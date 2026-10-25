import { getDatabaseInfo } from "../../services/databaseService";
import { noteService } from "../../services/noteService";
import { taskService } from "../../services/taskService";
import { localDateKey } from "../../services/dateService";
import { systemService } from "../../services/systemService";
import { aiRepository } from "../../repositories/aiRepository";
import type { AISettings, AIToolInput, AIToolName, AIToolPermission, AIToolResult, DailyCounters, Note, OllamaStatus, ProcessSnapshot, SystemSnapshot, Task } from "../../types";

export interface AzrielTool { name: AIToolName; description: string; domain: string; readonly: boolean; permission?: AIToolPermission; execute(input: AIToolInput): Promise<unknown> | unknown }
export interface ToolDependencies {
  tasks: { list(): Promise<Task[]>; today(): Promise<Task[]>; upcoming(): Promise<Task[]>; counters(): Promise<DailyCounters>; completeReferenced(conversationId: string, position: number): Promise<Task> };
  notes: { list(includeArchived?: boolean): Promise<Note[]> };
  databaseInfo(): Promise<{ schemaVersion: number }>;
  system: { snapshot(): Promise<SystemSnapshot>; processes(): Promise<ProcessSnapshot[]> };
  ollama: { settings(): Promise<AISettings>; status(endpoint: string, timeoutSeconds: number): Promise<OllamaStatus> };
}
const production: ToolDependencies = { tasks: taskService, notes: noteService, databaseInfo: getDatabaseInfo, system: systemService, ollama: { settings: aiRepository.getSettings, status: aiRepository.status } };
const unfinished = (task: Task) => !["completed", "cancelled"].includes(task.status);

function tools(deps: ToolDependencies): AzrielTool[] {
  return [
    { name:"get_today_tasks", domain:"Operações Diárias", description:"Tarefas de hoje", readonly:true, execute:()=>deps.tasks.today() },
    { name:"get_overdue_tasks", domain:"Operações Diárias", description:"Tarefas atrasadas", readonly:true, execute:async()=>{const now=localDateKey();return (await deps.tasks.list()).filter((t)=>unfinished(t)&&!!t.dueDate&&t.dueDate<now);} },
    { name:"get_upcoming_tasks", domain:"Operações Diárias", description:"Próximas tarefas", readonly:true, execute:()=>deps.tasks.upcoming() },
    { name:"get_critical_tasks", domain:"Operações Diárias", description:"Demandas críticas ativas", readonly:true, execute:async()=> (await deps.tasks.list()).filter((t)=>unfinished(t)&&t.priority==="critical") },
    { name:"get_priority_tasks", domain:"Operações Diárias", description:"Demandas de alta prioridade ativas", readonly:true, execute:async()=> (await deps.tasks.list()).filter((t)=>unfinished(t)&&["critical","high"].includes(t.priority)) },
    { name:"complete_task", domain:"Operações Diárias", description:"Conclui uma tarefa da lista mais recente", readonly:false, permission:"safe_write", execute:async(input)=>{if(!input.conversationId||!input.referencePosition)return {success:false,error:"Referência segura não encontrada."};try{return {success:true,task:await deps.tasks.completeReferenced(input.conversationId,input.referencePosition)}}catch(error){return {success:false,error:error instanceof Error?error.message:String(error)}}} },
    { name:"get_recent_notes", domain:"Operações Diárias", description:"Notas recentes", readonly:true, execute:async()=> (await deps.notes.list(false)).slice(0,5) },
    { name:"get_daily_operations_summary", domain:"Operações Diárias", description:"Resumo de tarefas e notas", readonly:true, execute:()=>deps.tasks.counters() },
    { name:"get_system_status", domain:"Sistema", description:"Telemetria local do dispositivo", readonly:true, execute:async()=>{const s=await deps.system.snapshot();return {cpuUsagePercent:s.cpu.usagePercent,memory:s.memory,uptimeSeconds:s.details.uptimeSeconds,errors:s.errors};} },
    { name:"get_cpu_status", domain:"Sistema", description:"Uso de CPU", readonly:true, execute:async()=> (await deps.system.snapshot()).cpu },
    { name:"get_memory_status", domain:"Sistema", description:"Uso de memória", readonly:true, execute:async()=> (await deps.system.snapshot()).memory },
    { name:"get_storage_status", domain:"Sistema", description:"Volumes locais", readonly:true, execute:async()=> (await deps.system.snapshot()).storage },
    { name:"get_network_status", domain:"Sistema", description:"Interfaces de rede", readonly:true, execute:async()=> (await deps.system.snapshot()).network },
    { name:"get_process_summary", domain:"Sistema", description:"Processos com maior consumo", readonly:true, execute:async()=> (await deps.system.processes()).sort((a,b)=>b.memoryBytes-a.memoryBytes).slice(0,10) },
    { name:"get_ollama_status", domain:"AI Core", description:"Estado do Ollama local", readonly:true, execute:async()=>{const s=await deps.ollama.settings();return deps.ollama.status(s.endpoint,Math.min(s.timeoutSeconds,8));} },
    { name:"get_azriel_status", domain:"Azriel", description:"Resumo dos serviços ativos", readonly:true, execute:async()=>({database:await deps.databaseInfo(),daily:await deps.tasks.counters(),ollama:await (async()=>{const s=await deps.ollama.settings();return deps.ollama.status(s.endpoint,Math.min(s.timeoutSeconds,8));})()}) },
    { name:"get_azriel_version", domain:"Azriel", description:"Versão do núcleo atual", readonly:true, execute:()=>({version:"core-only"}) },
  ];
}
export class ToolRegistry {
  private readonly indexed = new Map<AIToolName, AzrielTool>();
  constructor(dependencies: ToolDependencies = production) { tools(dependencies).forEach((tool)=>this.indexed.set(tool.name,tool)); }
  get(name: AIToolName) { const tool=this.indexed.get(name); if(!tool) throw new Error(`Ferramenta indisponível: ${name}`); return tool; }
  async execute(name: AIToolName,input:AIToolInput):Promise<AIToolResult>{const tool=this.get(name);const data=await tool.execute(input);return {name,domain:tool.domain,data,empty:Array.isArray(data)?data.length===0:data==null};}
  list(){return [...this.indexed.values()];}
}
