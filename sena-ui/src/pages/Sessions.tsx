import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Users,
  RefreshCw,
  Terminal,
  FolderOpen,
  Clock,
  Activity,
} from 'lucide-react';
import clsx from 'clsx';
import { useToast } from '../components/Toast';

interface CliSession {
  id: string;
  name: string;
  role: string;
  status: string;
  working_on: string | null;
  working_directory: string;
  joined_at: number;
  last_heartbeat: number;
}

const roleColors: Record<string, string> = {
  General: 'bg-blue-500/20 text-blue-400',
  Backend: 'bg-purple-500/20 text-purple-400',
  Web: 'bg-green-500/20 text-green-400',
  Android: 'bg-emerald-500/20 text-emerald-400',
  iOS: 'bg-pink-500/20 text-pink-400',
  IoT: 'bg-orange-500/20 text-orange-400',
};

const roleEmojis: Record<string, string> = {
  General: '🔧',
  Backend: '⚙️',
  Web: '🌐',
  Android: '🤖',
  iOS: '🍎',
  IoT: '📡',
};

function formatTime(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  return date.toLocaleTimeString();
}

function getIdleTime(lastHeartbeat: number): string {
  const now = Math.floor(Date.now() / 1000);
  const diff = now - lastHeartbeat;
  if (diff < 60) return 'Just now';
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  return `${Math.floor(diff / 3600)}h ago`;
}

export default function Sessions() {
  const [cliSessions, setCliSessions] = useState<CliSession[]>([]);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const toast = useToast();

  const loadCliSessions = useCallback(async () => {
    try {
      const sessions = await invoke<CliSession[]>('list_cli_sessions');
      setCliSessions(sessions);
      return sessions.length;
    } catch (error) {
      console.error('Failed to load CLI sessions:', error);
      toast.error(`Failed to load sessions: ${error}`);
      return 0;
    }
  }, [toast]);

  const refreshSessions = useCallback(async () => {
    setIsRefreshing(true);
    const count = await loadCliSessions();
    setIsRefreshing(false);
    if (count === 0) {
      toast.info('No CLI sessions found. Start one with: sena session start --name "Name" --role general');
    } else {
      toast.success(`Found ${count} CLI session(s)`);
    }
  }, [loadCliSessions, toast]);

  useEffect(() => {
    loadCliSessions();
    const interval = setInterval(loadCliSessions, 5000);
    return () => clearInterval(interval);
  }, [loadCliSessions]);

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-dark-100">
            Active Sessions
          </h1>
          <p className="text-dark-400 mt-1">
            View and manage CLI sessions running across terminals
          </p>
        </div>
        <div className="flex items-center gap-3">
          <button
            onClick={refreshSessions}
            disabled={isRefreshing}
            className="btn-secondary"
            title="Refresh sessions"
          >
            <RefreshCw className={clsx('w-5 h-5', isRefreshing && 'animate-spin')} />
          </button>
        </div>
      </div>

      {cliSessions.length === 0 ? (
        <div className="card flex flex-col items-center justify-center py-16 text-center">
          <div className="w-16 h-16 rounded-2xl bg-sena-500/10 flex items-center justify-center mb-4">
            <Users className="w-8 h-8 text-sena-400" />
          </div>
          <h2 className="text-lg font-semibold text-dark-200">
            No Active Sessions
          </h2>
          <p className="text-dark-400 mt-1 max-w-md">
            Start a CLI session in your terminal to see it here.
          </p>
          <div className="mt-6 p-4 rounded-lg bg-dark-800 text-left">
            <p className="text-xs text-dark-400 mb-2">Start a session with:</p>
            <code className="text-sm text-sena-400">
              sena session start --name "My Session" --role general
            </code>
          </div>
        </div>
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {cliSessions.map((session) => (
            <CliSessionCard key={session.id} session={session} />
          ))}
        </div>
      )}
    </div>
  );
}

function CliSessionCard({ session }: { session: CliSession }) {
  const roleColor = roleColors[session.role] || 'bg-gray-500/20 text-gray-400';
  const roleEmoji = roleEmojis[session.role] || '📦';

  return (
    <div className="card">
      <div className="flex items-start justify-between mb-4">
        <div>
          <div className="flex items-center gap-2">
            <span className="text-xl">{roleEmoji}</span>
            <h3 className="font-semibold text-dark-100">{session.name}</h3>
            <span className={clsx('badge', roleColor)}>{session.role}</span>
          </div>
          <p className="text-xs text-dark-500 mt-1">ID: {session.id}</p>
        </div>
        <div className="flex items-center gap-2">
          <span className={clsx(
            'badge',
            session.status === 'Active' ? 'badge-success' : 'badge-warning'
          )}>
            {session.status}
          </span>
        </div>
      </div>

      <div className="space-y-3 mb-4">
        <div className="flex items-center gap-2 text-sm">
          <FolderOpen className="w-4 h-4 text-dark-500" />
          <span className="text-dark-300 truncate" title={session.working_directory}>
            {session.working_directory}
          </span>
        </div>

        {session.working_on && (
          <div className="flex items-center gap-2 text-sm">
            <Activity className="w-4 h-4 text-sena-500" />
            <span className="text-sena-400">{session.working_on}</span>
          </div>
        )}
      </div>

      <div className="flex items-center justify-between pt-4 border-t border-dark-700">
        <div className="flex items-center gap-4 text-xs text-dark-400">
          <span className="flex items-center gap-1">
            <Terminal className="w-4 h-4" />
            CLI Session
          </span>
          <span className="flex items-center gap-1">
            <Clock className="w-4 h-4" />
            Started {formatTime(session.joined_at)}
          </span>
        </div>
        <span className="text-xs text-dark-500">
          {getIdleTime(session.last_heartbeat)}
        </span>
      </div>
    </div>
  );
}
