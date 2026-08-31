import { educationService } from "../../services/educationService";
import { getDatabaseInfo } from "../../services/databaseService";
import { diagnoseGaps, knowledgeService } from "../../services/knowledgeService";
import { noteService } from "../../services/noteService";
import { projectService } from "../../services/projectService";
import { taskService } from "../../services/taskService";
import { localDateKey } from "../../services/dateService";
import { systemService } from "../../services/systemService";
import { aiRepository } from "../../repositories/aiRepository";
import { automationService } from "../../services/automationService";
import type { ActionPermission, ActionRequest, ActionResult, AISettings, AIToolInput, AIToolName, AIToolResult, Application, DailyCounters, EducationItem, KnowledgeArea, KnowledgeHistory, Note, OllamaStatus, ProcessSnapshot, Project, RegisteredUrl, SystemSnapshot, Task, Workspace, WorkspaceStatus } from "../../types";

export interface AzrielTool {
  name: AIToolName;
  description: string;
  domain: string;
  readonly: boolean;
  permission?: ActionPermission;
  execute(input: AIToolInput): Promise<unknown>;
}

export interface ToolDependencies {
  tasks: { list(): Promise<Task[]>; today(): Promise<Task[]>; upcoming(): Promise<Task[]>; counters(): Promise<DailyCounters> };
  notes: { list(includeArchived?: boolean): Promise<Note[]> };
  projects: { list(): Promise<Project[]>; get(id: string): Promise<Project | null> };
  knowledge: { list(): Promise<KnowledgeArea[]>; get(id: string): Promise<KnowledgeArea | null>; history(id: string): Promise<KnowledgeHistory[]> };
  education: { list(): Promise<EducationItem[]> };
  databaseInfo(): Promise<{ schemaVersion: number; integrationValue: number }>;
  system: { snapshot(): Promise<SystemSnapshot>; processes(): Promise<ProcessSnapshot[]>; listWorkspaces(): Promise<Workspace[]>; workspaceStatus(id: string): Promise<WorkspaceStatus> };
  ollama: { settings(): Promise<AISettings>; status(endpoint: string, timeoutSeconds: number): Promise<OllamaStatus> };
  automation: { listApplications(): Promise<Application[]>; listUrls(): Promise<RegisteredUrl[]>; execute(request: ActionRequest): Promise<ActionResult> };
}

const productionDependencies: ToolDependencies = {
  tasks: taskService,
  notes: noteService,
  projects: projectService,
  knowledge: knowledgeService,
  education: educationService,
  databaseInfo: getDatabaseInfo,
  system: systemService,
  ollama: { settings: aiRepository.getSettings, status: aiRepository.status },
  automation: automationService,
};

const normalize = (value: string) => value.normalize("NFD").replace(/[\u0300-\u036f]/g, "").toLowerCase();
const unfinished = (task: Task) => !["completed", "cancelled"].includes(task.status);
const findProject = async (dependencies: ToolDependencies, input: AIToolInput) => {
  const projects = await dependencies.projects.list();
  const needle = normalize(input.entityId || input.term || input.query);
  return projects.find((project) => project.id === input.entityId || normalize(project.name).includes(needle) || needle.includes(normalize(project.name))) ?? null;
};
const findKnowledge = async (dependencies: ToolDependencies, input: AIToolInput) => {
  const areas = await dependencies.knowledge.list();
  const needle = normalize(input.entityId || input.term || input.query);
  return areas.find((area) => area.id === input.entityId || normalize(area.name).includes(needle) || needle.includes(normalize(area.name))) ?? null;
};
const findWorkspace = async (dependencies: ToolDependencies, input: AIToolInput) => {
  const workspaces = (await dependencies.system.listWorkspaces()).filter((workspace) => workspace.enabled);
  const needle = normalize(input.workspaceId || input.entityId || input.term || input.query);
  return workspaces.find((workspace) => workspace.id === input.workspaceId || workspace.id === input.entityId || normalize(workspace.name).includes(needle) || needle.includes(normalize(workspace.name))) ?? null;
};
const bestNamedMatch = <T extends { id: string; name: string; enabled: boolean }>(items: T[], query: string) => {
  const value = normalize(query);
  const matches = items.filter((item) => item.enabled && value.includes(normalize(item.name))).sort((a, b) => b.name.length - a.name.length);
  const best = matches[0];
  if (!best) return null;
  const bestName = normalize(best.name);
  const ambiguous = matches.slice(1).some((item) => {
    const candidate = normalize(item.name);
    return candidate === bestName || (!bestName.includes(candidate) && !candidate.includes(bestName));
  });
  return ambiguous ? null : best;
};
const executeAction = async (dependencies: ToolDependencies, actionId: ActionRequest["actionId"], targetId: string | undefined) =>
  dependencies.automation.execute({ actionId, source: "ai", targetId: targetId ?? "unregistered" });

