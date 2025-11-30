import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Settings as SettingsIcon,
  Moon,
  Bell,
  Shield,
  Database,
  Info,
  ExternalLink,
  RefreshCw,
  Trash2,
  Globe,
  Key,
  Copy,
  Eye,
  EyeOff,
  Edit2,
  Save,
  X,
} from 'lucide-react';
import clsx from 'clsx';
import { useToast } from '../components/Toast';

interface HubIdentity {
  hub_id: string;
  name: string;
  hostname: string;
  port: number;
  short_id: string;
}

export default function Settings() {
  const [isDarkMode, setIsDarkMode] = useState(true);
  const [notifications, setNotifications] = useState(true);
  const [autoSave, setAutoSave] = useState(true);
  const [version, setVersion] = useState('...');
  const [isClearing, setIsClearing] = useState(false);
  const [hubIdentity, setHubIdentity] = useState<HubIdentity | null>(null);
  const [authPasskey, setAuthPasskey] = useState<string | null>(null);
  const [showPasskey, setShowPasskey] = useState(false);
  const [editingName, setEditingName] = useState(false);
  const [newHubName, setNewHubName] = useState('');
  const [isGeneratingKey, setIsGeneratingKey] = useState(false);
  const toast = useToast();

  const loadVersion = useCallback(async () => {
    try {
      const ver = await invoke<string>('get_version');
      setVersion(ver);
    } catch {
<<<<<<< Updated upstream
      setVersion('13.0.2');
=======
      setVersion('13.1.3');
>>>>>>> Stashed changes
    }
  }, []);

  const loadHubIdentity = useCallback(async () => {
    try {
      const identity = await invoke<HubIdentity>('get_hub_identity');
      setHubIdentity(identity);
      setNewHubName(identity.name);
    } catch (error) {
      console.error('Failed to load hub identity:', error);
    }
  }, []);

  const loadAuthPasskey = useCallback(async () => {
    try {
      const passkey = await invoke<string>('get_hub_passkey');
      setAuthPasskey(passkey);
    } catch {
      setAuthPasskey(null);
    }
  }, []);

  useEffect(() => {
    loadVersion();
    loadHubIdentity();
    loadAuthPasskey();
  }, [loadVersion, loadHubIdentity, loadAuthPasskey]);

  const handleClearHistory = async () => {
    setIsClearing(true);
    try {
      await invoke('clear_message_history');
      toast.success('History cleared successfully');
    } catch (error) {
      toast.error(`Failed to clear history: ${error}`);
    } finally {
      setIsClearing(false);
    }
  };

  const handleToggleDarkMode = (checked: boolean) => {
    setIsDarkMode(checked);
    toast.info(checked ? 'Dark mode enabled' : 'Light mode enabled');
  };

  const handleToggleNotifications = (checked: boolean) => {
    setNotifications(checked);
    toast.info(checked ? 'Notifications enabled' : 'Notifications disabled');
  };

  const handleToggleAutoSave = (checked: boolean) => {
    setAutoSave(checked);
    toast.info(checked ? 'Auto-save enabled' : 'Auto-save disabled');
  };

  const handleSaveHubName = async () => {
    if (!newHubName.trim()) {
      toast.error('Hub name cannot be empty');
      return;
    }
    try {
      await invoke('set_hub_name', { name: newHubName.trim() });
      setEditingName(false);
      await loadHubIdentity();
      toast.success('Hub name updated');
    } catch (error) {
      toast.error(`Failed to update hub name: ${error}`);
    }
  };

  const handleGeneratePasskey = async () => {
    setIsGeneratingKey(true);
    try {
      const passkey = await invoke<string>('generate_hub_passkey');
      setAuthPasskey(passkey);
      toast.success('New auth passkey generated');
    } catch (error) {
      toast.error(`Failed to generate passkey: ${error}`);
    } finally {
      setIsGeneratingKey(false);
    }
  };

  const handleCopyPasskey = async () => {
    if (authPasskey) {
      await navigator.clipboard.writeText(authPasskey);
      toast.success('Passkey copied to clipboard');
    }
  };

  const handleCopyHubId = async () => {
    if (hubIdentity) {
      await navigator.clipboard.writeText(hubIdentity.hub_id);
      toast.success('Hub ID copied to clipboard');
    }
  };

  return (
    <div className="p-8 max-w-4xl">
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-dark-100">Settings</h1>
        <p className="text-dark-400 mt-1">
          Configure SENA preferences and behavior
        </p>
      </div>

      <div className="space-y-6">
        <SettingsSection title="Hub Identity & Credentials" icon={Globe}>
          {hubIdentity && (
            <div className="space-y-4">
              <SettingsRow
                label="Hub Name"
                description="Display name for this hub on the network"
              >
                {editingName ? (
                  <div className="flex items-center gap-2">
                    <input
                      type="text"
                      value={newHubName}
                      onChange={(e) => setNewHubName(e.target.value)}
                      className="input w-48"
                      placeholder="Enter hub name"
                      onKeyDown={(e) => e.key === 'Enter' && handleSaveHubName()}
                    />
                    <button onClick={handleSaveHubName} className="btn-primary p-2">
                      <Save className="w-4 h-4" />
                    </button>
                    <button
                      onClick={() => {
                        setEditingName(false);
                        setNewHubName(hubIdentity.name);
                      }}
                      className="btn-secondary p-2"
                    >
                      <X className="w-4 h-4" />
                    </button>
                  </div>
                ) : (
                  <div className="flex items-center gap-2">
                    <span className="text-dark-100 font-medium">{hubIdentity.name}</span>
                    <button
                      onClick={() => setEditingName(true)}
                      className="text-dark-500 hover:text-dark-300 transition-colors"
                    >
                      <Edit2 className="w-4 h-4" />
                    </button>
                  </div>
                )}
              </SettingsRow>

              <SettingsRow
                label="Hub ID"
                description="Unique identifier for this hub (share with others to connect)"
              >
                <div className="flex items-center gap-2">
                  <code className="text-xs text-sena-400 font-mono bg-dark-800 px-2 py-1 rounded">
                    {hubIdentity.short_id}
                  </code>
                  <button
                    onClick={handleCopyHubId}
                    className="text-dark-500 hover:text-sena-400 transition-colors"
                    title="Copy full Hub ID"
                  >
                    <Copy className="w-4 h-4" />
                  </button>
                </div>
              </SettingsRow>

              <SettingsRow
                label="Network Port"
                description="Port used for cross-hub communication"
              >
                <span className="text-dark-100 font-mono">{hubIdentity.port}</span>
              </SettingsRow>

              <SettingsRow
                label="Hostname"
                description="System hostname"
              >
                <span className="text-dark-300 text-sm">{hubIdentity.hostname}</span>
              </SettingsRow>
            </div>
          )}
        </SettingsSection>

        <SettingsSection title="Authentication Passkey" icon={Key}>
          <div className="space-y-4">
            <div className="p-4 rounded-lg bg-dark-800/50 border border-dark-700">
              <p className="text-sm text-dark-400 mb-3">
                Share this passkey with other hubs to allow them to connect to you.
                Generate a new passkey if you want to revoke access from previously shared keys.
              </p>

              {authPasskey ? (
                <div className="space-y-3">
                  <div className="flex items-center gap-2">
                    <div className="flex-1 bg-dark-900 rounded-lg px-3 py-2 font-mono text-sm">
                      {showPasskey ? (
                        <span className="text-sena-400">{authPasskey}</span>
                      ) : (
                        <span className="text-dark-500">••••••••••••••••••••••••</span>
                      )}
                    </div>
                    <button
                      onClick={() => setShowPasskey(!showPasskey)}
                      className="btn-secondary p-2"
                      title={showPasskey ? 'Hide passkey' : 'Show passkey'}
                    >
                      {showPasskey ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                    </button>
                    <button
                      onClick={handleCopyPasskey}
                      className="btn-secondary p-2"
                      title="Copy passkey"
                    >
                      <Copy className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              ) : (
                <p className="text-dark-500 text-sm">No passkey generated yet.</p>
              )}
            </div>

            <SettingsRow
              label="Generate New Passkey"
              description="Create a new authentication passkey (invalidates old one)"
            >
              <button
                onClick={handleGeneratePasskey}
                disabled={isGeneratingKey}
                className="btn-primary text-sm flex items-center gap-2"
              >
                {isGeneratingKey ? (
                  <RefreshCw className="w-4 h-4 animate-spin" />
                ) : (
                  <Key className="w-4 h-4" />
                )}
                {isGeneratingKey ? 'Generating...' : 'Generate Passkey'}
              </button>
            </SettingsRow>
          </div>
        </SettingsSection>

        <SettingsSection title="Appearance" icon={Moon}>
          <SettingsRow
            label="Dark Mode"
            description="Use dark theme for the interface"
          >
            <Toggle checked={isDarkMode} onChange={handleToggleDarkMode} />
          </SettingsRow>
        </SettingsSection>

        <SettingsSection title="Notifications" icon={Bell}>
          <SettingsRow
            label="Enable Notifications"
            description="Show notifications for AI responses and session updates"
          >
            <Toggle checked={notifications} onChange={handleToggleNotifications} />
          </SettingsRow>
        </SettingsSection>

        <SettingsSection title="Data" icon={Database}>
          <SettingsRow
            label="Auto-save Sessions"
            description="Automatically save session history"
          >
            <Toggle checked={autoSave} onChange={handleToggleAutoSave} />
          </SettingsRow>
          <SettingsRow
            label="Clear History"
            description="Remove all chat and session history"
          >
            <button
              onClick={handleClearHistory}
              disabled={isClearing}
              className="btn-secondary text-sm flex items-center gap-2"
            >
              {isClearing ? (
                <RefreshCw className="w-4 h-4 animate-spin" />
              ) : (
                <Trash2 className="w-4 h-4" />
              )}
              {isClearing ? 'Clearing...' : 'Clear All'}
            </button>
          </SettingsRow>
        </SettingsSection>

        <SettingsSection title="Security" icon={Shield}>
          <SettingsRow
            label="API Keys"
            description="API keys are stored as environment variables"
          >
            <span className="badge badge-success">Secure</span>
          </SettingsRow>
          <SettingsRow
            label="Data Location"
            description="Session data stored in ~/.sena/"
          >
            <code className="text-xs text-sena-400 font-mono">~/.sena/</code>
          </SettingsRow>
        </SettingsSection>

        <SettingsSection title="About" icon={Info}>
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-dark-300">Version</span>
              <span className="text-dark-100 font-mono">{version}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-dark-300">License</span>
              <span className="text-dark-100">MIT</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-dark-300">Author</span>
              <span className="text-dark-100">Sena1996</span>
            </div>
            <div className="pt-4 border-t border-dark-700">
              <a
                href="https://github.com/Sena1996/Sena1996-AI"
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-2 text-sena-400 hover:text-sena-300 transition-colors"
              >
                <ExternalLink className="w-4 h-4" />
                View on GitHub
              </a>
            </div>
          </div>
        </SettingsSection>

        <div className="card bg-gradient-to-br from-sena-500/10 to-sena-600/5 border-sena-500/20">
          <div className="flex items-center gap-4">
            <div className="w-12 h-12 rounded-xl bg-sena-500/20 flex items-center justify-center">
              <SettingsIcon className="w-6 h-6 text-sena-400" />
            </div>
            <div>
              <h3 className="font-semibold text-dark-100">
                SENA<span className="text-[0.6em] text-dark-300">1996</span> AI Collaboration Hub
              </h3>
              <p className="text-sm text-dark-400">
                Where Brilliant AIs Talk to Each Other
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function SettingsSection({
  title,
  icon: Icon,
  children,
}: {
  title: string;
  icon: React.ComponentType<{ className?: string }>;
  children: React.ReactNode;
}) {
  return (
    <div className="card">
      <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
        <Icon className="w-5 h-5 text-sena-400" />
        {title}
      </h2>
      <div className="space-y-4">{children}</div>
    </div>
  );
}

function SettingsRow({
  label,
  description,
  children,
}: {
  label: string;
  description: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between">
      <div>
        <p className="text-dark-200">{label}</p>
        <p className="text-sm text-dark-500">{description}</p>
      </div>
      {children}
    </div>
  );
}

function Toggle({
  checked,
  onChange,
}: {
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <button
      onClick={() => onChange(!checked)}
      className={clsx(
        'relative w-12 h-6 rounded-full transition-colors',
        checked ? 'bg-sena-500' : 'bg-dark-600'
      )}
    >
      <div
        className={clsx(
          'absolute top-1 w-4 h-4 rounded-full bg-white transition-transform',
          checked ? 'translate-x-7' : 'translate-x-1'
        )}
      />
    </button>
  );
}
