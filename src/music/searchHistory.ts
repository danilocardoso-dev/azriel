const KEY = "azriel.musicSearches";

/** Coloca a busca no topo, sem repetir (ignora maiúsculas), mantendo só as mais recentes. */
export function rememberSearch(list: string[], text: string, limit = 6): string[] {
  const clean = text.trim();
  if (!clean) return list;
  return [clean, ...list.filter((item) => item.toLowerCase() !== clean.toLowerCase())].slice(0, limit);
}

export function loadSearches(): string[] {
  try {
    const value: unknown = JSON.parse(localStorage.getItem(KEY) ?? "[]");
    return Array.isArray(value) ? value.filter((item): item is string => typeof item === "string").slice(0, 6) : [];
  } catch { return []; }
}

export function saveSearches(list: string[]) {
  try { localStorage.setItem(KEY, JSON.stringify(list)); } catch { /* preferência local, opcional */ }
}
