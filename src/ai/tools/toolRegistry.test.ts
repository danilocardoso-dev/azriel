import { describe, expect, it } from "vitest";
import type { ToolDependencies } from "./toolRegistry";
import { ToolRegistry } from "./toolRegistry";
import { FakeEngineeringService } from "../../engineering/fakeEngineeringService";

const dependencies: ToolDependencies = {
  tasks: {
    list: async () => [{ id: "late", title: "Atrasada", description: "", status: "pending", priority: "high", dueDate: "2026-08-30", projectId: "p1", knowledgeAreaId: "k1", createdAt: "", updatedAt: "", completedAt: null }],
    today: async () => [], upcoming: async () => [],
    counters: async () => ({ pending: 1, today: 0, overdue: 1, priority: 1, notes: 0, completed: 0 }),
  },
  notes: { list: async () => [] },
  projects: { list: async () => [{ id: "p1", name: "GeneScope", category: "bio", description: "", status: "active", knowledgeAreaIds: ["k1"], objective: "", progress: 20, nextStep: "", createdAt: "", updatedAt: "" }], get: async () => null },
  knowledge: { list: async () => [{ id: "k1", name: "Genética", category: "bio", description: "", coverage: 60, depth: 20, priority: "high", nodeType: "area", parentId: null, projectIds: ["p1"], createdAt: "", updatedAt: "" }], get: async () => null, history: async () => [] },
  education: { list: async () => [] },
  databaseInfo: async () => ({ schemaVersion: 5, integrationValue: 1 }),
  system: {
    snapshot: async () => ({ collectedAt: 0, details: { osName: "Windows", osVersion: "11", kernelVersion: "", architecture: "x86_64", hostname: "azriel", logicalCores: 8, physicalCores: 4, uptimeSeconds: 100 }, cpu: { usagePercent: 10, cores: [10] }, memory: { totalBytes: 1000, usedBytes: 500, availableBytes: 500, swapTotalBytes: 0, swapUsedBytes: 0 }, storage: [], network: [], errors: [] }),
    processes: async () => [{ pid: 1, name: "ollama", cpuPercent: 5, memoryBytes: 100 }],
    listWorkspaces: async () => [{ id: "w1", name: "Azriel", path: "C:\\azriel", projectId: "p1", applicationId: "code", enabled: true, createdAt: "", updatedAt: "" }],
    workspaceStatus: async () => ({ workspace: { id: "w1", name: "Azriel", path: "C:\\azriel", projectId: "p1", applicationId: "code", enabled: true, createdAt: "", updatedAt: "" }, pathAvailable: true, entryCount: 10, git: null, error: null }),
  },
  ollama: { settings: async () => ({ provider: "ollama", endpoint: "http://localhost:11434", model: "qwen", contextMessageLimit: 6, timeoutSeconds: 30, updatedAt: "" }), status: async () => ({ available: true, models: ["qwen"], error: null }) },
  automation: {
    listApplications: async () => [{ id: "code", name: "Visual Studio Code", path: "C:\\Code.exe", enabled: true, createdAt: "", updatedAt: "" }],
    listUrls: async () => [{ id: "github", name: "GitHub do Azriel", url: "https://example.com", enabled: true, createdAt: "", updatedAt: "" }],
    execute: async (request) => ({ success: true, message: "ok", errorCode: null, actionId: request.actionId, targetName: "alvo", historyId: 1, confirmation: null }),
    listRoutines: async () => [{ id: "dev", name: "Ambiente de Desenvolvimento", description: "Code e workspace", enabled: true, confirmationRequired: true, revision: 1, steps: [], createdAt: "", updatedAt: "" }],
    runRoutine: async (request) => ({ success: false, status: "waiting_confirmation", routineId: request.routineId, routineName: "Ambiente de Desenvolvimento", historyId: 2, completedSteps: 0, failedStep: null, error: null, confirmation: { historyId: 2, routineId: request.routineId, routineName: "Ambiente de Desenvolvimento", revision: 1, actions: [] } }),
  },
};

