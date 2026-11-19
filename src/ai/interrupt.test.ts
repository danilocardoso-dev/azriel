import { describe, expect, it } from "vitest";
import type { AgentRequest, AgentResponse, AISettings, Conversation, ConversationMessage, Memory } from "../types";
import { AICoreService, type ConversationGateway } from "./AICoreService";
import { ContextBuilder } from "./context/ContextBuilder";
import { abortable, InterruptedError } from "./interrupt";
import type { AIProvider } from "./providers/AIProvider";
import { ToolRegistry, type ToolDependencies } from "./tools/toolRegistry";

const settings = { model: "fake", timeoutSeconds: 30, contextMessageLimit: 8 } as AISettings;
const memories: Memory[] = [];
const deps = {
  tasks: { list: async () => [], today: async () => [], upcoming: async () => [], counters: async () => ({}) as never, completeReferenced: async () => ({}) as never },
  notes: { list: async () => [] },
  databaseInfo: async () => ({ schemaVersion: 1 }),
  system: { snapshot: async () => ({}) as never, processes: async () => [] },
  ollama: { settings: async () => ({}) as never, status: async () => ({}) as never },
  memory: { list: async () => memories, search: async () => [], save: async (input) => { const memory = { ...input, conversationId: null, createdAt: "", updatedAt: "" } as Memory; memories.push(memory); return memory; } },
} satisfies ToolDependencies;

/** Responde o roteiro e, quando acaba, fica "pensando" para sempre (até o ESC). */
class HangingProvider implements AIProvider {
  calls = 0;
  constructor(private readonly replies: Array<Partial<AgentResponse>>) {}
  async chat() { return new Promise<never>(() => undefined); }
  async chatWithTools(request: AgentRequest): Promise<AgentResponse> {
    void request;
    const reply = this.replies[this.calls++];
    if (!reply) return new Promise<never>(() => undefined);
    return { content: "", model: "fake", truncated: false, toolCalls: [], ...reply };
  }
  async isAvailable() { return { available: true, models: [], error: null }; }
  async listModels() { return []; }
}

const gateway = (): ConversationGateway & { stored: ConversationMessage[] } => {
  const stored: ConversationMessage[] = [];
  return {
    stored,
    create: async (title) => ({ id: "c1", title, createdAt: "", updatedAt: "" }) as Conversation,
    messages: async () => stored,
    addMessage: async (input) => { const message = { ...input, id: `m${stored.length}`, createdAt: "" }; stored.push(message); return message; },
    saveTaskReferences: async () => 0,
    deleteMessage: async (id) => { const index = stored.findIndex((message) => message.id === id); if (index >= 0) stored.splice(index, 1); },
  };
};

const until = async (condition: () => boolean) => { for (let i = 0; i < 200 && !condition(); i += 1) await new Promise((resolve) => setTimeout(resolve, 5)); };

describe("ESC / Parar", () => {
  it("abortable encerra a espera na hora", async () => {
    const controller = new AbortController();
    const waiting = abortable(new Promise(() => undefined), controller.signal);
    controller.abort();
    await expect(waiting).rejects.toBeInstanceOf(InterruptedError);
  });

  it("parado antes de qualquer ação: a mensagem some e o texto volta para o campo", async () => {
    const conversations = gateway();
    const provider = new HangingProvider([]);
    const controller = new AbortController();
    const sending = new AICoreService(provider, new ContextBuilder(new ToolRegistry(deps)), conversations, settings).send("escrevi erado", null, undefined, undefined, undefined, controller.signal);
    await until(() => provider.calls > 0);
    controller.abort();
    const error = await sending.catch((reason: unknown) => reason);
    expect(error).toBeInstanceOf(InterruptedError);
    expect((error as InterruptedError).restoredText).toBe("escrevi erado");
    expect(conversations.stored).toEqual([]);
  });

  it("parado depois de agir: a mensagem fica, com o aviso do que já foi feito", async () => {
    memories.length = 0;
    const conversations = gateway();
    const provider = new HangingProvider([{ toolCalls: [{ function: { name: "remember", arguments: { content: "O Senhor gosta de café", category: "preferencia" } } }] }]);
    const controller = new AbortController();
    const sending = new AICoreService(provider, new ContextBuilder(new ToolRegistry(deps)), conversations, settings).send("lembre que gosto de café", null, undefined, undefined, undefined, controller.signal);
    await until(() => provider.calls > 1);
    controller.abort();
    const error = await sending.catch((reason: unknown) => reason) as InterruptedError;
    expect(error.restoredText).toBeNull();
    expect(memories).toHaveLength(1);
    expect(conversations.stored.map((message) => message.role)).toEqual(["user", "assistant"]);
    expect(conversations.stored[1].content).toContain("Interrompido pelo Senhor");
    expect(conversations.stored[1].content).toContain("registrar memória");
  });
});
