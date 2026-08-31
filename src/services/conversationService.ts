import { aiRepository } from "../repositories/aiRepository";
import type { ConversationMessageInput } from "../types";

export const conversationService = {
  list: aiRepository.listConversations,
  create: (title: string) => aiRepository.createConversation(crypto.randomUUID(), title.trim().slice(0, 120) || "Nova conversa"),
  remove: aiRepository.deleteConversation,
  messages: aiRepository.listMessages,
  addMessage: (input: Omit<ConversationMessageInput, "id">) => aiRepository.addMessage({ ...input, id: crypto.randomUUID() }),
};
