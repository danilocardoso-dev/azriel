import { describe, expect, it } from "vitest";
import type { Memory } from "../types";
import { ageFrom, buildPersona, capabilitiesOf, formatMemories, loadPersona } from "./persona";
import { ToolRegistry, type ToolDependencies } from "./tools/toolRegistry";

const memory = (id: string, content: string, category: Memory["category"] = "fato"): Memory => ({ id, content, category, source: "azriel", conversationId: null, createdAt: "", updatedAt: "" });

const saved: Memory[] = [];
const deps: ToolDependencies = {
  tasks: { list: async () => [], today: async () => [], upcoming: async () => [], counters: async () => ({}) as never, completeReferenced: async () => ({}) as never },
  notes: { list: async () => [] },
  databaseInfo: async () => ({ schemaVersion: 1 }),
  system: { snapshot: async () => ({}) as never, processes: async () => [] },
  ollama: { settings: async () => ({}) as never, status: async () => ({}) as never },
  memory: {
    list: async () => saved,
    search: async (query) => saved.filter((item) => item.content.includes(query)),
    save: async (input) => { const item = memory(input.id, input.content, input.category); const index = saved.findIndex((m) => m.id === input.id); if (index >= 0) saved[index] = item; else saved.push(item); return item; },
  },
};

describe("persona do Azriel", () => {
  it("calcula a idade a partir da data de nascimento da identidade", () => {
    const identity = "Senhor Danilo, nascido em 25/12/2002.";
    expect(ageFrom(identity, new Date(2026, 10, 15))).toBe(23);
    expect(ageFrom(identity, new Date(2026, 11, 25))).toBe(24);
    expect(ageFrom("sem data", new Date())).toBeNull();
  });

  it("coloca identidade, capacidades, regras e memórias com id", () => {
    const persona = buildPersona({ identity: "QUEM EU SOU\nSou o Azriel. Nascido em 25/12/2002.", memories: [memory("m1", "O Senhor prefere café", "preferencia")], capabilities: ["Memória: Registra"], now: new Date(2026, 10, 15) });
    expect(persona.startsWith("QUEM EU SOU")).toBe(true);
    expect(persona).toContain("Hoje o Senhor tem 23 anos");
    expect(persona).toContain("O QUE POSSO FAZER AGORA\n- Memória: Registra");
    expect(persona).toContain("- [m1] (preferência) O Senhor prefere café");
  });

  it("respeita o teto de memórias e avisa quantas ficaram de fora", () => {
    const many = Array.from({ length: 50 }, (_, index) => memory(`m${index}`, "x".repeat(200)));
    const { shown } = formatMemories(many, 1_000);
    expect(shown).toBeLessThan(50);
    expect(buildPersona({ identity: "Eu", memories: many, capabilities: [], now: new Date() })).toMatch(/\+\d+ memórias mais antigas/);
  });

  it("sem identidade no banco, não inventa persona", async () => {
    expect(await loadPersona(new ToolRegistry(deps), { identity: async () => { throw new Error("offline"); }, memories: async () => [] })).toBe("");
  });
});

describe("ferramentas de memória", () => {
  it("o agente registra e corrige sozinho, mas não apaga", async () => {
    const registry = new ToolRegistry(deps);
    expect(registry.agentCanRun("remember")).toBe(true);
    expect(registry.agentCanRun("update_memory")).toBe(true);
    expect(registry.agentCanRun("complete_task")).toBe(false);
    const created = await registry.execute("remember", { query: "", conversationId: "c1", args: { content: "O Senhor faz aniversário em 25/12", category: "operador" } });
    const id = (created.data as { memoria: { id: string } }).memoria.id;
    expect(saved).toHaveLength(1);
    await registry.execute("update_memory", { query: "", args: { id, content: "O Senhor faz aniversário no Natal" } });
    expect(saved[0].content).toBe("O Senhor faz aniversário no Natal");
    expect(saved[0].category).toBe("operador");
    expect((await registry.execute("update_memory", { query: "", args: { id: "nao-existe", content: "x" } })).data).toEqual({ error: "Memória não encontrada" });
    expect((await registry.execute("search_memories", { query: "", args: { query: "Natal" } })).data).toHaveLength(1);
  });

  it("capacidades listam o que o agente pode usar", () => {
    const capabilities = capabilitiesOf(new ToolRegistry(deps));
    expect(capabilities.some((line) => line.startsWith("Memória:"))).toBe(true);
    expect(capabilities.join(" ")).not.toContain("Conclui uma tarefa");
  });
});
