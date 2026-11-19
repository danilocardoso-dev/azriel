import { aiRepository } from "../repositories/aiRepository";
import type { AgentStep, ConversationMessage, ConversationMessageInput } from "../types";

/** O backend devolve os passos do agente como JSON; a tela recebe a lista pronta. */
export function withSteps(message: ConversationMessage & { activity?: string | null }): ConversationMessage {
  const { activity, ...rest } = message;
  if (!activity) return rest;
  try {
    const steps = JSON.parse(activity) as unknown;
    return Array.isArray(steps) && steps.length ? { ...rest, steps: steps as AgentStep[] } : rest;
  } catch { return rest; }
}

export interface NewMessage { conversationId: string; role: ConversationMessageInput["role"]; content: string; steps?: AgentStep[] }

export const conversationService = {
  list: aiRepository.listConversations,
  create: (title: string) => aiRepository.createConversation(crypto.randomUUID(), title.trim().slice(0, 120) || "Nova conversa"),
  remove: aiRepository.deleteConversation,
  clearMessages: aiRepository.clearConversationMessages,
  messages: async (conversationId: string) => (await aiRepository.listMessages(conversationId)).map(withSteps),
  addMessage: async ({ steps, ...input }: NewMessage) => withSteps(await aiRepository.addMessage({ ...input, id: crypto.randomUUID(), activity: steps?.length ? JSON.stringify(steps) : null })),
  saveTaskReferences: aiRepository.saveTaskReferences,
  deleteMessage: aiRepository.deleteMessage,
};
