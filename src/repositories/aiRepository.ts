import type { Conversation, ConversationMessage } from "../types";
import { invokeDatabase } from "./tauri";

/**
 * Esqueleto do AI Core: sem motor de IA por enquanto. Ficam só as conversas salvas.
 */
export const aiRepository = {
  listConversations: () => invokeDatabase<Conversation[]>("list_conversations"),
  deleteConversation: (id: string) => invokeDatabase<void>("delete_conversation", { id }),
  clearConversationMessages: (conversationId: string) => invokeDatabase<number>("clear_conversation_messages", { conversationId }),
  listMessages: (conversationId: string) => invokeDatabase<ConversationMessage[]>("list_messages", { conversationId }),
};
