import type { MarketEquityPoint, MarketTimeframe } from "../../types";

export function annualizationFor(timeframe: MarketTimeframe): number {
  return timeframe === "15M" ? 252 * 26 : 252;
}

export function downsampleEquity(points: MarketEquityPoint[], maximum = 900): MarketEquityPoint[] {
  if (points.length <= maximum || maximum < 3) return points;
  const result = [points[0]];
  const bucketSize = (points.length - 2) / (maximum - 2);
  for (let bucket = 0; bucket < maximum - 2; bucket += 1) {
    const start = 1 + Math.floor(bucket * bucketSize);
    const end = Math.min(points.length - 1, 1 + Math.floor((bucket + 1) * bucketSize));
    let selected = points[start];
    for (let index = start + 1; index < end; index += 1) {
      if (Math.abs(points[index].equity - points[start - 1].equity) > Math.abs(selected.equity - points[start - 1].equity)) selected = points[index];
    }
    result.push(selected);
  }
  result.push(points[points.length - 1]);
  return result;
}
