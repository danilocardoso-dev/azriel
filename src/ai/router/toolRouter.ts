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

const contextualComponent = /\b(?:essa|esta|desta) peca\b|\b(?:esse|este|desse) componente\b|\bpeca selecionada\b|\bcomponente selecionado\b|\b(?:isso|ela|ele)\b/;

function engineeringComponentTerm(value: string, verbs: string): string | undefined {
  if (contextualComponent.test(value)) return undefined;
  const match = value.match(new RegExp(`(?:${verbs})\\s+(?:(?:o|a|ao|do|da|de|no|na)\\s+)?(.+)$`));
  return match?.[1]?.replace(/^(?:componente|peca)\s+/, "").trim();
}

function engineeringIntent(value: string): RoutedIntent | null {
  const subsystemTerm = value.match(/subsistema\s+(.+?)(?:\?|$)/)?.[1]?.trim();
  if (/\b(?:cobertura|percentual)\b/.test(value) && /\b(?:semantica|classificad)/.test(value)) return { intent: "semantic_coverage", scope: "azriel", tools: ["get_semantic_coverage"] };
  if (/\b(?:nao (?:foram )?classificados?|sem classificacao|pendentes? de classificacao)\b/.test(value)) return { intent: "unclassified_components", scope: "azriel", tools: ["get_unclassified_components"] };
  if (/\b(?:resuma|resumo)\b/.test(value) && /\b(?:montagem|assembly|estrutura)\b/.test(value)) return { intent: "assembly_graph_summary", scope: "azriel", tools: ["get_assembly_graph_summary"] };
  if (/\b(?:quais|liste|listar)\b/.test(value) && /\bsubsistemas?\b/.test(value) && !/\bcomponentes?\b/.test(value)) return { intent: "subsystem_list", scope: "azriel", tools: ["get_subsystems"] };
  if (/\bcomponentes?\b/.test(value) && /\b(?:pertencem|pertencentes|subsistema)\b/.test(value) && subsystemTerm) return { intent: "subsystem_components", scope: "azriel", term: subsystemTerm, tools: ["get_subsystem_components"] };
  if (/\b(?:relacoes|relacionamentos|conexoes|conectad[oa])\b/.test(value) && /\b(?:peca|componente|selecionad[ao]|essa|este|esta)\b/.test(value)) return { intent: "component_relationships", scope: "azriel", term: engineeringComponentTerm(value, "relacoes|relacionamentos|conexoes"), tools: ["get_component_relationships"] };
  if (/\b(?:funcao|papel|semantica|subsistema)\b/.test(value) && /\b(?:peca|componente|selecionad[ao]|essa|este|esta|desta|desse)\b/.test(value)) return { intent: "component_semantics", scope: "azriel", term: engineeringComponentTerm(value, "funcao|papel|semantica"), tools: ["get_component_semantics"] };
  const percentage = value.match(/(\d{1,3})(?:\s*por cento)?\b/);
  if (percentage && /\b(?:abra|abrir|exploda|explodir|explosao|fator)\b/.test(value)) return { intent: "explosion_adjust", scope: "azriel", tools: ["set_explosion_factor"], factor: Number(percentage[1]) / 100 };
  if (/\b(?:abra|abrir)\s+(?:um\s+)?(?:pouco\s+)?mais\b/.test(value)) return { intent: "explosion_adjust", scope: "azriel", tools: ["set_explosion_factor"], delta: 0.15 };
  if (/\b(?:feche|fechar)\s+(?:um\s+)?(?:pouco\s+)?(?:mais)?\b/.test(value)) return { intent: "explosion_adjust", scope: "azriel", tools: ["set_explosion_factor"], delta: -0.15 };
  if (/\b(?:reconstrua|reconstruir|reagrupe|reagrupar|remonte|remontar)\b/.test(value)) return { intent: "assembly_reassemble", scope: "azriel", tools: ["reassemble"] };
  if (/\b(?:resete|resetar|restaure|restaurar)\b/.test(value) && /\b(?:modelo|visualizacao|vista|camera)\b/.test(value)) return { intent: "model_view_reset", scope: "azriel", tools: ["reset_model_view"] };
  if (/\b(?:exploda|explodir)\b/.test(value) && /\b(?:montagem|modelo|tudo|todos)\b/.test(value)) return { intent: "assembly_explode", scope: "azriel", tools: ["explode_all"] };
  if (/\b(?:exploda|explodir)\b/.test(value)) return { intent: "component_explode", scope: "azriel", term: engineeringComponentTerm(value, "exploda|explodir"), tools: ["explode_component"] };
  if (/\b(?:mostre|mostrar|exiba|exibir|revele|revelar)\b/.test(value) && /\b(?:todos|todas)\b/.test(value) && /\bcomponentes?\b/.test(value)) return { intent: "component_show_all", scope: "azriel", tools: ["show_all_components"] };
  if (/\b(?:oculte|ocultar|esconda|esconder)\b/.test(value)) return { intent: "component_visibility", scope: "azriel", term: engineeringComponentTerm(value, "oculte|ocultar|esconda|esconder"), tools: ["hide_component"] };
  if (/\b(?:mostre|mostrar|exiba|exibir)\b/.test(value) && !/\b(?:pasta|mapa|projeto|nota|rotina|tarefa|formacao|sistema|status|workspace)\b/.test(value)) return { intent: "component_visibility", scope: "azriel", term: engineeringComponentTerm(value, "mostre|mostrar|exiba|exibir"), tools: ["show_component"] };
  if (/\b(?:isole|isolar)\b/.test(value)) return { intent: "component_isolate", scope: "azriel", term: engineeringComponentTerm(value, "isole|isolar"), tools: ["isolate_component"] };
  if (/\b(?:foque|focar|enquadre|enquadrar)\b/.test(value)) return { intent: "component_focus", scope: "azriel", term: engineeringComponentTerm(value, "foque|focar|enquadre|enquadrar"), tools: ["focus_component"] };
  if (/\b(?:selecione|selecionar|destaque|destacar|marque|marcar)\b/.test(value)) return { intent: "component_select", scope: "azriel", term: engineeringComponentTerm(value, "selecione|selecionar|destaque|destacar|marque|marcar"), tools: ["select_component"] };
  if (/\b(?:encontre|encontrar|localize|localizar|procure|procurar)\b/.test(value)) return { intent: "component_search", scope: "azriel", term: engineeringComponentTerm(value, "encontre|encontrar|localize|localizar|procure|procurar"), tools: ["find_component"] };
  if (/\b(?:qual|que)\b/.test(value) && /\b(?:peca|componente)\b/.test(value) && /\bselecionad[ao]\b/.test(value)) return { intent: "component_selected", scope: "azriel", tools: ["get_selected_component"] };
  if (contextualComponent.test(value) && /\b(?:o que e|qual e|fale sobre|informacoes sobre)\b/.test(value)) return { intent: "component_inspect", scope: "azriel", tools: ["get_component_details"] };
  if (/\b(?:detalhes?|inspecione|inspecionar)\b/.test(value) && !/\b(?:projeto|rotina|tarefa|sistema|workspace)\b/.test(value)) return { intent: "component_inspect", scope: "azriel", term: engineeringComponentTerm(value, "detalhe|detalhes|inspecione|inspecionar"), tools: ["get_component_details"] };
  if ((/\b(?:detalhes?|filhos?|visivel|material|dimensoes|transform)\b/.test(value) && (contextualComponent.test(value) || /\b(?:peca|componente)\b/.test(value)))) return { intent: "component_inspect", scope: "azriel", term: engineeringComponentTerm(value, "detalhe|detalhes|inspecione|inspecionar"), tools: ["get_component_details"] };
  if (/\b(?:liste|listar|quais)\b/.test(value) && /\bcomponentes?\b/.test(value)) return { intent: "component_list", scope: "azriel", tools: ["list_components"] };
  if (/\b(?:quantos|numero de|total de)\b/.test(value) && /\bcomponentes?\b/.test(value)) return { intent: "model_summary", scope: "azriel", tools: ["get_model_summary"] };
  if (/\b(?:explosao|exploded view)\b/.test(value) && /\b(?:estado|status|fator|quanto|como)\b/.test(value)) return { intent: "explosion_state", scope: "azriel", tools: ["get_explosion_state"] };
  if (/\bmodelo\b/.test(value) && /\b(?:carregado|aberto|atual|status)\b/.test(value)) return { intent: "model_status", scope: "azriel", tools: ["get_loaded_model"] };
  return null;
}

