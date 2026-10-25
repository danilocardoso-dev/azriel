import { describe, expect, it } from "vitest";
import { AICoreService, type ConversationGateway } from "../AICoreService";
import { ContextBuilder } from "../context/ContextBuilder";
import type { AIProvider } from "../providers/AIProvider";
import { ToolRegistry, type ToolDependencies } from "../tools/toolRegistry";
import type { AgentRequest, AgentResponse, AISettings, Conversation, ConversationMessage, Task } from "../../types";
import { AGENT_MAX_STEPS, runAgent } from "./agentLoop";

const task: Task = { id: "t1", title: "Revisar contrato", description: "", status: "pending", priority: "high", dueDate: "2026-10-25", createdAt: "", updatedAt: "", completedAt: null } as unknown as Task;
let completed = 0;
const deps: ToolDependencies = {
  tasks: { list: async () => [task], today: async () => [task], upcoming: async () => [], counters: async () => ({}) as never, completeReferenced: async () => { completed += 1; return task; } },
  notes: { list: async () => [] },
  databaseInfo: async () => ({ schemaVersion: 1 }),
  system: { snapshot: async () => ({ cpu: { usagePercent: 12 } }) as never, processes: async () => [] },
  ollama: { settings: async () => ({}) as never, status: async () => ({ available: true, models: [], error: null }) },
};

class ScriptedProvider implements AIProvider {
  requests: AgentRequest[] = [];
  constructor(private readonly replies: Array<Partial<AgentResponse> | Error>) {}
  async chat() { return { content: "legado", model: "fake", truncated: false }; }
  async chatWithTools(request: AgentRequest): Promise<AgentResponse> {
    this.requests.push(structuredClone(request));
    const reply = this.replies[Math.min(this.requests.length - 1, this.replies.length - 1)];
    if (reply instanceof Error) throw reply;
    return { content: "", model: "fake", truncated: false, toolCalls: [], ...reply };
  }
  async isAvailable() { return { available: true, models: ["fake"], error: null }; }
  async listModels() { return ["fake"]; }
}

const call = (name: string) => ({ function: { name, arguments: {} } });
const base = (provider: AIProvider) => ({ provider, registry: new ToolRegistry(deps), model: "fake", timeoutSeconds: 30, messages: [{ role: "user" as const, content: "O que tenho hoje?" }], query: "O que tenho hoje?" });

describe("agent loop", () => {
  it("executa as ferramentas escolhidas pelo modelo e devolve a resposta final", async () => {
    const provider = new ScriptedProvider([{ toolCalls: [call("get_today_tasks"), call("get_cpu_status")] }, { content: "Você tem 1 tarefa e a CPU está em 12%." }]);
    const result = await runAgent(base(provider));
    expect(result.content).toBe("Você tem 1 tarefa e a CPU está em 12%.");
    expect(result.toolsUsed).toEqual(["get_today_tasks", "get_cpu_status"]);
    const second = provider.requests[1].messages;
    expect(second.filter((message) => message.role === "tool").map((message) => message.tool_name)).toEqual(["get_today_tasks", "get_cpu_status"]);
    expect(second.find((message) => message.tool_name === "get_today_tasks")?.content).toContain("Revisar contrato");
  });

  it("nunca expõe nem executa ferramentas de escrita", async () => {
    completed = 0;
    const provider = new ScriptedProvider([{ toolCalls: [call("complete_task"), call("rm_rf")] }, { content: "ok" }]);
    await runAgent(base(provider));
    expect(provider.requests[0].tools.map((tool) => tool.function.name)).not.toContain("complete_task");
    expect(completed).toBe(0);
    expect(provider.requests[1].messages.filter((message) => message.role === "tool").every((message) => message.content.includes("indisponível"))).toBe(true);
  });

  it("limita as rodadas e tira as ferramentas na última para forçar resposta", async () => {
    const provider = new ScriptedProvider([{ toolCalls: [call("get_cpu_status")] }]);
    await runAgent(base(provider));
    expect(provider.requests).toHaveLength(AGENT_MAX_STEPS);
    expect(provider.requests.at(-1)?.tools).toEqual([]);
  });
});

describe("AICoreService com agente", () => {
  const settings: AISettings = { provider: "ollama", endpoint: "http://localhost:11434", model: "fake", contextMessageLimit: 8, timeoutSeconds: 30, updatedAt: "" };
  const gateway = (): ConversationGateway & { stored: ConversationMessage[] } => {
    const stored: ConversationMessage[] = [];
    return {
      stored,
      create: async (title) => ({ id: "c1", title, createdAt: "", updatedAt: "" }) as Conversation,
      messages: async () => stored,
      addMessage: async (input) => { const message = { ...input, id: `m${stored.length}`, createdAt: "" }; stored.push(message); return message; },
      saveTaskReferences: async () => 0,
    };
  };

  it("responde pelo agente sem depender de palavra-chave", async () => {
    const provider = new ScriptedProvider([{ toolCalls: [call("get_today_tasks")] }, { content: "Hoje: Revisar contrato." }]);
    const conversations = gateway();
    const exchange = await new AICoreService(provider, new ContextBuilder(new ToolRegistry(deps)), conversations, settings).send("me diz como tá meu dia", null);
    expect(exchange.assistantMessage.content).toBe("Hoje: Revisar contrato.");
    expect(provider.requests[0].messages[0].content).toContain("Data local atual");
  });

  it("volta ao roteador quando o modelo não suporta ferramentas", async () => {
    const provider = new ScriptedProvider([new Error("O modelo 'fake' não suporta ferramentas")]);
    const exchange = await new AICoreService(provider, new ContextBuilder(new ToolRegistry(deps)), gateway(), settings).send("explique o que é RSI", null);
    expect(exchange.assistantMessage.content).toBe("legado");
  });

  it("mantém o fluxo determinístico para demandas numeradas", async () => {
    const provider = new ScriptedProvider([{ content: "não deveria ser usado" }]);
    const exchange = await new AICoreService(provider, new ContextBuilder(new ToolRegistry(deps)), gateway(), settings).send("quais demandas de alta prioridade?", null);
    expect(provider.requests).toHaveLength(0);
    expect(exchange.assistantMessage.content).toContain("Revisar contrato");
  });
});
