import type { AzrielState, ModuleId } from "../types";

export const modules: Array<{ id: ModuleId; label: string; code: string; description: string }> = [
  { id: "command", label: "Command Center", code: "CMD", description: "Visão estratégica do sistema" },
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
  { name: "Interface Core", state: "online", detail: "React / HUD v0.4" },
  { name: "Project Core", state: "online", detail: "7 projetos mockados" },
  { name: "Knowledge Core", state: "online", detail: "13 domínios mapeados" },
  { name: "Memory Core", state: "standby", detail: "Previsto para v0.5" },
  { name: "AI Core", state: "standby", detail: "Previsto para v0.6" },
  { name: "System Core", state: "simulated", detail: "Integração real na v0.7" },
  { name: "Automation Core", state: "standby", detail: "Previsto para v0.8" },
  { name: "IoT Core", state: "standby", detail: "Previsto para v0.9" },
];
