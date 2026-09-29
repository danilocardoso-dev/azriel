import { createContext } from "react";
import type { AISettings, AISettingsInput, AzrielState, Conversation, ConversationMessage, LocalAiModelStatus, OllamaStatus } from "../types";

export interface AIContextValue {
  settings: AISettings | null;
  status: OllamaStatus | null;
  modelLifecycle: LocalAiModelStatus | null;
  modelTransition: "LOADING" | "UNLOADING" | null;
  conversations: Conversation[];
  selectedConversation: Conversation | null;
  messages: ConversationMessage[];
  coreState: AzrielState;
  phase: string;
  loading: boolean;
  sending: boolean;
  error: string | null;
  selectConversation: (conversation: Conversation) => Promise<void>;
  newConversation: () => void;
  deleteConversation: (conversation: Conversation) => Promise<void>;
  clearConversationHistory: (conversation: Conversation) => Promise<number>;
  reload: () => Promise<void>;
  send: (query: string) => Promise<void>;
  updateSettings: (input: AISettingsInput) => Promise<void>;
  probeOllama: (endpoint: string, timeoutSeconds: number) => Promise<OllamaStatus>;
  refreshStatus: () => Promise<OllamaStatus | null>;
  refreshModelLifecycle: () => Promise<LocalAiModelStatus | null>;
  loadLocalModel: () => Promise<LocalAiModelStatus>;
  unloadLocalModel: () => Promise<LocalAiModelStatus>;
}

export const AIContext = createContext<AIContextValue | null>(null);
