import type { Memory, MemoryCategory } from "../../types";

export const CATEGORY_NAMES: Record<MemoryCategory, string> = { operador: "Sobre o Senhor", preferencia: "Preferência", projeto: "Projeto", pessoa: "Pessoa", decisao: "Decisão", fato: "Fato" };

/** Filtra por categoria e por texto (sem diferenciar maiúsculas e acentos). */
export function filterMemories(memories: Memory[], category: MemoryCategory | "all", text: string): Memory[] {
  const fold = (value: string) => value.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();
  const needle = fold(text.trim());
  return memories.filter((memory) => (category === "all" || memory.category === category) && (!needle || fold(memory.content).includes(needle)));
}
