import { describe, expect, it } from "vitest";
import { ContextBuilder } from "./ContextBuilder";
import type { ToolDependencies } from "../tools/toolRegistry";
import { ToolRegistry } from "../tools/toolRegistry";

const emptyDependencies: ToolDependencies = {
  tasks: { list: async () => [], today: async () => [], upcoming: async () => [], counters: async () => ({ pending: 0, today: 0, overdue: 0, priority: 0, notes: 0 }) },
  notes: { list: async () => [] }, projects: { list: async () => [], get: async () => null },
  knowledge: { list: async () => [], get: async () => null, history: async () => [] },
  education: { list: async () => [] }, databaseInfo: async () => ({ schemaVersion: 5, integrationValue: 0 }),
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
