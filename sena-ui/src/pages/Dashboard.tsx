import { useEffect, useState } from 'react';
import {
  Activity,
  Bot,
  MessageSquare,
  Users,
  Cpu,
  Wifi,
  AlertCircle,
  CheckCircle2,
} from 'lucide-react';
import { useAppStore } from '../store';
import type { SystemHealth, Provider } from '../types';

const mockHealth: SystemHealth = {
  status: 'healthy',
  score: 95,
  version: '11.0.2',
  uptime: 3600,
  providers: { total: 5, connected: 3 },
  sessions: { active: 0, total: 0 },
};

const mockProviders: Provider[] = [
  {
    id: 'claude',
    name: 'Claude',
    status: 'connected',
    defaultModel: 'claude-sonnet-4-5',
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
    name: 'Gemini',
    status: 'disconnected',
    defaultModel: 'gemini-2.5-flash',
    hasApiKey: false,
    capabilities: { streaming: true, tools: true, vision: true },
  },
  {
    id: 'ollama',
    name: 'Ollama',
    status: 'connected',
    defaultModel: 'llama3.2',
    hasApiKey: true,
    capabilities: { streaming: true, tools: true, vision: false },
  },
  {
    id: 'mistral',
    name: 'Mistral',
    status: 'disconnected',
    defaultModel: 'mistral-large-latest',
    hasApiKey: false,
    capabilities: { streaming: true, tools: true, vision: true },
  },
];

export default function Dashboard() {
  const { setProviders, setHealth } = useAppStore();
  const [health] = useState<SystemHealth>(mockHealth);
  const [providers] = useState<Provider[]>(mockProviders);

  useEffect(() => {
    setProviders(mockProviders);
    setHealth(mockHealth);
  }, [setProviders, setHealth]);

  const connectedProviders = providers.filter((p) => p.status === 'connected');

  return (
    <div className="p-8">
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-dark-100">Dashboard</h1>
        <p className="text-dark-400 mt-1">
          Monitor your AI providers and collaboration sessions
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <StatCard
          title="System Health"
          value={`${health.score}%`}
          subtitle={health.status}
          icon={Activity}
          color="green"
        />
        <StatCard
          title="Providers"
          value={`${connectedProviders.length}/${providers.length}`}
          subtitle="Connected"
          icon={Bot}
          color="blue"
        />
        <StatCard
          title="Active Sessions"
          value={health.sessions.active.toString()}
          subtitle="Collaborations"
          icon={Users}
          color="purple"
        />
        <StatCard
          title="Messages Today"
          value="0"
          subtitle="AI Communications"
          icon={MessageSquare}
          color="orange"
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="card">
          <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
            <Bot className="w-5 h-5 text-sena-400" />
            AI Providers
          </h2>
          <div className="space-y-3">
            {providers.map((provider) => (
              <ProviderRow key={provider.id} provider={provider} />
            ))}
          </div>
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
              detail="Running v11.0.2"
            />
            <StatusRow
              label="Provider Router"
              status="online"
              detail="5 providers configured"
            />
            <StatusRow
              label="Collaboration Hub"
              status="online"
              detail="Ready for sessions"
            />
            <StatusRow
              label="Network Discovery"
              status="online"
              detail="mDNS active"
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
            label="New Chat"
            description="Start AI conversation"
            href="/chat"
          />
          <QuickAction
            label="New Session"
            description="Multi-AI collaboration"
            href="/sessions"
          />
          <QuickAction
            label="Test Providers"
            description="Check connectivity"
            href="/providers"
          />
          <QuickAction
            label="Settings"
            description="Configure SENA"
            href="/settings"
          />
        </div>
      </div>
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
          <p className="text-xs text-dark-500 mt-1">{subtitle}</p>
        </div>
        <div className={`p-3 rounded-xl ${colorClasses[color]}`}>
          <Icon className="w-6 h-6" />
        </div>
      </div>
    </div>
  );
}

function ProviderRow({ provider }: { provider: Provider }) {
  const statusColors = {
    connected: 'bg-green-500',
    disconnected: 'bg-dark-600',
    rate_limited: 'bg-yellow-500',
    error: 'bg-red-500',
  };

  return (
    <div className="flex items-center justify-between p-3 rounded-lg bg-dark-800/50">
      <div className="flex items-center gap-3">
        <div className={`w-2 h-2 rounded-full ${statusColors[provider.status]}`} />
        <div>
          <p className="font-medium text-dark-100">{provider.name}</p>
          <p className="text-xs text-dark-400">{provider.defaultModel}</p>
        </div>
      </div>
      <div className="flex items-center gap-2">
        {provider.capabilities.streaming && (
          <span className="badge badge-info">Stream</span>
        )}
        {provider.capabilities.tools && (
          <span className="badge badge-info">Tools</span>
        )}
        {provider.capabilities.vision && (
          <span className="badge badge-info">Vision</span>
        )}
      </div>
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
