import { describe, expect, it } from "vitest";
import type { MarketEquityPoint } from "../../types";
import { annualizationFor, downsampleEquity } from "./marketIntraday";

describe("Market Lab intraday utilities", () => {
  it("derives annualization from the timeframe", () => {
    expect(annualizationFor("1D")).toBe(252);
    expect(annualizationFor("15M")).toBe(6552);
  });

  it("downsamples only the visual series and preserves its endpoints", () => {
    const points: MarketEquityPoint[] = Array.from({ length: 5_000 }, (_, index) => ({ timestamp: String(index), agentId: "agent", equity: index, exposurePct: 0 }));
    const result = downsampleEquity(points, 500);
    expect(result).toHaveLength(500);
    expect(result[0]).toBe(points[0]);
    expect(result.at(-1)).toBe(points.at(-1));
  });
});
