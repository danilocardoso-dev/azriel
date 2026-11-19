import { describe, expect, it } from "vitest";
import type { AgentRequest, AISettings, ConversationMessage } from "../types";
import { AICoreService, withSingleSystem } from "./AICoreService";
import { ContextBuilder } from "./context/ContextBuilder";
import type { AIProvider } from "./providers/AIProvider";
import { ToolRegistry, type ToolDependencies } from "./tools/toolRegistry";

describe("mensagem de sistema única", () => {
  it("junta todas as mensagens de sistema no topo, na ordem", () => {
    const merged = withSingleSystem([
      { role: "system", content: "IDENTIDADE" },
      { role: "system", content: "REGRAS" },
      { role: "user", content: "oi" },
      { role: "system", content: "RETENTATIVA" },
    ]);
    expect(merged).toEqual([{ role: "system", content: "IDENTIDADE\n\nREGRAS\n\nRETENTATIVA" }, { role: "user", content: "oi" }]);
  });

  it("não mexe quando já há uma só no topo", () => {
    const messages = [{ role: "system", content: "A" }, { role: "user", content: "b" }];
    expect(withSingleSystem(messages)).toBe(messages);
  });

  it("o agente recebe a identidade dentro da única mensagem de sistema", async () => {
    const stored: ConversationMessage[] = [];
    const requests: AgentRequest[] = [];
    const provider: AIProvider = {
      chat: async () => ({ content: "", truncated: false }) as never,
      chatWithTools: async (request: AgentRequest) => { requests.push(request); return { content: "Às suas ordens, Senhor.", toolCalls: [], truncated: false } as never; },
    } as unknown as AIProvider;
    const conversations = {
      create: async () => ({ id: "c1", title: "t", createdAt: "", updatedAt: "" }),
      messages: async () => stored,
      addMessage: async (input: { conversationId: string; role: "user" | "assistant"; content: string }) => { const message = { id: String(stored.length), createdAt: "", ...input } as ConversationMessage; stored.push(message); return message; },
      saveTaskReferences: async () => 0,
      deleteMessage: async (id: string) => { const index = stored.findIndex((message) => message.id === id); if (index >= 0) stored.splice(index, 1); },
    };
    const registry = new ToolRegistry({} as ToolDependencies);
    const settings = { model: "m", timeoutSeconds: 10, contextMessageLimit: 10 } as AISettings;
    const service = new AICoreService(provider, new ContextBuilder(registry), conversations, settings, async () => "Sou o Azriel do Senhor Danilo.");
    await service.send("Quem é você?", null);
    const system = requests[0].messages.filter((message) => message.role === "system");
    expect(system).toHaveLength(1);
    expect(requests[0].messages[0].role).toBe("system");
    expect(system[0].content.startsWith("Sou o Azriel do Senhor Danilo.")).toBe(true);
    expect(system[0].content).toContain("REGRAS DE OPERAÇÃO");
  });
});
