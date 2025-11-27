import { useState } from 'react';
import {
  Users,
  Plus,
  Play,
  Pause,
  XCircle,
  MessageSquare,
  Bot,
  Clock,
} from 'lucide-react';
import clsx from 'clsx';
import type { Session, Participant } from '../types';

const mockSessions: Session[] = [];

const availableProviders = [
  { id: 'claude', name: 'Claude' },
  { id: 'openai', name: 'OpenAI' },
  { id: 'gemini', name: 'Gemini' },
  { id: 'ollama', name: 'Ollama' },
  { id: 'mistral', name: 'Mistral' },
];

export default function Sessions() {
  const [sessions, setSessions] = useState<Session[]>(mockSessions);
  const [showNewSession, setShowNewSession] = useState(false);
  const [newSessionName, setNewSessionName] = useState('');
  const [newSessionHost, setNewSessionHost] = useState('claude');

  const handleCreateSession = () => {
    if (!newSessionName.trim()) return;

    const newSession: Session = {
      id: `session_${crypto.randomUUID().split('-')[0]}`,
      name: newSessionName,
      state: 'initializing',
      createdAt: new Date(),
      participants: [
        {
          agentId: `${newSessionHost}_${crypto.randomUUID().split('-')[0]}`,
          provider: newSessionHost,
          model:
            availableProviders.find((p) => p.id === newSessionHost)?.name ||
            newSessionHost,
          isHost: true,
          status: 'idle',
          messageCount: 0,
        },
      ],
      messageCount: 0,
    };

    setSessions((prev) => [...prev, newSession]);
    setNewSessionName('');
    setShowNewSession(false);
  };

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-dark-100">
            Collaboration Sessions
          </h1>
          <p className="text-dark-400 mt-1">
            Create multi-AI collaboration sessions where AIs talk to each other
          </p>
        </div>
        <button
          onClick={() => setShowNewSession(true)}
          className="btn-primary"
        >
          <Plus className="w-5 h-5 mr-2" />
          New Session
        </button>
      </div>

      {showNewSession && (
        <div className="card mb-6">
          <h2 className="text-lg font-semibold text-dark-100 mb-4">
            Create New Session
          </h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm text-dark-300 mb-2">
                Session Name
              </label>
              <input
                type="text"
                value={newSessionName}
                onChange={(e) => setNewSessionName(e.target.value)}
                placeholder="e.g., Code Review Session"
                className="input"
              />
            </div>
            <div>
              <label className="block text-sm text-dark-300 mb-2">
                Host Provider
              </label>
              <div className="flex flex-wrap gap-2">
                {availableProviders.map((provider) => (
                  <button
                    key={provider.id}
                    onClick={() => setNewSessionHost(provider.id)}
                    className={clsx(
                      'px-4 py-2 rounded-lg text-sm font-medium transition-colors',
                      newSessionHost === provider.id
                        ? 'bg-sena-500 text-dark-950'
                        : 'bg-dark-700 text-dark-300 hover:bg-dark-600'
                    )}
                  >
                    {provider.name}
                  </button>
                ))}
              </div>
            </div>
            <div className="flex gap-3">
              <button onClick={handleCreateSession} className="btn-primary">
                Create Session
              </button>
              <button
                onClick={() => setShowNewSession(false)}
                className="btn-secondary"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}

      {sessions.length === 0 ? (
        <div className="card flex flex-col items-center justify-center py-16 text-center">
          <div className="w-16 h-16 rounded-2xl bg-sena-500/10 flex items-center justify-center mb-4">
            <Users className="w-8 h-8 text-sena-400" />
          </div>
          <h2 className="text-lg font-semibold text-dark-200">
            No Active Sessions
          </h2>
          <p className="text-dark-400 mt-1 max-w-md">
            Create a collaboration session to enable multiple AI providers to
            work together on a task. AIs can share context, review each other's
            work, and reach consensus.
          </p>
          <button
            onClick={() => setShowNewSession(true)}
            className="btn-primary mt-6"
          >
            <Plus className="w-5 h-5 mr-2" />
            Create Your First Session
          </button>
        </div>
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {sessions.map((session) => (
            <SessionCard
              key={session.id}
              session={session}
              onAddParticipant={(providerId) => {
                setSessions((prev) =>
                  prev.map((s) =>
                    s.id === session.id
                      ? {
                          ...s,
                          participants: [
                            ...s.participants,
                            {
                              agentId: `${providerId}_${crypto.randomUUID().split('-')[0]}`,
                              provider: providerId,
                              model:
                                availableProviders.find(
                                  (p) => p.id === providerId
                                )?.name || providerId,
                              isHost: false,
                              status: 'idle',
                              messageCount: 0,
                            },
                          ],
                        }
                      : s
                  )
                );
              }}
              onStart={() => {
                setSessions((prev) =>
                  prev.map((s) =>
                    s.id === session.id ? { ...s, state: 'active' } : s
                  )
                );
              }}
              onPause={() => {
                setSessions((prev) =>
                  prev.map((s) =>
                    s.id === session.id ? { ...s, state: 'paused' } : s
                  )
                );
              }}
              onEnd={() => {
                setSessions((prev) => prev.filter((s) => s.id !== session.id));
              }}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function SessionCard({
  session,
  onAddParticipant,
  onStart,
  onPause,
  onEnd,
}: {
  session: Session;
  onAddParticipant: (providerId: string) => void;
  onStart: () => void;
  onPause: () => void;
  onEnd: () => void;
}) {
  const [showAddParticipant, setShowAddParticipant] = useState(false);

  const stateColors = {
    initializing: 'badge-info',
    active: 'badge-success',
    paused: 'badge-warning',
    completed: 'badge bg-dark-600 text-dark-300',
    terminated: 'badge-error',
  };

  const existingProviders = session.participants.map((p) => p.provider);
  const availableToAdd = availableProviders.filter(
    (p) => !existingProviders.includes(p.id)
  );

  return (
    <div className="card">
      <div className="flex items-start justify-between mb-4">
        <div>
          <div className="flex items-center gap-2">
            <h3 className="font-semibold text-dark-100">{session.name}</h3>
            <span className={stateColors[session.state]}>{session.state}</span>
          </div>
          <p className="text-xs text-dark-400 mt-1">ID: {session.id}</p>
        </div>
        <div className="flex items-center gap-1">
          {session.state === 'initializing' && (
            <button onClick={onStart} className="btn-ghost p-2 text-green-400">
              <Play className="w-4 h-4" />
            </button>
          )}
          {session.state === 'active' && (
            <button onClick={onPause} className="btn-ghost p-2 text-yellow-400">
              <Pause className="w-4 h-4" />
            </button>
          )}
          {session.state === 'paused' && (
            <button onClick={onStart} className="btn-ghost p-2 text-green-400">
              <Play className="w-4 h-4" />
            </button>
          )}
          <button onClick={onEnd} className="btn-ghost p-2 text-red-400">
            <XCircle className="w-4 h-4" />
          </button>
        </div>
      </div>

      <div className="space-y-2 mb-4">
        <p className="text-xs text-dark-400 uppercase tracking-wider">
          Participants
        </p>
        {session.participants.map((participant) => (
          <ParticipantRow key={participant.agentId} participant={participant} />
        ))}
      </div>

      {showAddParticipant && availableToAdd.length > 0 && (
        <div className="mb-4 p-3 rounded-lg bg-dark-800/50">
          <p className="text-xs text-dark-400 mb-2">Add AI Participant:</p>
          <div className="flex flex-wrap gap-2">
            {availableToAdd.map((provider) => (
              <button
                key={provider.id}
                onClick={() => {
                  onAddParticipant(provider.id);
                  setShowAddParticipant(false);
                }}
                className="px-3 py-1 rounded-lg text-xs font-medium bg-dark-700 text-dark-300 hover:bg-dark-600"
              >
                {provider.name}
              </button>
            ))}
          </div>
        </div>
      )}

      <div className="flex items-center justify-between pt-4 border-t border-dark-700">
        <div className="flex items-center gap-4 text-xs text-dark-400">
          <span className="flex items-center gap-1">
            <MessageSquare className="w-4 h-4" />
            {session.messageCount} messages
          </span>
          <span className="flex items-center gap-1">
            <Clock className="w-4 h-4" />
            {session.createdAt.toLocaleTimeString()}
          </span>
        </div>
        {availableToAdd.length > 0 && (
          <button
            onClick={() => setShowAddParticipant(!showAddParticipant)}
            className="btn-ghost text-xs"
          >
            <Plus className="w-4 h-4 mr-1" />
            Add AI
          </button>
        )}
      </div>
    </div>
  );
}

function ParticipantRow({ participant }: { participant: Participant }) {
  const statusColors = {
    idle: 'bg-green-500',
    thinking: 'bg-yellow-500 animate-pulse',
    processing: 'bg-blue-500 animate-pulse',
    offline: 'bg-dark-600',
    error: 'bg-red-500',
  };

  return (
    <div className="flex items-center justify-between p-2 rounded-lg bg-dark-800/50">
      <div className="flex items-center gap-2">
        <Bot className="w-4 h-4 text-dark-400" />
        <span className="text-sm text-dark-200">{participant.provider}</span>
        {participant.isHost && (
          <span className="badge bg-sena-500/20 text-sena-400 text-xs">
            Host
          </span>
        )}
      </div>
      <div className="flex items-center gap-2">
        <span className="text-xs text-dark-500">
          {participant.messageCount} msgs
        </span>
        <div
          className={`w-2 h-2 rounded-full ${statusColors[participant.status]}`}
        />
      </div>
    </div>
  );
}
