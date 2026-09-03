import { describe, expect, it } from "vitest";
import { calculateRoadmapProgress, normalizeRoadmapOrder } from "./starkService";
import type { StudyRoadmapInput } from "../types";

const input: StudyRoadmapInput = { id: "roadmap", name: "  Controle  ", description: "  Base  ", status: "active", stages: [{ id: "stage", name: "Eletrônica", description: "", order: 9, topics: [{ id: "topic", name: "MOSFET", description: "", knowledgeNodeId: "electronics", state: "EXPOSED", order: 8, activities: [{ id: "a1", title: "Ler", description: "", activityType: "READING", status: "completed", completedAt: "2026-09-03", order: 7 }, { id: "a2", title: "Simular", description: "", activityType: "SIMULATION", status: "pending", completedAt: null, order: 6 }] }] }] };

describe("starkService", () => {
  it("calcula progresso apenas por atividades concluídas", () => expect(calculateRoadmapProgress(input)).toEqual({ completed: 1, total: 2, progress: 50 }));
  it("normaliza ordens hierárquicas e textos", () => {
    const result = normalizeRoadmapOrder(input);
    expect(result.name).toBe("Controle"); expect(result.description).toBe("Base");
    expect(result.stages[0].order).toBe(1); expect(result.stages[0].topics[0].order).toBe(1); expect(result.stages[0].topics[0].activities.map((item) => item.order)).toEqual([1, 2]);
  });
  it("não confunde progresso vazio com conhecimento", () => expect(calculateRoadmapProgress({ stages: [] })).toEqual({ completed: 0, total: 0, progress: 0 }));
});
