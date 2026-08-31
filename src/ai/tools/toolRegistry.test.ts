import { describe, expect, it } from "vitest";
import type { ToolDependencies } from "./toolRegistry";
import { ToolRegistry } from "./toolRegistry";

const dependencies: ToolDependencies = {
  tasks: {
    list: async () => [{ id: "late", title: "Atrasada", description: "", status: "pending", priority: "high", dueDate: "2026-08-30", projectId: "p1", knowledgeAreaId: "k1", createdAt: "", updatedAt: "", completedAt: null }],
    today: async () => [], upcoming: async () => [],
    counters: async () => ({ pending: 1, today: 0, overdue: 1, priority: 1, notes: 0 }),
  },
  notes: { list: async () => [] },
  projects: { list: async () => [{ id: "p1", name: "GeneScope", category: "bio", description: "", status: "active", knowledgeAreaIds: ["k1"], objective: "", progress: 20, nextStep: "", createdAt: "", updatedAt: "" }], get: async () => null },
  knowledge: { list: async () => [{ id: "k1", name: "Genética", category: "bio", description: "", coverage: 60, depth: 20, priority: "high", projectIds: ["p1"], createdAt: "", updatedAt: "" }], get: async () => null, history: async () => [] },
  education: { list: async () => [] },
  databaseInfo: async () => ({ schemaVersion: 5, integrationValue: 1 }),
};

describe("Tool Registry", () => {
  it("expõe as 19 tools iniciais somente como read-only", () => {
    const tools = new ToolRegistry(dependencies).list();
    expect(tools).toHaveLength(19);
    expect(tools.every((tool) => tool.readonly)).toBe(true);
  });
  it("calcula tarefas atrasadas sem delegar ao modelo", async () => {
    const result = await new ToolRegistry(dependencies).execute("get_overdue_tasks", { query: "atrasadas" });
    expect(result.empty).toBe(false);
    expect(result.data).toHaveLength(1);
  });
  it("recupera conhecimentos por relação persistida", async () => {
    const result = await new ToolRegistry(dependencies).execute("get_project_knowledge", { query: "GeneScope", term: "GeneScope" });
    expect(result.data).toEqual([expect.objectContaining({ name: "Genética" })]);
  });
});
