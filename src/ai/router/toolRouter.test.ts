import { describe, expect, it } from "vitest";
import { routeIntent } from "./toolRouter";

describe("Tool Router", () => {
  it.each([
    ["O que tenho para hoje?", "get_today_tasks"],
    ["Tenho alguma atividade atrasada?", "get_overdue_tasks"],
    ["Quais são minhas demandas críticas?", "get_critical_tasks"],
    ["Liste minhas tarefas CRITICAS", "get_critical_tasks"],
    ["Quais são minhas demandas prioritárias?", "get_priority_tasks"],
    ["Quais são minhas prioridades?", "get_priority_tasks"],
    ["Tenho pendências de prioridade alta?", "get_priority_tasks"],
    ["Quais são meus projetos?", "list_projects"],
    ["Azriel, situação.", "get_daily_operations_summary"],
    ["Quais processos estão consumindo mais memória?", "get_process_summary"],
    ["Como está o uso da CPU?", "get_cpu_status"],
    ["Quais workspaces estão cadastrados?", "list_workspaces"],
    ["Quais projetos Git possuem alterações?", "get_git_status"],
    ["O Ollama está disponível?", "get_ollama_status"],
    ["Quais rotinas eu tenho?", "list_routines"],
    ["Quais roadmaps estão ativos?", "list_study_roadmaps"],
    ["Como estão meus Estudos?", "list_study_roadmaps"],
    ["Como está meu antigo Mapa Stark?", "list_study_roadmaps"],
    ["Como está meu roadmap de Controle e Automação?", "get_study_roadmap"],
    ["Qual é minha próxima atividade?", "get_current_study_position"],
    ["Onde parei nos estudos?", "get_current_study_position"],
  ])("roteia %s para %s", (query, tool) => expect(routeIntent(query).tools).toContain(tool));

  it("não confunde criticidade geral ou lacunas críticas com demandas críticas", () => {
    expect(routeIntent("O que é criticidade?").tools).toEqual([]);
    expect(routeIntent("Quais são minhas lacunas críticas?").tools).toContain("get_knowledge_gaps");
    expect(routeIntent("Quais são minhas lacunas críticas?").tools).not.toContain("get_critical_tasks");
  });

  it("mantém o resumo geral alinhado aos módulos ativos", () => {
    const route = routeIntent("Azriel, situação.");
    expect(route.tools).toContain("list_study_roadmaps");
    expect(route.tools).not.toContain("get_current_education");
  });

  it.each([
    "Como está minha formação?",
    "Exploda a montagem.",
    "Selecione o rotor.",
    "Qual modelo está carregado?",
  ])("não roteia recursos removidos ou congelados: %s", (query) => {
    const route = routeIntent(query);
    expect(route.tools.some((tool) => tool.includes("education") || tool.includes("component") || tool.includes("model") || tool.includes("explosion") || tool === "explode_all")).toBe(false);
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
    ["Finaliza a 1 para mim.", 1],
    ["Conclua a tarefa 2.", 2],
    ["Marque a terceira como concluída.", 3],
  ])("resolve conclusão explícita por referência: %s", (query, position) => {
    expect(routeIntent(query)).toMatchObject({ intent: "complete_task_reference", scope: "azriel", tools: ["complete_task"], referencePosition: position });
  });

  it.each([
    "Talvez eu finalize a 1 depois.",
    "A primeira está quase pronta.",
  ])("não conclui tarefa com linguagem vaga: %s", (query) => {
    expect(routeIntent(query).tools).not.toContain("complete_task");
  });

  it("mantém comando interno sem referência fora do conhecimento geral", () => {
    expect(routeIntent("Finalize essa demanda.")).toMatchObject({ intent: "unsupported_internal", scope: "azriel", tools: [] });
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

  it.each([
    ["Azriel, abra o Visual Studio Code.", "open_application"],
    ["Abra o workspace do Azriel.", "open_workspace"],
    ["Abra o GeneScope.", "open_project"],
    ["Mostre a pasta do ArcCore.", "reveal_workspace"],
    ["Abra o GitHub do Azriel.", "open_registered_url"],
  ])("aceita intenção explícita: %s", (query, tool) => expect(routeIntent(query).tools).toEqual([tool]));

  it.each([
    "Talvez eu trabalhe no GeneScope hoje.",
    "Eu poderia abrir o Visual Studio Code depois.",
    "O GitHub do Azriel é importante.",
  ])("não executa intenção vaga: %s", (query) => {
    expect(routeIntent(query).tools.some((tool) => tool.startsWith("open_") || tool === "reveal_workspace")).toBe(false);
  });
});
