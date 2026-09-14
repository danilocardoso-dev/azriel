import { describe, expect, it } from "vitest";
import type { MarketAiDecisionLog } from "../../types";
import { compareAiDecisions } from "./marketAiIntraday";

const decision = (
  agentId: string,
  timestamp: string,
  intent: string,
  callStatus = "VALID",
) => ({ agentId, timestamp, intent, callStatus }) as MarketAiDecisionLog;

describe("compareAiDecisions", () => {
  it("compara apenas oportunidades em que ambas as IAs foram chamadas", () => {
    const result = compareAiDecisions(
      ["ai-technical-v4-2", "ai-intraday-v1"],
      [
        decision("ai-technical-v4-2", "T1", "ENTER"),
        decision("ai-intraday-v1", "T1", "HOLD"),
        decision("ai-technical-v4-2", "T2", "HOLD"),
        decision("ai-intraday-v1", "T2", "HOLD"),
        decision("ai-technical-v4-2", "T3", "EXIT"),
        decision("ai-intraday-v1", "T3", "EXIT", "NO_LLM_CALL"),
      ],
    );

    expect(result).toEqual({
      left: "ai-technical-v4-2",
      right: "ai-intraday-v1",
      opportunities: 2,
      disagreements: 1,
      rate: 50,
    });
  });

  it("não cria comparação quando existe somente uma IA", () => {
    expect(compareAiDecisions(["ai-intraday-v1"], [])).toBeNull();
  });
});
