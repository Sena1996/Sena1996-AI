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
} from 'lucide-react';
import { useToast } from '../components/Toast';
import clsx from 'clsx';

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

export default function Providers() {
  const [providers, setProviders] = useState<ProviderInfo[]>([]);
  const [testing, setTesting] = useState<string | null>(null);
  const [defaultProvider, setDefaultProvider] = useState('claude');
  const [isLoading, setIsLoading] = useState(true);
  const toast = useToast();

  const loadProviders = useCallback(async () => {
    try {
      const data = await invoke<ProviderInfo[]>('get_providers');
      setProviders(data);
    } catch (error) {
      console.error('Failed to load providers:', error);
      toast.error(`Failed to load providers: ${error}`);
    } finally {
      setIsLoading(false);
    }
  }, [toast]);

  const refreshProviders = useCallback(async () => {
    setIsLoading(true);
    await loadProviders();
    toast.success('Providers refreshed');
  }, [loadProviders, toast]);

  useEffect(() => {
    loadProviders();
  }, [loadProviders]);

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
      await loadProviders();
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

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-dark-100">AI Providers</h1>
          <p className="text-dark-400 mt-1">
            Manage and configure your AI provider connections
          </p>
        </div>
        <button
          onClick={refreshProviders}
          disabled={isLoading}
          className="btn-secondary"
          title="Refresh providers"
        >
          <RefreshCw className={clsx('w-5 h-5', isLoading && 'animate-spin')} />
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {providers.map((provider) => (
          <ProviderCard
            key={provider.id}
            provider={provider}
            isDefault={provider.id === defaultProvider}
            isTesting={testing === provider.id}
            onTest={() => handleTest(provider.id)}
            onSetDefault={() => handleSetDefault(provider.id)}
          />
        ))}
      </div>

      <div className="mt-8 card">
        <h2 className="text-lg font-semibold text-dark-100 mb-4">
          Configuration
        </h2>
        <div className="space-y-4 text-sm">
          <div className="p-4 rounded-lg bg-dark-800/50">
            <p className="text-dark-300 mb-2">
              Set API keys via environment variables:
            </p>
            <code className="block text-sena-400 font-mono">
              export ANTHROPIC_API_KEY=your-key # Claude
            </code>
            <code className="block text-sena-400 font-mono mt-1">
              export OPENAI_API_KEY=your-key # OpenAI
            </code>
            <code className="block text-sena-400 font-mono mt-1">
              export GOOGLE_API_KEY=your-key # Gemini
            </code>
            <code className="block text-sena-400 font-mono mt-1">
              export MISTRAL_API_KEY=your-key # Mistral
            </code>
          </div>
          <p className="text-dark-400">
            Ollama runs locally and doesn't require an API key. Make sure the
            Ollama service is running on localhost:11434.
          </p>
        </div>
      </div>
    </div>
  );
}

function ProviderCard({
  provider,
  isDefault,
  isTesting,
  onTest,
  onSetDefault,
}: {
  provider: ProviderInfo;
  isDefault: boolean;
  isTesting: boolean;
  onTest: () => void;
  onSetDefault: () => void;
}) {
  const isConnected = provider.status === 'connected';
  const isError = provider.status === 'error';

  const StatusIcon = isConnected ? CheckCircle2 : isError ? AlertCircle : XCircle;

  const statusColor = isConnected
    ? 'text-green-400'
    : isError
      ? 'text-red-400'
      : 'text-dark-500';

  const statusBg = isConnected
    ? 'bg-green-500/10'
    : isError
      ? 'bg-red-500/10'
      : 'bg-dark-800';

  return (
    <div className="card">
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center gap-3">
          <div
            className={`w-12 h-12 rounded-xl flex items-center justify-center ${statusBg}`}
          >
            <Bot className={`w-6 h-6 ${statusColor}`} />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h3 className="font-semibold text-dark-100">{provider.name}</h3>
              {isDefault && (
                <span className="badge bg-sena-500/20 text-sena-400">
                  Default
                </span>
              )}
            </div>
            <p className="text-sm text-dark-400">{provider.default_model}</p>
          </div>
        </div>
        <StatusIcon className={`w-5 h-5 ${statusColor}`} />
      </div>

      <div className="flex flex-wrap gap-2 mb-4">
        {provider.capabilities.streaming && (
          <span className="badge badge-info">Streaming</span>
        )}
        {provider.capabilities.tools && (
          <span className="badge badge-info">Tool Use</span>
        )}
        {provider.capabilities.vision && (
          <span className="badge badge-info">Vision</span>
        )}
        {provider.has_api_key ? (
          <span className="badge badge-success">API Key Set</span>
        ) : (
          <span className="badge badge-warning">No API Key</span>
        )}
      </div>

      <div className="flex gap-2">
        <button
          onClick={onTest}
          disabled={isTesting || !provider.has_api_key}
          className="btn-secondary flex-1"
        >
          {isTesting ? (
            <>
              <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
              Testing...
            </>
          ) : (
            <>
              <Zap className="w-4 h-4 mr-2" />
              Test
            </>
          )}
        </button>
        {!isDefault && isConnected && (
          <button onClick={onSetDefault} className="btn-primary">
            <Settings className="w-4 h-4 mr-2" />
            Set Default
          </button>
        )}
      </div>
    </div>
  );
}
