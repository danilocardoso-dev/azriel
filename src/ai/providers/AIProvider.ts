import type { AgentRequest, AgentResponse, AIRequest, AIResponse, OllamaStatus } from "../../types";

export interface AIProvider {
  chat(request: AIRequest): Promise<AIResponse>;
  /** Uma rodada de tool calling. Providers sem suporte simplesmente não implementam. */
  chatWithTools?(request: AgentRequest): Promise<AgentResponse>;
  isAvailable(): Promise<OllamaStatus>;
  listModels(): Promise<string[]>;
}
