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
  ])("roteia %s para %s", (query, tool) => expect(routeIntent(query).tools).toContain(tool));

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
