import { getDatabaseInfo } from "../../services/databaseService";
import { noteService } from "../../services/noteService";
import { taskService } from "../../services/taskService";
import { localDateKey } from "../../services/dateService";
import { systemService } from "../../services/systemService";
import { aiRepository } from "../../repositories/aiRepository";
import { tradingRepository } from "../../repositories/tradingRepository";
import type { TradingLive, TradingOverview } from "../../types";
import type { AgentToolSchema, AISettings, AIToolInput, AIToolName, AIToolPermission, AIToolResult, DailyCounters, MemorySnapshot, Note, OllamaStatus, ProcessSnapshot, StorageSnapshot, SystemSnapshot, Task } from "../../types";

export interface AzrielTool { name: AIToolName; description: string; domain: string; readonly: boolean; permission?: AIToolPermission; execute(input: AIToolInput): Promise<unknown> | unknown }
export interface ToolDependencies {
  tasks: { list(): Promise<Task[]>; today(): Promise<Task[]>; upcoming(): Promise<Task[]>; counters(): Promise<DailyCounters>; completeReferenced(conversationId: string, position: number): Promise<Task> };
  notes: { list(includeArchived?: boolean): Promise<Note[]> };
  databaseInfo(): Promise<{ schemaVersion: number }>;
  system: { snapshot(): Promise<SystemSnapshot>; processes(): Promise<ProcessSnapshot[]> };
  ollama: { settings(): Promise<AISettings>; status(endpoint: string, timeoutSeconds: number): Promise<OllamaStatus> };
  trading?: { overview(): Promise<TradingOverview>; live(): Promise<TradingLive> };
}
const production: ToolDependencies = { tasks: taskService, notes: noteService, databaseInfo: getDatabaseInfo, system: systemService, ollama: { settings: aiRepository.getSettings, status: aiRepository.status }, trading: tradingRepository };

