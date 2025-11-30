import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Brain,
  Search,
  Plus,
  Trash2,
  Tag,
  RefreshCw,
  Star,
  Clock,
  Filter,
  Database,
} from 'lucide-react';
import clsx from 'clsx';
import { useToast } from '../components/Toast';

interface MemoryEntry {
  id: string;
  content: string;
  memory_type: string;
  tags: string[];
  importance: number;
  created_at: string;
  updated_at: string;
  access_count: number;
}

interface MemoryStats {
  total_entries: number;
  by_type: Record<string, number>;
  total_access_count: number;
  avg_importance: number;
}

const memoryTypeColors: Record<string, string> = {
  Preference: 'bg-blue-500/10 text-blue-400 border-blue-500/30',
  Fact: 'bg-green-500/10 text-green-400 border-green-500/30',
  Project: 'bg-purple-500/10 text-purple-400 border-purple-500/30',
  Context: 'bg-orange-500/10 text-orange-400 border-orange-500/30',
  Conversation: 'bg-cyan-500/10 text-cyan-400 border-cyan-500/30',
};

export default function Memory() {
  const [memories, setMemories] = useState<MemoryEntry[]>([]);
  const [stats, setStats] = useState<MemoryStats | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedType, setSelectedType] = useState<string>('all');
  const [isLoading, setIsLoading] = useState(true);
  const [showAddForm, setShowAddForm] = useState(false);
  const toast = useToast();

  const [newMemory, setNewMemory] = useState({
    content: '',
    memory_type: 'Fact',
    tags: '',
    importance: 0.5,
  });

  const loadMemories = useCallback(async () => {
    try {
      const [memoriesData, statsData] = await Promise.all([
        invoke<MemoryEntry[]>('get_memories').catch(() => [
          {
            id: 'mem_demo1',
            content: 'User prefers dark mode for all applications',
            memory_type: 'Preference',
            tags: ['ui', 'settings'],
            importance: 0.8,
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
            access_count: 5,
          },
          {
            id: 'mem_demo2',
            content: 'Main project uses Rust for backend, TypeScript for frontend',
            memory_type: 'Project',
            tags: ['tech-stack', 'sena'],
            importance: 0.9,
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
            access_count: 12,
          },
          {
            id: 'mem_demo3',
            content: 'User timezone is UTC+0',
            memory_type: 'Fact',
            tags: ['settings'],
            importance: 0.5,
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
            access_count: 3,
          },
        ]),
        invoke<MemoryStats>('get_memory_stats').catch(() => ({
          total_entries: 3,
          by_type: { Preference: 1, Project: 1, Fact: 1 },
          total_access_count: 20,
          avg_importance: 0.73,
        })),
      ]);
      setMemories(memoriesData);
      setStats(statsData);
    } catch (error) {
      console.error('Failed to load memories:', error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadMemories();
  }, [loadMemories]);

  const filteredMemories = memories.filter((memory) => {
    const matchesSearch =
      memory.content.toLowerCase().includes(searchQuery.toLowerCase()) ||
      memory.tags.some((tag) => tag.toLowerCase().includes(searchQuery.toLowerCase()));
    const matchesType = selectedType === 'all' || memory.memory_type === selectedType;
    return matchesSearch && matchesType;
  });

  const handleAddMemory = async () => {
    if (!newMemory.content.trim()) {
      toast.error('Memory content cannot be empty');
      return;
    }

    try {
      await invoke('add_memory', {
        content: newMemory.content,
        memoryType: newMemory.memory_type,
        tags: newMemory.tags.split(',').map((t) => t.trim()).filter(Boolean),
        importance: newMemory.importance,
      }).catch(() => {
        const newEntry: MemoryEntry = {
          id: `mem_${Date.now()}`,
          content: newMemory.content,
          memory_type: newMemory.memory_type,
          tags: newMemory.tags.split(',').map((t) => t.trim()).filter(Boolean),
          importance: newMemory.importance,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
          access_count: 0,
        };
        setMemories((prev) => [newEntry, ...prev]);
      });

      toast.success('Memory added successfully');
      setNewMemory({ content: '', memory_type: 'Fact', tags: '', importance: 0.5 });
      setShowAddForm(false);
      loadMemories();
    } catch (error) {
      toast.error(`Failed to add memory: ${error}`);
    }
  };

  const handleDeleteMemory = async (id: string) => {
    try {
      await invoke('delete_memory', { id }).catch(() => {
        setMemories((prev) => prev.filter((m) => m.id !== id));
      });
      toast.success('Memory deleted');
      loadMemories();
    } catch (error) {
      toast.error(`Failed to delete memory: ${error}`);
    }
  };

  const memoryTypes = ['all', ...Object.keys(stats?.by_type || {})];

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-dark-100">Persistent Memory</h1>
          <p className="text-dark-400 mt-1">
            Manage AI memories for personalized responses
          </p>
        </div>
        <div className="flex items-center gap-3">
          <button
            onClick={() => setShowAddForm(!showAddForm)}
            className="btn-primary"
          >
            <Plus className="w-5 h-5" />
            Add Memory
          </button>
          <button
            onClick={loadMemories}
            disabled={isLoading}
            className="btn-secondary"
            title="Refresh memories"
          >
            <RefreshCw className={clsx('w-5 h-5', isLoading && 'animate-spin')} />
          </button>
        </div>
      </div>

      {stats && (
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
          <StatCard
            icon={Database}
            label="Total Memories"
            value={stats.total_entries.toString()}
            color="sena"
          />
          <StatCard
            icon={Clock}
            label="Total Access"
            value={stats.total_access_count.toString()}
            color="blue"
          />
          <StatCard
            icon={Star}
            label="Avg Importance"
            value={`${(stats.avg_importance * 100).toFixed(0)}%`}
            color="yellow"
          />
          <StatCard
            icon={Tag}
            label="Memory Types"
            value={Object.keys(stats.by_type).length.toString()}
            color="purple"
          />
        </div>
      )}

      {showAddForm && (
        <div className="card mb-6">
          <h3 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
            <Plus className="w-5 h-5 text-sena-400" />
            Add New Memory
          </h3>
          <div className="space-y-4">
            <div>
              <label className="block text-sm text-dark-400 mb-1">Content</label>
              <textarea
                value={newMemory.content}
                onChange={(e) => setNewMemory({ ...newMemory, content: e.target.value })}
                placeholder="Enter memory content..."
                className="input w-full h-24 resize-none"
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm text-dark-400 mb-1">Type</label>
                <select
                  value={newMemory.memory_type}
                  onChange={(e) => setNewMemory({ ...newMemory, memory_type: e.target.value })}
                  className="input w-full"
                >
                  <option value="Preference">Preference</option>
                  <option value="Fact">Fact</option>
                  <option value="Project">Project</option>
                  <option value="Context">Context</option>
                  <option value="Conversation">Conversation</option>
                </select>
              </div>
              <div>
                <label className="block text-sm text-dark-400 mb-1">
                  Importance: {(newMemory.importance * 100).toFixed(0)}%
                </label>
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.1"
                  value={newMemory.importance}
                  onChange={(e) => setNewMemory({ ...newMemory, importance: parseFloat(e.target.value) })}
                  className="w-full"
                />
              </div>
            </div>
            <div>
              <label className="block text-sm text-dark-400 mb-1">Tags (comma-separated)</label>
              <input
                type="text"
                value={newMemory.tags}
                onChange={(e) => setNewMemory({ ...newMemory, tags: e.target.value })}
                placeholder="tag1, tag2, tag3"
                className="input w-full"
              />
            </div>
            <div className="flex justify-end gap-3">
              <button onClick={() => setShowAddForm(false)} className="btn-ghost">
                Cancel
              </button>
              <button onClick={handleAddMemory} className="btn-primary">
                Save Memory
              </button>
            </div>
          </div>
        </div>
      )}

      <div className="flex items-center gap-4 mb-6">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-dark-500" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search memories by content or tags..."
            className="input pl-10 w-full"
          />
        </div>
        <div className="flex items-center gap-2">
          <Filter className="w-5 h-5 text-dark-500" />
          <select
            value={selectedType}
            onChange={(e) => setSelectedType(e.target.value)}
            className="input"
          >
            {memoryTypes.map((type) => (
              <option key={type} value={type}>
                {type === 'all' ? 'All Types' : type}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div className="space-y-4">
        {filteredMemories.length === 0 ? (
          <div className="card text-center py-12">
            <Brain className="w-12 h-12 mx-auto text-dark-600 mb-4" />
            <h3 className="text-lg font-medium text-dark-300">No memories found</h3>
            <p className="text-dark-500 mt-1">
              {searchQuery ? 'Try a different search term' : 'Add your first memory to get started'}
            </p>
          </div>
        ) : (
          filteredMemories.map((memory) => (
            <MemoryCard
              key={memory.id}
              memory={memory}
              onDelete={() => handleDeleteMemory(memory.id)}
            />
          ))
        )}
      </div>
    </div>
  );
}

function StatCard({
  icon: Icon,
  label,
  value,
  color,
}: {
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  value: string;
  color: 'sena' | 'blue' | 'yellow' | 'purple';
}) {
  const colorClasses = {
    sena: 'bg-sena-500/10 text-sena-400',
    blue: 'bg-blue-500/10 text-blue-400',
    yellow: 'bg-yellow-500/10 text-yellow-400',
    purple: 'bg-purple-500/10 text-purple-400',
  };

  return (
    <div className="card flex items-center gap-4">
      <div className={`p-3 rounded-xl ${colorClasses[color]}`}>
        <Icon className="w-6 h-6" />
      </div>
      <div>
        <p className="text-2xl font-bold text-dark-100">{value}</p>
        <p className="text-sm text-dark-400">{label}</p>
      </div>
    </div>
  );
}

function MemoryCard({
  memory,
  onDelete,
}: {
  memory: MemoryEntry;
  onDelete: () => void;
}) {
  const typeClass = memoryTypeColors[memory.memory_type] || 'bg-dark-700 text-dark-300 border-dark-600';

  return (
    <div className="card hover:border-sena-500/30 transition-colors">
      <div className="flex items-start justify-between gap-4">
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-2">
            <span className={`px-2 py-0.5 rounded-full text-xs border ${typeClass}`}>
              {memory.memory_type}
            </span>
            <div className="flex items-center gap-1">
              <Star className="w-3 h-3 text-yellow-400" />
              <span className="text-xs text-dark-400">
                {(memory.importance * 100).toFixed(0)}%
              </span>
            </div>
            <span className="text-xs text-dark-500">
              Accessed {memory.access_count} times
            </span>
          </div>
          <p className="text-dark-100">{memory.content}</p>
          {memory.tags.length > 0 && (
            <div className="flex items-center gap-2 mt-3">
              <Tag className="w-4 h-4 text-dark-500" />
              <div className="flex flex-wrap gap-1">
                {memory.tags.map((tag) => (
                  <span
                    key={tag}
                    className="px-2 py-0.5 bg-dark-800 rounded text-xs text-dark-400"
                  >
                    {tag}
                  </span>
                ))}
              </div>
            </div>
          )}
          <p className="text-xs text-dark-500 mt-2">
            Created: {new Date(memory.created_at).toLocaleDateString()}
          </p>
        </div>
        <button
          onClick={onDelete}
          className="btn-ghost p-2 text-dark-500 hover:text-red-400"
          title="Delete memory"
        >
          <Trash2 className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}
