import { financeRepository } from "../repositories/financeRepository";
import type { FinancialCategoryInput, FinancialTransactionInput } from "../types";

export function parseCurrencyToCents(value: string): number {
  const normalized = value.trim().replace(/\s/g, "").replace(/^R\$/i, "");
  if (!normalized) throw new Error("Informe o valor do lançamento.");
  const canonical = normalized.includes(",")
    ? normalized.replace(/\./g, "").replace(",", ".")
    : normalized;
  if (!/^\d+(?:\.\d{1,2})?$/.test(canonical)) throw new Error("Informe um valor monetário válido.");
  const [integer, decimal = ""] = canonical.split(".");
  const cents = Number(integer) * 100 + Number(decimal.padEnd(2, "0"));
  if (!Number.isSafeInteger(cents) || cents <= 0) throw new Error("O valor deve ser maior que zero.");
  return cents;
}

export function formatCurrency(cents: number): string {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(cents / 100);
}

export function formatCurrencyInput(cents: number): string {
  return new Intl.NumberFormat("pt-BR", { minimumFractionDigits: 2, maximumFractionDigits: 2 }).format(cents / 100);
}

export function maskBrazilianDate(value: string): string {
  const digits = value.replace(/\D/g, "").slice(0, 8);
  if (digits.length <= 2) return digits;
  if (digits.length <= 4) return `${digits.slice(0, 2)}/${digits.slice(2)}`;
  return `${digits.slice(0, 2)}/${digits.slice(2, 4)}/${digits.slice(4)}`;
}

export function formatBrazilianDate(value: string | null): string {
  if (!value) return "";
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  return match ? `${match[3]}/${match[2]}/${match[1]}` : value;
}

export function parseBrazilianDate(value: string, label: string, required = true): string | null {
  const normalized = value.trim();
  if (!normalized) {
    if (required) throw new Error(`Informe ${label}.`);
    return null;
  }
  const match = /^(\d{2})\/(\d{2})\/(\d{4})$/.exec(normalized);
  if (!match) throw new Error(`Informe ${label} no formato DD/MM/AAAA.`);
  const day = Number(match[1]);
  const month = Number(match[2]);
  const year = Number(match[3]);
  const date = new Date(year, month - 1, day);
  if (date.getFullYear() !== year || date.getMonth() !== month - 1 || date.getDate() !== day) {
    throw new Error(`Informe uma data válida para ${label}.`);
  }
  return `${match[3]}-${match[2]}-${match[1]}`;
}

export function formatMonth(month: string): string {
  const [year, monthNumber] = month.split("-").map(Number);
  return new Intl.DateTimeFormat("pt-BR", { month: "long", year: "numeric", timeZone: "UTC" }).format(new Date(Date.UTC(year, monthNumber - 1, 1)));
}

export function shiftMonth(month: string, offset: number): string {
  const match = /^(\d{4})-(\d{2})$/.exec(month);
  if (!match) throw new Error("Mês inválido; use AAAA-MM.");
  const date = new Date(Date.UTC(Number(match[1]), Number(match[2]) - 1 + offset, 1));
  return `${date.getUTCFullYear()}-${String(date.getUTCMonth() + 1).padStart(2, "0")}`;
}

export function validateFinancialTransaction(input: FinancialTransactionInput): FinancialTransactionInput {
  if (!input.description.trim()) throw new Error("Informe a descrição do lançamento.");
  if (!Number.isSafeInteger(input.amountCents) || input.amountCents <= 0) throw new Error("O valor deve ser maior que zero.");
  if (!input.categoryId) throw new Error("Selecione uma categoria.");
  if (!/^\d{4}-\d{2}-\d{2}$/.test(input.competenceDate)) throw new Error("Informe a data de competência.");
  if (input.status === "PAID" && !input.paidAt) throw new Error("Informe a data de pagamento ou recebimento.");
  return { ...input, description: input.description.trim(), notes: input.notes?.trim() || null };
}

export function validateFinancialCategory(input: FinancialCategoryInput): FinancialCategoryInput {
  if (!input.name.trim()) throw new Error("Informe o nome da categoria.");
  return { ...input, name: input.name.trim() };
}

export const financeService = {
  ...financeRepository,
  saveCategory: (input: FinancialCategoryInput) => financeRepository.saveCategory(validateFinancialCategory(input)),
  saveTransaction: (input: FinancialTransactionInput) => financeRepository.saveTransaction(validateFinancialTransaction(input)),
};
