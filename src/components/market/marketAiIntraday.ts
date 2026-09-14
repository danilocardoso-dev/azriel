import type { MarketAiDecisionLog } from "../../types";

export interface AiDecisionComparison {
  left: string;
  right: string;
  opportunities: number;
  disagreements: number;
  rate: number;
}

export function compareAiDecisions(
  aiIds: string[],
  decisions: MarketAiDecisionLog[],
): AiDecisionComparison | null {
  if (aiIds.length < 2) return null;
  const [left, right] = aiIds;
  const actionable = (agentId: string) => new Map(
    decisions
      .filter((item) => item.agentId === agentId && item.callStatus !== "NO_LLM_CALL")
      .map((item) => [item.timestamp, item]),
  );
  const leftDecisions = actionable(left);
  const rightDecisions = actionable(right);
  const comparable = [...leftDecisions.keys()].filter((timestamp) => rightDecisions.has(timestamp));
  const disagreements = comparable.filter(
    (timestamp) => leftDecisions.get(timestamp)?.intent !== rightDecisions.get(timestamp)?.intent,
  ).length;
  return {
    left,
    right,
    opportunities: comparable.length,
    disagreements,
    rate: comparable.length === 0 ? 0 : disagreements / comparable.length * 100,
  };
}
