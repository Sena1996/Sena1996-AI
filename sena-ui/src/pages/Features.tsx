import { useState } from 'react';
import {
  Book,
  Bot,
  Brain,
  Code,
  Command,
  GitBranch,
  Globe,
  HelpCircle,
  Layers,
  Play,
  Search,
  Settings,
  Sparkles,
  Terminal,
  Users,
  Wrench,
  Zap,
} from 'lucide-react';
import clsx from 'clsx';

interface Feature {
  id: string;
  name: string;
  description: string;
  icon: React.ComponentType<{ className?: string }>;
  category: string;
  commands: string[];
  examples: string[];
  tips: string[];
}

const features: Feature[] = [
  {
    id: 'providers',
    name: 'Multi-Provider AI',
    description: 'Connect to multiple AI providers (Claude, OpenAI, Gemini, Ollama) with automatic fallback and load balancing.',
    icon: Bot,
    category: 'Core',
    commands: [
      'sena provider list',
      'sena provider models <provider>',
      'sena provider set-default <provider>',
    ],
    examples: [
      'sena provider list                    # List all configured providers',
      'sena provider models claude           # Show Claude models',
      'sena chat "Hello" --provider openai  # Use specific provider',
    ],
    tips: [
      'Set API keys in ~/.sena/providers.toml',
      'Use Ollama for local, private AI',
      'Provider fallback happens automatically on errors',
    ],
  },
  {
    id: 'hub',
    name: 'Hub Collaboration',
    description: 'Central hub for coordinating multiple AI sessions. Send tasks, broadcast messages, and monitor all sessions.',
    icon: Globe,
    category: 'Core',
    commands: [
      'sena hub status',
      'sena hub broadcast <message>',
      'sena session list',
      'sena session start --name <n> --role <r>',
    ],
    examples: [
      'sena hub status                       # Check hub health',
      'sena hub broadcast "Run tests"        # Message all sessions',
      'sena session start --name Backend --role backend',
    ],
    tips: [
      'Use the Chat page for smart messaging',
      'Sessions auto-register with the hub',
      'Cross-hub federation connects multiple hubs',
    ],
  },
  {
    id: 'tools',
    name: 'Tool System',
    description: 'Provider-agnostic tool/function calling. Execute file operations, shell commands, web searches, and more.',
    icon: Wrench,
    category: 'Automation',
    commands: [
      'sena tools list',
      'sena tools execute <name> --params <json>',
      'sena tools categories',
    ],
    examples: [
      'sena tools list                       # List all tools',
      'sena tools execute read_file --params \'{"path": "src/main.rs"}\'',
      'sena tools execute search_files --params \'{"pattern": "*.tsx"}\'',
    ],
    tips: [
      'Tools work with any AI provider',
      'Create custom tools in ~/.sena/tools/',
      'Tools are sandboxed for security',
    ],
  },
  {
    id: 'memory',
    name: 'Persistent Memory',
    description: 'Store and retrieve memories across sessions. The AI remembers preferences, facts, and project context.',
    icon: Brain,
    category: 'Intelligence',
    commands: [
      'sena memory add "<content>" --type <type>',
      'sena memory search <query>',
      'sena memory list',
      'sena memory stats',
    ],
    examples: [
      'sena memory add "User prefers Rust" --type preference',
      'sena memory search "programming"',
      'sena memory list --type project',
    ],
    tips: [
      'Memory types: Preference, Fact, Project, Context',
      'Higher importance = more relevant in search',
      'Memories improve AI personalization',
    ],
  },
  {
    id: 'autonomous',
    name: 'Autonomous Agent',
    description: 'Let AI plan and execute multi-step tasks autonomously. It breaks down tasks, uses tools, and reports progress.',
    icon: Zap,
    category: 'Automation',
    commands: [
      'sena auto "<task>"',
      'sena auto "<task>" --max-steps 10',
      'sena auto "<task>" --confirm',
    ],
    examples: [
      'sena auto "Refactor the authentication module"',
      'sena auto "Write tests for user service" --max-steps 5',
      'sena auto "Fix all linting errors" --confirm',
    ],
    tips: [
      'Use --confirm for step-by-step approval',
      'Max steps prevents runaway execution',
      'Agent uses tools and memory automatically',
    ],
  },
  {
    id: 'git',
    name: 'Git Integration',
    description: 'AI-powered git operations. Generate commit messages, review changes, and manage branches intelligently.',
    icon: GitBranch,
    category: 'Development',
    commands: [
      'sena git status',
      'sena git commit',
      'sena git log',
      'sena git diff',
    ],
    examples: [
      'sena git status                       # Enhanced status with insights',
      'sena git commit                       # AI-generated commit message',
      'sena git log --pretty                 # Formatted commit history',
    ],
    tips: [
      'AI analyzes diff to suggest commit messages',
      'Works with any git repository',
      'Respects .gitignore and git config',
    ],
  },
  {
    id: 'streaming',
    name: 'Real-time Streaming',
    description: 'Stream AI responses in real-time. WebSocket support for live updates in the UI.',
    icon: Play,
    category: 'Core',
    commands: [
      'sena chat "<message>" --stream',
    ],
    examples: [
      'sena chat "Explain quantum computing" --stream',
    ],
    tips: [
      'Streaming reduces perceived latency',
      'WebSocket broadcasts to connected clients',
      'JSON mode available for structured output',
    ],
  },
  {
    id: 'semantic-search',
    name: 'Semantic Search',
    description: 'Vector embeddings for intelligent memory search. Find related memories even with different wording.',
    icon: Search,
    category: 'Intelligence',
    commands: [
      'sena memory search "<query>" --semantic',
    ],
    examples: [
      'sena memory search "coding preferences" --semantic',
    ],
    tips: [
      'Combines keyword and semantic matching',
      'Uses cosine similarity for relevance',
      'Works offline with local embeddings',
    ],
  },
  {
    id: 'collab',
    name: 'AI Collaboration',
    description: 'Multiple AI agents working together. Create sessions, add participants, and facilitate AI discussions.',
    icon: Users,
    category: 'Advanced',
    commands: [
      'sena collab create "<topic>"',
      'sena collab add <session> --provider <p>',
      'sena collab discuss "<prompt>"',
    ],
    examples: [
      'sena collab create "Architecture Review"',
      'sena collab add arch-review --provider claude',
      'sena collab discuss "How should we handle auth?"',
    ],
    tips: [
      'Mix providers for diverse perspectives',
      'Host agent coordinates the discussion',
      'Export conversations for documentation',
    ],
  },
  {
    id: 'thinking',
    name: 'Deep Thinking',
    description: 'SENA 7 Ancient Wisdom Layers for complex problem solving. Multi-perspective analysis with depth control.',
    icon: Sparkles,
    category: 'Intelligence',
    commands: [
      'sena think "<query>"',
      'sena think "<query>" --depth deep',
      'sena think "<query>" --depth maximum',
    ],
    examples: [
      'sena think "Best architecture for microservices"',
      'sena think "Security implications of OAuth" --depth deep',
    ],
    tips: [
      'Depth levels: quick, normal, deep, maximum',
      '7 wisdom layers: Pattern, Knowledge, Logic, Creative, etc.',
      'Use /sena-think for quick access',
    ],
  },
];

