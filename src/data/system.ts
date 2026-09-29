import type { AzrielState, ModuleId } from "../types";

export const modules: Array<{ id: ModuleId; label: string; code: string; description: string }> = [
  { id: "command", label: "Command Center", code: "CMD", description: "Visão estratégica do sistema" },
  { id: "ai", label: "AI Core", code: "AIC", description: "Conversa e consultas locais" },
  { id: "daily", label: "Operações Diárias", code: "OPS", description: "Tarefas, notas e prioridades" },
  { id: "projects", label: "Projetos", code: "PRJ", description: "Projetos e objetivos" },
  { id: "studies", label: "Estudos", code: "STD", description: "Roadmaps, etapas e atividades de estudo" },
  { id: "market", label: "Market Lab", code: "LAB", description: "Backtests determinísticos e experimentais" },
  { id: "automation", label: "Automação", code: "AUT", description: "Ações locais autorizadas" },
  { id: "settings", label: "Configurações", code: "CFG", description: "Preferências da interface" },
];

export const azrielStates: Record<AzrielState, { label: string; message: string }> = {
  idle: { label: "ONLINE", message: "Núcleo disponível. Aguardando comando." },
  processing: { label: "PROCESSANDO", message: "Organizando relações entre módulos." },
  tool: { label: "CONSULTANDO", message: "Recuperando dados estruturados dos núcleos." },
  executing: { label: "EXECUTANDO AÇÃO", message: "Executando uma ação previamente autorizada." },
  engineering: { label: "ENGINEERING COMMAND", message: "Manipulando o estado visual do modelo 3D." },
  routine: { label: "EXECUTANDO ROTINA", message: "Executando passos autorizados em sequência." },
  alert: { label: "ALERTA", message: "Lacunas críticas requerem atenção." },
  offline: { label: "OFFLINE", message: "Ollama local indisponível; demais núcleos continuam ativos." },
};
