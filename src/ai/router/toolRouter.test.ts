import { describe, expect, it } from "vitest";
import { routeIntent } from "./toolRouter";

describe("Tool Router", () => {
  it.each([
    ["O que tenho para hoje?", "get_today_tasks"],
    ["Tenho alguma atividade atrasada?", "get_overdue_tasks"],
    ["Quais são meus projetos?", "list_projects"],
    ["Qual minha maior lacuna?", "get_knowledge_gaps"],
    ["Como está meu Mapa Stark?", "get_stark_map"],
    ["Como está minha formação?", "get_current_education"],
    ["Azriel, situação.", "get_daily_operations_summary"],
    ["Quais processos estão consumindo mais memória?", "get_process_summary"],
    ["Como está o uso da CPU?", "get_cpu_status"],
    ["Quais workspaces estão cadastrados?", "list_workspaces"],
    ["Quais projetos Git possuem alterações?", "get_git_status"],
    ["O Ollama está disponível?", "get_ollama_status"],
    ["Quais rotinas eu tenho?", "list_routines"],
    ["Quais roadmaps estão ativos?", "list_study_roadmaps"],
    ["Quais tópicos ainda não comecei?", "list_study_roadmaps"],
    ["Como está meu roadmap de Controle e Automação?", "get_study_roadmap"],
    ["Quais pesquisas estão relacionadas a Bioinformática?", "list_research_items"],
    ["Por que minha cobertura em Eletrônica está nesse nível?", "explain_knowledge_level"],
    ["Qual modelo está carregado?", "get_loaded_model"],
    ["Quantos componentes ele possui?", "get_model_summary"],
    ["Encontre o rotor.", "find_component"],
    ["Selecione o rotor.", "select_component"],
    ["Isole o rotor.", "isolate_component"],
    ["Exploda a montagem.", "explode_all"],
    ["Reconstrua a montagem.", "reassemble"],
    ["Qual peça está selecionada?", "get_selected_component"],
  ])("roteia %s para %s", (query, tool) => expect(routeIntent(query).tools).toContain(tool));

  it.each([
    ["Qual é a função desta peça?", "get_component_semantics"],
    ["Quais subsistemas existem?", "get_subsystems"],
    ["Quais componentes pertencem ao subsistema Drive Train?", "get_subsystem_components"],
    ["Quais relações possui esse componente?", "get_component_relationships"],
    ["Quais componentes ainda não foram classificados?", "get_unclassified_components"],
    ["Qual a cobertura semântica?", "get_semantic_coverage"],
    ["Resuma a estrutura da montagem.", "get_assembly_graph_summary"],
  ])("roteia leitura semântica %s", (query, tool) => expect(routeIntent(query).tools).toEqual([tool]));

  it("resolve fatores absolutos e incrementais no software", () => {
    expect(routeIntent("Abra em 50%.")).toMatchObject({ tools: ["set_explosion_factor"], factor: 0.5 });
    expect(routeIntent("Exploda a montagem em 70%.")).toMatchObject({ tools: ["set_explosion_factor"], factor: 0.7 });
    expect(routeIntent("Abra mais.")).toMatchObject({ tools: ["set_explosion_factor"], delta: 0.15 });
    expect(routeIntent("Feche um pouco.")).toMatchObject({ tools: ["set_explosion_factor"], delta: -0.15 });
  });

  it.each([
    ["Mostre todos os componentes.", "show_all_components"],
    ["Oculte o rotor.", "hide_component"],
    ["Mostre o rotor.", "show_component"],
    ["Foque o rotor.", "focus_component"],
    ["Exploda o conjunto do eixo.", "explode_component"],
    ["Resete a visualização do modelo.", "reset_model_view"],
  ])("roteia ação visual %s", (query, tool) => expect(routeIntent(query).tools).toEqual([tool]));

  it("mantém referências contextuais sem inventar um ID", () => {
    expect(routeIntent("Isole essa peça.")).toMatchObject({ tools: ["isolate_component"], term: undefined });
    expect(routeIntent("Esse componente está visível?")).toMatchObject({ tools: ["get_component_details"], term: undefined });
    expect(routeIntent("Azriel, o que é essa peça?")).toMatchObject({ tools: ["get_component_details"] });
    expect(routeIntent("Quantos filhos ela possui?")).toMatchObject({ tools: ["get_component_details"], term: undefined });
  });

  it("extrai o nome real do componente sem delegar IDs ao modelo", () => {
    expect(routeIntent("Destaque o rotor.")).toMatchObject({ tools: ["select_component"], term: "rotor" });
    expect(routeIntent("Detalhes do rotor.")).toMatchObject({ tools: ["get_component_details"], term: "rotor" });
  });

  it("não captura consultas de outros módulos como ações do Engineering", () => {
    expect(routeIntent("Mostre meu Mapa Stark.").tools).toContain("get_stark_map");
    expect(routeIntent("Mostre a pasta do ArcCore.").tools).toEqual(["reveal_workspace"]);
  });

  it("combina domínios relacionados a bioinformática", () => {
    const route = routeIntent("O que estou fazendo relacionado a bioinformática?");
    expect(route.intent).toBe("cross_domain_activity");
    expect(route.tools).toEqual(expect.arrayContaining(["list_projects", "list_knowledge_areas", "get_today_tasks"]));
    expect(route.term).toBe("bioinformática");
  });

  it("executa rotina apenas com intenção explícita", () => {
    expect(routeIntent("Azriel, execute a rotina Ambiente de Desenvolvimento.").tools).toEqual(["run_routine"]);
    expect(routeIntent("Talvez eu programe um pouco hoje.").tools).not.toContain("run_routine");
  });

  it.each([
    "Explique o que é física quântica.",
    "Por que o céu é azul?",
    "Quanto é 5 * 542?",
    "Quem criou a teoria da gravidade?",
  ])("mantém a pergunta geral fora das tools internas: %s", (query) => {
    const route = routeIntent(query);
    expect(route.scope).toBe("general");
    expect(route.tools).toEqual([]);
  });

  it("só consulta o estado interno quando o Azriel é mencionado", () => {
    expect(routeIntent("Qual é o status do Azriel?")).toMatchObject({ scope: "azriel", intent: "azriel_status" });
  });

  it.each([
    ["Azriel, abra o Visual Studio Code.", "open_application"],
    ["Abra o workspace do Azriel.", "open_workspace"],
    ["Abra o GeneScope.", "open_project"],
    ["Mostre a pasta do ArcCore.", "reveal_workspace"],
    ["Abra o GitHub do Azriel.", "open_registered_url"],
  ])("aceita intenção explícita: %s", (query, tool) => {
    expect(routeIntent(query).tools).toEqual([tool]);
  });

  it.each([
    "Talvez eu trabalhe no GeneScope hoje.",
    "Eu poderia abrir o Visual Studio Code depois.",
    "O GitHub do Azriel é importante.",
  ])("não executa intenção vaga: %s", (query) => {
    expect(routeIntent(query).tools.some((tool) => tool.startsWith("open_") || tool === "reveal_workspace")).toBe(false);
  });
});
