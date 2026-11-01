/** Valores em centavos (inteiros) de ponta a ponta: o dinheiro nunca passa por ponto flutuante fora daqui. */

const MONTHS = ["janeiro", "fevereiro", "março", "abril", "maio", "junho", "julho", "agosto", "setembro", "outubro", "novembro", "dezembro"];

export function formatMoney(cents: number): string {
  const sign = cents < 0 ? "-" : "";
  const whole = Math.floor(Math.abs(cents) / 100).toString().replace(/\B(?=(\d{3})+(?!\d))/g, ".");
  return `${sign}R$ ${whole},${String(Math.abs(cents) % 100).padStart(2, "0")}`;
}

/** Lê o que a pessoa digita: "1500", "1500,50", "1.500,50", "R$ 1.500,50" ou "1500.5". Devolve centavos, ou null se não for um valor. */
export function parseAmount(text: string): number | null {
  let value = text.replace(/R\$/gi, "").replace(/\s/g, "");
  if (!value) return null;
  if (value.includes(",")) value = value.replace(/\./g, "").replace(",", ".");
  else if (/^\d{1,3}(\.\d{3})+$/.test(value)) value = value.replace(/\./g, "");
  if (!/^\d+(\.\d{1,2})?$/.test(value)) return null;
  return Math.round(Number(value) * 100);
}

/** Centavos de volta para o campo de edição: 150050 → "1500,50". */
export function amountField(cents: number): string {
  return `${Math.floor(cents / 100)},${String(cents % 100).padStart(2, "0")}`;
}

export const monthOf = (date: string) => date.slice(0, 7);

export function shiftMonth(month: string, delta: number): string {
  const [year, number] = month.split("-").map(Number);
  const index = year * 12 + (number - 1) + delta;
  return `${String(Math.floor(index / 12)).padStart(4, "0")}-${String((index % 12) + 1).padStart(2, "0")}`;
}

export function monthLabel(month: string): string {
  const [year, number] = month.split("-").map(Number);
  return `${MONTHS[number - 1]} de ${year}`;
}

export const dayOf = (date: string) => date.slice(8, 10);

/** 2026-10-05 → "05/10/2026". */
export const formatDate = (date: string) => `${date.slice(8, 10)}/${date.slice(5, 7)}/${date.slice(0, 4)}`;

/** Máscara de digitação: "05102026" → "05/10/2026". */
export function maskDate(text: string): string {
  const digits = text.replace(/\D/g, "").slice(0, 8);
  return [digits.slice(0, 2), digits.slice(2, 4), digits.slice(4)].filter(Boolean).join("/");
}

/** "05/10/2026" (ou 5/10/2026, 05-10-2026) → "2026-10-05"; null se não for uma data real. */
export function parseDate(text: string): string | null {
  const match = /^(\d{1,2})[/.-](\d{1,2})[/.-](\d{4})$/.exec(text.trim());
  if (!match) return null;
  const [day, month, year] = [Number(match[1]), Number(match[2]), Number(match[3])];
  const real = new Date(Date.UTC(year, month - 1, day));
  if (real.getUTCFullYear() !== year || real.getUTCMonth() !== month - 1 || real.getUTCDate() !== day) return null;
  return `${String(year).padStart(4, "0")}-${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
}
