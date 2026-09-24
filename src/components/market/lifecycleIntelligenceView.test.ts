import { describe, expect, it } from "vitest";
import type { LifecycleEventView, LifecycleHealthTrace } from "../../types";
import { aggregateHasPostDecisionEvidence, lifecycleTimeline } from "./lifecycleIntelligenceView";

const trace = (candleIndex: number, positionHealth: LifecycleHealthTrace["positionHealth"]): LifecycleHealthTrace => ({ lifecycleId: "life-1", candleIndex, timestamp: `2026-01-01T10:${String(candleIndex).padStart(2, "0")}:00Z`, positionAgeCandles: candleIndex, positionAgeMarketMinutes: candleIndex * 15, marketPrice: 100, exposurePct: 25, returnSinceEntryPct: 0, mfeSinceEntryPct: 0, maeSinceEntryPct: 0, distanceFromEntryAtr: null, distanceFromMfePct: 0, givebackFromMfePct: 0, givebackRelativePct: null, vwapChangeSinceEntry: null, emaSpreadChangeSinceEntry: null, rsiChangeSinceEntry: null, trendChanged: false, momentumChanged: false, trendComponent: 0, momentumComponent: 0, givebackComponent: 0, vwapComponent: 0, volatilityComponent: 0, timeComponent: 0, deteriorationScore: 0, deteriorationLevel: "NONE", positionHealth, availableAtT: true });

const event: LifecycleEventView = { id: 1, lifecycleId: "life-1", decisionId: 1, executionId: 1, timestamp: "2026-01-01T10:02:00Z", candleIndex: 2, eventType: "OPEN", stateBefore: "FLAT", stateAfter: "LONG", positionBefore: "FLAT", positionAfter: "LONG", exposureBeforePct: 0, exposureAfterPct: 25, marketPrice: 100, unrealizedPnlPct: 0, mfeSoFarPct: 0, maeSoFarPct: 0, aiIntent: "ENTER", confidence: .8, reasonCode: "TREND", triggerReason: "SIGNAL", riskResult: "APPROVED", executionResult: "FILLED" };

describe("lifecycle intelligence view", () => {
  it("keeps event and available-at-T health changes in chronological order", () => {
    const result = lifecycleTimeline([event], [trace(1, "HEALTHY"), trace(2, "HEALTHY"), trace(3, "WEAKENING")]);
    expect(result.map((item) => item.label)).toEqual(["HEALTHY", "OPEN", "WEAKENING"]);
  });

  it("only labels populated finite post-decision aggregates as evidence", () => {
    expect(aggregateHasPostDecisionEvidence({ label: "HIGH", count: 2, averageForward1: -.1, averageForward5: -.2, averageMfe5: .1, averageMae5: -.4 })).toBe(true);
    expect(aggregateHasPostDecisionEvidence({ label: "HIGH", count: 0, averageForward1: 0, averageForward5: 0, averageMfe5: 0, averageMae5: 0 })).toBe(false);
  });
});
