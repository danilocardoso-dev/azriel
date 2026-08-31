import type { AIRequest, AIResponse, AISettings, AISettingsInput, Conversation, ConversationMessage, ConversationMessageInput, OllamaStatus } from "../types";
import { invokeDatabase } from "./tauri";

export const aiRepository = {
  getSettings: () => invokeDatabase<AISettings>("get_ai_settings"),
  updateSettings: (input: AISettingsInput) => invokeDatabase<AISettings>("update_ai_settings", { input }),
  listConversations: () => invokeDatabase<Conversation[]>("list_conversations"),
  createConversation: (id: string, title: string) => invokeDatabase<Conversation>("create_conversation", { input: { id, title } }),
  deleteConversation: (id: string) => invokeDatabase<void>("delete_conversation", { id }),
  listMessages: (conversationId: string) => invokeDatabase<ConversationMessage[]>("list_messages", { conversationId }),
  addMessage: (input: ConversationMessageInput) => invokeDatabase<ConversationMessage>("add_message", { input }),
  status: (endpoint: string, timeoutSeconds: number) => invokeDatabase<OllamaStatus>("ollama_status", { endpoint, timeoutSeconds }),
  chat: (endpoint: string, request: AIRequest) => invokeDatabase<AIResponse>("ollama_chat", { endpoint, model: request.model, messages: request.messages, timeoutSeconds: request.timeoutSeconds, generationProfile: request.generationProfile ?? "standard" }),
};
