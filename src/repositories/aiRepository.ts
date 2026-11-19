import type { Conversation, ConversationMessage, RemoteAISettings, RemoteAISettingsInput } from "../types";
import { invokeDatabase } from "./tauri";

/**
 * Esqueleto do AI Core: sem motor de IA por enquanto. Ficam as conversas salvas e a chave do
 * Google usada pela chamada de voz.
 */
export const aiRepository = {
  listConversations: () => invokeDatabase<Conversation[]>("list_conversations"),
  deleteConversation: (id: string) => invokeDatabase<void>("delete_conversation", { id }),
  clearConversationMessages: (conversationId: string) => invokeDatabase<number>("clear_conversation_messages", { conversationId }),
  listMessages: (conversationId: string) => invokeDatabase<ConversationMessage[]>("list_messages", { conversationId }),
  getRemoteSettings: () => invokeDatabase<RemoteAISettings>("get_remote_ai_settings"),
  updateRemoteSettings: (input: RemoteAISettingsInput) => invokeDatabase<RemoteAISettings>("update_remote_ai_settings", { input: { ...input, apiKey: input.apiKey ?? null } }),
};
