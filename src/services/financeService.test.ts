import { describe, expect, it } from "vitest";
import { formatBrazilianDate, formatCurrency, formatCurrencyInput, formatMonth, maskBrazilianDate, parseBrazilianDate, parseCurrencyToCents, shiftMonth, validateFinancialTransaction } from "./financeService";

describe("financeService", () => {
  it("converte moeda brasileira para centavos sem ponto flutuante", () => {
    expect(parseCurrencyToCents("R$ 1.234,56")).toBe(123456);
    expect(parseCurrencyToCents("42,5")).toBe(4250);
    expect(parseCurrencyToCents("10.25")).toBe(1025);
    expect(() => parseCurrencyToCents("0")).toThrow(/maior que zero/i);
    expect(() => parseCurrencyToCents("12,345")).toThrow(/válido/i);
  });

  it("formata valores e meses para apresentação", () => {
    expect(formatCurrency(123456)).toContain("1.234,56");
    expect(formatCurrencyInput(90000)).toBe("900,00");
    expect(formatMonth("2026-10")).toBe("outubro de 2026");
    expect(shiftMonth("2026-10", -1)).toBe("2026-09");
    expect(shiftMonth("2026-12", 1)).toBe("2027-01");
    expect(shiftMonth("2026-01", -1)).toBe("2025-12");
    expect(() => shiftMonth("10/2026", 1)).toThrow(/mês inválido/i);
  });

  it("converte datas brasileiras sem alterar o formato persistido", () => {
    expect(maskBrazilianDate("13102026")).toBe("13/10/2026");
    expect(formatBrazilianDate("2026-10-13")).toBe("13/10/2026");
    expect(parseBrazilianDate("13/10/2026", "a data")).toBe("2026-10-13");
    expect(parseBrazilianDate("", "a data opcional", false)).toBeNull();
    expect(() => parseBrazilianDate("31/02/2026", "a data")).toThrow(/data válida/i);
    expect(() => parseBrazilianDate("10/13/2026", "a data")).toThrow(/data válida/i);
  });

  it("exige pagamento para lançamento realizado", () => {
    expect(() => validateFinancialTransaction({ id: "x", transactionType: "INCOME", scope: "PERSONAL", description: "Receita", amountCents: 100, categoryId: "personal-other", competenceDate: "2026-10-01", dueDate: null, paidAt: null, status: "PAID", notes: null })).toThrow(/pagamento/i);
  });
});
