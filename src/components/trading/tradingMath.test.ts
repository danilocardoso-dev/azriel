import { describe, expect, it } from "vitest";
import { change, signedPct, signedUsd, sqliteDate, trend } from "./tradingMath";

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

  it("interpreta o horário UTC do SQLite", () => {
    expect(sqliteDate("2026-10-25 13:00:00").toISOString()).toBe("2026-10-25T13:00:00.000Z");
  });
});
