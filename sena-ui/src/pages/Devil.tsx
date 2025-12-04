import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Flame,
  Zap,
  RefreshCw,
  Play,
  CheckCircle2,
  XCircle,
  Clock,
  AlertTriangle,
  Bot,
  Activity,
  BarChart3,
  Layers,
} from 'lucide-react';
import clsx from 'clsx';
import { useToast } from '../components/Toast';

interface DevilStatus {
  enabled: boolean;
  timeoutSecs: number;
  minProviders: number;
  synthesisMethod: string;
  consensusThreshold: number;
  waitMode: string;
  availableProviders: DevilProvider[];
}

interface DevilProvider {
  id: string;
  status: string;
}

interface DevilExecuteResult {
  content: string;
  consensusScore: number;
  synthesisMethod: string;
  totalLatencyMs: number;
  factsVerified: number;
  factsRejected: number;
  providerResponses: ProviderResponse[];
}

interface ProviderResponse {
  providerId: string;
  model: string;
  status: string;
  latencyMs: number;
  contentPreview: string | null;
}

export default function Devil() {
  const [status, setStatus] = useState<DevilStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [prompt, setPrompt] = useState('');
  const [timeout, setTimeout] = useState(30);
  const [executeResult, setExecuteResult] = useState<DevilExecuteResult | null>(null);
  const [executionHistory, setExecutionHistory] = useState<DevilExecuteResult[]>([]);
  const [isExecuting, setIsExecuting] = useState(false);
  const [isTesting, setIsTesting] = useState(false);
  const toast = useToast();

  const loadStatus = useCallback(async () => {
    try {
      const statusData = await invoke<DevilStatus>('get_devil_status');
      setStatus(statusData);
    } catch (error) {
      console.error('Failed to load Devil status:', error);
      setStatus({
        enabled: true,
        timeoutSecs: 30,
        minProviders: 2,
        synthesisMethod: 'CrossVerification',
        consensusThreshold: 0.6,
        waitMode: 'WaitForAll',
        availableProviders: [],
      });
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadStatus();
  }, [loadStatus]);

  const handleExecute = async () => {
    if (!prompt.trim()) {
      toast.error('Please enter a prompt');
      return;
    }

    setIsExecuting(true);
    try {
      const result = await invoke<DevilExecuteResult>('devil_execute', {
        prompt,
        timeout,
      });
      setExecuteResult(result);
      setExecutionHistory((prev) => [result, ...prev.slice(0, 4)]);

      const successCount = result.providerResponses.filter(
        (r) => r.status === 'Success'
      ).length;
      toast.success(`Devil Mode completed: ${successCount}/${result.providerResponses.length} providers responded`);
    } catch (error) {
      toast.error(`Execution failed: ${error}`);
    } finally {
      setIsExecuting(false);
    }
  };

  const handleTest = async () => {
    setIsTesting(true);
    try {
      const result = await invoke<DevilExecuteResult>('devil_test', {
        prompt: 'Test prompt for Devil Mode',
      });
      setExecuteResult(result);
      toast.success('Devil Mode test completed');
    } catch (error) {
      toast.error(`Test failed: ${error}`);
    } finally {
      setIsTesting(false);
    }
  };

  const getStatusColor = (providerStatus: string) => {
    if (providerStatus.includes('Success') || providerStatus === 'Connected') {
      return 'text-green-400';
    }
    if (providerStatus.includes('Error')) {
      return 'text-red-400';
    }
    if (providerStatus.includes('Timeout')) {
      return 'text-yellow-400';
    }
    return 'text-dark-400';
  };

  const getConsensusColor = (score: number) => {
    if (score >= 0.8) return 'text-green-400';
    if (score >= 0.6) return 'text-yellow-400';
    return 'text-red-400';
  };

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-dark-100 flex items-center gap-3">
            <Flame className="w-8 h-8 text-orange-500" />
            Devil Mode
          </h1>
          <p className="text-dark-400 mt-1">
            Parallel AI execution with cross-verification consensus
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={handleTest}
            disabled={isTesting}
            className="btn-secondary"
            title="Run test"
          >
            {isTesting ? (
              <RefreshCw className="w-5 h-5 animate-spin" />
            ) : (
              <Zap className="w-5 h-5" />
            )}
            Test
          </button>
          <button
            onClick={loadStatus}
            disabled={isLoading}
            className="btn-secondary"
            title="Refresh status"
          >
            <RefreshCw className={clsx('w-5 h-5', isLoading && 'animate-spin')} />
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-4 mb-8">
        <StatusCard
          icon={status?.enabled ? Flame : XCircle}
          label="Status"
          value={status?.enabled ? 'Active' : 'Inactive'}
          color={status?.enabled ? 'orange' : 'red'}
        />
        <StatusCard
          icon={Bot}
          label="Available Providers"
          value={`${status?.availableProviders.length || 0}`}
          color="blue"
        />
        <StatusCard
          icon={Layers}
          label="Synthesis Method"
          value={status?.synthesisMethod || 'CrossVerification'}
          color="purple"
        />
        <StatusCard
          icon={Activity}
          label="Consensus Threshold"
          value={`${((status?.consensusThreshold || 0.6) * 100).toFixed(0)}%`}
          color="sena"
        />
      </div>

      {status?.availableProviders && status.availableProviders.length > 0 && (
        <div className="card mb-6">
          <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
            <Bot className="w-5 h-5 text-sena-400" />
            Available Providers
          </h2>
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-3">
            {status.availableProviders.map((provider) => (
              <div
                key={provider.id}
                className="p-3 rounded-lg bg-dark-800/50 border border-dark-700"
              >
                <div className="flex items-center gap-2">
                  <div className={clsx(
                    'w-2 h-2 rounded-full',
                    provider.status.includes('Connected') ? 'bg-green-500' : 'bg-dark-500'
                  )} />
                  <span className="text-sm font-medium text-dark-200">{provider.id}</span>
                </div>
                <p className={clsx('text-xs mt-1', getStatusColor(provider.status))}>
                  {provider.status}
                </p>
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        <div className="card">
          <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
            <Play className="w-5 h-5 text-sena-400" />
            Execute Query
          </h2>
          <p className="text-dark-400 text-sm mb-4">
            Send a prompt to all available providers simultaneously and synthesize results.
          </p>
          <div className="space-y-4">
            <textarea
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              placeholder="Enter your prompt for parallel AI execution..."
              className="input w-full h-32 resize-none"
            />
            <div className="flex items-center gap-4">
              <label className="text-sm text-dark-400">
                Timeout (seconds):
              </label>
              <input
                type="number"
                value={timeout}
                onChange={(e) => setTimeout(Number(e.target.value))}
                min={5}
                max={120}
                className="input w-24"
              />
            </div>
            <button
              onClick={handleExecute}
              disabled={isExecuting || !prompt.trim()}
              className="btn-primary w-full"
            >
              {isExecuting ? (
                <>
                  <RefreshCw className="w-4 h-4 animate-spin" />
                  Executing...
                </>
              ) : (
                <>
                  <Flame className="w-4 h-4" />
                  Execute Devil Mode
                </>
              )}
            </button>
          </div>
        </div>

        <div className="card">
          <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
            <BarChart3 className="w-5 h-5 text-sena-400" />
            Execution Stats
          </h2>
          {executeResult ? (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="p-3 rounded-lg bg-dark-800/50">
                  <p className="text-xs text-dark-500">Consensus Score</p>
                  <p className={clsx('text-2xl font-bold', getConsensusColor(executeResult.consensusScore))}>
                    {(executeResult.consensusScore * 100).toFixed(0)}%
                  </p>
                </div>
                <div className="p-3 rounded-lg bg-dark-800/50">
                  <p className="text-xs text-dark-500">Total Latency</p>
                  <p className="text-2xl font-bold text-dark-100">
                    {executeResult.totalLatencyMs}ms
                  </p>
                </div>
                <div className="p-3 rounded-lg bg-dark-800/50">
                  <p className="text-xs text-dark-500">Facts Verified</p>
                  <p className="text-2xl font-bold text-green-400">
                    {executeResult.factsVerified}
                  </p>
                </div>
                <div className="p-3 rounded-lg bg-dark-800/50">
                  <p className="text-xs text-dark-500">Facts Rejected</p>
                  <p className="text-2xl font-bold text-red-400">
                    {executeResult.factsRejected}
                  </p>
                </div>
              </div>

              <div className="pt-4 border-t border-dark-700">
                <h3 className="text-sm font-medium text-dark-300 mb-3">Provider Responses</h3>
                <div className="space-y-2">
                  {executeResult.providerResponses.map((response, idx) => (
                    <div
                      key={idx}
                      className={clsx(
                        'p-3 rounded-lg',
                        response.status === 'Success'
                          ? 'bg-green-500/10 border border-green-500/20'
                          : response.status.includes('Error')
                          ? 'bg-red-500/10 border border-red-500/20'
                          : 'bg-yellow-500/10 border border-yellow-500/20'
                      )}
                    >
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          {response.status === 'Success' ? (
                            <CheckCircle2 className="w-4 h-4 text-green-400" />
                          ) : response.status.includes('Error') ? (
                            <XCircle className="w-4 h-4 text-red-400" />
                          ) : (
                            <AlertTriangle className="w-4 h-4 text-yellow-400" />
                          )}
                          <span className="text-sm font-medium text-dark-200">
                            {response.providerId}
                          </span>
                          <span className="text-xs text-dark-500">
                            {response.model}
                          </span>
                        </div>
                        <div className="flex items-center gap-2">
                          <Clock className="w-3 h-3 text-dark-500" />
                          <span className="text-xs text-dark-400">
                            {response.latencyMs}ms
                          </span>
                        </div>
                      </div>
                      {response.contentPreview && (
                        <p className="text-xs text-dark-400 mt-2 truncate">
                          {response.contentPreview}
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            </div>
          ) : (
            <div className="text-center py-8">
              <Flame className="w-12 h-12 mx-auto text-dark-600 mb-4" />
              <p className="text-dark-500">
                Execute a query to see results here
              </p>
            </div>
          )}
        </div>
      </div>

      {executeResult && (
        <div className="card">
          <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
            <Layers className="w-5 h-5 text-sena-400" />
            Synthesized Response
          </h2>
          <div className="p-4 rounded-lg bg-dark-800/50 border border-dark-700">
            <p className="text-dark-200 whitespace-pre-wrap">
              {executeResult.content}
            </p>
          </div>
          <div className="mt-4 flex items-center gap-4 text-sm text-dark-400">
            <span>Synthesis Method: <span className="text-sena-400">{executeResult.synthesisMethod}</span></span>
            <span>|</span>
            <span>Consensus: <span className={getConsensusColor(executeResult.consensusScore)}>{(executeResult.consensusScore * 100).toFixed(0)}%</span></span>
          </div>
        </div>
      )}

      {executionHistory.length > 1 && (
        <div className="card mt-6">
          <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
            <Activity className="w-5 h-5 text-sena-400" />
            Execution History
          </h2>
          <div className="space-y-2">
            {executionHistory.slice(1).map((result, idx) => (
              <div
                key={idx}
                className="p-3 rounded-lg bg-dark-800/50 flex items-center justify-between"
              >
                <div className="flex items-center gap-4">
                  <span className={clsx('text-lg font-bold', getConsensusColor(result.consensusScore))}>
                    {(result.consensusScore * 100).toFixed(0)}%
                  </span>
                  <span className="text-sm text-dark-400">
                    {result.providerResponses.filter((r) => r.status === 'Success').length}/{result.providerResponses.length} providers
                  </span>
                </div>
                <div className="flex items-center gap-2 text-sm text-dark-500">
                  <Clock className="w-4 h-4" />
                  {result.totalLatencyMs}ms
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function StatusCard({
  icon: Icon,
  label,
  value,
  color,
}: {
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  value: string;
  color: 'green' | 'red' | 'orange' | 'blue' | 'purple' | 'sena';
}) {
  const colorMap = {
    green: 'text-green-400 bg-green-500/10',
    red: 'text-red-400 bg-red-500/10',
    orange: 'text-orange-400 bg-orange-500/10',
    blue: 'text-blue-400 bg-blue-500/10',
    purple: 'text-purple-400 bg-purple-500/10',
    sena: 'text-sena-400 bg-sena-500/10',
  };

  return (
    <div className="card">
      <div className="flex items-center gap-3">
        <div className={clsx('w-10 h-10 rounded-lg flex items-center justify-center', colorMap[color])}>
          <Icon className="w-5 h-5" />
        </div>
        <div>
          <p className="text-xs text-dark-500">{label}</p>
          <p className="text-lg font-semibold text-dark-100">{value}</p>
        </div>
      </div>
    </div>
  );
}
