import { aiRepository } from "../../repositories/aiRepository";
import type { AIRequest, AISettings } from "../../types";
import type { AIProvider } from "./AIProvider";

export class OllamaProvider implements AIProvider {
  constructor(private readonly settings: AISettings) {}
  chat(request: AIRequest) { return aiRepository.chat(this.settings.endpoint, request); }
  isAvailable() { return aiRepository.status(this.settings.endpoint, Math.min(this.settings.timeoutSeconds, 8)); }
  async listModels() { return (await this.isAvailable()).models; }
}
