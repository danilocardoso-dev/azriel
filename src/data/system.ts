import type { AzrielState, ModuleId } from "../types";
export const modules: Array<{ id: ModuleId; label: string; code: string; description: string }> = [
  { id: "command", label: "Command Center", code: "CMD", description: "Visão operacional" },
  { id: "ai", label: "AI Core", code: "AIC", description: "Conversa local" },
  { id: "daily", label: "Operações Diárias", code: "OPS", description: "Tarefas e notas" },
  { id: "trading", label: "Trading", code: "TRD", description: "Agente na conta demo" },
  { id: "settings", label: "Configurações", code: "CFG", description: "Modelo local" },
];
export const azrielStates: Record<AzrielState, { label: string; message: string }> = {
  idle: { label: "ONLINE", message: "Núcleo disponível." },
  processing: { label: "PROCESSANDO", message: "Organizando o contexto local." },
  tool: { label: "CONSULTANDO", message: "Recuperando dados locais." },
  executing: { label: "EXECUTANDO", message: "Executando operação autorizada." },
  engineering: { label: "PAUSADO", message: "Engineering View está congelado." },
  routine: { label: "PAUSADO", message: "Automação foi removida." },
  alert: { label: "ALERTA", message: "Uma demanda requer atenção." },
  offline: { label: "OFFLINE", message: "Ollama local indisponível." },
};
