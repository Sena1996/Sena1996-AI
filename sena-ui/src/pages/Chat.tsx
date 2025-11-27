import { useState, useRef, useEffect } from 'react';
import { Send, Bot, User, Loader2, Settings2 } from 'lucide-react';
import clsx from 'clsx';

interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  provider?: string;
  model?: string;
  timestamp: Date;
}

const providers = [
  { id: 'claude', name: 'Claude', model: 'claude-sonnet-4-5' },
  { id: 'openai', name: 'OpenAI', model: 'gpt-4.1' },
  { id: 'gemini', name: 'Gemini', model: 'gemini-2.5-flash' },
  { id: 'ollama', name: 'Ollama', model: 'llama3.2' },
  { id: 'mistral', name: 'Mistral', model: 'mistral-large-latest' },
];

export default function Chat() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [selectedProvider, setSelectedProvider] = useState('claude');
  const [showSettings, setShowSettings] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || isLoading) return;

    const userMessage: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'user',
      content: input.trim(),
      timestamp: new Date(),
    };

    setMessages((prev) => [...prev, userMessage]);
    setInput('');
    setIsLoading(true);

    await new Promise((resolve) => setTimeout(resolve, 1500));

    const provider = providers.find((p) => p.id === selectedProvider);
    const assistantMessage: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'assistant',
      content: `This is a demo response from ${provider?.name}. In the full application, this would call the Tauri backend to get a real response from the ${provider?.model} model.\n\nYour message was: "${userMessage.content}"`,
      provider: provider?.name,
      model: provider?.model,
      timestamp: new Date(),
    };

    setMessages((prev) => [...prev, assistantMessage]);
    setIsLoading(false);
  };

  return (
    <div className="flex flex-col h-full">
      <div className="border-b border-dark-800 p-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-xl font-bold text-dark-100">AI Chat</h1>
            <p className="text-sm text-dark-400">
              Chat with any configured AI provider
            </p>
          </div>
          <button
            onClick={() => setShowSettings(!showSettings)}
            className={clsx(
              'btn-ghost p-2',
              showSettings && 'bg-dark-800 text-sena-400'
            )}
          >
            <Settings2 className="w-5 h-5" />
          </button>
        </div>

        {showSettings && (
          <div className="mt-4 p-4 rounded-lg bg-dark-800/50">
            <label className="block text-sm text-dark-300 mb-2">
              Select Provider
            </label>
            <div className="flex flex-wrap gap-2">
              {providers.map((provider) => (
                <button
                  key={provider.id}
                  onClick={() => setSelectedProvider(provider.id)}
                  className={clsx(
                    'px-4 py-2 rounded-lg text-sm font-medium transition-colors',
                    selectedProvider === provider.id
                      ? 'bg-sena-500 text-dark-950'
                      : 'bg-dark-700 text-dark-300 hover:bg-dark-600'
                  )}
                >
                  {provider.name}
                </button>
              ))}
            </div>
            <p className="text-xs text-dark-500 mt-2">
              Current model:{' '}
              {providers.find((p) => p.id === selectedProvider)?.model}
            </p>
          </div>
        )}
      </div>

      <div className="flex-1 overflow-auto p-4 space-y-4 scrollbar-thin">
        {messages.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <div className="w-16 h-16 rounded-2xl bg-sena-500/10 flex items-center justify-center mb-4">
              <Bot className="w-8 h-8 text-sena-400" />
            </div>
            <h2 className="text-lg font-semibold text-dark-200">
              Start a conversation
            </h2>
            <p className="text-dark-400 mt-1 max-w-md">
              Send a message to chat with{' '}
              {providers.find((p) => p.id === selectedProvider)?.name}. You can
              switch providers using the settings button above.
            </p>
          </div>
        ) : (
          messages.map((message) => (
            <MessageBubble key={message.id} message={message} />
          ))
        )}
        {isLoading && (
          <div className="flex items-center gap-3 text-dark-400">
            <Loader2 className="w-5 h-5 animate-spin" />
            <span>
              {providers.find((p) => p.id === selectedProvider)?.name} is
              thinking...
            </span>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      <form onSubmit={handleSubmit} className="p-4 border-t border-dark-800">
        <div className="flex gap-3">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Type your message..."
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

function MessageBubble({ message }: { message: ChatMessage }) {
  const isUser = message.role === 'user';

  return (
    <div className={clsx('flex gap-3', isUser && 'flex-row-reverse')}>
      <div
        className={clsx(
          'w-8 h-8 rounded-lg flex items-center justify-center shrink-0',
          isUser ? 'bg-sena-500' : 'bg-dark-700'
        )}
      >
        {isUser ? (
          <User className="w-4 h-4 text-dark-950" />
        ) : (
          <Bot className="w-4 h-4 text-dark-300" />
        )}
      </div>
      <div
        className={clsx(
          'max-w-[80%] rounded-2xl px-4 py-3',
          isUser ? 'bg-sena-500 text-dark-950' : 'bg-dark-800 text-dark-100'
        )}
      >
        {!isUser && message.provider && (
          <p className="text-xs text-dark-400 mb-1">
            {message.provider} • {message.model}
          </p>
        )}
        <p className="whitespace-pre-wrap">{message.content}</p>
        <p
          className={clsx(
            'text-xs mt-2',
            isUser ? 'text-dark-950/60' : 'text-dark-500'
          )}
        >
          {message.timestamp.toLocaleTimeString()}
        </p>
      </div>
    </div>
  );
}
