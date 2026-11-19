import { aiRepository } from "../repositories/aiRepository";

/** Conversas salvas do AI Core (só leitura e limpeza enquanto não há motor de IA). */
export const conversationService = {
  list: aiRepository.listConversations,
  remove: aiRepository.deleteConversation,
  clearMessages: aiRepository.clearConversationMessages,
  messages: aiRepository.listMessages,
};
