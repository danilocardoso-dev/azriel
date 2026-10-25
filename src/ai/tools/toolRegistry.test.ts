import { describe, expect, it } from "vitest";
import type { TradingOverview } from "../../types";
import { tradingStatus } from "./toolRegistry";

const overview: TradingOverview = {
  settings: { apiKeyConfigured: true, symbol: "BTC-USDT", candleInterval: "15m", cycleMinutes: 15, marginUsdt: 20, leverage: 5, stopLossPct: 1, takeProfitPct: 2, maxDailyLossPct: 5, minConfidence: 60, engine: "gemini", running: true, updatedAt: "" },
  engine: { running: true, cycleInProgress: false, lastCycleAt: null, nextCycleAt: null, lastError: null },
  decisions: [{ id: 1, createdAt: "2026-10-25 13:00:00", symbol: "BTC-USDT", price: 67000, positionBefore: "FLAT", proposedAction: "OPEN_LONG", finalAction: "OPEN_LONG", confidence: 74, reason: "Tendência alinhada", riskNote: "", engine: "gemini", model: "gemini-flash", orderStatus: "FILLED", orderDetail: "", equity: 1000 }],
  equity: [{ createdAt: "", equity: 1000, balance: 1000, unrealized: 0 }],
  firstEquity: 1000,
  dayStartEquity: 1000,
};

describe("ferramenta de trading do Az", () => {
  it("resume o trading com resultado pronto e sem dados sensíveis", async () => {
    const status = await tradingStatus({ overview: async () => overview, live: async () => ({ equity: 1012.5, balance: 1000, unrealized: 12.5, availableMargin: 980, position: { side: "LONG", quantity: 0.002, entryPrice: 67000, unrealized: 12.5, leverage: 5 } }) });
    expect(status.resultadoHoje).toEqual({ usdt: 12.5, pct: 1.25 });
    expect(status.posicao?.side).toBe("LONG");
    expect(status.ultimasDecisoes[0].acao).toBe("OPEN_LONG");
    expect(JSON.stringify(status)).not.toMatch(/apiKey|secret/i);
  });

  it("usa o último patrimônio salvo quando a BingX não responde", async () => {
    const status = await tradingStatus({ overview: async () => overview, live: async () => { throw new Error("offline"); } });
    expect(status.patrimonioUsdt).toBe(1000);
    expect(status.posicao).toBeNull();
  });
});
