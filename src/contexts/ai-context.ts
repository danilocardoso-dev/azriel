import { createContext } from "react";
import type { AzrielState, Conversation, ConversationMessage } from "../types";

/** Esqueleto do AI Core: conversas salvas, sem motor de IA por enquanto. */
export interface AIContextValue {
  conversations: Conversation[];
  selectedConversation: Conversation | null;
  messages: ConversationMessage[];
  coreState: AzrielState;
  loading: boolean;
  error: string | null;
  /** Sempre falso por enquanto: o Azriel está sem motor de IA. */
  engineAvailable: boolean;
  engineLabel: string;
  selectConversation: (conversation: Conversation) => Promise<void>;
  newConversation: () => void;
  deleteConversation: (conversation: Conversation) => Promise<void>;
  reload: () => Promise<void>;
}

export const AIContext = createContext<AIContextValue | null>(null);
