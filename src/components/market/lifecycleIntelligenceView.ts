import type { LifecycleEventView, LifecycleHealthTrace, LifecycleOutcomeAggregate } from "../../types";

export interface LifecycleTimelineItem {
  key: string;
  candleIndex: number;
  timestamp: string;
  kind: "EVENT" | "HEALTH";
  label: string;
  detail: string;
}

export function lifecycleTimeline(events: LifecycleEventView[], traces: LifecycleHealthTrace[]): LifecycleTimelineItem[] {
  const changes = traces.filter((trace, index) => index === 0 || traces[index - 1]?.positionHealth !== trace.positionHealth);
  return [
    ...events.map((event) => ({ key: `event-${event.id}`, candleIndex: event.candleIndex, timestamp: event.timestamp, kind: "EVENT" as const, label: event.eventType, detail: `${event.stateBefore} → ${event.stateAfter} · ${event.exposureBeforePct.toFixed(1)}% → ${event.exposureAfterPct.toFixed(1)}%` })),
    ...changes.map((trace) => ({ key: `health-${trace.candleIndex}`, candleIndex: trace.candleIndex, timestamp: trace.timestamp, kind: "HEALTH" as const, label: trace.positionHealth, detail: `${trace.deteriorationLevel} · score ${trace.deteriorationScore.toFixed(3)}` })),
  ].sort((left, right) => left.candleIndex - right.candleIndex || left.kind.localeCompare(right.kind));
}

export function aggregateHasPostDecisionEvidence(row: LifecycleOutcomeAggregate): boolean {
  return row.count > 0 && [row.averageForward1, row.averageForward5, row.averageMfe5, row.averageMae5].every(Number.isFinite);
}
