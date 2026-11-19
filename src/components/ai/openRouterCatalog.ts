import type { OpenRouterModel } from "../../types";

/** "US$ 0,27 / 1,10" (entrada / saída por milhão de tokens) ou "GRÁTIS". */
export function formatPrice(model: OpenRouterModel): string {
  if (model.free) return "GRÁTIS";
  const money = (value: number) => value.toLocaleString("pt-BR", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  return `US$ ${money(model.promptPerMillion)} / ${money(model.completionPerMillion)}`;
}

/** "128 mil" / "1 mi" tokens de contexto. */
export function formatContext(tokens: number): string {
  if (!tokens) return "—";
  if (tokens >= 1_000_000) return `${(tokens / 1_000_000).toLocaleString("pt-BR", { maximumFractionDigits: 1 })} mi`;
  return `${Math.round(tokens / 1_000)} mil`;
}

/**
 * Busca no catálogo por nome ou id (todas as palavras precisam aparecer), favoritos primeiro.
 * O catálogo tem centenas de modelos: a tela mostra só os primeiros `limit`.
 */
export function filterCatalog(models: OpenRouterModel[], text: string, onlyFree: boolean, favorites: string[], limit = 60): OpenRouterModel[] {
  const words = text.toLowerCase().split(/\s+/).filter(Boolean);
  return models
    .filter((model) => (!onlyFree || model.free) && words.every((word) => `${model.id} ${model.name}`.toLowerCase().includes(word)))
    .sort((a, b) => Number(favorites.includes(b.id)) - Number(favorites.includes(a.id)) || a.id.localeCompare(b.id))
    .slice(0, limit);
}

/** Liga/desliga um favorito sem duplicar. */
export function toggleFavorite(favorites: string[], id: string): string[] {
  return favorites.includes(id) ? favorites.filter((item) => item !== id) : [...favorites, id];
}
