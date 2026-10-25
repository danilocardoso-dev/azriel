import { describe, expect, it } from "vitest";
import { change, intervalMs, niceTicks, price, priceDigits, signedPct, signedUsd, sqliteDate, trend } from "./tradingMath";

describe("trading math", () => {
  it("calcula variação contra a base e trata base ausente", () => {
    expect(change(1050, 1000)).toEqual({ value: 50, pct: 5 });
    expect(change(950, 1000)?.pct).toBe(-5);
    expect(change(1000, null)).toBeNull();
    expect(change(1000, 0)).toBeNull();
  });

  it("sempre mostra o sinal, para o ganho e a perda não dependerem só da cor", () => {
    expect(signedUsd(12.5)).toBe("+12,50");
    expect(signedUsd(-3)).toBe("−3,00");
    expect(signedPct(0)).toBe("0,00%");
    expect(trend(-0.01)).toBe("down");
  });

  it("formata preço pela magnitude do ativo", () => {
    expect(priceDigits(80633.2)).toBe(1);
    expect(price(182.456)).toBe("182,46");
    expect(priceDigits(0.000012)).toBe(8);
  });

  it("gera ticks redondos dentro da faixa e converte intervalos", () => {
    expect(niceTicks(80_120, 80_980)).toEqual([80_200, 80_400, 80_600, 80_800]);
    expect(niceTicks(5, 5)).toEqual([5]);
    expect(intervalMs("15m")).toBe(900_000);
    expect(intervalMs("4h")).toBe(14_400_000);
  });

  it("interpreta o horário UTC do SQLite", () => {
    expect(sqliteDate("2026-10-25 13:00:00").toISOString()).toBe("2026-10-25T13:00:00.000Z");
  });
});
