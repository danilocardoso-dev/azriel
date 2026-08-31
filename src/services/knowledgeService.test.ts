import { describe, expect, it } from "vitest";
import { calculateGap, diagnoseGaps, validateMetrics } from "./knowledgeService";
import type { KnowledgeArea } from "../types";

const area = (id: string, priority: KnowledgeArea["priority"], coverage: number, depth: number): KnowledgeArea => ({
  id, name: id, category: "Teste", description: "", priority, coverage, depth,
  projectIds: [], createdAt: "2026-08-31", updatedAt: "2026-08-31",
});

describe("knowledgeService", () => {
  it("calcula a diferença entre cobertura e profundidade", () => {
    expect(calculateGap(area("physics", "high", 45, 20))).toBe(25);
  });

  it("diagnostica apenas prioridades críticas e altas", () => {
    const result = diagnoseGaps([
      area("medium", "medium", 80, 10), area("high", "high", 40, 20), area("critical", "critical", 20, 5),
    ]);
    expect(result.map((item) => item.id)).toEqual(["critical", "high"]);
  });

  it("rejeita métricas fora do intervalo e motivo vazio", () => {
    expect(() => validateMetrics({ knowledgeId: "ai", coverage: 101, depth: 20, reason: "ok" })).toThrow(/entre 0 e 100/);
    expect(() => validateMetrics({ knowledgeId: "ai", coverage: 60, depth: 20, reason: " " })).toThrow(/motivo/);
  });
});
