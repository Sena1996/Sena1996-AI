import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Bot,
  CheckCircle2,
  XCircle,
  AlertCircle,
  RefreshCw,
  Settings,
  Zap,
  Key,
  LogIn,
  LogOut,
  Loader2,
  ExternalLink,
  Shield,
} from 'lucide-react';
import { useToast } from '../components/Toast';
import ProviderConfigModal from '../components/ProviderConfigModal';
import clsx from 'clsx';
import type { ProviderMetadata, CredentialStatus } from '../types';

interface ProviderInfo {
  id: string;
  name: string;
  status: string;
  default_model: string;
  has_api_key: boolean;
  capabilities: {
    streaming: boolean;
    tools: boolean;
    vision: boolean;
  };
}

interface ProviderStatus {
  metadata: ProviderMetadata;
  credentialStatus: CredentialStatus;
  connectionInfo?: ProviderInfo;
  isOAuthLoggedIn: boolean;
}

export default function Providers() {
  const [allProviderStatuses, setAllProviderStatuses] = useState<ProviderStatus[]>([]);
  const [testing, setTesting] = useState<string | null>(null);
  const [oauthLoggingIn, setOAuthLoggingIn] = useState<string | null>(null);
  const [defaultProvider, setDefaultProvider] = useState('claude');
  const [isLoading, setIsLoading] = useState(true);
  const [selectedProvider, setSelectedProvider] = useState<string | null>(null);
  const toast = useToast();

  const loadAllProviderData = useCallback(async () => {
    try {
      const metadata = await invoke<ProviderMetadata[]>('get_all_provider_metadata_cmd');
      const connectionInfo = await invoke<ProviderInfo[]>('get_providers').catch(() => []);

      const statuses: ProviderStatus[] = await Promise.all(
        metadata.map(async (meta) => {
          const credentialStatus = await invoke<CredentialStatus>('get_credential_status_cmd', {
            providerId: meta.id,
          }).catch(() => ({
            providerId: meta.id,
            hasCredential: false,
            source: 'none' as const,
            isValid: null,
            canImportFromEnv: false,
          }));

          const isOAuthLoggedIn = meta.oauthSupported
            ? await invoke<boolean>('check_oauth_status', { providerId: meta.id }).catch(() => false)
            : false;

          const connection = connectionInfo.find((p) => p.id === meta.id);

          return {
            metadata: meta,
            credentialStatus,
            connectionInfo: connection,
            isOAuthLoggedIn,
          };
        })
      );

      setAllProviderStatuses(statuses);
    } catch (error) {
      console.error('Failed to load provider data:', error);
      toast.error(`Failed to load providers: ${error}`);
    } finally {
      setIsLoading(false);
    }
  }, [toast]);

  const refreshProviders = useCallback(async () => {
    setIsLoading(true);
    await loadAllProviderData();
    toast.success('Providers refreshed');
  }, [loadAllProviderData, toast]);

  useEffect(() => {
    loadAllProviderData();
  }, [loadAllProviderData]);

  const handleCredentialSaved = useCallback(async () => {
    await loadAllProviderData();
    toast.success('Credentials saved successfully');
  }, [loadAllProviderData, toast]);

  const selectedProviderStatus = allProviderStatuses.find((s) => s.metadata.id === selectedProvider);

  const handleTest = async (providerId: string) => {
    setTesting(providerId);
    try {
      const result = await invoke<{ success: boolean; message: string }>('test_provider', {
        providerId,
      });
      if (result.success) {
        toast.success(`${providerId} connection successful`);
      } else {
        toast.error(`${providerId}: ${result.message}`);
      }
    } catch (error) {
      toast.error(`Test failed: ${error}`);
    } finally {
      setTesting(null);
      await loadAllProviderData();
    }
  };

  const handleSetDefault = async (providerId: string) => {
    try {
      await invoke('set_default_provider', { providerId });
      setDefaultProvider(providerId);
      toast.success(`${providerId} set as default provider`);
    } catch (error) {
      toast.error(`Failed to set default: ${error}`);
    }
  };

  const handleOAuthLogin = async (providerId: string) => {
    setOAuthLoggingIn(providerId);
    try {
      await invoke('initiate_oauth_login', { providerId });
      await loadAllProviderData();
      toast.success(`Logged in to ${providerId} via OAuth`);
    } catch (error) {
      toast.error(`OAuth login failed: ${error}`);
    } finally {
      setOAuthLoggingIn(null);
    }
  };

  const handleOAuthLogout = async (providerId: string) => {
    try {
      await invoke('logout_oauth', { providerId });
      await loadAllProviderData();
      toast.success(`Logged out from ${providerId}`);
    } catch (error) {
      toast.error(`Logout failed: ${error}`);
    }
  };

  const oauthProviders = allProviderStatuses.filter((p) => p.metadata.oauthSupported);
  const apiKeyProviders = allProviderStatuses.filter(
    (p) => !p.metadata.oauthSupported && p.metadata.authSchema.authType === 'api_key'
  );
  const localProviders = allProviderStatuses.filter(
    (p) => p.metadata.authSchema.authType === 'local' || p.metadata.authSchema.authType === 'none'
  );

  return (
    <div className="p-8 max-w-7xl mx-auto">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold text-dark-100">AI Providers</h1>
          <p className="text-dark-400 mt-2">
            Connect to AI providers via OAuth or API keys. Supports 12 providers across 3 authentication types.
          </p>
        </div>
        <button
          onClick={refreshProviders}
          disabled={isLoading}
          className="btn-secondary flex items-center gap-2"
          title="Refresh providers"
        >
          <RefreshCw className={clsx('w-5 h-5', isLoading && 'animate-spin')} />
          Refresh
        </button>
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center py-20">
          <Loader2 className="w-8 h-8 animate-spin text-sena-400" />
        </div>
      ) : (
        <div className="space-y-8">
          {oauthProviders.length > 0 && (
            <ProviderSection
              title="OAuth Authentication"
              description="Connect with your account for enhanced security and features"
              icon={<Shield className="w-5 h-5" />}
              providers={oauthProviders}
              defaultProvider={defaultProvider}
              testing={testing}
              oauthLoggingIn={oauthLoggingIn}
              onTest={handleTest}
              onSetDefault={handleSetDefault}
              onConfigure={(id) => setSelectedProvider(id)}
              onOAuthLogin={handleOAuthLogin}
              onOAuthLogout={handleOAuthLogout}
            />
          )}

          {apiKeyProviders.length > 0 && (
            <ProviderSection
              title="API Key Authentication"
              description="Use API keys from provider platforms"
              icon={<Key className="w-5 h-5" />}
              providers={apiKeyProviders}
              defaultProvider={defaultProvider}
              testing={testing}
              oauthLoggingIn={oauthLoggingIn}
              onTest={handleTest}
              onSetDefault={handleSetDefault}
              onConfigure={(id) => setSelectedProvider(id)}
              onOAuthLogin={handleOAuthLogin}
              onOAuthLogout={handleOAuthLogout}
            />
          )}

          {localProviders.length > 0 && (
            <ProviderSection
              title="Local Models"
              description="Run AI models locally without API keys"
              icon={<Bot className="w-5 h-5" />}
              providers={localProviders}
              defaultProvider={defaultProvider}
              testing={testing}
              oauthLoggingIn={oauthLoggingIn}
              onTest={handleTest}
              onSetDefault={handleSetDefault}
              onConfigure={(id) => setSelectedProvider(id)}
              onOAuthLogin={handleOAuthLogin}
              onOAuthLogout={handleOAuthLogout}
            />
          )}
        </div>
      )}

      {selectedProviderStatus && (
        <ProviderConfigModal
          provider={selectedProviderStatus.metadata}
          credentialStatus={selectedProviderStatus.credentialStatus}
          isOpen={selectedProvider !== null}
          onClose={() => setSelectedProvider(null)}
          onSave={handleCredentialSaved}
        />
      )}
    </div>
  );
}

