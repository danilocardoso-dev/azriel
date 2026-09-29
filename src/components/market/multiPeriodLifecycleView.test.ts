import { describe, expect, it } from "vitest";
import type { LifecycleValidationLifecycle } from "../../types";
import { chartBars, filterValidationLifecycles, responseLabel } from "./multiPeriodLifecycleView";

const row = (overrides: Partial<LifecycleValidationLifecycle> = {}): LifecycleValidationLifecycle => ({
  periodId: "p1", sourceRole: "DEVELOPMENT", experimentId: "e1", lifecycleId: "l1", entryAt: "2026-01-01", exitAt: null,
  status: "OPEN", durationMinutes: 60, pnlPct: null, mfePct: 2, maePct: -1, givebackPct: 1, worstHealth: "WEAKENING",
  dominantHealth: "HEALTHY", healthDistribution: { strongPct: 0, healthyPct: 80, weakeningPct: 20, deterioratingPct: 0, criticalPct: 0 },
  maxDeteriorationScore: .5, maxDeteriorationLevel: "MODERATE", firstTimestampAtMaxLevel: "2026-01-01", lateReductionEvents: 0,
  hasLateReduction: false, responseDelayMinutes: null, responseStatus: "UNRESOLVED", responseFromFirstDeteriorationMinutes: null,
  responseFromHighMinutes: null, responseFromCriticalMinutes: null, overnight: false, censoredAtDatasetEnd: true,
  averageComponents: { trend: 0, momentum: 0, giveback: 0, vwap: 0, volatility: 0, time: 0 },
  maximumComponents: { trend: 0, momentum: 0, giveback: 0, vwap: 0, volatility: 0, time: 0 },
  componentAtFirstHigh: null, componentAtFirstCritical: null, dominantDeteriorationComponent: "GIVEBACK", ...overrides,
});

describe("multi-period lifecycle view", () => {
  it("filters by period, lifecycle state, deterioration reach and observational flags", () => {
    const rows = [row(), row({ lifecycleId: "l2", periodId: "p2", status: "CLOSED", maxDeteriorationLevel: "HIGH", hasLateReduction: true, overnight: true })];
    expect(filterValidationLifecycles(rows, { periodId: "p2", status: "CLOSED", deterioration: "HIGH", lateReduction: "YES", overnight: "YES" }).map((item) => item.lifecycleId)).toEqual(["l2"]);
  });

  it("keeps unresolved response explicit and builds deterministic chart buckets", () => {
    expect(responseLabel(null, "UNRESOLVED")).toBe("UNRESOLVED");
    expect(chartBars([1, 2, 3, 4], 4).reduce((sum, value) => sum + value, 0)).toBe(4);
  });
});
