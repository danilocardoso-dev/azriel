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

/** SQLite grava CURRENT_TIMESTAMP em UTC sem fuso: converte para Date local. */
export const sqliteDate = (value: string) => new Date(value.includes("T") ? value : `${value.replace(" ", "T")}Z`);
export const shortTime = (value: string) => sqliteDate(value).toLocaleString("pt-BR", { day: "2-digit", month: "2-digit", hour: "2-digit", minute: "2-digit" });
