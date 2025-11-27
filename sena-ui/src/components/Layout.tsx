import { NavLink } from 'react-router-dom';
import {
  LayoutDashboard,
  Bot,
  MessageSquare,
  Users,
  Settings,
  Zap,
} from 'lucide-react';
import clsx from 'clsx';

interface LayoutProps {
  children: React.ReactNode;
}

const navItems = [
  { path: '/', icon: LayoutDashboard, label: 'Dashboard' },
  { path: '/providers', icon: Bot, label: 'Providers' },
  { path: '/chat', icon: MessageSquare, label: 'Chat' },
  { path: '/sessions', icon: Users, label: 'Sessions' },
  { path: '/settings', icon: Settings, label: 'Settings' },
];

export default function Layout({ children }: LayoutProps) {
  return (
    <div className="flex h-screen bg-dark-950">
      <aside className="w-64 border-r border-dark-800 bg-dark-900 flex flex-col">
        <div className="p-6 border-b border-dark-800">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-sena-400 to-sena-600 flex items-center justify-center">
              <Zap className="w-6 h-6 text-dark-950" />
            </div>
            <div>
              <h1 className="font-bold text-lg text-dark-100">SENA</h1>
              <p className="text-xs text-dark-400">AI Collaboration Hub</p>
            </div>
          </div>
        </div>

        <nav className="flex-1 p-4 space-y-1">
          {navItems.map((item) => (
            <NavLink
              key={item.path}
              to={item.path}
              className={({ isActive }) =>
                clsx(
                  'flex items-center gap-3 px-4 py-3 rounded-lg transition-colors',
                  isActive
                    ? 'bg-sena-500/10 text-sena-400'
                    : 'text-dark-400 hover:text-dark-100 hover:bg-dark-800'
                )
              }
            >
              <item.icon className="w-5 h-5" />
              <span className="font-medium">{item.label}</span>
            </NavLink>
          ))}
        </nav>

        <div className="p-4 border-t border-dark-800">
          <div className="card bg-dark-800/50 p-4">
            <div className="flex items-center gap-2 text-xs text-dark-400">
              <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
              <span>System Online</span>
            </div>
            <p className="text-xs text-dark-500 mt-1">v11.0.2</p>
          </div>
        </div>
      </aside>

      <main className="flex-1 overflow-auto scrollbar-thin">{children}</main>
    </div>
  );
}
