import { createContext } from "react";
import type { AISettings, AISettingsInput, AzrielState, Conversation, ConversationMessage, OllamaStatus } from "../types";

export interface AIContextValue {
  settings: AISettings | null;
  status: OllamaStatus | null;
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
  send: (query: string) => Promise<void>;
  updateSettings: (input: AISettingsInput) => Promise<void>;
  refreshStatus: () => Promise<OllamaStatus | null>;
}

export const AIContext = createContext<AIContextValue | null>(null);
