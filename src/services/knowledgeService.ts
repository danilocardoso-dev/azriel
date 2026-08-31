import { knowledgeRepository } from "../repositories/knowledgeRepository";
import type { KnowledgeArea, MetricsInput } from "../types";

export const calculateGap = (area: Pick<KnowledgeArea, "coverage" | "depth">) => area.coverage - area.depth;
export function diagnoseGaps(areas: KnowledgeArea[]) {
  const weight = { critical: 4, high: 3, medium: 2, low: 1 };
  return areas.filter((area) => area.priority === "critical" || area.priority === "high")
    .sort((left, right) => weight[right.priority] - weight[left.priority] || calculateGap(right) - calculateGap(left));
}
export function validateMetrics(input: MetricsInput) {
  if (![input.coverage, input.depth].every((value) => Number.isInteger(value) && value >= 0 && value <= 100))
    throw new Error("Cobertura e profundidade devem ser números inteiros entre 0 e 100.");
  if (!input.reason.trim()) throw new Error("Informe o motivo da atualização.");
  return { ...input, reason: input.reason.trim() };
}
export const knowledgeService = {
  list: knowledgeRepository.list, get: knowledgeRepository.get, save: knowledgeRepository.save, remove: knowledgeRepository.remove,
  history: knowledgeRepository.history,
  updateMetrics: (input: MetricsInput) => knowledgeRepository.updateMetrics(validateMetrics(input)),
};