const categories = [...new Set(features.map((f) => f.category))];

export default function Features() {
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [expandedFeature, setExpandedFeature] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');

  const filteredFeatures = features.filter((feature) => {
    const matchesCategory = selectedCategory === 'all' || feature.category === selectedCategory;
    const matchesSearch =
      feature.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      feature.description.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesCategory && matchesSearch;
  });

  return (
    <div className="p-8">
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-dark-100 flex items-center gap-3">
          <Book className="w-8 h-8 text-sena-400" />
          Features & Documentation
        </h1>
        <p className="text-dark-400 mt-2">
          Learn how to use SENA's features. Click any feature for detailed commands and examples.
        </p>
      </div>

      <div className="flex items-center gap-4 mb-6">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-dark-500" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search features..."
            className="input pl-10 w-full"
          />
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setSelectedCategory('all')}
            className={clsx(
              'px-3 py-2 rounded-lg text-sm font-medium transition-colors',
              selectedCategory === 'all'
                ? 'bg-sena-500 text-dark-950'
                : 'bg-dark-800 text-dark-300 hover:bg-dark-700'
            )}
          >
            All
          </button>
          {categories.map((cat) => (
            <button
              key={cat}
              onClick={() => setSelectedCategory(cat)}
              className={clsx(
                'px-3 py-2 rounded-lg text-sm font-medium transition-colors',
                selectedCategory === cat
                  ? 'bg-sena-500 text-dark-950'
                  : 'bg-dark-800 text-dark-300 hover:bg-dark-700'
              )}
            >
              {cat}
            </button>
          ))}
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {filteredFeatures.map((feature) => (
          <FeatureCard
            key={feature.id}
            feature={feature}
            isExpanded={expandedFeature === feature.id}
            onToggle={() =>
              setExpandedFeature(expandedFeature === feature.id ? null : feature.id)
            }
          />
        ))}
      </div>

      {filteredFeatures.length === 0 && (
        <div className="card text-center py-12">
          <HelpCircle className="w-12 h-12 mx-auto text-dark-600 mb-4" />
          <h3 className="text-lg font-medium text-dark-300">No features found</h3>
          <p className="text-dark-500 mt-1">Try a different search term or category</p>
        </div>
      )}

      <div className="mt-8 card bg-gradient-to-br from-sena-500/10 to-transparent border-sena-500/30">
        <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
          <Terminal className="w-5 h-5 text-sena-400" />
          Quick Reference
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <QuickRef icon={Command} title="Slash Commands">
            <code>/sena-think</code> - Deep thinking<br />
            <code>/sena-search</code> - Knowledge search<br />
            <code>/sena-analyze</code> - Code analysis
          </QuickRef>
          <QuickRef icon={Settings} title="Configuration">
            <code>~/.sena/</code> - Config directory<br />
            <code>providers.toml</code> - AI providers<br />
            <code>memory/</code> - Memory storage
          </QuickRef>
          <QuickRef icon={Layers} title="Hub Files">
            <code>~/.claude/hub/</code> - Hub data<br />
            <code>sessions.json</code> - Active sessions<br />
            <code>messages/</code> - Message queue
          </QuickRef>
        </div>
      </div>
    </div>
  );
}

