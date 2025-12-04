import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Shield,
  ShieldCheck,
  ShieldAlert,
  ShieldX,
  RefreshCw,
  Terminal,
  AlertTriangle,
  CheckCircle2,
  XCircle,
  Eye,
  Activity,
  Search,
} from 'lucide-react';
import clsx from 'clsx';
import { useToast } from '../components/Toast';

interface GuardianStatus {
  enabled: boolean;
  sandboxLevel: string;
  hallucinationMode: string;
  threshold: number;
}

interface ValidationResult {
  command: string;
  allowed: boolean;
  riskScore: number;
  reason: string | null;
  matchedPatterns: string[];
}

interface HallucinationCheck {
  isHallucination: boolean;
  riskScore: number;
  response: string;
  harmonyStatus: string;
  warnings: string[];
  details: {
    consistencyScore: number;
    semanticEntropy: number;
    factValidationScore: number;
    suspiciousPatterns: string[];
  };
}

export default function Guardian() {
  const [status, setStatus] = useState<GuardianStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [commandInput, setCommandInput] = useState('');
  const [contentInput, setContentInput] = useState('');
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);
  const [hallucinationResult, setHallucinationResult] = useState<HallucinationCheck | null>(null);
  const [validationHistory, setValidationHistory] = useState<ValidationResult[]>([]);
  const [isValidating, setIsValidating] = useState(false);
  const [isChecking, setIsChecking] = useState(false);
  const toast = useToast();

  const loadStatus = useCallback(async () => {
    try {
      const statusData = await invoke<GuardianStatus>('get_guardian_status');
      setStatus(statusData);
    } catch (error) {
      console.error('Failed to load Guardian status:', error);
      setStatus({
        enabled: true,
        sandboxLevel: 'Full',
        hallucinationMode: 'All',
        threshold: 0.7,
      });
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadStatus();
  }, [loadStatus]);

  const handleValidate = async () => {
    if (!commandInput.trim()) {
      toast.error('Please enter a command to validate');
      return;
    }

    setIsValidating(true);
    try {
      const result = await invoke<ValidationResult>('guardian_validate', {
        command: commandInput,
      });
      setValidationResult(result);
      setValidationHistory((prev) => [result, ...prev.slice(0, 9)]);

      if (result.allowed) {
        toast.success('Command is safe to execute');
      } else {
        toast.warning('Command blocked by Guardian');
      }
    } catch (error) {
      toast.error(`Validation failed: ${error}`);
    } finally {
      setIsValidating(false);
    }
  };

  const handleCheck = async () => {
    if (!contentInput.trim()) {
      toast.error('Please enter content to check');
      return;
    }

    setIsChecking(true);
    try {
      const result = await invoke<HallucinationCheck>('guardian_check', {
        content: contentInput,
      });
      setHallucinationResult(result);

      if (result.isHallucination) {
        toast.warning('Potential hallucination detected');
      } else {
        toast.success('Content appears factual');
      }
    } catch (error) {
      toast.error(`Check failed: ${error}`);
    } finally {
      setIsChecking(false);
    }
  };

  const getRiskColor = (score: number) => {
    if (score < 0.3) return 'text-green-400';
    if (score < 0.6) return 'text-yellow-400';
    return 'text-red-400';
  };

  const getRiskBg = (score: number) => {
    if (score < 0.3) return 'bg-green-500/10';
    if (score < 0.6) return 'bg-yellow-500/10';
    return 'bg-red-500/10';
  };

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-dark-100 flex items-center gap-3">
            <Shield className="w-8 h-8 text-sena-400" />
            Guardian Middleware
          </h1>
          <p className="text-dark-400 mt-1">
            Command validation and hallucination detection
          </p>
        </div>
        <button
          onClick={loadStatus}
          disabled={isLoading}
          className="btn-secondary"
          title="Refresh status"
        >
          <RefreshCw className={clsx('w-5 h-5', isLoading && 'animate-spin')} />
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-4 mb-8">
        <StatusCard
          icon={status?.enabled ? ShieldCheck : ShieldX}
          label="Status"
          value={status?.enabled ? 'Active' : 'Inactive'}
          color={status?.enabled ? 'green' : 'red'}
        />
        <StatusCard
          icon={Shield}
          label="Sandbox Level"
          value={status?.sandboxLevel || 'Full'}
          color="blue"
        />
        <StatusCard
          icon={Eye}
          label="Hallucination Mode"
          value={status?.hallucinationMode || 'All'}
          color="purple"
        />
        <StatusCard
          icon={Activity}
          label="Threshold"
          value={`${((status?.threshold || 0.7) * 100).toFixed(0)}%`}
          color="sena"
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        <div className="card">
          <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
            <Terminal className="w-5 h-5 text-sena-400" />
            Command Validation
          </h2>
          <p className="text-dark-400 text-sm mb-4">
            Validate commands before execution to detect potentially dangerous operations.
          </p>
          <div className="space-y-4">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-dark-500" />
              <input
                type="text"
                value={commandInput}
                onChange={(e) => setCommandInput(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleValidate()}
                placeholder="Enter command to validate (e.g., rm -rf /)"
                className="input pl-10 w-full font-mono"
              />
            </div>
            <button
              onClick={handleValidate}
              disabled={isValidating || !commandInput.trim()}
              className="btn-primary w-full"
            >
              {isValidating ? (
                <>
                  <RefreshCw className="w-4 h-4 animate-spin" />
                  Validating...
                </>
              ) : (
                <>
                  <ShieldCheck className="w-4 h-4" />
                  Validate Command
                </>
              )}
            </button>
          </div>

          {validationResult && (
            <div className={clsx(
              'mt-4 p-4 rounded-lg border',
              validationResult.allowed
                ? 'bg-green-500/10 border-green-500/30'
                : 'bg-red-500/10 border-red-500/30'
            )}>
              <div className="flex items-center gap-2 mb-2">
                {validationResult.allowed ? (
                  <CheckCircle2 className="w-5 h-5 text-green-400" />
                ) : (
                  <XCircle className="w-5 h-5 text-red-400" />
                )}
                <span className={validationResult.allowed ? 'text-green-400' : 'text-red-400'}>
                  {validationResult.allowed ? 'Command Allowed' : 'Command Blocked'}
                </span>
              </div>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-dark-400">Risk Score:</span>
                  <span className={getRiskColor(validationResult.riskScore)}>
                    {(validationResult.riskScore * 100).toFixed(0)}%
                  </span>
                </div>
                {validationResult.reason && (
                  <div>
                    <span className="text-dark-400">Reason:</span>
                    <p className="text-dark-200 mt-1">{validationResult.reason}</p>
                  </div>
                )}
                {validationResult.matchedPatterns.length > 0 && (
                  <div>
                    <span className="text-dark-400">Matched Patterns:</span>
                    <div className="flex flex-wrap gap-1 mt-1">
                      {validationResult.matchedPatterns.map((pattern, idx) => (
                        <span key={idx} className="badge badge-error text-xs">
                          {pattern}
                        </span>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>

        <div className="card">
          <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
            <Eye className="w-5 h-5 text-sena-400" />
            Hallucination Detection
          </h2>
          <p className="text-dark-400 text-sm mb-4">
            Analyze AI-generated content for potential hallucinations and factual inconsistencies.
          </p>
          <div className="space-y-4">
            <textarea
              value={contentInput}
              onChange={(e) => setContentInput(e.target.value)}
              placeholder="Enter AI-generated content to check for hallucinations..."
              className="input w-full h-32 resize-none"
            />
            <button
              onClick={handleCheck}
              disabled={isChecking || !contentInput.trim()}
              className="btn-primary w-full"
            >
              {isChecking ? (
                <>
                  <RefreshCw className="w-4 h-4 animate-spin" />
                  Checking...
                </>
              ) : (
                <>
                  <Eye className="w-4 h-4" />
                  Check Content
                </>
              )}
            </button>
          </div>

          {hallucinationResult && (
            <div className={clsx(
              'mt-4 p-4 rounded-lg border',
              hallucinationResult.isHallucination
                ? 'bg-yellow-500/10 border-yellow-500/30'
                : 'bg-green-500/10 border-green-500/30'
            )}>
              <div className="flex items-center gap-2 mb-2">
                {hallucinationResult.isHallucination ? (
                  <AlertTriangle className="w-5 h-5 text-yellow-400" />
                ) : (
                  <CheckCircle2 className="w-5 h-5 text-green-400" />
                )}
                <span className={hallucinationResult.isHallucination ? 'text-yellow-400' : 'text-green-400'}>
                  {hallucinationResult.isHallucination ? 'Potential Hallucination' : 'Content Verified'}
                </span>
              </div>
              <div className="grid grid-cols-2 gap-2 text-sm mt-3">
                <div>
                  <span className="text-dark-500 text-xs">Consistency</span>
                  <div className={clsx('font-medium', getRiskColor(1 - hallucinationResult.details.consistencyScore))}>
                    {(hallucinationResult.details.consistencyScore * 100).toFixed(0)}%
                  </div>
                </div>
                <div>
                  <span className="text-dark-500 text-xs">Entropy</span>
                  <div className={clsx('font-medium', getRiskColor(hallucinationResult.details.semanticEntropy))}>
                    {(hallucinationResult.details.semanticEntropy * 100).toFixed(0)}%
                  </div>
                </div>
                <div>
                  <span className="text-dark-500 text-xs">Fact Score</span>
                  <div className={clsx('font-medium', getRiskColor(1 - hallucinationResult.details.factValidationScore))}>
                    {(hallucinationResult.details.factValidationScore * 100).toFixed(0)}%
                  </div>
                </div>
                <div>
                  <span className="text-dark-500 text-xs">Status</span>
                  <div className="font-medium text-sena-400">
                    {hallucinationResult.harmonyStatus}
                  </div>
                </div>
              </div>
              {hallucinationResult.warnings.length > 0 && (
                <div className="mt-3 pt-3 border-t border-dark-700">
                  <span className="text-dark-400 text-xs">Warnings:</span>
                  <ul className="mt-1 space-y-1">
                    {hallucinationResult.warnings.map((warning, idx) => (
                      <li key={idx} className="text-xs text-yellow-400 flex items-center gap-1">
                        <AlertTriangle className="w-3 h-3" />
                        {warning}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      <div className="card">
        <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
          <Activity className="w-5 h-5 text-sena-400" />
          Validation History
        </h2>
        {validationHistory.length === 0 ? (
          <p className="text-dark-500 text-sm text-center py-8">
            No validation history yet. Validate some commands to see results here.
          </p>
        ) : (
          <div className="space-y-2">
            {validationHistory.map((result, idx) => (
              <div
                key={idx}
                className={clsx(
                  'p-3 rounded-lg flex items-center justify-between',
                  getRiskBg(result.riskScore)
                )}
              >
                <div className="flex items-center gap-3">
                  {result.allowed ? (
                    <CheckCircle2 className="w-4 h-4 text-green-400" />
                  ) : (
                    <ShieldAlert className="w-4 h-4 text-red-400" />
                  )}
                  <code className="text-sm text-dark-200 font-mono">
                    {result.command.length > 50
                      ? `${result.command.slice(0, 50)}...`
                      : result.command}
                  </code>
                </div>
                <div className="flex items-center gap-4">
                  <span className={clsx('text-sm', getRiskColor(result.riskScore))}>
                    {(result.riskScore * 100).toFixed(0)}% risk
                  </span>
                  <span className={clsx(
                    'badge',
                    result.allowed ? 'badge-success' : 'badge-error'
                  )}>
                    {result.allowed ? 'Allowed' : 'Blocked'}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
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
  color: 'green' | 'red' | 'blue' | 'purple' | 'sena';
}) {
  const colorMap = {
    green: 'text-green-400 bg-green-500/10',
    red: 'text-red-400 bg-red-500/10',
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
