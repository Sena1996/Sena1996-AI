import { useState, useRef, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Send,
  Radio,
  MessageSquare,
  ArrowRight,
  RefreshCw,
  Terminal,
  Globe,
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
}

interface FederatedSession {
  hub_id: string;
  hub_name: string;
  session_id: string;
  session_name: string;
  role: string;
  status: string;
  working_on: string | null;
  working_directory: string;
  is_local: boolean;
}

interface HubMessage {
  id: string;
  from: string;
  to: string;
  content: string;
  message_type: string;
  timestamp: number;
  read: boolean;
}

interface ChatMessage {
  id: string;
  type: 'outgoing' | 'system' | 'info';
  from: string;
  to: string;
  content: string;
  timestamp: Date;
}

const roleEmojis: Record<string, string> = {
  General: '🔧',
  Backend: '⚙️',
  Web: '🌐',
  Android: '🤖',
  iOS: '🍎',
  IoT: '📡',
  hub: '🦁',
};

export default function Chat() {
  const [sessions, setSessions] = useState<CliSession[]>([]);
  const [federatedSessions, setFederatedSessions] = useState<FederatedSession[]>([]);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [selectedTarget, setSelectedTarget] = useState<string>('broadcast');
  const [isLoading, setIsLoading] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const messagesContainerRef = useRef<HTMLDivElement>(null);
  const toast = useToast();

  const scrollToBottom = useCallback(() => {
    if (messagesContainerRef.current) {
      const container = messagesContainerRef.current;
      container.scrollTop = container.scrollHeight;
    }
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth', block: 'end' });
  }, []);

  useEffect(() => {
    const timeoutId = setTimeout(scrollToBottom, 100);
    return () => clearTimeout(timeoutId);
  }, [messages, scrollToBottom]);

  const loadSessions = useCallback(async () => {
    try {
      const [localData, federatedData] = await Promise.all([
        invoke<CliSession[]>('list_cli_sessions'),
        invoke<FederatedSession[]>('get_federated_sessions').catch(() => []),
      ]);
      setSessions(localData);
      setFederatedSessions(federatedData);
    } catch (error) {
      console.error('Failed to load sessions:', error);
    }
  }, []);

  const loadMessages = useCallback(async () => {
    try {
      const hubMessages = await invoke<HubMessage[]>('get_all_messages');
      const chatMessages: ChatMessage[] = hubMessages.map((m) => ({
        id: m.id,
        type: m.from === 'hub' ? 'outgoing' : 'info',
        from: m.from,
        to: m.to,
        content: m.content,
        timestamp: new Date(m.timestamp * 1000),
      }));
      setMessages(chatMessages);
    } catch (error) {
      console.error('Failed to load messages:', error);
    }
  }, []);

  useEffect(() => {
    loadSessions();
    loadMessages();
    const interval = setInterval(() => {
      loadSessions();
      loadMessages();
    }, 3000);
    return () => clearInterval(interval);
  }, [loadSessions, loadMessages]);

  const parseSmartInput = (text: string): { target: string; message: string } | null => {
    const crossHubTellMatch = text.match(/^tell\s+([\w-]+):([\w-]+)\s+(.+)$/i);
    if (crossHubTellMatch) {
      return { target: `${crossHubTellMatch[1]}:${crossHubTellMatch[2]}`, message: crossHubTellMatch[3] };
    }

    const tellMatch = text.match(/^tell\s+([\w-]+)\s+(.+)$/i);
    if (tellMatch) {
      return { target: tellMatch[1], message: tellMatch[2] };
    }

    const crossHubAtMatch = text.match(/^@([\w-]+):([\w-]+)\s+(.+)$/);
    if (crossHubAtMatch) {
      return { target: `${crossHubAtMatch[1]}:${crossHubAtMatch[2]}`, message: crossHubAtMatch[3] };
    }

    const atMatch = text.match(/^@([\w-]+)\s+(.+)$/);
    if (atMatch) {
      return { target: atMatch[1], message: atMatch[2] };
    }

    const crossHubColonMatch = text.match(/^([\w-]+):([\w-]+):\s*(.+)$/);
    if (crossHubColonMatch) {
      return { target: `${crossHubColonMatch[1]}:${crossHubColonMatch[2]}`, message: crossHubColonMatch[3] };
    }

    const toMatch = text.match(/^([\w-]+):\s*(.+)$/);
    if (toMatch) {
      const possibleTarget = toMatch[1].toLowerCase();
      const session = sessions.find(
        (s) => s.name.toLowerCase() === possibleTarget || s.role.toLowerCase() === possibleTarget
      );
      if (session) {
        return { target: session.name, message: toMatch[2] };
      }
    }

    return null;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || isLoading) return;

    const trimmedInput = input.trim();
    setIsLoading(true);

    try {
      const parsed = parseSmartInput(trimmedInput);

      if (parsed) {
        const result = await invoke<{ success: boolean; target: string }>('send_message_to_session', {
          targetSession: parsed.target,
          message: parsed.message,
        });

        if (result.success) {
          toast.success(`Message sent to ${result.target}`);
          setInput('');
          await loadMessages();
        }
      } else if (selectedTarget === 'broadcast') {
        const result = await invoke<{ success: boolean }>('broadcast_message', {
          message: trimmedInput,
        });

        if (result.success) {
          toast.success('Broadcast sent to all sessions');
          setInput('');
          await loadMessages();
        }
      } else {
        const result = await invoke<{ success: boolean; target: string }>('send_message_to_session', {
          targetSession: selectedTarget,
          message: trimmedInput,
        });

        if (result.success) {
          toast.success(`Message sent to ${result.target}`);
          setInput('');
          await loadMessages();
        }
      }
    } catch (error) {
      toast.error(`Failed to send: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const getSessionEmoji = (name: string): string => {
    const session = sessions.find((s) => s.name === name || s.id === name);
    if (session) {
      return roleEmojis[session.role] || '📦';
    }
    const fedSession = federatedSessions.find(
      (s) => s.session_name === name || `${s.hub_name}:${s.session_name}` === name
    );
    if (fedSession) {
      return roleEmojis[fedSession.role] || '📦';
    }
    return roleEmojis[name] || '📦';
  };

  const remoteSessions = federatedSessions.filter((s) => !s.is_local);

  return (
    <div className="flex flex-col h-full">
      <div className="border-b border-dark-800 p-4">
        <div className="flex items-center justify-between mb-4">
          <div>
            <h1 className="text-xl font-bold text-dark-100">Smart Hub Chat</h1>
            <p className="text-sm text-dark-400">
              Send messages to sessions • Use @name or "tell name message"
            </p>
          </div>
          <button
            onClick={() => { loadSessions(); loadMessages(); }}
            className="btn-ghost p-2"
            title="Refresh"
          >
            <RefreshCw className="w-5 h-5" />
          </button>
        </div>

        <div className="flex items-center gap-2 overflow-x-auto pb-2">
          <button
            onClick={() => setSelectedTarget('broadcast')}
            className={clsx(
              'flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium whitespace-nowrap transition-colors',
              selectedTarget === 'broadcast'
                ? 'bg-sena-500 text-dark-950'
                : 'bg-dark-800 text-dark-300 hover:bg-dark-700'
            )}
          >
            <Radio className="w-4 h-4" />
            Broadcast All
          </button>

          {sessions.map((session) => (
            <button
              key={session.id}
              onClick={() => setSelectedTarget(session.name)}
              className={clsx(
                'flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium whitespace-nowrap transition-colors',
                selectedTarget === session.name
                  ? 'bg-sena-500 text-dark-950'
                  : 'bg-dark-800 text-dark-300 hover:bg-dark-700'
              )}
            >
              <span>{roleEmojis[session.role] || '📦'}</span>
              {session.name}
            </button>
          ))}

          {remoteSessions.length > 0 && (
            <>
              <div className="h-6 w-px bg-dark-700 mx-1" />
              <span className="text-xs text-dark-500 flex items-center gap-1">
                <Globe className="w-3 h-3" />
                Remote
              </span>
            </>
          )}

          {remoteSessions.map((session) => {
            const fullAddress = `${session.hub_name}:${session.session_name}`;
            return (
              <button
                key={`${session.hub_id}-${session.session_id}`}
                onClick={() => setSelectedTarget(fullAddress)}
                className={clsx(
                  'flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium whitespace-nowrap transition-colors border',
                  selectedTarget === fullAddress
                    ? 'bg-blue-500 text-white border-blue-400'
                    : 'bg-dark-800 text-dark-300 hover:bg-dark-700 border-blue-500/30'
                )}
              >
                <span>{roleEmojis[session.role] || '📦'}</span>
                <span className="text-xs text-dark-400">{session.hub_name}:</span>
                {session.session_name}
              </button>
            );
          })}
        </div>
      </div>

      <div
        ref={messagesContainerRef}
        className="flex-1 overflow-y-auto p-4 space-y-4 scrollbar-thin"
        style={{ maxHeight: 'calc(100vh - 280px)' }}
      >
        {messages.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <div className="w-16 h-16 rounded-2xl bg-sena-500/10 flex items-center justify-center mb-4">
              <MessageSquare className="w-8 h-8 text-sena-400" />
            </div>
            <h2 className="text-lg font-semibold text-dark-200">
              Start communicating
            </h2>
            <p className="text-dark-400 mt-1 max-w-md">
              Send messages to your running sessions. Use smart commands:
            </p>
            <div className="mt-4 space-y-2 text-left">
              <code className="block text-sm text-sena-400 bg-dark-800 px-3 py-2 rounded">
                tell Android implement user auth
              </code>
              <code className="block text-sm text-sena-400 bg-dark-800 px-3 py-2 rounded">
                @WebBackend add API endpoint for /users
              </code>
              <code className="block text-sm text-sena-400 bg-dark-800 px-3 py-2 rounded">
                Android: run unit tests
              </code>
              <p className="text-xs text-dark-500 mt-3">Cross-hub messaging:</p>
              <code className="block text-sm text-blue-400 bg-dark-800 px-3 py-2 rounded">
                @RemoteHub:Backend sync database schema
              </code>
            </div>
          </div>
        ) : (
          messages.map((message) => (
            <MessageBubble
              key={message.id}
              message={message}
              sessions={sessions}
              getEmoji={getSessionEmoji}
            />
          ))
        )}
        <div ref={messagesEndRef} />
      </div>

      <form onSubmit={handleSubmit} className="p-4 border-t border-dark-800">
        <div className="flex items-center gap-2 mb-2">
          <Terminal className="w-4 h-4 text-dark-500" />
          <span className="text-xs text-dark-500">
            Target: {selectedTarget === 'broadcast' ? 'All Sessions' : selectedTarget}
          </span>
          {selectedTarget !== 'broadcast' && (
            <>
              <ArrowRight className="w-3 h-3 text-dark-600" />
              <span className="text-xs text-sena-400">
                {sessions.find((s) => s.name === selectedTarget)?.role || 'Session'}
              </span>
            </>
          )}
        </div>
        <div className="flex gap-3">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder={
              selectedTarget === 'broadcast'
                ? 'Type message to broadcast, or use @name / tell name...'
                : `Message to ${selectedTarget}...`
            }
            className="input flex-1"
            disabled={isLoading}
          />
          <button
            type="submit"
            disabled={!input.trim() || isLoading}
            className="btn-primary px-6"
          >
            <Send className="w-5 h-5" />
          </button>
        </div>
      </form>
    </div>
  );
}

function MessageBubble({
  message,
  sessions,
  getEmoji,
}: {
  message: ChatMessage;
  sessions: CliSession[];
  getEmoji: (name: string) => string;
}) {
  const isOutgoing = message.from === 'hub';
  const targetSession = sessions.find((s) => s.id === message.to || s.name === message.to);

  return (
    <div className={clsx('flex gap-3', isOutgoing && 'flex-row-reverse')}>
      <div
        className={clsx(
          'w-8 h-8 rounded-lg flex items-center justify-center shrink-0 text-lg',
          isOutgoing ? 'bg-sena-500' : 'bg-dark-700'
        )}
      >
        {isOutgoing ? '🦁' : getEmoji(message.from)}
      </div>
      <div
        className={clsx(
          'max-w-[80%] rounded-2xl px-4 py-3',
          isOutgoing ? 'bg-sena-500 text-dark-950' : 'bg-dark-800 text-dark-100'
        )}
      >
        <div className="flex items-center gap-2 mb-1">
          <span className={clsx('text-xs font-medium', isOutgoing ? 'text-dark-950/70' : 'text-dark-400')}>
            {isOutgoing ? 'Hub' : message.from}
          </span>
          <ArrowRight className={clsx('w-3 h-3', isOutgoing ? 'text-dark-950/50' : 'text-dark-500')} />
          <span className={clsx('text-xs', isOutgoing ? 'text-dark-950/70' : 'text-dark-400')}>
            {message.to === 'all' ? 'All Sessions' : targetSession?.name || message.to}
          </span>
        </div>
        <p className="whitespace-pre-wrap">{message.content}</p>
        <p className={clsx('text-xs mt-2', isOutgoing ? 'text-dark-950/60' : 'text-dark-500')}>
          {message.timestamp.toLocaleTimeString()}
        </p>
      </div>
    </div>
  );
}
