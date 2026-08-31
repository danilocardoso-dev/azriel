import { open } from "@tauri-apps/plugin-dialog";
import { systemRepository } from "../repositories/systemRepository";
import type { ProcessSnapshot } from "../types";

export type ProcessSort = "memory" | "cpu" | "name" | "pid";

export function formatBytes(value: number): string {
  if (!Number.isFinite(value) || value <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const index = Math.min(Math.floor(Math.log(value) / Math.log(1024)), units.length - 1);
  const amount = value / 1024 ** index;
  return `${amount >= 100 || index === 0 ? amount.toFixed(0) : amount.toFixed(1)} ${units[index]}`;
}

export function filterAndSortProcesses(processes: ProcessSnapshot[], query: string, sort: ProcessSort, descending = true) {
  const needle = query.trim().toLocaleLowerCase("pt-BR");
  const filtered = processes.filter((process) => !needle || process.name.toLocaleLowerCase("pt-BR").includes(needle) || String(process.pid).includes(needle));
  const direction = descending ? -1 : 1;
  return [...filtered].sort((a, b) => {
    if (sort === "name") return a.name.localeCompare(b.name) * direction;
    if (sort === "pid") return (a.pid - b.pid) * direction;
    if (sort === "cpu") return (a.cpuPercent - b.cpuPercent) * direction;
    return (a.memoryBytes - b.memoryBytes) * direction;
  });
}

export const systemService = {
  ...systemRepository,
  async selectDirectory() {
    const selected = await open({ directory: true, multiple: false, title: "Autorizar workspace no Azriel" });
    return typeof selected === "string" ? selected : null;
  },
};
