import type { AIRequest, AIResponse, AISettings, AISettingsInput, Conversation, ConversationMessage, ConversationMessageInput, LocalAiModelStatus, OllamaStatus } from "../types";
import { invokeDatabase } from "./tauri";

export const aiRepository = {
  getSettings: () => invokeDatabase<AISettings>("get_ai_settings"),
  updateSettings: (input: AISettingsInput) => invokeDatabase<AISettings>("update_ai_settings", { input }),
  listConversations: () => invokeDatabase<Conversation[]>("list_conversations"),
  createConversation: (id: string, title: string) => invokeDatabase<Conversation>("create_conversation", { input: { id, title } }),
  deleteConversation: (id: string) => invokeDatabase<void>("delete_conversation", { id }),
  clearConversationMessages: (conversationId: string) => invokeDatabase<number>("clear_conversation_messages", { conversationId }),
  listMessages: (conversationId: string) => invokeDatabase<ConversationMessage[]>("list_messages", { conversationId }),
  addMessage: (input: ConversationMessageInput) => invokeDatabase<ConversationMessage>("add_message", { input }),
  saveTaskReferences: (conversationId: string, sourceMessageId: string, taskIds: string[]) => invokeDatabase<number>("save_ai_task_references", { input: { conversationId, sourceMessageId, taskIds } }),
  status: (endpoint: string, timeoutSeconds: number) => invokeDatabase<OllamaStatus>("ollama_status", { endpoint, timeoutSeconds }),
  modelLifecycleStatus: () => invokeDatabase<LocalAiModelStatus>("get_local_ai_model_status"),
  loadModel: () => invokeDatabase<LocalAiModelStatus>("load_local_ai_model"),
  unloadModel: () => invokeDatabase<LocalAiModelStatus>("unload_local_ai_model"),
  chat: (endpoint: string, request: AIRequest) => invokeDatabase<AIResponse>("ollama_chat", {
    endpoint,
    model: request.model,
    messages: request.messages,
    timeoutSeconds: request.timeoutSeconds,
    generationProfile: request.generationProfile ?? "standard",
    structuredOutputSchema: request.structuredOutputSchema ?? null,
    requestMetadata: request.requestMetadata ?? null,
  }),
};
