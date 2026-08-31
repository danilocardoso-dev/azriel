import type { AzrielState, ModuleId } from "../types";

export const modules: Array<{ id: ModuleId; label: string; code: string; description: string }> = [
  { id: "command", label: "Command Center", code: "CMD", description: "Visão estratégica do sistema" },
  { id: "daily", label: "Operações Diárias", code: "OPS", description: "Tarefas, notas e prioridades" },
  { id: "projects", label: "Projetos", code: "PRJ", description: "Projetos e objetivos" },
  { id: "knowledge", label: "Conhecimento", code: "KNO", description: "Áreas e métricas" },
  { id: "stark", label: "Mapa Stark", code: "STK", description: "Cobertura e profundidade" },
  { id: "education", label: "Formação", code: "EDU", description: "Trajetória acadêmica" },
  { id: "research", label: "Pesquisa", code: "R&D", description: "Fila de pesquisa e estudo" },
  { id: "system", label: "Sistema", code: "SYS", description: "Estado simulado dos núcleos" },
  { id: "settings", label: "Configurações", code: "CFG", description: "Preferências da interface" },
];

export const azrielStates: Record<AzrielState, { label: string; message: string }> = {
  idle: { label: "ONLINE", message: "Núcleo disponível. Aguardando comando." },
  processing: { label: "PROCESSANDO", message: "Organizando relações entre módulos." },
  alert: { label: "ALERTA", message: "Lacunas críticas requerem atenção." },
  offline: { label: "OFFLINE", message: "Simulação de núcleo indisponível." },
};

export const systemNodes = [
  { name: "Interface Core", state: "online", detail: "React / HUD v0.5.1" },
  { name: "Project Core", state: "online", detail: "Projetos persistidos em SQLite" },
  { name: "Knowledge Core", state: "online", detail: "Domínios e histórico persistidos" },
  { name: "Daily Operations", state: "online", detail: "Tarefas e notas locais" },
  { name: "Memory Core", state: "online", detail: "SQLite / schema versionado" },
  { name: "AI Core", state: "standby", detail: "Previsto para v0.6" },
  { name: "System Core", state: "simulated", detail: "Integração real na v0.7" },
  { name: "Automation Core", state: "standby", detail: "Previsto para v0.8" },
  { name: "IoT Core", state: "standby", detail: "Previsto para v0.9" },
];
