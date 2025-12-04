import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  X,
  Key,
  Shield,
  FileText,
  Download,
  CheckCircle2,
  AlertCircle,
  Loader2,
  ExternalLink,
} from 'lucide-react';
import DynamicAuthForm from './DynamicAuthForm';
import type {
  ProviderMetadata,
  CredentialStatus,
  StorageOptions,
  StorageType,
  ValidationResult,
} from '../types';

interface ProviderConfigModalProps {
  provider: ProviderMetadata;
  credentialStatus: CredentialStatus;
  isOpen: boolean;
  onClose: () => void;
  onSave: () => void;
}

export default function ProviderConfigModal({
  provider,
  credentialStatus,
  isOpen,
  onClose,
  onSave,
}: ProviderConfigModalProps) {
  const [values, setValues] = useState<Record<string, string>>({});
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [storageType, setStorageType] = useState<StorageType>('keychain');
  const [storageOptions, setStorageOptions] = useState<StorageOptions | null>(null);
  const [isValidating, setIsValidating] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);

  useEffect(() => {
    if (isOpen) {
      invoke<StorageOptions>('get_storage_options_cmd')
        .then((options) => {
          setStorageOptions(options);
          if (!options.keychainAvailable) {
            setStorageType('config');
          }
        })
        .catch(console.error);

      const loadCredentials = async () => {
        const loadedValues: Record<string, string> = {};

        for (const field of provider.authSchema.fields) {
          if (field.defaultValue) {
            loadedValues[field.id] = field.defaultValue;
          }

          try {
            const storedValue = await invoke<string | null>('get_credential', {
              providerId: provider.id,
              fieldId: field.id,
            });
            if (storedValue) {
              loadedValues[field.id] = storedValue;
            }
          } catch (error) {
            console.error(`Failed to load credential for ${field.id}:`, error);
          }
        }

        setValues(loadedValues);
      };

      loadCredentials();
      setErrors({});
      setValidationResult(null);
    }
  }, [isOpen, provider]);

  const handleFieldChange = (fieldId: string, value: string) => {
    setValues((prev) => ({ ...prev, [fieldId]: value }));
    setErrors((prev) => {
      const next = { ...prev };
      delete next[fieldId];
      return next;
    });
    setValidationResult(null);
  };

  const validateFields = (): boolean => {
    const newErrors: Record<string, string> = {};

    provider.authSchema.fields.forEach((field) => {
      const value = values[field.id] || '';

      if (field.required && !value.trim()) {
        newErrors[field.id] = `${field.displayName} is required`;
      } else if (field.validationPattern && value) {
        const regex = new RegExp(field.validationPattern);
        if (!regex.test(value)) {
          newErrors[field.id] = `Invalid ${field.displayName.toLowerCase()} format`;
        }
      }
    });

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleValidate = async () => {
    if (!validateFields()) return;

    setIsValidating(true);
    setValidationResult(null);

    try {
      const firstField = provider.authSchema.fields[0];
      const credentialValue = firstField ? values[firstField.id] : '';

      if (!credentialValue && provider.id !== 'ollama') {
        setValidationResult({ valid: false, error: 'No credential provided' });
        return;
      }

      const result = await invoke<ValidationResult>('validate_api_key_cmd', {
        providerId: provider.id,
        apiKey: credentialValue || '',
      });
      setValidationResult(result);
    } catch (error) {
      setValidationResult({ valid: false, error: String(error) });
    } finally {
      setIsValidating(false);
    }
  };

  const handleImportFromEnv = async () => {
    const envVarField = provider.authSchema.fields.find((f) => f.envVarName);
    if (!envVarField?.envVarName) return;

    try {
      await invoke('import_env_to_storage', {
        providerId: provider.id,
        envVar: envVarField.envVarName,
        storageType,
      });
      onSave();
      onClose();
    } catch (error) {
      setErrors({ [envVarField.id]: String(error) });
    }
  };

  const handleSave = async () => {
    if (!validateFields()) return;

    setIsSaving(true);

    try {
      for (const field of provider.authSchema.fields) {
        const value = values[field.id];
        if (value) {
          await invoke('save_credential', {
            providerId: provider.id,
            fieldId: field.id,
            value,
            storageType,
          });
        }
      }
      onSave();
      onClose();
    } catch (error) {
      setErrors({ _general: String(error) });
    } finally {
      setIsSaving(false);
    }
  };

  if (!isOpen) return null;

  const sourceLabel = {
    keychain: 'OS Keychain',
    config: 'Config File',
    environment: 'Environment Variable',
    none: 'Not Configured',
  }[credentialStatus.source];

  const sourceColor = {
    keychain: 'text-green-400 bg-green-500/10',
    config: 'text-blue-400 bg-blue-500/10',
    environment: 'text-yellow-400 bg-yellow-500/10',
    none: 'text-dark-400 bg-dark-700',
  }[credentialStatus.source];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onClose}
      />

      <div className="relative w-full max-w-lg bg-dark-900 rounded-2xl shadow-2xl border border-dark-700">
        <div className="flex items-center justify-between p-6 border-b border-dark-700">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-sena-500/20 flex items-center justify-center">
              <Key className="w-5 h-5 text-sena-400" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-dark-100">
                Configure {provider.displayName}
              </h2>
              <p className="text-sm text-dark-400">{provider.description}</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-2 text-dark-400 hover:text-dark-200 rounded-lg hover:bg-dark-800 transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="p-6 space-y-6">
          <div className="flex items-center justify-between p-3 rounded-lg bg-dark-800/50">
            <span className="text-sm text-dark-300">Current Status</span>
            <span className={`px-2.5 py-1 rounded-full text-xs font-medium ${sourceColor}`}>
              {sourceLabel}
            </span>
          </div>

          <DynamicAuthForm
            fields={provider.authSchema.fields}
            values={values}
            onChange={handleFieldChange}
            errors={errors}
            disabled={isSaving}
          />

          {storageOptions && (
            <div className="space-y-3">
              <label className="text-sm font-medium text-dark-200">
                Storage Location
              </label>
              <div className="grid grid-cols-2 gap-3">
                <button
                  type="button"
                  onClick={() => setStorageType('keychain')}
                  disabled={!storageOptions.keychainAvailable || isSaving}
                  className={`p-4 rounded-xl border-2 text-left transition-all ${
                    storageType === 'keychain'
                      ? 'border-sena-500 bg-sena-500/10'
                      : 'border-dark-700 hover:border-dark-600'
                  } ${
                    !storageOptions.keychainAvailable
                      ? 'opacity-50 cursor-not-allowed'
                      : ''
                  }`}
                >
                  <Shield className={`w-5 h-5 mb-2 ${
                    storageType === 'keychain' ? 'text-sena-400' : 'text-dark-400'
                  }`} />
                  <p className={`text-sm font-medium ${
                    storageType === 'keychain' ? 'text-sena-400' : 'text-dark-200'
                  }`}>
                    OS Keychain
                  </p>
                  <p className="text-xs text-dark-400 mt-1">
                    More secure
                  </p>
                </button>

                <button
                  type="button"
                  onClick={() => setStorageType('config')}
                  disabled={isSaving}
                  className={`p-4 rounded-xl border-2 text-left transition-all ${
                    storageType === 'config'
                      ? 'border-sena-500 bg-sena-500/10'
                      : 'border-dark-700 hover:border-dark-600'
                  }`}
                >
                  <FileText className={`w-5 h-5 mb-2 ${
                    storageType === 'config' ? 'text-sena-400' : 'text-dark-400'
                  }`} />
                  <p className={`text-sm font-medium ${
                    storageType === 'config' ? 'text-sena-400' : 'text-dark-200'
                  }`}>
                    Config File
                  </p>
                  <p className="text-xs text-dark-400 mt-1">
                    Portable
                  </p>
                </button>
              </div>
              {storageType === 'config' && storageOptions.configFilePath && (
                <p className="text-xs text-dark-500">
                  File: <code className="text-dark-400">{storageOptions.configFilePath}</code>
                </p>
              )}
            </div>
          )}

          {credentialStatus.canImportFromEnv && (
            <button
              type="button"
              onClick={handleImportFromEnv}
              disabled={isSaving}
              className="w-full flex items-center justify-center gap-2 p-3 rounded-lg border border-dashed border-dark-600 text-dark-300 hover:text-dark-100 hover:border-dark-500 transition-colors"
            >
              <Download className="w-4 h-4" />
              Import from Environment Variable
            </button>
          )}

          {validationResult && (
            <div className={`flex items-center gap-3 p-3 rounded-lg ${
              validationResult.valid
                ? 'bg-green-500/10 text-green-400'
                : 'bg-red-500/10 text-red-400'
            }`}>
              {validationResult.valid ? (
                <CheckCircle2 className="w-5 h-5" />
              ) : (
                <AlertCircle className="w-5 h-5" />
              )}
              <span className="text-sm">
                {validationResult.valid
                  ? 'API key is valid'
                  : validationResult.error || 'Invalid API key'}
              </span>
            </div>
          )}

          {errors._general && (
            <div className="flex items-center gap-2 p-3 rounded-lg bg-red-500/10 text-red-400">
              <AlertCircle className="w-5 h-5" />
              <span className="text-sm">{errors._general}</span>
            </div>
          )}
        </div>

        <div className="flex items-center justify-between p-6 border-t border-dark-700">
          <button
            type="button"
            onClick={() => {
              const apiKeyUrls: Record<string, string> = {
                claude: 'https://console.anthropic.com/settings/keys',
                openai: 'https://platform.openai.com/api-keys',
                gemini: 'https://aistudio.google.com/apikey',
                mistral: 'https://console.mistral.ai/api-keys',
                ollama: 'https://ollama.com/download',
              };
              const url = apiKeyUrls[provider.id] || provider.website;
              invoke('open_external_url', { url }).catch(console.error);
            }}
            className="flex items-center gap-2 text-sm text-dark-400 hover:text-sena-400 transition-colors"
          >
            <ExternalLink className="w-4 h-4" />
            Get API Key
          </button>

          <div className="flex items-center gap-3">
            <button
              type="button"
              onClick={handleValidate}
              disabled={isValidating || isSaving || !Object.values(values).some(Boolean)}
              className="btn-secondary"
            >
              {isValidating ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Validating...
                </>
              ) : (
                'Validate'
              )}
            </button>

            <button
              type="button"
              onClick={onClose}
              disabled={isSaving}
              className="btn-secondary"
            >
              Cancel
            </button>

            <button
              type="button"
              onClick={handleSave}
              disabled={isSaving || !Object.values(values).some(Boolean)}
              className="btn-primary"
            >
              {isSaving ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Saving...
                </>
              ) : (
                'Save'
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
