import { describe, expect, it } from "vitest";
import { amountField, dayOf, formatDate, formatMoney, maskDate, monthLabel, monthOf, parseAmount, parseDate, shiftMonth } from "./format";

describe("finanças: valores", () => {
  it("formata centavos em reais", () => {
    expect(formatMoney(0)).toBe("R$ 0,00");
    expect(formatMoney(5)).toBe("R$ 0,05");
    expect(formatMoney(150050)).toBe("R$ 1.500,50");
    expect(formatMoney(123456789)).toBe("R$ 1.234.567,89");
    expect(formatMoney(-2500)).toBe("-R$ 25,00");
  });

  it("lê os jeitos comuns de digitar um valor", () => {
    expect(parseAmount("1500")).toBe(150000);
    expect(parseAmount("1500,50")).toBe(150050);
    expect(parseAmount("1.500,50")).toBe(150050);
    expect(parseAmount("R$ 1.500,50")).toBe(150050);
    expect(parseAmount("1500.5")).toBe(150050);
    expect(parseAmount("1.500")).toBe(150000);
    expect(parseAmount("0,99")).toBe(99);
    expect(parseAmount("19.99")).toBe(1999);
  });

  it("recusa o que não é valor", () => {
    for (const text of ["", "abc", "12,345", "1,2,3", "-5", "1e3", "R$"]) expect(parseAmount(text)).toBeNull();
  });

  it("volta para o campo de edição sem perder centavos", () => {
    expect(amountField(150050)).toBe("1500,50");
    expect(parseAmount(amountField(150005))).toBe(150005);
  });
});

describe("finanças: meses", () => {
  it("navega entre meses virando o ano", () => {
    expect(shiftMonth("2026-12", 1)).toBe("2027-01");
    expect(shiftMonth("2026-01", -1)).toBe("2025-12");
    expect(shiftMonth("2026-10", 0)).toBe("2026-10");
    expect(shiftMonth("2026-10", -22)).toBe("2024-12");
  });

  it("nomeia o mês e corta partes da data", () => {
    expect(monthLabel("2026-03")).toBe("março de 2026");
    expect(monthOf("2026-10-05")).toBe("2026-10");
    expect(dayOf("2026-10-05")).toBe("05");
  });
});

describe("finanças: datas no padrão brasileiro", () => {
  it("mostra dd/mm/aaaa", () => {
    expect(formatDate("2026-10-05")).toBe("05/10/2026");
  });

  it("aplica a máscara enquanto digita", () => {
    expect(maskDate("0510")).toBe("05/10");
    expect(maskDate("05102026")).toBe("05/10/2026");
    expect(maskDate("05/10/20269")).toBe("05/10/2026");
  });

  it("lê dd/mm/aaaa e recusa datas inexistentes", () => {
    expect(parseDate("05/10/2026")).toBe("2026-10-05");
    expect(parseDate("5/1/2026")).toBe("2026-01-05");
    expect(parseDate("29/02/2028")).toBe("2028-02-29");
    expect(parseDate("29/02/2026")).toBeNull();
    expect(parseDate("31/04/2026")).toBeNull();
    expect(parseDate("2026-10-05")).toBeNull();
    expect(parseDate("")).toBeNull();
  });
});
