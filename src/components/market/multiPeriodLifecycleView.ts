import type { DeteriorationLevel, LifecycleValidationLifecycle } from "../../types";

export interface LifecycleValidationFilters {
  periodId: string;
  status: "ALL" | "OPEN" | "CLOSED";
  deterioration: "ALL" | DeteriorationLevel;
  lateReduction: "ALL" | "YES" | "NO";
  overnight: "ALL" | "YES" | "NO";
}

const levelRank: Record<DeteriorationLevel, number> = { NONE: 0, LOW: 1, MODERATE: 2, HIGH: 3, CRITICAL: 4 };

export function filterValidationLifecycles(rows: LifecycleValidationLifecycle[], filters: LifecycleValidationFilters) {
  return rows.filter((row) =>
    (!filters.periodId || row.periodId === filters.periodId)
    && (filters.status === "ALL" || row.status === filters.status)
    && (filters.deterioration === "ALL" || levelRank[row.maxDeteriorationLevel] >= levelRank[filters.deterioration])
    && (filters.lateReduction === "ALL" || row.hasLateReduction === (filters.lateReduction === "YES"))
    && (filters.overnight === "ALL" || row.overnight === (filters.overnight === "YES"))
  );
}

export function chartBars(values: number[], buckets = 8): number[] {
  if (values.length === 0) return Array.from({ length: buckets }, () => 0);
  const minimum = Math.min(...values);
  const maximum = Math.max(...values);
  if (minimum === maximum) return Array.from({ length: buckets }, (_, index) => index === Math.floor(buckets / 2) ? values.length : 0);
  const width = (maximum - minimum) / buckets;
  const result = Array.from({ length: buckets }, () => 0);
  values.forEach((value) => {
    const index = Math.min(buckets - 1, Math.floor((value - minimum) / width));
    result[index] += 1;
  });
  return result;
}

export function responseLabel(value: number | null, status: LifecycleValidationLifecycle["responseStatus"]) {
  if (status === "UNRESOLVED") return "UNRESOLVED";
  if (status === "NOT_APPLICABLE") return "N/A";
  return value == null ? "N/A" : `${value} min`;
}
