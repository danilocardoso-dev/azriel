import { describe, expect, it } from "vitest";
import type { AISettings, Conversation, ConversationMessage } from "../types";
import { AICoreService, hasPathologicalRepetition, selectConversationHistory, type ConversationGateway } from "./AICoreService";
import { ContextBuilder } from "./context/ContextBuilder";
import { FakeAIProvider } from "./providers/FakeAIProvider";
import { ToolRegistry, type ToolDependencies } from "./tools/toolRegistry";

const settings: AISettings = { provider: "ollama", endpoint: "http://localhost:11434", model: "qwen2.5:0.5b", contextMessageLimit: 6, timeoutSeconds: 30, updatedAt: "" };
const dependencies: ToolDependencies = {
  tasks: { list: async () => [], today: async () => [{ id: "t", title: "Testar", description: "", status: "pending", priority: "high", dueDate: null, projectId: null, knowledgeAreaId: null, createdAt: "", updatedAt: "", completedAt: null }], upcoming: async () => [], counters: async () => ({ pending: 1, today: 1, overdue: 0, priority: 1, notes: 0, completed: 0 }), completeReferenced: async () => { throw new Error("referência ausente"); } },
  notes: { list: async () => [] }, projects: { list: async () => [], get: async () => null }, knowledge: { list: async () => [], get: async () => null, history: async () => [] }, databaseInfo: async () => ({ schemaVersion: 5, integrationValue: 0 }),
  system: {
    snapshot: async () => ({ collectedAt: 0, details: { osName: "Windows", osVersion: "11", kernelVersion: "", architecture: "x86_64", hostname: "azriel", logicalCores: 8, physicalCores: 4, uptimeSeconds: 100 }, cpu: { usagePercent: 10, cores: [10] }, memory: { totalBytes: 1000, usedBytes: 500, availableBytes: 500, swapTotalBytes: 0, swapUsedBytes: 0 }, storage: [], network: [], errors: [] }),
    processes: async () => [], listWorkspaces: async () => [], workspaceStatus: async () => { throw new Error("workspace ausente"); },
  },
  ollama: { settings: async () => settings, status: async () => ({ available: true, models: [settings.model], error: null }) },
  automation: { listApplications: async () => [], listUrls: async () => [], listRoutines: async () => [], runRoutine: async (request) => ({ success: false, status: "failed", routineId: request.routineId, routineName: "", historyId: 1, completedSteps: 0, failedStep: null, error: "não registrada", confirmation: null }), execute: async (request) => ({ success: false, message: "não registrado", errorCode: "TARGET_NOT_FOUND", actionId: request.actionId, targetName: null, historyId: 1, confirmation: null }) },
};

class MemoryConversations implements ConversationGateway {
  conversation: Conversation = { id: "c1", title: "Teste", createdAt: "", updatedAt: "" };
  stored: ConversationMessage[] = [];
  references: Array<{ conversationId: string; sourceMessageId: string; taskIds: string[] }> = [];
  async create(title: string) { this.conversation = { ...this.conversation, title }; return this.conversation; }
  async messages() { return this.stored; }
  async addMessage(input: { conversationId: string; role: "user" | "assistant"; content: string }) { const message = { id: `m${this.stored.length}`, createdAt: "", ...input }; this.stored.push(message); return message; }
  async saveTaskReferences(conversationId: string, sourceMessageId: string, taskIds: string[]) { this.references.push({ conversationId, sourceMessageId, taskIds }); return taskIds.length; }
}

