import { invoke } from '@tauri-apps/api/core';
import type {
  Provider,
  Session,
  Message,
  ChatResponse,
  SystemHealth,
  Model,
} from '../types';

export class TauriError extends Error {
  constructor(
    message: string,
    public readonly originalError?: unknown
  ) {
    super(message);
    this.name = 'TauriError';
  }
}

async function safeInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new TauriError(`Command '${command}' failed: ${message}`, error);
  }
}

export async function getProviders(): Promise<Provider[]> {
  return safeInvoke<Provider[]>('get_providers');
}

export async function getProviderStatus(): Promise<Provider[]> {
  return safeInvoke<Provider[]>('get_provider_status');
}

export async function getModels(providerId?: string): Promise<Model[]> {
  return safeInvoke<Model[]>('get_models', { providerId });
}

export async function sendChat(
  message: string,
  provider?: string,
  model?: string
): Promise<ChatResponse> {
  if (!message.trim()) {
    throw new TauriError('Message cannot be empty');
  }
  return safeInvoke<ChatResponse>('send_chat', { message, provider, model });
}

export async function setDefaultProvider(providerId: string): Promise<void> {
  if (!providerId.trim()) {
    throw new TauriError('Provider ID cannot be empty');
  }
  return safeInvoke<void>('set_default_provider', { providerId });
}

export async function testProvider(providerId: string): Promise<boolean> {
  if (!providerId.trim()) {
    throw new TauriError('Provider ID cannot be empty');
  }
  return safeInvoke<boolean>('test_provider', { providerId });
}

export async function createSession(
  name: string,
  hostProvider: string
): Promise<Session> {
  if (!name.trim()) {
    throw new TauriError('Session name cannot be empty');
  }
  if (name.length > 200) {
    throw new TauriError('Session name too long (max 200 characters)');
  }
  return safeInvoke<Session>('create_session', { name, hostProvider });
}

export async function listSessions(): Promise<Session[]> {
  return safeInvoke<Session[]>('list_sessions');
}

export async function joinSession(
  sessionId: string,
  providerId: string
): Promise<string> {
  return safeInvoke<string>('join_session', { sessionId, providerId });
}

export async function startSession(sessionId: string): Promise<void> {
  return safeInvoke<void>('start_session', { sessionId });
}

export async function sendSessionMessage(
  sessionId: string,
  message: string,
  senderId?: string
): Promise<void> {
  return safeInvoke<void>('send_session_message', { sessionId, message, senderId });
}

export async function broadcastToSession(
  sessionId: string,
  message: string
): Promise<Message[]> {
  return safeInvoke<Message[]>('broadcast_to_session', { sessionId, message });
}

export async function getSessionInfo(sessionId: string): Promise<Session> {
  return safeInvoke<Session>('get_session_info', { sessionId });
}

export async function endSession(sessionId: string): Promise<void> {
  return safeInvoke<void>('end_session', { sessionId });
}

export async function getHealth(): Promise<SystemHealth> {
  return safeInvoke<SystemHealth>('get_health');
}

export async function getVersion(): Promise<string> {
  return safeInvoke<string>('get_version');
}