function createTools(dependencies: ToolDependencies): AzrielTool[] {
  return [
    { name: "get_today_tasks", domain: "Operações Diárias", description: "Tarefas de hoje e atrasadas ainda abertas", readonly: true, execute: () => dependencies.tasks.today() },
    { name: "get_overdue_tasks", domain: "Operações Diárias", description: "Tarefas atrasadas", readonly: true, execute: async () => { const today = localDateKey(); return (await dependencies.tasks.list()).filter((task) => unfinished(task) && task.dueDate !== null && task.dueDate < today); } },
    { name: "get_upcoming_tasks", domain: "Operações Diárias", description: "Próximas tarefas", readonly: true, execute: () => dependencies.tasks.upcoming() },
    { name: "get_recent_notes", domain: "Operações Diárias", description: "Notas ativas recentes", readonly: true, execute: async () => (await dependencies.notes.list(false)).slice(0, 5) },
    { name: "get_daily_operations_summary", domain: "Operações Diárias", description: "Contadores reais das operações", readonly: true, execute: () => dependencies.tasks.counters() },
    { name: "list_projects", domain: "Projetos", description: "Projetos registrados", readonly: true, execute: () => dependencies.projects.list() },
    { name: "get_project", domain: "Projetos", description: "Detalhes de um projeto", readonly: true, execute: (input) => findProject(dependencies, input) },
    { name: "get_project_tasks", domain: "Projetos", description: "Tarefas relacionadas a um projeto", readonly: true, execute: async (input) => { const project = await findProject(dependencies, input); if (!project) return []; return (await dependencies.tasks.list()).filter((task) => task.projectId === project.id); } },
    { name: "get_project_knowledge", domain: "Projetos", description: "Conhecimentos relacionados a um projeto", readonly: true, execute: async (input) => { const project = await findProject(dependencies, input); if (!project) return []; const areas = await dependencies.knowledge.list(); return areas.filter((area) => project.knowledgeAreaIds.includes(area.id)); } },
    { name: "list_knowledge_areas", domain: "Knowledge Core", description: "Áreas de conhecimento registradas", readonly: true, execute: () => dependencies.knowledge.list() },
    { name: "get_knowledge_area", domain: "Knowledge Core", description: "Detalhes de uma área de conhecimento", readonly: true, execute: (input) => findKnowledge(dependencies, input) },
    { name: "get_knowledge_gaps", domain: "Knowledge Core", description: "Maiores lacunas calculadas", readonly: true, execute: async () => diagnoseGaps(await dependencies.knowledge.list()) },
    { name: "get_stark_map", domain: "Knowledge Core", description: "Dados reais do Mapa Stark", readonly: true, execute: async () => { const areas = await dependencies.knowledge.list(); const average = (field: "coverage" | "depth") => areas.length ? Math.round(areas.reduce((total, area) => total + area[field], 0) / areas.length) : 0; return { areas, coverageAverage: average("coverage"), depthAverage: average("depth") }; } },
    { name: "get_knowledge_history", domain: "Knowledge Core", description: "Histórico de uma área", readonly: true, execute: async (input) => { const area = await findKnowledge(dependencies, input); return area ? dependencies.knowledge.history(area.id) : []; } },
    { name: "get_education", domain: "Formação", description: "Formação completa", readonly: true, execute: () => dependencies.education.list() },
    { name: "get_current_education", domain: "Formação", description: "Formação atual", readonly: true, execute: async () => (await dependencies.education.list()).filter((item) => item.status === "in_progress") },
    { name: "get_planned_education", domain: "Formação", description: "Formação planejada", readonly: true, execute: async () => (await dependencies.education.list()).filter((item) => item.status === "planned") },
    { name: "get_system_status", domain: "System Core", description: "Resumo do sistema operacional e da telemetria local", readonly: true, execute: async () => { const snapshot = await dependencies.system.snapshot(); return { collectedAt: snapshot.collectedAt, details: snapshot.details, cpuUsagePercent: snapshot.cpu.usagePercent, memory: snapshot.memory, storage: snapshot.storage.map((disk) => ({ name: disk.name, mountPoint: disk.mountPoint, totalBytes: disk.totalBytes, availableBytes: disk.availableBytes })), network: { interfaces: snapshot.network.length, receivedBytes: snapshot.network.reduce((sum, item) => sum + item.receivedBytes, 0), transmittedBytes: snapshot.network.reduce((sum, item) => sum + item.transmittedBytes, 0) }, errors: snapshot.errors }; } },
    { name: "get_cpu_status", domain: "System Core", description: "Uso atual de CPU", readonly: true, execute: async () => (await dependencies.system.snapshot()).cpu },
    { name: "get_memory_status", domain: "System Core", description: "Uso atual de memória e swap", readonly: true, execute: async () => (await dependencies.system.snapshot()).memory },
    { name: "get_storage_status", domain: "System Core", description: "Volumes e espaço disponível", readonly: true, execute: async () => (await dependencies.system.snapshot()).storage },
    { name: "get_network_status", domain: "System Core", description: "Interfaces e tráfego da amostra mais recente", readonly: true, execute: async () => (await dependencies.system.snapshot()).network },
    { name: "get_process_summary", domain: "System Core", description: "Até dez processos com maior consumo, sem argumentos ou caminhos sensíveis", readonly: true, execute: async (input) => { const processes = await dependencies.system.processes(); const byCpu = normalize(input.query).includes("cpu"); return [...processes].sort((a, b) => byCpu ? b.cpuPercent - a.cpuPercent : b.memoryBytes - a.memoryBytes).slice(0, 10); } },
    { name: "list_workspaces", domain: "System Core", description: "Workspaces atualmente autorizados e habilitados", readonly: true, execute: async () => (await dependencies.system.listWorkspaces()).filter((workspace) => workspace.enabled).map(({ id, name, projectId, enabled }) => ({ id, name, projectId, enabled })) },
    { name: "get_workspace_status", domain: "System Core", description: "Estado de um workspace autorizado identificado internamente", readonly: true, execute: async (input) => { const workspace = await findWorkspace(dependencies, input); return workspace ? dependencies.system.workspaceStatus(workspace.id) : null; } },
    { name: "get_git_status", domain: "System Core", description: "Estado Git dos workspaces autorizados", readonly: true, execute: async (input) => { const workspace = await findWorkspace(dependencies, input); if (workspace) return dependencies.system.workspaceStatus(workspace.id); const enabled = (await dependencies.system.listWorkspaces()).filter((item) => item.enabled); return Promise.all(enabled.slice(0, 20).map(async (item) => { const status = await dependencies.system.workspaceStatus(item.id); return { workspace: item.name, git: status.git }; })); } },
    { name: "get_recent_commits", domain: "System Core", description: "Commits recentes de um workspace autorizado", readonly: true, execute: async (input) => { const workspace = await findWorkspace(dependencies, input); if (!workspace) return []; return (await dependencies.system.workspaceStatus(workspace.id)).git?.recentCommits ?? []; } },
    { name: "get_ollama_status", domain: "System Core", description: "Estado do Ollama e modelos locais usando as configurações existentes", readonly: true, execute: async () => { const settings = await dependencies.ollama.settings(); return dependencies.ollama.status(settings.endpoint, Math.min(settings.timeoutSeconds, 8)); } },
    { name: "get_azriel_status", domain: "Azriel", description: "Estado consolidado do Azriel", readonly: true, execute: async () => { const [database, daily, system, workspaces] = await Promise.all([dependencies.databaseInfo(), dependencies.tasks.counters(), dependencies.system.snapshot(), dependencies.system.listWorkspaces()]); return { version: "0.8.0", database, daily, system: { cpuUsagePercent: system.cpu.usagePercent, memory: system.memory, errors: system.errors }, workspaces: { enabled: workspaces.filter((item) => item.enabled).length, total: workspaces.length }, aiCore: "online quando Ollama disponível", automationCore: "safe mode", writeAccess: "somente ações previamente autorizadas" }; } },
    { name: "get_azriel_version", domain: "Azriel", description: "Versão atual", readonly: true, execute: async () => ({ version: "0.8.0", name: "Automation Core / Safe Actions" }) },
    { name: "open_application", domain: "Automation Core", description: "Abre somente um aplicativo autorizado identificado pelo nome", readonly: false, permission: "safe_write", execute: async (input) => { const target = bestNamedMatch(await dependencies.automation.listApplications(), input.query); return executeAction(dependencies, "open_application", target?.id); } },
    { name: "open_workspace", domain: "Automation Core", description: "Abre somente um workspace autorizado identificado pelo nome", readonly: false, permission: "safe_write", execute: async (input) => { const target = bestNamedMatch(await dependencies.system.listWorkspaces(), input.query); return executeAction(dependencies, "open_workspace", target?.id); } },
    { name: "open_project", domain: "Automation Core", description: "Abre um projeto registrado por seu workspace autorizado", readonly: false, permission: "safe_write", execute: async (input) => { const target = bestNamedMatch((await dependencies.projects.list()).map((project) => ({ ...project, enabled: true })), input.query); return executeAction(dependencies, "open_project", target?.id); } },
    { name: "reveal_workspace", domain: "Automation Core", description: "Revela somente a pasta de um workspace autorizado", readonly: false, permission: "safe_write", execute: async (input) => { const target = bestNamedMatch(await dependencies.system.listWorkspaces(), input.query); return executeAction(dependencies, "reveal_workspace", target?.id); } },
    { name: "open_registered_url", domain: "Automation Core", description: "Abre somente uma URL previamente cadastrada", readonly: false, permission: "safe_write", execute: async (input) => { const target = bestNamedMatch(await dependencies.automation.listUrls(), input.query); return executeAction(dependencies, "open_registered_url", target?.id); } },
  ];
}

function isEmpty(data: unknown): boolean {
  if (data === null || data === undefined) return true;
  if (Array.isArray(data)) return data.length === 0;
  if (typeof data === "object") return Object.keys(data).length === 0;
  return data === "";
}

export class ToolRegistry {
  private readonly tools: Map<AIToolName, AzrielTool>;
  constructor(dependencies: ToolDependencies = productionDependencies) {
    this.tools = new Map(createTools(dependencies).map((tool) => [tool.name, tool]));
  }
  list() { return [...this.tools.values()]; }
  get(name: AIToolName) { const tool = this.tools.get(name); if (!tool) throw new Error(`Tool não registrada: ${name}`); return tool; }
  async execute(name: AIToolName, input: AIToolInput): Promise<AIToolResult> {
    const tool = this.get(name);
    try { const data = await tool.execute(input); return { name, domain: tool.domain, data, empty: isEmpty(data) }; }
    catch (error) { console.error(`[AI_CORE] Falha na tool ${name}`); throw error; }
  }
}
