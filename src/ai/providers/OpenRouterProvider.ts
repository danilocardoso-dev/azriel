import { aiRepository } from "../../repositories/aiRepository";
import type { AgentRequest, AIRequest, OllamaStatus, OpenRouterSettings } from "../../types";
import type { AIProvider } from "./AIProvider";

/** OpenRouter: mesmo formato do Gemini (API compatível com OpenAI), com vários modelos por trás. */
export class OpenRouterProvider implements AIProvider {
  constructor(private readonly settings: OpenRouterSettings) {}
  chatWithTools(request: AgentRequest) { return aiRepository.openRouterAgentChat(request); }
  async chat(request: AIRequest) {
    const response = await this.chatWithTools({ model: request.model, timeoutSeconds: request.timeoutSeconds, tools: [], messages: request.messages });
    return { content: response.content, model: response.model, truncated: response.truncated };
  }
  async isAvailable(): Promise<OllamaStatus> {
    const ready = this.settings.apiKeyConfigured && !!this.settings.model;
    return { available: ready, models: ready ? [this.settings.model] : [], error: ready ? null : "Configure a chave do OpenRouter e escolha um modelo." };
  }
  async listModels() { return this.settings.favorites; }
}
