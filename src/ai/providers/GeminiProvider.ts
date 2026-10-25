import { aiRepository } from "../../repositories/aiRepository";
import type { AgentRequest, AIRequest, OllamaStatus, RemoteAISettings } from "../../types";
import type { AIProvider } from "./AIProvider";

export class GeminiProvider implements AIProvider {
  constructor(private readonly settings: RemoteAISettings) {}
  chatWithTools(request: AgentRequest) { return aiRepository.remoteAgentChat(request); }
  async chat(request: AIRequest) {
    const response = await this.chatWithTools({ model: request.model, timeoutSeconds: request.timeoutSeconds, tools: [], messages: request.messages });
    return { content: response.content, model: response.model, truncated: response.truncated };
  }
  async isAvailable(): Promise<OllamaStatus> {
    const ready = this.settings.apiKeyConfigured && !!this.settings.model;
    return { available: ready, models: ready ? [this.settings.model] : [], error: ready ? null : "Configure a chave do Gemini e selecione um modelo." };
  }
  listModels() { return aiRepository.listRemoteModels(); }
}
