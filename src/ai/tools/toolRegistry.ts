import { educationService } from "../../services/educationService";
import { getDatabaseInfo } from "../../services/databaseService";
import { diagnoseGaps, knowledgeService } from "../../services/knowledgeService";
import { noteService } from "../../services/noteService";
import { projectService } from "../../services/projectService";
import { taskService } from "../../services/taskService";
import { localDateKey } from "../../services/dateService";
import type { AIToolInput, AIToolName, AIToolResult, DailyCounters, EducationItem, KnowledgeArea, KnowledgeHistory, Note, Project, Task } from "../../types";

export interface AzrielTool {
  name: AIToolName;
  description: string;
  domain: string;
  readonly: true;
  execute(input: AIToolInput): Promise<unknown>;
}

export interface ToolDependencies {
  tasks: { list(): Promise<Task[]>; today(): Promise<Task[]>; upcoming(): Promise<Task[]>; counters(): Promise<DailyCounters> };
  notes: { list(includeArchived?: boolean): Promise<Note[]> };
  projects: { list(): Promise<Project[]>; get(id: string): Promise<Project | null> };
  knowledge: { list(): Promise<KnowledgeArea[]>; get(id: string): Promise<KnowledgeArea | null>; history(id: string): Promise<KnowledgeHistory[]> };
  education: { list(): Promise<EducationItem[]> };
  databaseInfo(): Promise<{ schemaVersion: number; integrationValue: number }>;
}

const productionDependencies: ToolDependencies = {
  tasks: taskService,
  notes: noteService,
  projects: projectService,
  knowledge: knowledgeService,
  education: educationService,
  databaseInfo: getDatabaseInfo,
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
    { name: "get_azriel_status", domain: "Azriel", description: "Estado interno persistente do Azriel", readonly: true, execute: async () => { const [database, daily] = await Promise.all([dependencies.databaseInfo(), dependencies.tasks.counters()]); return { version: "0.6.0", database, daily, aiCore: "online quando Ollama disponível", writeAccess: false }; } },
    { name: "get_azriel_version", domain: "Azriel", description: "Versão atual", readonly: true, execute: async () => ({ version: "0.6.0", name: "AI Core" }) },
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