interface ProviderSectionProps {
  title: string;
  description: string;
  icon: React.ReactNode;
  providers: ProviderStatus[];
  defaultProvider: string;
  testing: string | null;
  oauthLoggingIn: string | null;
  onTest: (id: string) => void;
  onSetDefault: (id: string) => void;
  onConfigure: (id: string) => void;
  onOAuthLogin: (id: string) => void;
  onOAuthLogout: (id: string) => void;
}

function ProviderSection({
  title,
  description,
  icon,
  providers,
  defaultProvider,
  testing,
  oauthLoggingIn,
  onTest,
  onSetDefault,
  onConfigure,
  onOAuthLogin,
  onOAuthLogout,
}: ProviderSectionProps) {
  return (
    <div>
      <div className="flex items-center gap-3 mb-4">
        <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-sena-500/10 text-sena-400">
          {icon}
        </div>
        <div>
          <h2 className="text-xl font-semibold text-dark-100">{title}</h2>
          <p className="text-sm text-dark-400">{description}</p>
        </div>
      </div>
      <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
        {providers.map((providerStatus) => (
          <ProviderCard
            key={providerStatus.metadata.id}
            providerStatus={providerStatus}
            isDefault={providerStatus.metadata.id === defaultProvider}
            isTesting={testing === providerStatus.metadata.id}
            isOAuthLoggingIn={oauthLoggingIn === providerStatus.metadata.id}
            onTest={() => onTest(providerStatus.metadata.id)}
            onSetDefault={() => onSetDefault(providerStatus.metadata.id)}
            onConfigure={() => onConfigure(providerStatus.metadata.id)}
            onOAuthLogin={() => onOAuthLogin(providerStatus.metadata.id)}
            onOAuthLogout={() => onOAuthLogout(providerStatus.metadata.id)}
          />
        ))}
      </div>
    </div>
  );
}

