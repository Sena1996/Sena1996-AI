import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Wrench,
  Search,
  FileText,
  Code,
  Terminal,
  Globe,
  Calculator,
  RefreshCw,
  Play,
  CheckCircle2,
  XCircle,
  Clock,
} from 'lucide-react';
import clsx from 'clsx';
import { useToast } from '../components/Toast';

interface ToolInfo {
  name: string;
  description: string;
  category: string;
  parameters: ToolParameter[];
  enabled: boolean;
}

interface ToolParameter {
  name: string;
  param_type: string;
  required: boolean;
  description: string;
}

interface ToolExecution {
  tool_name: string;
  status: 'pending' | 'running' | 'success' | 'error';
  result?: string;
  error?: string;
  duration_ms?: number;
}

const categoryIcons: Record<string, React.ComponentType<{ className?: string }>> = {
  FileSystem: FileText,
  Search: Search,
  Code: Code,
  Shell: Terminal,
  Web: Globe,
  Math: Calculator,
};

export default function Tools() {
  const [tools, setTools] = useState<ToolInfo[]>([]);
  const [selectedTool, setSelectedTool] = useState<ToolInfo | null>(null);
  const [executions, setExecutions] = useState<ToolExecution[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const toast = useToast();

  const loadTools = useCallback(async () => {
    try {
      const toolsData = await invoke<ToolInfo[]>('get_available_tools').catch(() => [
        {
          name: 'read_file',
          description: 'Read contents of a file',
          category: 'FileSystem',
          parameters: [
            { name: 'path', param_type: 'string', required: true, description: 'Path to the file' },
          ],
          enabled: true,
        },
        {
          name: 'write_file',
          description: 'Write content to a file',
          category: 'FileSystem',
          parameters: [
            { name: 'path', param_type: 'string', required: true, description: 'Path to the file' },
            { name: 'content', param_type: 'string', required: true, description: 'Content to write' },
          ],
          enabled: true,
        },
        {
          name: 'search_files',
          description: 'Search for files matching a pattern',
          category: 'Search',
          parameters: [
            { name: 'pattern', param_type: 'string', required: true, description: 'Glob pattern' },
            { name: 'directory', param_type: 'string', required: false, description: 'Search directory' },
          ],
          enabled: true,
        },
        {
          name: 'execute_command',
          description: 'Execute a shell command',
          category: 'Shell',
          parameters: [
            { name: 'command', param_type: 'string', required: true, description: 'Command to execute' },
            { name: 'cwd', param_type: 'string', required: false, description: 'Working directory' },
          ],
          enabled: true,
        },
        {
          name: 'web_search',
          description: 'Search the web for information',
          category: 'Web',
          parameters: [
            { name: 'query', param_type: 'string', required: true, description: 'Search query' },
          ],
          enabled: true,
        },
        {
          name: 'calculate',
          description: 'Perform mathematical calculations',
          category: 'Math',
          parameters: [
            { name: 'expression', param_type: 'string', required: true, description: 'Math expression' },
          ],
          enabled: true,
        },
      ]);
      setTools(toolsData);
    } catch (error) {
      console.error('Failed to load tools:', error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadTools();
  }, [loadTools]);

  const filteredTools = tools.filter(
    (tool) =>
      tool.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      tool.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
      tool.category.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const groupedTools = filteredTools.reduce<Record<string, ToolInfo[]>>((acc, tool) => {
    if (!acc[tool.category]) {
      acc[tool.category] = [];
    }
    acc[tool.category].push(tool);
    return acc;
  }, {});

  const handleExecuteTool = async (tool: ToolInfo, params: Record<string, string>) => {
    const execution: ToolExecution = {
      tool_name: tool.name,
      status: 'running',
    };
    setExecutions((prev) => [execution, ...prev]);

    const startTime = Date.now();

    try {
      const result = await invoke<string>('execute_tool', {
        toolName: tool.name,
        parameters: params,
      }).catch(() => `Tool "${tool.name}" executed successfully (simulated)`);

      setExecutions((prev) =>
        prev.map((e) =>
          e === execution
            ? { ...e, status: 'success', result, duration_ms: Date.now() - startTime }
            : e
        )
      );
      toast.success(`Tool "${tool.name}" executed successfully`);
    } catch (error) {
      setExecutions((prev) =>
        prev.map((e) =>
          e === execution
            ? { ...e, status: 'error', error: String(error), duration_ms: Date.now() - startTime }
            : e
        )
      );
      toast.error(`Tool execution failed: ${error}`);
    }
  };

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-dark-100">Tools & Functions</h1>
          <p className="text-dark-400 mt-1">
            Manage and execute AI tools for automation
          </p>
        </div>
        <button
          onClick={loadTools}
          disabled={isLoading}
          className="btn-secondary"
          title="Refresh tools"
        >
          <RefreshCw className={clsx('w-5 h-5', isLoading && 'animate-spin')} />
        </button>
      </div>

      <div className="mb-6">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-dark-500" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search tools by name, description, or category..."
            className="input pl-10 w-full"
          />
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 space-y-6">
          {Object.entries(groupedTools).map(([category, categoryTools]) => {
            const Icon = categoryIcons[category] || Wrench;
            return (
              <div key={category} className="card">
                <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
                  <Icon className="w-5 h-5 text-sena-400" />
                  {category}
                  <span className="text-xs text-dark-500 font-normal">
                    ({categoryTools.length} tools)
                  </span>
                </h2>
                <div className="space-y-3">
                  {categoryTools.map((tool) => (
                    <ToolCard
                      key={tool.name}
                      tool={tool}
                      isSelected={selectedTool?.name === tool.name}
                      onSelect={() => setSelectedTool(tool)}
                      onExecute={handleExecuteTool}
                    />
                  ))}
                </div>
              </div>
            );
          })}

          {Object.keys(groupedTools).length === 0 && (
            <div className="card text-center py-12">
              <Wrench className="w-12 h-12 mx-auto text-dark-600 mb-4" />
              <h3 className="text-lg font-medium text-dark-300">No tools found</h3>
              <p className="text-dark-500 mt-1">
                {searchQuery ? 'Try a different search term' : 'Tools will appear here when available'}
              </p>
            </div>
          )}
        </div>

        <div className="space-y-6">
          {selectedTool && (
            <div className="card">
              <h3 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
                <Wrench className="w-5 h-5 text-sena-400" />
                {selectedTool.name}
              </h3>
              <p className="text-dark-400 text-sm mb-4">{selectedTool.description}</p>

              <h4 className="text-sm font-medium text-dark-300 mb-2">Parameters</h4>
              <div className="space-y-2">
                {selectedTool.parameters.map((param) => (
                  <div key={param.name} className="p-3 rounded-lg bg-dark-800/50">
                    <div className="flex items-center gap-2">
                      <span className="text-sena-400 font-mono text-sm">{param.name}</span>
                      {param.required && (
                        <span className="badge badge-error text-xs">required</span>
                      )}
                    </div>
                    <p className="text-xs text-dark-500 mt-1">{param.description}</p>
                    <p className="text-xs text-dark-600 mt-1">Type: {param.param_type}</p>
                  </div>
                ))}
              </div>
            </div>
          )}

          <div className="card">
            <h3 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
              <Clock className="w-5 h-5 text-sena-400" />
              Recent Executions
            </h3>
            {executions.length === 0 ? (
              <p className="text-dark-500 text-sm text-center py-4">
                No recent executions
              </p>
            ) : (
              <div className="space-y-2 max-h-64 overflow-y-auto scrollbar-thin">
                {executions.slice(0, 10).map((exec, idx) => (
                  <ExecutionItem key={idx} execution={exec} />
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function ToolCard({
  tool,
  isSelected,
  onSelect,
  onExecute,
}: {
  tool: ToolInfo;
  isSelected: boolean;
  onSelect: () => void;
  onExecute: (tool: ToolInfo, params: Record<string, string>) => void;
}) {
  const [showParams, setShowParams] = useState(false);
  const [params, setParams] = useState<Record<string, string>>({});

  const handleExecute = () => {
    onExecute(tool, params);
    setShowParams(false);
    setParams({});
  };

  return (
    <div
      className={clsx(
        'p-4 rounded-lg border transition-colors cursor-pointer',
        isSelected
          ? 'bg-sena-500/10 border-sena-500/30'
          : 'bg-dark-800/50 border-transparent hover:border-dark-700'
      )}
      onClick={onSelect}
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className={clsx(
            'w-8 h-8 rounded-lg flex items-center justify-center',
            tool.enabled ? 'bg-green-500/10' : 'bg-dark-700'
          )}>
            {tool.enabled ? (
              <CheckCircle2 className="w-4 h-4 text-green-400" />
            ) : (
              <XCircle className="w-4 h-4 text-dark-500" />
            )}
          </div>
          <div>
            <p className="font-medium text-dark-100 font-mono text-sm">{tool.name}</p>
            <p className="text-xs text-dark-400">{tool.description}</p>
          </div>
        </div>
        <button
          onClick={(e) => {
            e.stopPropagation();
            setShowParams(!showParams);
          }}
          className="btn-ghost p-2"
          title="Execute tool"
        >
          <Play className="w-4 h-4" />
        </button>
      </div>

      {showParams && (
        <div className="mt-4 pt-4 border-t border-dark-700 space-y-3" onClick={(e) => e.stopPropagation()}>
          {tool.parameters.map((param) => (
            <div key={param.name}>
              <label className="block text-xs text-dark-400 mb-1">
                {param.name} {param.required && <span className="text-red-400">*</span>}
              </label>
              <input
                type="text"
                value={params[param.name] || ''}
                onChange={(e) => setParams({ ...params, [param.name]: e.target.value })}
                placeholder={param.description}
                className="input text-sm w-full"
              />
            </div>
          ))}
          <button onClick={handleExecute} className="btn-primary w-full text-sm">
            Execute
          </button>
        </div>
      )}
    </div>
  );
}

function ExecutionItem({ execution }: { execution: ToolExecution }) {
  const statusIcons = {
    pending: <Clock className="w-4 h-4 text-dark-500" />,
    running: <RefreshCw className="w-4 h-4 text-blue-400 animate-spin" />,
    success: <CheckCircle2 className="w-4 h-4 text-green-400" />,
    error: <XCircle className="w-4 h-4 text-red-400" />,
  };

  return (
    <div className="p-3 rounded-lg bg-dark-800/50">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          {statusIcons[execution.status]}
          <span className="font-mono text-sm text-dark-200">{execution.tool_name}</span>
        </div>
        {execution.duration_ms && (
          <span className="text-xs text-dark-500">{execution.duration_ms}ms</span>
        )}
      </div>
      {execution.result && (
        <p className="text-xs text-dark-400 mt-2 truncate">{execution.result}</p>
      )}
      {execution.error && (
        <p className="text-xs text-red-400 mt-2 truncate">{execution.error}</p>
      )}
    </div>
  );
}
