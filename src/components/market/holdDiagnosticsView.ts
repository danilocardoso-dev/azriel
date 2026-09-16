import type { HoldDiagnosticsReport, HoldQuality } from "../../types";

export const qualityLabel = (quality: HoldQuality, positionState?: "FLAT" | "LONG") => {
  if (quality === "GOOD_HOLD" && positionState === "FLAT") return "GOOD AVOIDANCE";
  return ({
    GOOD_HOLD: "GOOD HOLD",
    POTENTIAL_MISSED_OPPORTUNITY: "POTENTIAL MISSED OPPORTUNITY",
    POTENTIAL_LATE_REDUCTION: "POTENTIAL LATE REDUCTION",
    INCONCLUSIVE: "INCONCLUSIVE",
  } as const)[quality];
};

export const qualityRate = (report: HoldDiagnosticsReport, count: number) =>
  report.totalHolds === 0 ? 0 : (count / report.totalHolds) * 100;

export const diagnosticAvailability = (agentIds: string[], timeframe: string, status: string) =>
  agentIds.includes("ai-intraday-v1") && timeframe === "15M" && status === "completed";
