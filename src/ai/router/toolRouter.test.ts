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
  ])("roteia %s para %s", (query, tool) => expect(routeIntent(query).tools).toContain(tool));

  it("combina domínios relacionados a bioinformática", () => {
    const route = routeIntent("O que estou fazendo relacionado a bioinformática?");
    expect(route.intent).toBe("cross_domain_activity");
    expect(route.tools).toEqual(expect.arrayContaining(["list_projects", "list_knowledge_areas", "get_today_tasks"]));
    expect(route.term).toBe("bioinformática");
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
});