function FeatureCard({
  feature,
  isExpanded,
  onToggle,
}: {
  feature: Feature;
  isExpanded: boolean;
  onToggle: () => void;
}) {
  const Icon = feature.icon;

  return (
    <div
      className={clsx(
        'card cursor-pointer transition-all',
        isExpanded && 'ring-2 ring-sena-500/50'
      )}
      onClick={onToggle}
    >
      <div className="flex items-start gap-4">
        <div className="p-3 rounded-xl bg-sena-500/10">
          <Icon className="w-6 h-6 text-sena-400" />
        </div>
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-1">
            <h3 className="text-lg font-semibold text-dark-100">{feature.name}</h3>
            <span className="px-2 py-0.5 bg-dark-800 rounded text-xs text-dark-400">
              {feature.category}
            </span>
          </div>
          <p className="text-dark-400 text-sm">{feature.description}</p>
        </div>
      </div>

      {isExpanded && (
        <div className="mt-4 pt-4 border-t border-dark-700 space-y-4" onClick={(e) => e.stopPropagation()}>
          <div>
            <h4 className="text-sm font-medium text-dark-300 mb-2 flex items-center gap-2">
              <Terminal className="w-4 h-4" />
              Commands
            </h4>
            <div className="space-y-1">
              {feature.commands.map((cmd, idx) => (
                <code
                  key={idx}
                  className="block text-sm text-sena-400 bg-dark-800 px-3 py-2 rounded font-mono"
                >
                  {cmd}
                </code>
              ))}
            </div>
          </div>

          <div>
            <h4 className="text-sm font-medium text-dark-300 mb-2 flex items-center gap-2">
              <Code className="w-4 h-4" />
              Examples
            </h4>
            <div className="space-y-1">
              {feature.examples.map((ex, idx) => (
                <code
                  key={idx}
                  className="block text-xs text-dark-400 bg-dark-800/50 px-3 py-2 rounded font-mono whitespace-pre"
                >
                  {ex}
                </code>
              ))}
            </div>
          </div>

          <div>
            <h4 className="text-sm font-medium text-dark-300 mb-2 flex items-center gap-2">
              <Sparkles className="w-4 h-4" />
              Tips
            </h4>
            <ul className="space-y-1">
              {feature.tips.map((tip, idx) => (
                <li key={idx} className="text-sm text-dark-400 flex items-start gap-2">
                  <span className="text-sena-400 mt-1">•</span>
                  {tip}
                </li>
              ))}
            </ul>
          </div>
        </div>
      )}
    </div>
  );
}

function QuickRef({
  icon: Icon,
  title,
  children,
}: {
  icon: React.ComponentType<{ className?: string }>;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="p-4 rounded-lg bg-dark-800/50">
      <h3 className="font-medium text-dark-200 mb-2 flex items-center gap-2">
        <Icon className="w-4 h-4 text-sena-400" />
        {title}
      </h3>
      <div className="text-sm text-dark-400 font-mono">{children}</div>
    </div>
  );
}
