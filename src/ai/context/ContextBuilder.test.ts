import { describe, expect, it } from "vitest";
import { ContextBuilder } from "./ContextBuilder";
import type { ToolDependencies } from "../tools/toolRegistry";
import { ToolRegistry } from "../tools/toolRegistry";

const settings = { provider: "ollama" as const, endpoint: "http://localhost:11434", model: "qwen", contextMessageLimit: 6, timeoutSeconds: 30, updatedAt: "" };

const emptyDependencies: ToolDependencies = {
  tasks: { list: async () => [], today: async () => [], upcoming: async () => [], counters: async () => ({ pending: 0, today: 0, overdue: 0, priority: 0, notes: 0 }) },
  notes: { list: async () => [] }, projects: { list: async () => [], get: async () => null },
  knowledge: { list: async () => [], get: async () => null, history: async () => [] },
  education: { list: async () => [] }, databaseInfo: async () => ({ schemaVersion: 5, integrationValue: 0 }),
  system: {
    snapshot: async () => ({ collectedAt: 0, details: { osName: "Windows", osVersion: "11", kernelVersion: "", architecture: "x86_64", hostname: "azriel", logicalCores: 8, physicalCores: 4, uptimeSeconds: 100 }, cpu: { usagePercent: 10, cores: [10] }, memory: { totalBytes: 1000, usedBytes: 500, availableBytes: 500, swapTotalBytes: 0, swapUsedBytes: 0 }, storage: [], network: [], errors: [] }),
    processes: async () => [], listWorkspaces: async () => [], workspaceStatus: async () => { throw new Error("workspace ausente"); },
  },
  ollama: { settings: async () => settings, status: async () => ({ available: true, models: [settings.model], error: null }) },
  automation: { listApplications: async () => [], listUrls: async () => [], listRoutines: async () => [], runRoutine: async (request) => ({ success: false, status: "failed", routineId: request.routineId, routineName: "", historyId: 1, completedSteps: 0, failedStep: null, error: "não registrada", confirmation: null }), execute: async (request) => ({ success: false, message: "não registrado", errorCode: "TARGET_NOT_FOUND", actionId: request.actionId, targetName: null, historyId: 1, confirmation: null }) },
};

describe("Context Builder", () => {
  it("executa somente as tools roteadas e limita o contexto", async () => {
    const context = await new ContextBuilder(new ToolRegistry(emptyDependencies), 400).build("projetos", { intent: "projects", scope: "azriel", tools: ["list_projects"] });
    expect(context.results).toHaveLength(1);
    expect(context.results[0].name).toBe("list_projects");
    expect(context.empty).toBe(true);
    expect(context.text.length).toBeLessThanOrEqual(440);
  });
});
