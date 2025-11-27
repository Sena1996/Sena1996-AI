export interface Provider {
  id: string;
  name: string;
  status: 'connected' | 'disconnected' | 'rate_limited' | 'error';
  defaultModel: string;
  hasApiKey: boolean;
  capabilities: {
    streaming: boolean;
    tools: boolean;
    vision: boolean;
  };
}

export interface Model {
  id: string;
  name: string;
  provider: string;
  contextLength: number;
  supportsVision: boolean;
  supportsTools: boolean;
  supportsStreaming: boolean;
}

export interface Message {
  id: string;
  sessionId: string;
  senderId: string;
  senderName: string;
  senderProvider: string;
  content: string;
  timestamp: Date;
  isUser: boolean;
}

export interface Session {
  id: string;
  name: string;
  state: 'initializing' | 'active' | 'paused' | 'completed' | 'terminated';
  createdAt: Date;
  participants: Participant[];
  messageCount: number;
}

export interface Participant {
  agentId: string;
  provider: string;
  model: string;
  isHost: boolean;
  status: 'idle' | 'thinking' | 'processing' | 'offline' | 'error';
  messageCount: number;
}

export interface ChatRequest {
  sessionId?: string;
  provider?: string;
  model?: string;
  message: string;
}

export interface ChatResponse {
  id: string;
  provider: string;
  model: string;
  content: string;
  usage: {
    promptTokens: number;
    completionTokens: number;
    totalTokens: number;
  };
}

export interface SystemHealth {
  status: 'healthy' | 'degraded' | 'unhealthy';
  score: number;
  version: string;
  uptime: number;
  providers: {
    total: number;
    connected: number;
  };
  sessions: {
    active: number;
    total: number;
  };
}
