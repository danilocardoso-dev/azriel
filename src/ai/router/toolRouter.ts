import type { AIToolName, RoutedIntent } from "../../types";
const normalize=(value:string)=>value.normalize("NFD").replace(/[\u0300-\u036f]/g,"").toLowerCase();
const has=(value:string, expression:RegExp)=>expression.test(value);
export function routeIntent(query:string):RoutedIntent {
  const value=normalize(query);
  if(!value.trim()) return {intent:"empty",scope:"general",tools:[]};
  if(/^(?:azriel\s+)?(?:finaliza|finalize|conclua|concluir|marque)\b/.test(value)) {
    const match=value.match(/\b(?:a|o)\s+(\d+)\b/);
    return match?{intent:"complete_task_reference",scope:"azriel",tools:["complete_task"],referencePosition:Number(match[1])}:{intent:"unsupported_internal",scope:"azriel",tools:[]};
  }
  if(has(value,/\b(demanda|demandas|tarefa|tarefas|pendencia|pendencias)\b/) && has(value,/\bcritica/)) return {intent:"critical_tasks",scope:"azriel",tools:["get_critical_tasks"]};
  if(has(value,/\b(demanda|demandas|tarefa|tarefas|pendencia|pendencias|prioridade)\b/) && has(value,/\b(alta|alto|priorit)/)) return {intent:"priority_tasks",scope:"azriel",tools:["get_priority_tasks"]};
  if(value.includes("atrasad")) return {intent:"overdue",scope:"azriel",tools:["get_overdue_tasks"]};
  if(value.includes("hoje")) return {intent:"today",scope:"azriel",tools:["get_today_tasks"]};
  if(value.includes("proxim")) return {intent:"upcoming",scope:"azriel",tools:["get_upcoming_tasks"]};
  if(value.includes("nota")) return {intent:"notes",scope:"azriel",tools:["get_recent_notes"]};
  if(value.includes("ollama")||value.includes("modelo")) return {intent:"ollama",scope:"azriel",tools:["get_ollama_status"]};
  if(value.includes("cpu")||value.includes("processador")) return {intent:"cpu",scope:"azriel",tools:["get_cpu_status"]};
  if(value.includes("memoria")||value.includes("ram")) return {intent:"memory",scope:"azriel",tools:["get_memory_status"]};
  if(value.includes("disco")||value.includes("armazenamento")) return {intent:"storage",scope:"azriel",tools:["get_storage_status"]};
  if(value.includes("rede")) return {intent:"network",scope:"azriel",tools:["get_network_status"]};
  if(value.includes("process")) return {intent:"processes",scope:"azriel",tools:["get_process_summary"]};
  if(value.includes("situacao")||value.includes("status do azriel")) return {intent:"status",scope:"azriel",tools:["get_daily_operations_summary","get_system_status","get_ollama_status"]};
  if(value.includes("versao")) return {intent:"version",scope:"azriel",tools:["get_azriel_version"]};
  const tools: AIToolName[]=[];
  return {intent:"general_knowledge",scope:"general",tools};
}
