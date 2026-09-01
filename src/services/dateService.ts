import type { Task } from "../types";

export function localDateKey(date = new Date()) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function parseLocalDate(value: string) {
  const [year, month, day] = value.split("-").map(Number);
  return new Date(year, month - 1, day);
}

export function formatLocalDate(value: string) {
  return parseLocalDate(value).toLocaleDateString("pt-BR");
}

export function parseBrazilianDateInput(value: string) {
  const normalized = value.trim();
  if (!normalized) return null;
  const match = /^(\d{2})\/(\d{2})\/(\d{4})$/.exec(normalized);
  if (!match) throw new Error("Informe o prazo no formato DD/MM/AAAA.");
  const [, dayText, monthText, yearText] = match;
  const day = Number(dayText);
  const month = Number(monthText);
  const year = Number(yearText);
  const date = new Date(year, month - 1, day);
  if (date.getFullYear() !== year || date.getMonth() !== month - 1 || date.getDate() !== day) {
    throw new Error("Informe uma data válida no formato DD/MM/AAAA.");
  }
  return localDateKey(date);
}

export function formatTimestamp(value: string) {
  const normalized = value.includes("T") ? value : `${value.replace(" ", "T")}Z`;
  const date = new Date(normalized);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString("pt-BR");
}

export function groupUpcoming(tasks: Task[], now = new Date()) {
  const tomorrow = new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1);
  const endOfWeek = new Date(now.getFullYear(), now.getMonth(), now.getDate() + (7 - now.getDay()));
  const tomorrowKey = localDateKey(tomorrow);
  const weekKey = localDateKey(endOfWeek);
  return {
    tomorrow: tasks.filter((task) => task.dueDate === tomorrowKey),
    thisWeek: tasks.filter((task) => task.dueDate && task.dueDate > tomorrowKey && task.dueDate <= weekKey),
    later: tasks.filter((task) => task.dueDate && task.dueDate > weekKey && task.dueDate !== tomorrowKey),
  };
}
