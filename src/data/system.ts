import type { AzrielState, ModuleId } from "../types";
export const modules: Array<{ id: ModuleId; label: string; code: string; glyph: string; description: string }> = [
  { id: "command", label: "Núcleo", code: "CORE", glyph: "◉", description: "Visão central" },
  { id: "ai", label: "AI Core", code: "AIC", glyph: "⌬", description: "Conversa" },
  { id: "daily", label: "Operações", code: "OPS", glyph: "≡", description: "Tarefas e notas" },
  { id: "trading", label: "Trading", code: "TRD", glyph: "⟋", description: "Agente na conta demo" },
  { id: "settings", label: "Configurações", code: "CFG", glyph: "⚙", description: "Motores e chaves" },
];
export const azrielStates: Record<AzrielState, { label: string; message: string }> = {
  idle: { label: "ONLINE", message: "Todos os sistemas nominais." },
  processing: { label: "PROCESSANDO", message: "Organizando o contexto." },
  tool: { label: "CONSULTANDO", message: "Recuperando dados." },
  executing: { label: "EXECUTANDO", message: "Executando operação autorizada." },
  engineering: { label: "PAUSADO", message: "Engineering View está congelado." },
  routine: { label: "PAUSADO", message: "Automação foi removida." },
  alert: { label: "ALERTA", message: "Uma demanda requer atenção." },
  offline: { label: "OFFLINE", message: "Motor de IA indisponível." },
};
