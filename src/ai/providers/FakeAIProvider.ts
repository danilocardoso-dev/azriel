import type { AIRequest, AIResponse, OllamaStatus } from "../../types";
import type { AIProvider } from "./AIProvider";

export class FakeAIProvider implements AIProvider {
  public requests: AIRequest[] = [];
  constructor(private readonly response: string | Array<string | { content: string; truncated: boolean }> = "Resposta simulada do provider de teste.", private readonly available = true) {}
  async chat(request: AIRequest): Promise<AIResponse> { this.requests.push(request); const selected = Array.isArray(this.response) ? this.response[Math.min(this.requests.length - 1, this.response.length - 1)] : this.response; return { content: typeof selected === "string" ? selected : selected.content, model: "fake:test", truncated: typeof selected === "string" ? false : selected.truncated }; }
  async isAvailable(): Promise<OllamaStatus> { return { available: this.available, models: this.available ? ["fake:test"] : [], error: this.available ? null : "Provider indisponível" }; }
  async listModels() { return (await this.isAvailable()).models; }
}