function ProviderCard({
  providerStatus,
  isDefault,
  isTesting,
  isOAuthLoggingIn,
  onTest,
  onSetDefault,
  onConfigure,
  onOAuthLogin,
  onOAuthLogout,
}: {
  providerStatus: ProviderStatus;
  isDefault: boolean;
  isTesting: boolean;
  isOAuthLoggingIn: boolean;
  onTest: () => void;
  onSetDefault: () => void;
  onConfigure: () => void;
  onOAuthLogin: () => void;
  onOAuthLogout: () => void;
}) {
  const { metadata, credentialStatus, connectionInfo, isOAuthLoggedIn } = providerStatus;
  const isConnected = connectionInfo?.status === 'connected';
  const isError = connectionInfo?.status === 'error';
  const hasAuth = isOAuthLoggedIn || credentialStatus.hasCredential;

  const StatusIcon = isConnected ? CheckCircle2 : isError ? AlertCircle : XCircle;

  const statusColor = isConnected
    ? 'text-green-400'
    : isError
      ? 'text-red-400'
      : 'text-dark-500';

  return (
    <div className="card hover:border-sena-500/30 transition-all duration-200">
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-sena-500/20 to-sena-600/20 flex items-center justify-center border border-sena-500/30">
            <Bot className="w-6 h-6 text-sena-400" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h3 className="font-semibold text-dark-100">{metadata.displayName}</h3>
              {isDefault && (
                <span className="badge bg-sena-500/20 text-sena-400 border border-sena-500/30">
                  Default
                </span>
              )}
            </div>
            <p className="text-xs text-dark-400 mt-0.5">{metadata.description}</p>
          </div>
        </div>
        <StatusIcon className={`w-5 h-5 ${statusColor} flex-shrink-0`} />
      </div>

      {metadata.oauthSupported && (
        <div className="mb-4">
          {isOAuthLoggedIn ? (
            <div className="flex items-center justify-between p-3 rounded-lg border border-green-500/30 bg-green-500/10">
              <div className="flex items-center gap-2">
                <CheckCircle2 className="w-4 h-4 text-green-400" />
                <span className="text-sm font-medium text-green-400">OAuth Connected</span>
              </div>
              <button
                onClick={onOAuthLogout}
                className="text-xs text-dark-400 hover:text-dark-100 flex items-center gap-1"
              >
                <LogOut className="w-3 h-3" />
                Logout
              </button>
            </div>
          ) : (
            <button
              onClick={onOAuthLogin}
              disabled={isOAuthLoggingIn}
              className="w-full flex items-center justify-center gap-2 p-3 rounded-lg border border-sena-500/50 bg-sena-500/10 hover:bg-sena-500/20 text-sena-400 hover:text-sena-300 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isOAuthLoggingIn ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  <span className="text-sm font-medium">Authenticating...</span>
                </>
              ) : (
                <>
                  <LogIn className="w-4 h-4" />
                  <span className="text-sm font-medium">Login with OAuth</span>
                </>
              )}
            </button>
          )}
        </div>
      )}

      <div className="flex flex-wrap gap-2 mb-4">
        {connectionInfo?.capabilities.streaming && (
          <span className="badge badge-info text-xs">Streaming</span>
        )}
        {connectionInfo?.capabilities.tools && (
          <span className="badge badge-info text-xs">Tools</span>
        )}
        {connectionInfo?.capabilities.vision && (
          <span className="badge badge-info text-xs">Vision</span>
        )}
        {credentialStatus.source !== 'none' && (
          <span
            className={`badge text-xs ${
              credentialStatus.source === 'keychain'
                ? 'bg-green-500/20 text-green-400 border border-green-500/30'
                : credentialStatus.source === 'config'
                  ? 'bg-blue-500/20 text-blue-400 border border-blue-500/30'
                  : 'bg-yellow-500/20 text-yellow-400 border border-yellow-500/30'
            }`}
          >
            {credentialStatus.source === 'keychain' && 'Keychain'}
            {credentialStatus.source === 'config' && 'Config'}
            {credentialStatus.source === 'environment' && 'Env'}
          </span>
        )}
      </div>

      <div className="space-y-2">
        {metadata.authSchema.authType !== 'none' && metadata.authSchema.authType !== 'local' && (
          <button
            onClick={onConfigure}
            className="w-full btn-secondary flex items-center justify-center gap-2 text-sm"
          >
            <Key className="w-4 h-4" />
            {credentialStatus.hasCredential ? 'Update API Key' : 'Configure API Key'}
          </button>
        )}

        <div className="flex gap-2">
          <button
            onClick={onTest}
            disabled={isTesting || !hasAuth}
            className="btn-secondary flex-1 flex items-center justify-center gap-2 text-sm disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isTesting ? (
              <>
                <Loader2 className="w-4 h-4 animate-spin" />
                Testing...
              </>
            ) : (
              <>
                <Zap className="w-4 h-4" />
                Test
              </>
            )}
          </button>
          {!isDefault && isConnected && (
            <button
              onClick={onSetDefault}
              className="btn-primary flex items-center gap-2 text-sm"
            >
              <Settings className="w-4 h-4" />
              Set Default
            </button>
          )}
        </div>

        {metadata.website && (
          <a
            href={metadata.website}
            target="_blank"
            rel="noopener noreferrer"
            className="w-full flex items-center justify-center gap-2 p-2 text-xs text-dark-400 hover:text-sena-400 transition-colors"
          >
            <ExternalLink className="w-3 h-3" />
            Visit {metadata.displayName}
          </a>
        )}
      </div>
    </div>
  );
}
