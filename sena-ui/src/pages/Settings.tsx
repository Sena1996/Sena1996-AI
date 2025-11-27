import { useState } from 'react';
import {
  Settings as SettingsIcon,
  Moon,
  Sun,
  Bell,
  Shield,
  Database,
  Info,
  ExternalLink,
} from 'lucide-react';
import clsx from 'clsx';

export default function Settings() {
  const [isDarkMode, setIsDarkMode] = useState(true);
  const [notifications, setNotifications] = useState(true);
  const [autoSave, setAutoSave] = useState(true);

  return (
    <div className="p-8 max-w-4xl">
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-dark-100">Settings</h1>
        <p className="text-dark-400 mt-1">
          Configure SENA preferences and behavior
        </p>
      </div>

      <div className="space-y-6">
        <SettingsSection title="Appearance" icon={Moon}>
          <SettingsRow
            label="Dark Mode"
            description="Use dark theme for the interface"
          >
            <Toggle checked={isDarkMode} onChange={setIsDarkMode} />
          </SettingsRow>
        </SettingsSection>

        <SettingsSection title="Notifications" icon={Bell}>
          <SettingsRow
            label="Enable Notifications"
            description="Show notifications for AI responses and session updates"
          >
            <Toggle checked={notifications} onChange={setNotifications} />
          </SettingsRow>
        </SettingsSection>

        <SettingsSection title="Data" icon={Database}>
          <SettingsRow
            label="Auto-save Sessions"
            description="Automatically save session history"
          >
            <Toggle checked={autoSave} onChange={setAutoSave} />
          </SettingsRow>
          <SettingsRow
            label="Clear History"
            description="Remove all chat and session history"
          >
            <button className="btn-secondary text-sm">Clear All</button>
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
              <span className="text-dark-100 font-mono">11.0.2</span>
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
                SENA AI Collaboration Hub
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
