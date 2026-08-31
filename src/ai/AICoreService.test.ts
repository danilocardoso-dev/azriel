import { describe, expect, it } from "vitest";
import type { AISettings, Conversation, ConversationMessage } from "../types";
import { AICoreService, hasPathologicalRepetition, type ConversationGateway } from "./AICoreService";
import { ContextBuilder } from "./context/ContextBuilder";
import { FakeAIProvider } from "./providers/FakeAIProvider";
import { ToolRegistry, type ToolDependencies } from "./tools/toolRegistry";

const settings: AISettings = { provider: "ollama", endpoint: "http://localhost:11434", model: "qwen2.5:0.5b", contextMessageLimit: 6, timeoutSeconds: 30, updatedAt: "" };
const dependencies: ToolDependencies = {
  tasks: { list: async () => [], today: async () => [{ id: "t", title: "Testar", description: "", status: "pending", priority: "high", dueDate: null, projectId: null, knowledgeAreaId: null, createdAt: "", updatedAt: "", completedAt: null }], upcoming: async () => [], counters: async () => ({ pending: 1, today: 1, overdue: 0, priority: 1, notes: 0 }) },
  notes: { list: async () => [] }, projects: { list: async () => [], get: async () => null }, knowledge: { list: async () => [], get: async () => null, history: async () => [] }, education: { list: async () => [] }, databaseInfo: async () => ({ schemaVersion: 5, integrationValue: 0 }),
  system: {
    snapshot: async () => ({ collectedAt: 0, details: { osName: "Windows", osVersion: "11", kernelVersion: "", architecture: "x86_64", hostname: "azriel", logicalCores: 8, physicalCores: 4, uptimeSeconds: 100 }, cpu: { usagePercent: 10, cores: [10] }, memory: { totalBytes: 1000, usedBytes: 500, availableBytes: 500, swapTotalBytes: 0, swapUsedBytes: 0 }, storage: [], network: [], errors: [] }),
    processes: async () => [], listWorkspaces: async () => [], workspaceStatus: async () => { throw new Error("workspace ausente"); },
  },
  ollama: { settings: async () => settings, status: async () => ({ available: true, models: [settings.model], error: null }) },
  automation: { listApplications: async () => [], listUrls: async () => [], execute: async (request) => ({ success: false, message: "não registrado", errorCode: "TARGET_NOT_FOUND", actionId: request.actionId, targetName: null, historyId: 1, confirmation: null }) },
};

class MemoryConversations implements ConversationGateway {
  conversation: Conversation = { id: "c1", title: "Teste", createdAt: "", updatedAt: "" };
  stored: ConversationMessage[] = [];
  async create(title: string) { this.conversation = { ...this.conversation, title }; return this.conversation; }
  async messages() { return this.stored; }
  async addMessage(input: { conversationId: string; role: "user" | "assistant"; content: string }) { const message = { id: `m${this.stored.length}`, createdAt: "", ...input }; this.stored.push(message); return message; }
}

describe("AI Core Service", () => {
  it("usa provider desacoplado e persiste os dois lados da conversa", async () => {
    const provider = new FakeAIProvider("Você possui uma tarefa hoje.");
    const gateway = new MemoryConversations();
    const service = new AICoreService(provider, new ContextBuilder(new ToolRegistry(dependencies)), gateway, settings);
    const result = await service.send("O que tenho para hoje?", null);
    expect(result.assistantMessage.content).toContain("uma tarefa");
    expect(gateway.stored.map((message) => message.role)).toEqual(["user", "assistant"]);
    expect(provider.requests[0].messages.some((message) => message.content.includes("get_today_tasks"))).toBe(true);
  });

  it("não chama o modelo quando a consulta não encontra dados", async () => {
    const provider = new FakeAIProvider("Não deveria ser usada");
    const gateway = new MemoryConversations();
    const service = new AICoreService(provider, new ContextBuilder(new ToolRegistry(dependencies)), gateway, settings);
    const result = await service.send("Quais são meus projetos?", null);
    expect(result.assistantMessage.content).toBe("Não encontrei essa informação registrada no Azriel.");
    expect(provider.requests).toHaveLength(0);
  });

  it("responde conhecimento geral sem anexar dados internos do Azriel", async () => {
    const provider = new FakeAIProvider("A luz azul sofre maior espalhamento na atmosfera.");
    const gateway = new MemoryConversations();
    const service = new AICoreService(provider, new ContextBuilder(new ToolRegistry(dependencies)), gateway, settings);
    await service.send("Por que o céu é azul?", null);
    expect(provider.requests).toHaveLength(1);
    expect(provider.requests[0].messages.some((message) => message.content.includes("DADOS ESTRUTURADOS"))).toBe(false);
    expect(provider.requests[0].messages[0].content).toContain("conhecimento geral");
  });

  it("regenera respostas repetitivas com o perfil protegido", async () => {
    const repeated = Array.from({ length: 8 }, () => "A lei de Newton explica a lei de Newton.").join(" ");
    const provider = new FakeAIProvider([repeated, "Newton formulou suas leis a partir de estudos matemáticos e observações."]);
    const gateway = new MemoryConversations();
    const service = new AICoreService(provider, new ContextBuilder(new ToolRegistry(dependencies)), gateway, settings);
    const result = await service.send("Quem foi Newton?", null);
    expect(hasPathologicalRepetition(repeated)).toBe(true);
    expect(provider.requests).toHaveLength(2);
    expect(provider.requests[1].generationProfile).toBe("repetition-retry");
    expect(result.assistantMessage.content).toContain("formulou suas leis");
  });

  it("não devolve nem reaproveita uma resposta que continua degenerada", async () => {
    const repeated = Array.from({ length: 10 }, () => "gravidade gravidade gravidade e lei de Newton").join(" ");
    const provider = new FakeAIProvider(repeated);
    const gateway = new MemoryConversations();
    gateway.stored.push({ id: "bad", conversationId: "c1", role: "assistant", content: repeated, createdAt: "" });
    const service = new AICoreService(provider, new ContextBuilder(new ToolRegistry(dependencies)), gateway, settings);
    const result = await service.send("Explique novamente.", gateway.conversation);
    expect(provider.requests[0].messages.some((message) => message.content === repeated)).toBe(false);
    expect(result.assistantMessage.content).toContain("Não consegui formular uma resposta confiável");
  });

  it("regenera uma resposta interrompida pelo limite de geração", async () => {
    const provider = new FakeAIProvider([
      { content: "Uma explicação que terminou no meio da", truncated: true },
      { content: "Uma explicação curta e completa.", truncated: false },
    ]);
    const gateway = new MemoryConversations();
    const service = new AICoreService(provider, new ContextBuilder(new ToolRegistry(dependencies)), gateway, settings);
    const result = await service.send("Explique um assunto complexo.", null);
    expect(provider.requests).toHaveLength(2);
    expect(provider.requests[1].generationProfile).toBe("repetition-retry");
    expect(result.assistantMessage.content).toBe("Uma explicação curta e completa.");
  });
});
