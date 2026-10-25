import type { TradingAction } from "../../types";

export const usd = (value: number) => `${value >= 0 ? "" : "−"}${Math.abs(value).toLocaleString("pt-BR", { minimumFractionDigits: 2, maximumFractionDigits: 2 })} USDT`;
export const signedUsd = (value: number) => `${value > 0 ? "+" : value < 0 ? "−" : ""}${Math.abs(value).toLocaleString("pt-BR", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
export const signedPct = (value: number) => `${value > 0 ? "+" : value < 0 ? "−" : ""}${Math.abs(value).toLocaleString("pt-BR", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}%`;

/** Variação absoluta e percentual em relação a uma base; null quando não há base. */
export function change(current: number | null | undefined, base: number | null | undefined) {
  if (current == null || base == null || base === 0) return null;
  return { value: current - base, pct: ((current - base) / base) * 100 };
}

export const trend = (value: number) => value > 0 ? "up" : value < 0 ? "down" : "flat";

export const actionLabels: Record<TradingAction, string> = {
  OPEN_LONG: "COMPRA",
  OPEN_SHORT: "VENDA",
  CLOSE: "FECHAR",
  HOLD: "AGUARDAR",
  ERROR: "ERRO",
  MANUAL_CLOSE: "FECHAMENTO MANUAL",
};

/** Casas decimais de preço pela magnitude (BTC 80.633,2 · SOL 182,45 · PEPE 0,000012). */
export function priceDigits(price: number) {
  const value = Math.abs(price);
  return value >= 1000 ? 1 : value >= 10 ? 2 : value >= 1 ? 3 : value >= 0.01 ? 5 : 8;
}
export const price = (value: number, digits = priceDigits(value)) => value.toLocaleString("pt-BR", { minimumFractionDigits: digits, maximumFractionDigits: digits });

/** Passos "redondos" para o eixo de preço (1, 2, 2,5, 5 × 10ⁿ). */
export function niceTicks(min: number, max: number, count = 5) {
  if (!(max > min)) return [min];
  const raw = (max - min) / count;
  const power = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 2.5, 5, 10].map((factor) => factor * power).find((candidate) => candidate >= raw) ?? raw;
  const ticks: number[] = [];
  for (let tick = Math.ceil(min / step) * step; tick <= max + step * 1e-9; tick += step) ticks.push(Number(tick.toPrecision(12)));
  return ticks;
}

export const intervalMs = (interval: string) => {
  const amount = Number.parseInt(interval, 10) || 1;
  const unit = interval.slice(-1);
  return amount * (unit === "m" ? 60_000 : unit === "h" ? 3_600_000 : unit === "d" ? 86_400_000 : 60_000);
};

/** SQLite grava CURRENT_TIMESTAMP em UTC sem fuso: converte para Date local. */
export const sqliteDate = (value: string) => new Date(value.includes("T") ? value : `${value.replace(" ", "T")}Z`);
export const shortTime = (value: string) => sqliteDate(value).toLocaleString("pt-BR", { day: "2-digit", month: "2-digit", hour: "2-digit", minute: "2-digit" });