describe("AI Core Service", () => {
  it("limita o histórico por tamanho e preserva as mensagens mais recentes", () => {
    const messages: ConversationMessage[] = [
      { id: "m1", conversationId: "c1", role: "user", content: "a".repeat(700), createdAt: "" },
      { id: "m2", conversationId: "c1", role: "assistant", content: "b".repeat(700), createdAt: "" },
      { id: "m3", conversationId: "c1", role: "user", content: "pergunta atual", createdAt: "" },
    ];

    expect(selectConversationHistory(messages, 20, 800).map((message) => message.id)).toEqual(["m2", "m3"]);
  });

  it("remove tentativas consecutivas duplicadas do contexto", () => {
    const messages: ConversationMessage[] = [
      { id: "m1", conversationId: "c1", role: "user", content: "Explique biologia molecular", createdAt: "" },
      { id: "m2", conversationId: "c1", role: "user", content: "Explique biologia molecular", createdAt: "" },
      { id: "m3", conversationId: "c1", role: "user", content: "Resuma biologia molecular", createdAt: "" },
    ];

    expect(selectConversationHistory(messages, 20).map((message) => message.id)).toEqual(["m1", "m3"]);
  });

  it("usa provider desacoplado e persiste os dois lados da conversa", async () => {
    const provider = new FakeAIProvider("Você possui uma tarefa hoje.");
    const gateway = new MemoryConversations();
    const service = new AICoreService(provider, new ContextBuilder(new ToolRegistry(dependencies)), gateway, settings);
    const result = await service.send("O que tenho para hoje?", null);
    expect(result.assistantMessage.content).toContain("uma tarefa");
    expect(gateway.stored.map((message) => message.role)).toEqual(["user", "assistant"]);
    expect(provider.requests[0].messages.some((message) => message.content.includes("get_today_tasks"))).toBe(true);
  });

  it("responde demandas críticas diretamente do SQLite sem depender do Ollama", async () => {
    const provider = new FakeAIProvider("Não deveria ser chamada");
    const gateway = new MemoryConversations();
    const criticalDependencies: ToolDependencies = {
      ...dependencies,
      tasks: { ...dependencies.tasks, list: async () => [{ id: "critical", title: "Revisar redespacho", description: "", status: "pending", priority: "critical", dueDate: null, projectId: null, knowledgeAreaId: null, createdAt: "", updatedAt: "", completedAt: null }] },
    };
    const service = new AICoreService(provider, new ContextBuilder(new ToolRegistry(criticalDependencies)), gateway, settings);
    const result = await service.send("Quais são minhas demandas críticas?", null);
    expect(provider.requests).toHaveLength(0);
    expect(result.assistantMessage.content).toContain("1 demanda crítica ativa");
    expect(result.assistantMessage.content).toContain("Revisar redespacho — CRÍTICA · sem prazo");
    expect(gateway.references).toEqual([{ conversationId: "c1", sourceMessageId: "m1", taskIds: ["critical"] }]);
  });

  it("conclui a referência numérica explicitamente sem chamar o Ollama", async () => {
    const provider = new FakeAIProvider("Não deveria ser chamada");
    const gateway = new MemoryConversations();
    const calls: Array<{ conversationId: string; position: number }> = [];
    const completeDependencies: ToolDependencies = {
      ...dependencies,
      tasks: { ...dependencies.tasks, completeReferenced: async (conversationId, position) => {
        calls.push({ conversationId, position });
        return { id: "critical", title: "Adicionar feature de redespacho", description: "", status: "completed", priority: "critical", dueDate: null, projectId: null, knowledgeAreaId: null, createdAt: "", updatedAt: "", completedAt: "" };
      } },
    };
    const result = await new AICoreService(provider, new ContextBuilder(new ToolRegistry(completeDependencies)), gateway, settings)
      .send("Finaliza a 1 para mim.", gateway.conversation);
    expect(calls).toEqual([{ conversationId: "c1", position: 1 }]);
    expect(provider.requests).toHaveLength(0);
    expect(result.assistantMessage.content).toBe("Demanda concluída:\n\nAdicionar feature de redespacho");
  });

  it("responde prioridades altas e críticas sem incluir tarefas médias ou concluídas", async () => {
    const provider = new FakeAIProvider("Não deveria ser chamada");
    const gateway = new MemoryConversations();
    const priorityDependencies: ToolDependencies = {
      ...dependencies,
      tasks: { ...dependencies.tasks, list: async () => [
        { id: "critical", title: "Crítica", description: "", status: "pending", priority: "critical", dueDate: null, projectId: null, knowledgeAreaId: null, createdAt: "", updatedAt: "", completedAt: null },
        { id: "high", title: "Alta", description: "", status: "in_progress", priority: "high", dueDate: "2026-09-30", projectId: null, knowledgeAreaId: null, createdAt: "", updatedAt: "", completedAt: null },
        { id: "medium", title: "Média", description: "", status: "pending", priority: "medium", dueDate: null, projectId: null, knowledgeAreaId: null, createdAt: "", updatedAt: "", completedAt: null },
        { id: "done", title: "Concluída", description: "", status: "completed", priority: "critical", dueDate: null, projectId: null, knowledgeAreaId: null, createdAt: "", updatedAt: "", completedAt: "" },
      ] },
    };
    const result = await new AICoreService(provider, new ContextBuilder(new ToolRegistry(priorityDependencies)), gateway, settings)
      .send("Quais são minhas prioridades?", null);
    expect(provider.requests).toHaveLength(0);
    expect(result.assistantMessage.content).toContain("2 demandas prioritárias ativas");
    expect(result.assistantMessage.content).toContain("Crítica — CRÍTICA");
    expect(result.assistantMessage.content).toContain("Alta — ALTA · prazo: 2026-09-30");
    expect(result.assistantMessage.content).not.toContain("Média");
    expect(result.assistantMessage.content).not.toContain("Concluída");
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

  it("não ativa o Engineering congelado por linguagem natural", async () => {
    const provider = new FakeAIProvider("O módulo Engineering está congelado nesta versão.");
    const gateway = new MemoryConversations();
    const phases: string[] = [];
    const service = new AICoreService(provider, new ContextBuilder(new ToolRegistry(dependencies)), gateway, settings);
    await service.send("Exploda a montagem.", null, (state) => phases.push(state));
    expect(phases).not.toContain("engineering");
    expect(provider.requests[0].messages.some((message) => message.content.includes("DADOS ESTRUTURADOS"))).toBe(false);
  });
});
