import { aiRepository } from "../../repositories/aiRepository";
import type { AgentRequest, AIRequest, DeepSeekSettings, OllamaStatus } from "../../types";
import type { AIProvider } from "./AIProvider";

/** DeepSeek pela API oficial: mesmo formato do Gemini (compatível com OpenAI). */
export class DeepSeekProvider implements AIProvider {
  constructor(private readonly settings: DeepSeekSettings) {}
  chatWithTools(request: AgentRequest) { return aiRepository.deepSeekAgentChat(request); }
  async chat(request: AIRequest) {
    const response = await this.chatWithTools({ model: request.model, timeoutSeconds: request.timeoutSeconds, tools: [], messages: request.messages });
    return { content: response.content, model: response.model, truncated: response.truncated };
  }
  async isAvailable(): Promise<OllamaStatus> {
    const ready = this.settings.apiKeyConfigured && !!this.settings.model;
    return { available: ready, models: ready ? [this.settings.model] : [], error: ready ? null : "Configure a chave do DeepSeek e escolha um modelo." };
  }
  listModels() { return aiRepository.listDeepSeekModels(); }
}
