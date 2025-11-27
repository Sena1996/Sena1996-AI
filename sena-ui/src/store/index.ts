import { create } from 'zustand';
import type { Provider, Session, Message, SystemHealth } from '../types';

interface AppState {
  providers: Provider[];
  sessions: Session[];
  messages: Message[];
  activeSessionId: string | null;
  health: SystemHealth | null;
  isLoading: boolean;
  error: string | null;

  setProviders: (providers: Provider[]) => void;
  setSessions: (sessions: Session[]) => void;
  addSession: (session: Session) => void;
  setActiveSession: (sessionId: string | null) => void;
  addMessage: (message: Message) => void;
  setMessages: (messages: Message[]) => void;
  setHealth: (health: SystemHealth) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  clearError: () => void;
}

export const useAppStore = create<AppState>((set) => ({
  providers: [],
  sessions: [],
  messages: [],
  activeSessionId: null,
  health: null,
  isLoading: false,
  error: null,

  setProviders: (providers) => set({ providers }),
  setSessions: (sessions) => set({ sessions }),
  addSession: (session) =>
    set((state) => ({ sessions: [...state.sessions, session] })),
  setActiveSession: (sessionId) => set({ activeSessionId: sessionId }),
  addMessage: (message) =>
    set((state) => ({ messages: [...state.messages, message] })),
  setMessages: (messages) => set({ messages }),
  setHealth: (health) => set({ health }),
  setLoading: (isLoading) => set({ isLoading }),
  setError: (error) => set({ error }),
  clearError: () => set({ error: null }),
}));

interface ThemeState {
  isDark: boolean;
  toggleTheme: () => void;
}

export const useThemeStore = create<ThemeState>((set) => ({
  isDark: true,
  toggleTheme: () => set((state) => ({ isDark: !state.isDark })),
}));
