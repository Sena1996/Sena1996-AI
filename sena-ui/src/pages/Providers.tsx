import { useState } from 'react';
import {
  Bot,
  CheckCircle2,
  XCircle,
  AlertCircle,
  RefreshCw,
  Settings,
  Zap,
} from 'lucide-react';
import type { Provider } from '../types';

const mockProviders: Provider[] = [
  {
    id: 'claude',
    name: 'Claude (Anthropic)',
    status: 'connected',
    defaultModel: 'claude-sonnet-4-5-20250929',
    hasApiKey: true,
    capabilities: { streaming: true, tools: true, vision: true },
  },
  {
    id: 'openai',
    name: 'OpenAI',
    status: 'disconnected',
    defaultModel: 'gpt-4.1',
    hasApiKey: false,
    capabilities: { streaming: true, tools: true, vision: true },
  },
  {
    id: 'gemini',
    name: 'Google Gemini',
    status: 'disconnected',
    defaultModel: 'gemini-2.5-flash',
    hasApiKey: false,
    capabilities: { streaming: true, tools: true, vision: true },
  },
  {
    id: 'ollama',
    name: 'Ollama (Local)',
    status: 'connected',
    defaultModel: 'llama3.2',
    hasApiKey: true,
    capabilities: { streaming: true, tools: true, vision: false },
  },
  {
    id: 'mistral',
    name: 'Mistral AI',
    status: 'disconnected',
    defaultModel: 'mistral-large-latest',
    hasApiKey: false,
    capabilities: { streaming: true, tools: true, vision: true },
  },
];

export default function Providers() {
  const [providers] = useState<Provider[]>(mockProviders);
  const [testing, setTesting] = useState<string | null>(null);
  const [defaultProvider, setDefaultProvider] = useState('claude');

  const handleTest = async (providerId: string) => {
    setTesting(providerId);
    await new Promise((resolve) => setTimeout(resolve, 1500));
    setTesting(null);
  };

  const handleSetDefault = (providerId: string) => {
    setDefaultProvider(providerId);
  };

  return (
    <div className="p-8">
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-dark-100">AI Providers</h1>
        <p className="text-dark-400 mt-1">
          Manage and configure your AI provider connections
        </p>
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
  provider: Provider;
  isDefault: boolean;
  isTesting: boolean;
  onTest: () => void;
  onSetDefault: () => void;
}) {
  const StatusIcon =
    provider.status === 'connected'
      ? CheckCircle2
      : provider.status === 'error'
        ? AlertCircle
        : XCircle;

  const statusColor =
    provider.status === 'connected'
      ? 'text-green-400'
      : provider.status === 'error'
        ? 'text-red-400'
        : 'text-dark-500';

  const statusBg =
    provider.status === 'connected'
      ? 'bg-green-500/10'
      : provider.status === 'error'
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
            <p className="text-sm text-dark-400">{provider.defaultModel}</p>
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
        {provider.hasApiKey ? (
          <span className="badge badge-success">API Key Set</span>
        ) : (
          <span className="badge badge-warning">No API Key</span>
        )}
      </div>

      <div className="flex gap-2">
        <button
          onClick={onTest}
          disabled={isTesting || !provider.hasApiKey}
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
        {!isDefault && provider.status === 'connected' && (
          <button onClick={onSetDefault} className="btn-primary">
            <Settings className="w-4 h-4 mr-2" />
            Set Default
          </button>
        )}
      </div>
    </div>
  );
}
