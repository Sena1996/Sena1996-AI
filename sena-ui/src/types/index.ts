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

export interface ProviderMetadata {
  id: string;
  displayName: string;
  description: string;
  icon: string | null;
  website: string;
  documentationUrl: string | null;
  authSchema: AuthSchema;
}

export interface AuthSchema {
  authType: 'api_key' | 'oauth2' | 'basic_auth' | 'local' | 'none';
  fields: AuthField[];
}

export interface AuthField {
  id: string;
  displayName: string;
  fieldType: 'text' | 'password' | 'url' | 'number' | 'toggle';
  required: boolean;
  sensitive: boolean;
  placeholder: string | null;
  helpText: string | null;
  defaultValue: string | null;
  envVarName: string | null;
  validationPattern: string | null;
}

export interface CredentialStatus {
  providerId: string;
  hasCredential: boolean;
  source: 'keychain' | 'config' | 'environment' | 'none';
  isValid: boolean | null;
  canImportFromEnv: boolean;
}

export interface ValidationResult {
  valid: boolean;
  error: string | null;
}

export interface StorageOptions {
  keychainAvailable: boolean;
  configFilePath: string;
}

export type StorageType = 'keychain' | 'config';

export interface GuardianStatus {
  enabled: boolean;
  sandboxLevel: string;
  hallucinationMode: string;
  threshold: number;
}

export interface GuardianValidationResult {
  command: string;
  allowed: boolean;
  riskScore: number;
  reason: string | null;
  matchedPatterns: string[];
}

export interface HallucinationCheck {
  isHallucination: boolean;
  riskScore: number;
  response: string;
  harmonyStatus: string;
  warnings: string[];
  details: {
    consistencyScore: number;
    semanticEntropy: number;
    factValidationScore: number;
    suspiciousPatterns: string[];
  };
}

export interface DevilStatus {
  enabled: boolean;
  timeoutSecs: number;
  minProviders: number;
  synthesisMethod: string;
  consensusThreshold: number;
  waitMode: string;
  availableProviders: DevilProvider[];
}

export interface DevilProvider {
  id: string;
  status: string;
}

export interface DevilExecuteResult {
  content: string;
  consensusScore: number;
  synthesisMethod: string;
  totalLatencyMs: number;
  factsVerified: number;
  factsRejected: number;
  providerResponses: DevilProviderResponse[];
}

export interface DevilProviderResponse {
  providerId: string;
  model: string;
  status: string;
  latencyMs: number;
  contentPreview: string | null;
}
