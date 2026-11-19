import type { AIEngine } from "../../types";

export interface ModelGroup { engine: AIEngine; label: string; models: string[] }

export interface ModelSources {
  gemini: { configured: boolean; models: string[]; current: string };
  deepseek: { configured: boolean; models: string[]; current: string };
  local: { available: boolean; models: string[]; current: string };
  engine: AIEngine;
}

const withCurrent = (models: string[], current: string) => current && !models.includes(current) ? [current, ...models] : models;

/**
 * Um seletor só, agrupado por provedor. Só entra o provedor configurado (ou o local, se estiver
 * ligado); o motor em uso sempre aparece, para o seletor nunca ficar sem o valor atual.
 */
export function modelGroups(sources: ModelSources): ModelGroup[] {
  const groups: ModelGroup[] = [];
  if (sources.gemini.configured || sources.engine === "gemini") groups.push({ engine: "gemini", label: "Gemini", models: withCurrent(sources.gemini.models, sources.gemini.current) });
  if (sources.deepseek.configured || sources.engine === "deepseek") groups.push({ engine: "deepseek", label: "DeepSeek", models: withCurrent(sources.deepseek.models, sources.deepseek.current) });
  if (sources.local.available || sources.engine === "ollama") groups.push({ engine: "ollama", label: sources.local.available ? "Local (Ollama)" : "Local (Ollama desligado)", models: withCurrent(sources.local.models, sources.local.current) });
  return groups.filter((group) => group.models.length > 0);
}

/** O valor de cada opção guarda motor e modelo juntos: "gemini::gemini-3.8-flash". */
export const optionValue = (engine: AIEngine, model: string) => `${engine}::${model}`;

export function parseOption(value: string): { engine: AIEngine; model: string } | null {
  const [engine, ...rest] = value.split("::");
  const model = rest.join("::");
  return ["gemini", "deepseek", "ollama"].includes(engine) && model ? { engine: engine as AIEngine, model } : null;
}
