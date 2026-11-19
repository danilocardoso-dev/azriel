import { describe, expect, it } from "vitest";
import type { OpenRouterModel } from "../../types";
import { filterCatalog, formatContext, formatPrice, toggleFavorite } from "./openRouterCatalog";

const model = (id: string, name: string, prompt = 0.27, completion = 1.1, free = false, contextLength = 128_000): OpenRouterModel => ({ id, name, promptPerMillion: prompt, completionPerMillion: completion, free, contextLength });
const catalog = [
  model("openai/gpt-x", "GPT X"),
  model("deepseek/deepseek-chat", "DeepSeek V3"),
  model("moonshotai/kimi-k3", "Kimi K3"),
  model("moonshotai/kimi-k2:free", "Kimi K2 (free)", 0, 0, true),
];

describe("catálogo do OpenRouter", () => {
  it("mostra preço por milhão ou GRÁTIS, e o contexto legível", () => {
    expect(formatPrice(catalog[1])).toBe("US$ 0,27 / 1,10");
    expect(formatPrice(catalog[3])).toBe("GRÁTIS");
    expect(formatContext(128_000)).toBe("128 mil");
    expect(formatContext(1_048_576)).toBe("1 mi");
    expect(formatContext(0)).toBe("—");
  });

  it("busca por todas as palavras em nome e id, com favoritos primeiro", () => {
    expect(filterCatalog(catalog, "kimi", false, []).map((m) => m.id)).toEqual(["moonshotai/kimi-k2:free", "moonshotai/kimi-k3"]);
    expect(filterCatalog(catalog, "kimi k3", false, []).map((m) => m.id)).toEqual(["moonshotai/kimi-k3"]);
    expect(filterCatalog(catalog, "", true, []).map((m) => m.id)).toEqual(["moonshotai/kimi-k2:free"]);
    expect(filterCatalog(catalog, "", false, ["openai/gpt-x"])[0].id).toBe("openai/gpt-x");
    expect(filterCatalog(catalog, "", false, [], 2)).toHaveLength(2);
  });

  it("favoritar e desfavoritar sem duplicar", () => {
    expect(toggleFavorite(["a"], "b")).toEqual(["a", "b"]);
    expect(toggleFavorite(["a", "b"], "a")).toEqual(["b"]);
  });
});
