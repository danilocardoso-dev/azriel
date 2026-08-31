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
  system: {
    snapshot: async () => ({ collectedAt: 0, details: { osName: "Windows", osVersion: "11", kernelVersion: "", architecture: "x86_64", hostname: "azriel", logicalCores: 8, physicalCores: 4, uptimeSeconds: 100 }, cpu: { usagePercent: 10, cores: [10] }, memory: { totalBytes: 1000, usedBytes: 500, availableBytes: 500, swapTotalBytes: 0, swapUsedBytes: 0 }, storage: [], network: [], errors: [] }),
    processes: async () => [{ pid: 1, name: "ollama", cpuPercent: 5, memoryBytes: 100 }],
    listWorkspaces: async () => [{ id: "w1", name: "Azriel", path: "C:\\azriel", projectId: "p1", enabled: true, createdAt: "", updatedAt: "" }],
    workspaceStatus: async () => ({ workspace: { id: "w1", name: "Azriel", path: "C:\\azriel", projectId: "p1", enabled: true, createdAt: "", updatedAt: "" }, pathAvailable: true, entryCount: 10, git: null, error: null }),
  },
  ollama: { settings: async () => ({ provider: "ollama", endpoint: "http://localhost:11434", model: "qwen", contextMessageLimit: 6, timeoutSeconds: 30, updatedAt: "" }), status: async () => ({ available: true, models: ["qwen"], error: null }) },
};

describe("Tool Registry", () => {
  it("expõe as 30 tools somente como read-only", () => {
    const tools = new ToolRegistry(dependencies).list();
    expect(tools).toHaveLength(30);
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
  it("não envia caminhos ao listar workspaces para a IA", async () => {
    const result = await new ToolRegistry(dependencies).execute("list_workspaces", { query: "workspaces" });
    expect(result.data).toEqual([{ id: "w1", name: "Azriel", projectId: "p1", enabled: true }]);
  });
});