export function routeIntent(query: string): RoutedIntent {
  const value = normalize(query);
  const term = termFrom(query);
  if (!value) return { intent: "empty", scope: "general", tools: [] };
  const engineering = engineeringIntent(value);
  if (engineering) return engineering;
  const explicitOpen = /^(?:azriel\s+)?(?:abra|abrir)\b/.test(value);
  const explicitReveal = /^(?:azriel\s+)?(?:mostre|mostrar|revele|revelar)\b/.test(value) && value.includes("pasta");
  const explicitRoutine = /^(?:azriel\s+)?(?:execute|executar|inicie|iniciar|rode|rodar)\b/.test(value) && value.includes("rotina");
  if (explicitRoutine) return { intent: "run_routine", scope: "azriel", term: value, tools: ["run_routine"] };
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
  if (value.includes("por que") && (value.includes("cobertura") || value.includes("profundidade") || value.includes("nivel") || value.includes("aument"))) return { intent: "knowledge_explanation", scope: "azriel", term, tools: ["explain_knowledge_level"] };
  if (value.includes("contribuiu") || value.includes("evidencia")) return { intent: "knowledge_evidence", scope: "azriel", term, tools: ["get_knowledge_evidence"] };
  if (value.includes("evoluiram") || (value.includes("eventos") && value.includes("conhecimento"))) return { intent: "recent_learning", scope: "azriel", tools: ["get_recent_knowledge_events"] };
  if (value.includes("dominio") || (value.includes("nivel") && value.includes("topico"))) return { intent: "topic_mastery", scope: "azriel", term, tools: ["get_topic_mastery"] };
  if (value.includes("roadmap")) {
    const details = value.includes("como esta") || value.includes("topico") || value.includes("atividade") || value.includes("conhecimento");
    if (value.includes("contribuiu") || value.includes("aprendizado") || value.includes("evolucao")) return { intent: "roadmap_learning", scope: "azriel", term, tools: ["get_roadmap_learning_status"] };
    return { intent: details ? "roadmap_details" : "roadmaps", scope: "azriel", term, tools: [details ? "get_study_roadmap" : "list_study_roadmaps"] };
  }
  if (value.includes("topico") && (value.includes("nao comecei") || value.includes("nao iniciado"))) return { intent: "roadmap_topics_not_started", scope: "azriel", tools: ["list_study_roadmaps"] };
  if ((value.includes("pesquisa") || value.includes("pesquisas")) && (value.includes("minha") || value.includes("minhas") || value.includes("relacionad") || term)) return { intent: "research", scope: "azriel", term, tools: ["list_research_items"] };
  if (value.includes("rotina")) return { intent: "routines", scope: "azriel", term: value, tools: ["list_routines"] };
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
