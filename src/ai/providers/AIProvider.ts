import type { AIRequest, AIResponse, OllamaStatus } from "../../types";

export interface AIProvider {
  chat(request: AIRequest): Promise<AIResponse>;
  isAvailable(): Promise<OllamaStatus>;
  listModels(): Promise<string[]>;
}
