import type { AIToolName, RoutedIntent } from "../../types";

const normalize = (value: string) => value.normalize("NFD").replace(/[\u0300-\u036f]/g, "").toLowerCase().replace(/[^a-z0-9\s]/g, " ").replace(/\s+/g, " ").trim();
const domains = ["bioinformática", "genética", "biologia molecular", "iot", "eletrônica", "robótica", "inteligência artificial", "big data", "biotecnologia"];
const unique = (tools: AIToolName[]) => [...new Set(tools)];

function termFrom(query: string) {
  const normalized = normalize(query);
  return domains.find((domain) => normalized.includes(normalize(domain)))
    ?? normalized.match(/relacionad[oa]s? (?:a|com) ([a-z0-9 ]+)/)?.[1]
    ?? normalized.match(/trabalham? ([a-z0-9 ]+)/)?.[1];
}

export function routeIntent(query: string): RoutedIntent {
  const value = normalize(query);
  const term = termFrom(query);
  if (!value) return { intent: "empty", scope: "general", tools: [] };
  const explicitOpen = /^(?:azriel\s+)?(?:abra|abrir)\b/.test(value);
  const explicitReveal = /^(?:azriel\s+)?(?:mostre|mostrar|revele|revelar)\b/.test(value) && value.includes("pasta");
  if (explicitReveal) return { intent: "reveal_workspace", scope: "azriel", term: value, tools: ["reveal_workspace"] };
  if (explicitOpen && value.includes("workspace")) return { intent: "open_workspace", scope: "azriel", term: value, tools: ["open_workspace"] };
  if (explicitOpen && /(github|url|site|pagina|link)\b/.test(value)) return { intent: "open_registered_url", scope: "azriel", term: value, tools: ["open_registered_url"] };
  if (explicitOpen && /(aplicativo|programa|visual studio code|vscode|photoshop|chrome|ollama)\b/.test(value)) return { intent: "open_application", scope: "azriel", term: value, tools: ["open_application"] };
  if (explicitOpen) return { intent: "open_project", scope: "azriel", term: value, tools: ["open_project"] };
  if (value === "situacao" || value.includes("azriel situacao") || value.includes("resumo da situacao")) {
    return { intent: "situation", scope: "azriel", tools: ["get_daily_operations_summary", "get_today_tasks", "get_overdue_tasks", "list_projects", "get_knowledge_gaps", "get_current_education", "get_system_status", "list_workspaces", "get_ollama_status"] };
  }
  if (value.includes("process")) return { intent: "processes", scope: "azriel", tools: ["get_process_summary"] };
  if (value.includes("cpu") || value.includes("processador")) return { intent: "cpu", scope: "azriel", tools: ["get_cpu_status"] };
  if (value.includes("memoria") || value.includes(" ram")) return { intent: "memory", scope: "azriel", tools: ["get_memory_status"] };
  if (value.includes("disco") || value.includes("armazenamento") || value.includes("espaco livre")) return { intent: "storage", scope: "azriel", tools: ["get_storage_status"] };
  if (value.includes("rede") || value.includes("network")) return { intent: "network", scope: "azriel", tools: ["get_network_status"] };
  if (value.includes("ollama")) return { intent: "ollama", scope: "azriel", tools: ["get_ollama_status"] };
  if (value.includes("commit")) return { intent: "git_commits", scope: "azriel", term: value, tools: ["list_workspaces", "get_recent_commits"] };
  if (value.includes("git") || value.includes("repositorio")) return { intent: "git", scope: "azriel", term: value, tools: ["list_workspaces", "get_git_status"] };
  if (value.includes("workspace") || value.includes("pasta autorizada")) return { intent: "workspaces", scope: "azriel", term: value, tools: ["list_workspaces", "get_workspace_status"] };
  if (value.includes("projet") && value.includes("lacuna")) return { intent: "projects_for_gaps", scope: "azriel", tools: ["list_projects", "list_knowledge_areas", "get_knowledge_gaps"] };
  if ((value.includes("fazendo") || value.includes("atividade") || value.includes("trabalhando")) && term) {
    return { intent: "cross_domain_activity", scope: "azriel", term, tools: ["list_projects", "list_knowledge_areas", "get_today_tasks", "get_upcoming_tasks", "get_recent_notes"] };
  }
  if (value.includes("atrasad")) return { intent: "overdue", scope: "azriel", tools: ["get_overdue_tasks"] };
  if (value.includes("hoje")) return { intent: "today", scope: "azriel", tools: ["get_today_tasks"] };
  if (value.includes("proxim")) return { intent: "upcoming", scope: "azriel", tools: ["get_upcoming_tasks"] };
  if (value.includes("nota")) return { intent: "notes", scope: "azriel", tools: ["get_recent_notes"] };
  if (value.includes("mapa stark") || value.includes("stark")) return { intent: "stark_map", scope: "azriel", tools: ["get_stark_map"] };
  if (value.includes("maior lacuna") || value.includes("lacuna")) return { intent: "knowledge_gaps", scope: "azriel", tools: ["get_knowledge_gaps"] };
  if (value.includes("formacao") || value.includes("faculdade") || value.includes("curso")) return { intent: "education", scope: "azriel", tools: ["get_current_education", "get_planned_education"] };
  if (value.includes("projet")) {
    const tools: AIToolName[] = term ? ["list_projects", "list_knowledge_areas"] : ["list_projects"];
    return { intent: term ? "projects_by_knowledge" : "projects", scope: "azriel", term, tools: unique(tools) };
  }
  if (term || value.includes("conhecimento")) return { intent: "knowledge", scope: "azriel", term, tools: ["list_knowledge_areas"] };
  if (value.includes("versao") && (value.includes("azriel") || value.includes("sistema"))) return { intent: "version", scope: "azriel", tools: ["get_azriel_version"] };
  if ((value.includes("status") || value.includes("estado")) && (value.includes("azriel") || value.includes("sistema"))) return { intent: "azriel_status", scope: "azriel", tools: ["get_azriel_status"] };
  return { intent: "general_knowledge", scope: "general", tools: [] };
}