describe("Tool Registry", () => {
  it("expõe consultas, cinco safe actions e execução controlada de rotina", () => {
    const tools = new ToolRegistry(dependencies).list();
    expect(tools).toHaveLength(66);
    expect(tools.filter((tool) => tool.permission === "visual_action")).toHaveLength(11);
    expect(tools.filter((tool) => tool.domain === "Engineering Core" && tool.readonly)).toHaveLength(7);
    expect(tools.filter((tool) => tool.domain === "Assembly Intelligence" && tool.readonly)).toHaveLength(7);
    expect(tools.filter((tool) => !tool.readonly).map((tool) => tool.name)).toEqual([
      "select_component", "focus_component", "isolate_component", "show_all_components", "hide_component", "show_component",
      "set_explosion_factor", "explode_all", "explode_component", "reassemble", "reset_model_view",
      "run_routine", "open_application", "open_workspace", "open_project", "reveal_workspace", "open_registered_url",
    ]);
    expect(tools.find((tool) => tool.name === "run_routine")?.permission).toBe("confirm_write");
  });
  it("executa rotina somente pelo ID resolvido no registro", async () => {
    const calls: unknown[] = [];
    const registry = new ToolRegistry({ ...dependencies, automation: { ...dependencies.automation, runRoutine: async (request) => { calls.push(request); return dependencies.automation.runRoutine(request); } } });
    await registry.execute("run_routine", { query: "Execute a rotina Ambiente de Desenvolvimento" });
    expect(calls).toEqual([{ routineId: "dev", source: "ai" }]);
  });
  it("resolve ações por ID e não envia caminho ou URL no request", async () => {
    const calls: unknown[] = [];
    const registry = new ToolRegistry({ ...dependencies, automation: { ...dependencies.automation, execute: async (request) => { calls.push(request); return { success: true, message: "ok", errorCode: null, actionId: request.actionId, targetName: "alvo", historyId: 1, confirmation: null }; } } });
    await registry.execute("open_application", { query: "Abra o Visual Studio Code." });
    await registry.execute("open_registered_url", { query: "Abra o GitHub do Azriel." });
    expect(calls).toEqual([
      { actionId: "open_application", source: "ai", targetId: "code" },
      { actionId: "open_registered_url", source: "ai", targetId: "github" },
    ]);
    expect(JSON.stringify(calls)).not.toContain("Code.exe");
    expect(JSON.stringify(calls)).not.toContain("https://");
  });
  it("não escolhe silenciosamente entre dois alvos diferentes", async () => {
    const calls: unknown[] = [];
    const registry = new ToolRegistry({ ...dependencies, automation: { ...dependencies.automation, listApplications: async () => [
      { id: "code", name: "Visual Studio Code", path: "C:\\Code.exe", enabled: true, createdAt: "", updatedAt: "" },
      { id: "chrome", name: "Chrome", path: "C:\\Chrome.exe", enabled: true, createdAt: "", updatedAt: "" },
    ], execute: async (request) => { calls.push(request); return { success: false, message: "alvo não resolvido", errorCode: "TARGET_NOT_FOUND", actionId: request.actionId, targetName: null, historyId: 2, confirmation: null }; } } });
    await registry.execute("open_application", { query: "Abra Visual Studio Code e Chrome" });
    expect(calls).toEqual([{ actionId: "open_application", source: "ai", targetId: "unregistered" }]);
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
  it("consulta roadmaps, pesquisas e origem sem criar evolução", async () => {
    const registry = new ToolRegistry({ ...dependencies, stark: {
      roadmaps: async () => [
        { id: "control", name: "Controle e Automação", description: "", status: "active", completedActivities: 1, totalActivities: 2, progress: 50, stages: [], createdAt: "", updatedAt: "" },
        { id: "future", name: "Futuro", description: "", status: "planned", completedActivities: 0, totalActivities: 0, progress: 0, stages: [], createdAt: "", updatedAt: "" },
      ],
      research: async () => [{ id: "pid", title: "Controle PID", domain: "Controle", objective: "", description: "", kind: "research", status: "active", impact: "", knowledgeNodeId: "k1", roadmapId: "control", roadmapTopicId: null, projectId: "p1", createdAt: "", updatedAt: "" }],
      baselines: async () => [{ knowledgeAreaId: "k1", coverage: 60, depth: 20, recordedAt: "" }], events: async () => [],
    } });
    expect((await registry.execute("list_study_roadmaps", { query: "ativos" })).data).toEqual([expect.objectContaining({ id: "control" })]);
    expect((await registry.execute("list_research_items", { query: "pesquisas" })).empty).toBe(false);
    expect(await registry.execute("get_knowledge_origin", { query: "Genética", term: "genética" })).toMatchObject({ data: { baseline: { coverage: 60, depth: 20 }, automaticLearningEnabled: false } });
  });
  it("não envia caminhos ao listar workspaces para a IA", async () => {
    const result = await new ToolRegistry(dependencies).execute("list_workspaces", { query: "workspaces" });
    expect(result.data).toEqual([{ id: "w1", name: "Azriel", projectId: "p1", enabled: true }]);
  });
  it("executa visual actions somente pelo gateway controlado do Engineering Core", async () => {
    const engineering = new FakeEngineeringService();
    const registry = new ToolRegistry({ ...dependencies, engineering });
    await registry.execute("select_component", { query: "Selecione o rotor", term: "rotor" });
    await registry.execute("set_explosion_factor", { query: "Abra em 50%", factor: 0.5 });
    expect(engineering.calls).toEqual([
      { command: "select_component", value: "component-rotor" },
      { command: "set_explosion_factor", value: 0.5 },
    ]);
  });
  it("expõe Assembly Intelligence somente por tools de leitura", () => {
    const tools = new ToolRegistry({ ...dependencies, engineering: new FakeEngineeringService() }).list().filter((tool) => tool.domain === "Assembly Intelligence");
    expect(tools).toHaveLength(7);
    expect(tools.every((tool) => tool.readonly && tool.permission === "read")).toBe(true);
  });
});