/** Resumo do trading para o agente: números prontos, sem chaves nem detalhes internos. */
export async function tradingStatus(trading: NonNullable<ToolDependencies["trading"]>) {
  const overview = await trading.overview();
  const live = await trading.live().catch(() => null);
  const equity = live?.equity ?? overview.equity.at(-1)?.equity ?? null;
  const delta = (base: number | null) => equity != null && base ? { usdt: Math.round((equity - base) * 100) / 100, pct: Math.round(((equity - base) / base) * 10_000) / 100 } : null;
  return {
    account: "BingX DEMO (dinheiro virtual)",
    motorLigado: overview.engine.running,
    par: overview.settings.symbol,
    patrimonioUsdt: equity,
    resultadoHoje: delta(overview.dayStartEquity),
    resultadoTotal: delta(overview.firstEquity),
    posicao: live?.position ?? null,
    ultimoErro: overview.engine.lastError,
    ultimasDecisoes: overview.decisions.slice(0, 5).map((d) => ({ quando: d.createdAt, acao: d.finalAction, confianca: d.confidence, motivo: d.reason, ordem: d.orderStatus })),
  };
}
const unfinished = (task: Task) => !["completed", "cancelled"].includes(task.status);
// Modelos pequenos erram conta com bytes: as ferramentas já entregam GB e percentuais.
const gb = (bytes: number) => Math.round((bytes / 1024 ** 3) * 10) / 10;
const percent = (part: number, total: number) => total > 0 ? Math.round((part / total) * 100) : 0;
const memoryView = (memory: MemorySnapshot) => ({ totalGb: gb(memory.totalBytes), usedGb: gb(memory.usedBytes), availableGb: gb(memory.availableBytes), usedPercent: percent(memory.usedBytes, memory.totalBytes) });
const storageView = (volume: StorageSnapshot) => ({ name: volume.name, mountPoint: volume.mountPoint, totalGb: gb(volume.totalBytes), freeGb: gb(volume.availableBytes), usedPercent: percent(volume.totalBytes - volume.availableBytes, volume.totalBytes) });

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
    { name:"get_system_status", domain:"Sistema", description:"Resumo do PC do operador: CPU, memória RAM e tempo ligado", readonly:true, execute:async()=>{const s=await deps.system.snapshot();return {cpuUsagePercent:s.cpu.usagePercent,memoryRam:memoryView(s.memory),uptimeHours:Math.round(s.details.uptimeSeconds/360)/10,errors:s.errors};} },
    { name:"get_cpu_status", domain:"Sistema", description:"Uso de CPU do PC do operador", readonly:true, execute:async()=> (await deps.system.snapshot()).cpu },
    { name:"get_memory_status", domain:"Sistema", description:"Memória RAM do PC do operador, em GB", readonly:true, execute:async()=> memoryView((await deps.system.snapshot()).memory) },
    { name:"get_storage_status", domain:"Sistema", description:"Discos do PC do operador, espaço livre em GB", readonly:true, execute:async()=> (await deps.system.snapshot()).storage.map(storageView) },
    { name:"get_network_status", domain:"Sistema", description:"Interfaces de rede", readonly:true, execute:async()=> (await deps.system.snapshot()).network },
    { name:"get_process_summary", domain:"Sistema", description:"Processos que mais usam memória, em MB", readonly:true, execute:async()=> (await deps.system.processes()).sort((a,b)=>b.memoryBytes-a.memoryBytes).slice(0,10).map((p)=>({name:p.name,pid:p.pid,cpuPercent:Math.round(p.cpuPercent),memoryMb:Math.round(p.memoryBytes/1024**2)})) },
    { name:"get_ollama_status", domain:"AI Core", description:"Estado do Ollama local", readonly:true, execute:async()=>{const s=await deps.ollama.settings();return deps.ollama.status(s.endpoint,Math.min(s.timeoutSeconds,8));} },
    { name:"get_azriel_status", domain:"Azriel", description:"Resumo dos serviços ativos", readonly:true, execute:async()=>({database:{schemaVersion:(await deps.databaseInfo()).schemaVersion},daily:await deps.tasks.counters(),ollama:await (async()=>{const s=await deps.ollama.settings();return deps.ollama.status(s.endpoint,Math.min(s.timeoutSeconds,8));})()}) },
    { name:"get_trading_status", domain:"Trading", description:"Situação do agente de trading na conta demo: motor, patrimônio, resultado de hoje e total, posição e últimas decisões", readonly:true, execute:()=> deps.trading ? tradingStatus(deps.trading) : { error:"Trading indisponível" } },
    { name:"get_azriel_version", domain:"Azriel", description:"Versão do núcleo atual", readonly:true, execute:()=>({version:"core-only"}) },
  ];
}
export class ToolRegistry {
  private readonly indexed = new Map<AIToolName, AzrielTool>();
  constructor(dependencies: ToolDependencies = production) { tools(dependencies).forEach((tool)=>this.indexed.set(tool.name,tool)); }
  has(name: string): name is AIToolName { return this.indexed.has(name as AIToolName); }
  /** Ferramentas expostas ao loop de agente: somente leitura. Escritas seguem pelos comandos explícitos. */
  agentTools(): AgentToolSchema[] {
    return this.list().filter((tool)=>tool.readonly).map((tool)=>({ type:"function", function:{ name:tool.name, description:`${tool.description} (${tool.domain})`, parameters:{ type:"object", properties:{} } } }));
  }
  get(name: AIToolName) { const tool=this.indexed.get(name); if(!tool) throw new Error(`Ferramenta indisponível: ${name}`); return tool; }
  async execute(name: AIToolName,input:AIToolInput):Promise<AIToolResult>{const tool=this.get(name);const data=await tool.execute(input);return {name,domain:tool.domain,data,empty:Array.isArray(data)?data.length===0:data==null};}
  list(){return [...this.indexed.values()];}
}
