import { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Activity,
  Bot,
  MessageSquare,
  Users,
  Cpu,
  Wifi,
  AlertCircle,
  CheckCircle2,
  RefreshCw,
  Terminal,
} from 'lucide-react';
import { useToast } from '../components/Toast';
import clsx from 'clsx';

interface HealthData {
  status: string;
  score: number;
  version: string;
  uptime: number;
  providers: { total: number; connected: number };
  sessions: { active: number; total: number };
}

interface CliSession {
  id: string;
  name: string;
  role: string;
  status: string;
}

interface HubMessage {
  id: string;
  from: string;
  to: string;
  content: string;
  timestamp: number;
}

export default function Dashboard() {
  const [health, setHealth] = useState<HealthData | null>(null);
  const [sessions, setSessions] = useState<CliSession[]>([]);
  const [messages, setMessages] = useState<HubMessage[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const toast = useToast();

  const loadData = useCallback(async () => {
    try {
      const [healthData, sessionsData, messagesData] = await Promise.all([
        invoke<HealthData>('get_health'),
        invoke<CliSession[]>('list_cli_sessions'),
        invoke<HubMessage[]>('get_all_messages'),
      ]);
      setHealth(healthData);
      setSessions(sessionsData);
      setMessages(messagesData);
    } catch (error) {
      console.error('Failed to load dashboard data:', error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const refreshData = useCallback(async () => {
    setIsLoading(true);
    await loadData();
    toast.success('Dashboard refreshed');
  }, [loadData, toast]);

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 5000);
    return () => clearInterval(interval);
  }, [loadData]);

  const connectedProviders = health?.providers.connected || 0;
  const totalProviders = health?.providers.total || 0;

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-dark-100">Dashboard</h1>
          <p className="text-dark-400 mt-1">
            Monitor your AI providers and collaboration sessions
          </p>
        </div>
        <button
          onClick={refreshData}
          disabled={isLoading}
          className="btn-secondary"
          title="Refresh dashboard"
        >
          <RefreshCw className={clsx('w-5 h-5', isLoading && 'animate-spin')} />
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <StatCard
          title="System Health"
          value={health ? `${health.score}%` : '--'}
          subtitle={health?.status || 'Loading...'}
          icon={Activity}
          color="green"
        />
        <StatCard
          title="Providers"
          value={`${connectedProviders}/${totalProviders}`}
          subtitle="Connected"
          icon={Bot}
          color="blue"
        />
        <StatCard
          title="CLI Sessions"
          value={sessions.length.toString()}
          subtitle={sessions.length > 0 ? sessions.map(s => s.name).join(', ') : 'None active'}
          icon={Users}
          color="purple"
        />
        <StatCard
          title="Hub Messages"
          value={messages.length.toString()}
          subtitle="Total communications"
          icon={MessageSquare}
          color="orange"
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="card">
          <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
            <Terminal className="w-5 h-5 text-sena-400" />
            Active CLI Sessions
          </h2>
          {sessions.length === 0 ? (
            <div className="text-center py-8 text-dark-400">
              <Terminal className="w-8 h-8 mx-auto mb-2 opacity-50" />
              <p>No CLI sessions active</p>
              <p className="text-xs mt-1">Start one with: sena session start --name "Name" --role general</p>
            </div>
          ) : (
            <div className="space-y-3">
              {sessions.map((session) => (
                <SessionRow key={session.id} session={session} />
              ))}
            </div>
          )}
        </div>

        <div className="card">
          <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
            <Cpu className="w-5 h-5 text-sena-400" />
            System Status
          </h2>
          <div className="space-y-4">
            <StatusRow
              label="Core Engine"
              status="online"
              detail={`Running v${health?.version || '...'}`}
            />
            <StatusRow
              label="Provider Router"
              status={connectedProviders > 0 ? 'online' : 'warning'}
              detail={`${totalProviders} providers configured`}
            />
            <StatusRow
              label="Hub Messaging"
              status="online"
              detail={`${messages.length} messages in queue`}
            />
            <StatusRow
              label="Session Manager"
              status={sessions.length > 0 ? 'online' : 'warning'}
              detail={sessions.length > 0 ? `${sessions.length} active` : 'No sessions'}
            />
          </div>
        </div>
      </div>

      <div className="mt-6 card">
        <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
          <Wifi className="w-5 h-5 text-sena-400" />
          Quick Actions
        </h2>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <QuickAction
            label="Smart Chat"
            description="Send to sessions"
            href="/chat"
          />
          <QuickAction
            label="Sessions"
            description="View CLI sessions"
            href="/sessions"
          />
          <QuickAction
            label="Providers"
            description="Configure AI"
            href="/providers"
          />
          <QuickAction
            label="Settings"
            description="Configure SENA"
            href="/settings"
          />
        </div>
      </div>

      {messages.length > 0 && (
        <div className="mt-6 card">
          <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
            <MessageSquare className="w-5 h-5 text-sena-400" />
            Recent Messages
          </h2>
          <div className="space-y-2 max-h-48 overflow-y-auto scrollbar-thin">
            {messages.slice(0, 5).map((msg) => (
              <div key={msg.id} className="flex items-center gap-3 p-2 rounded-lg bg-dark-800/50 text-sm">
                <span className="text-sena-400 font-medium">{msg.from}</span>
                <span className="text-dark-500">→</span>
                <span className="text-dark-300">{msg.to}</span>
                <span className="text-dark-400 flex-1 truncate">{msg.content}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function StatCard({
  title,
  value,
  subtitle,
  icon: Icon,
  color,
}: {
  title: string;
  value: string;
  subtitle: string;
  icon: React.ComponentType<{ className?: string }>;
  color: 'green' | 'blue' | 'purple' | 'orange';
}) {
  const colorClasses = {
    green: 'bg-green-500/10 text-green-400',
    blue: 'bg-blue-500/10 text-blue-400',
    purple: 'bg-purple-500/10 text-purple-400',
    orange: 'bg-sena-500/10 text-sena-400',
  };

  return (
    <div className="card">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm text-dark-400">{title}</p>
          <p className="text-2xl font-bold text-dark-100 mt-1">{value}</p>
          <p className="text-xs text-dark-500 mt-1 truncate max-w-[150px]">{subtitle}</p>
        </div>
        <div className={`p-3 rounded-xl ${colorClasses[color]}`}>
          <Icon className="w-6 h-6" />
        </div>
      </div>
    </div>
  );
}

const roleEmojis: Record<string, string> = {
  General: '🔧',
  Backend: '⚙️',
  Web: '🌐',
  Android: '🤖',
  iOS: '🍎',
  IoT: '📡',
};

function SessionRow({ session }: { session: CliSession }) {
  const emoji = roleEmojis[session.role] || '📦';

  return (
    <div className="flex items-center justify-between p-3 rounded-lg bg-dark-800/50">
      <div className="flex items-center gap-3">
        <span className="text-xl">{emoji}</span>
        <div>
          <p className="font-medium text-dark-100">{session.name}</p>
          <p className="text-xs text-dark-400">{session.role} • {session.id}</p>
        </div>
      </div>
      <span className={clsx(
        'badge',
        session.status === 'Active' ? 'badge-success' : 'badge-warning'
      )}>
        {session.status}
      </span>
    </div>
  );
}

function StatusRow({
  label,
  status,
  detail,
}: {
  label: string;
  status: 'online' | 'offline' | 'warning';
  detail: string;
}) {
  const Icon = status === 'online' ? CheckCircle2 : AlertCircle;
  const iconColor =
    status === 'online'
      ? 'text-green-400'
      : status === 'warning'
        ? 'text-yellow-400'
        : 'text-red-400';

  return (
    <div className="flex items-center justify-between">
      <div className="flex items-center gap-3">
        <Icon className={`w-5 h-5 ${iconColor}`} />
        <span className="text-dark-200">{label}</span>
      </div>
      <span className="text-sm text-dark-400">{detail}</span>
    </div>
  );
}

function QuickAction({
  label,
  description,
  href,
}: {
  label: string;
  description: string;
  href: string;
}) {
  return (
    <a
      href={href}
      className="p-4 rounded-lg bg-dark-800/50 hover:bg-dark-800 transition-colors border border-transparent hover:border-sena-500/30"
    >
      <p className="font-medium text-dark-100">{label}</p>
      <p className="text-xs text-dark-400 mt-1">{description}</p>
    </a>
  );
}
