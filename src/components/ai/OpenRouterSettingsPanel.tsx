import { useCallback, useEffect, useMemo, useState, type FormEvent } from "react";
import { useAI } from "../../contexts/useAI";
import { aiRepository } from "../../repositories/aiRepository";
import type { OpenRouterModel } from "../../types";
import { filterCatalog, formatContext, formatPrice, toggleFavorite } from "./openRouterCatalog";
import "../../styles/mind.css";

const messageOf = (reason: unknown) => reason instanceof Error ? reason.message : String(reason);

/** OpenRouter: uma chave, vários modelos (DeepSeek, Kimi, Claude...). Os favoritos vão para o seletor do AI Core. */
export function OpenRouterSettingsPanel() {
  const { openRouter, saveOpenRouter } = useAI();
  const [apiKey, setApiKey] = useState("");
  const [catalog, setCatalog] = useState<OpenRouterModel[] | null>(null);
  const [search, setSearch] = useState("");
  const [onlyFree, setOnlyFree] = useState(false);
  const [busy, setBusy] = useState<string | null>(null);
  const [message, setMessage] = useState<{ kind: "success" | "error"; text: string } | null>(null);
  const configured = Boolean(openRouter?.apiKeyConfigured);
  const favorites = useMemo(() => openRouter?.favorites ?? [], [openRouter]);

  const loadCatalog = useCallback(async () => {
    setBusy("catalog");
    try { setCatalog(await aiRepository.listOpenRouterModels()); }
    catch (reason) { setMessage({ kind: "error", text: messageOf(reason) }); }
    finally { setBusy(null); }
  }, []);
  useEffect(() => { if (configured && !catalog) { const first = window.setTimeout(() => void loadCatalog(), 0); return () => window.clearTimeout(first); } }, [configured, catalog, loadCatalog]);

  async function save(kind: string, change: { model?: string; favorites?: string[]; apiKey?: string | null; enabled?: boolean }, done: string) {
    if (!openRouter) return;
    setBusy(kind); setMessage(null);
    try {
      await saveOpenRouter({ enabled: change.enabled ?? openRouter.enabled, model: change.model ?? openRouter.model, favorites: change.favorites ?? favorites, apiKey: change.apiKey ?? null });
      setMessage({ kind: "success", text: done });
    } catch (reason) { setMessage({ kind: "error", text: messageOf(reason) }); }
    finally { setBusy(null); }
  }

  function submitKey(event: FormEvent) {
    event.preventDefault();
    const key = apiKey.trim();
    if (!key) return;
    void save("key", { apiKey: key }, "Chave salva. Escolha os modelos favoritos abaixo e ligue o OpenRouter no topo do AI Core.").then(() => setApiKey(""));
  }

  const visible = useMemo(() => filterCatalog(catalog ?? [], search, onlyFree, favorites), [catalog, search, onlyFree, favorites]);
  const byId = useMemo(() => new Map((catalog ?? []).map((model) => [model.id, model])), [catalog]);

  function use(id: string) {
    void save(`use:${id}`, { model: id, favorites: favorites.includes(id) ? favorites : [...favorites, id] }, `Modelo em uso: ${id}.`);
  }

  return <section className="ai-settings mind-panel">
    <header><span>MOTOR REMOTO // OPENROUTER</span><i data-state={configured ? "online" : "offline"}>{!openRouter ? "CARREGANDO" : openRouter.enabled ? "LIGADO" : configured ? "CHAVE CONFIGURADA" : "SEM CHAVE"}</i></header>
    <div className="mind-panel__body">
      <p className="mind-panel__hint">Uma chave do openrouter.ai dá acesso a DeepSeek, Kimi, Claude, GPT e outros, pagos por uso (créditos pré-pagos). Só aparecem modelos que aceitam ferramentas. Com o OpenRouter ligado, suas perguntas e os dados consultados vão para o OpenRouter e para a empresa do modelo escolhido.</p>

      <form className="mind-add mind-add--key" onSubmit={submitKey}>
        <input type="password" autoComplete="off" spellCheck={false} value={apiKey} placeholder={configured ? "•••••••• (substituir chave)" : "Cole a chave do OpenRouter (sk-or-...)"} onChange={(event) => setApiKey(event.target.value)} aria-label="Chave do OpenRouter" />
        <button disabled={busy !== null || !apiKey.trim()}>{busy === "key" ? "SALVANDO..." : "SALVAR CHAVE"}</button>
        {configured && <button type="button" className="mind-button--danger" disabled={busy !== null} onClick={() => void save("remove", { apiKey: "", enabled: false }, "Chave removida.")}>REMOVER CHAVE</button>}
      </form>

      {message && <p className="mind-panel__status" role="status" data-kind={message.kind}>{message.text}</p>}

      <div className="router-favorites">
        <strong>FAVORITOS ({favorites.length}) · aparecem no seletor do AI Core</strong>
        {!favorites.length && <small>Nenhum ainda. Marque ★ nos modelos do catálogo.</small>}
        <ul>
          {favorites.map((id) => <li key={id} data-active={openRouter?.model === id}>
            <button type="button" className="router-favorites__use" disabled={busy !== null || openRouter?.model === id} onClick={() => use(id)} title="Usar este modelo">{openRouter?.model === id ? "● " : ""}{id}</button>
            {byId.get(id) && <small>{formatPrice(byId.get(id)!)}</small>}
            <button type="button" className="router-favorites__remove" disabled={busy !== null} onClick={() => void save("fav", { favorites: toggleFavorite(favorites, id) }, `${id} saiu dos favoritos.`)} aria-label={`Tirar ${id} dos favoritos`}>×</button>
          </li>)}
        </ul>
      </div>

      <div className="mind-filters">
        <input value={search} placeholder="Buscar no catálogo (ex.: deepseek, kimi, claude)" onChange={(event) => setSearch(event.target.value)} aria-label="Buscar modelo" />
        <div role="group" aria-label="Filtro de preço">
          <button type="button" data-active={!onlyFree} onClick={() => setOnlyFree(false)}>Todos</button>
          <button type="button" data-active={onlyFree} onClick={() => setOnlyFree(true)}>Só grátis</button>
          <button type="button" disabled={busy !== null} onClick={() => void loadCatalog()}>{busy === "catalog" ? "Carregando..." : catalog ? "Atualizar" : "Carregar catálogo"}</button>
        </div>
      </div>

      <ul className="mind-list router-catalog">
        {catalog && !visible.length && <li className="mind-list__empty">Nenhum modelo com esse filtro.</li>}
        {!catalog && busy !== "catalog" && <li className="mind-list__empty">Carregue o catálogo para escolher os modelos.</li>}
        {visible.map((model) => {
          const favorite = favorites.includes(model.id);
          return <li key={model.id} className="router-model" data-favorite={favorite}>
            <button type="button" className="router-model__star" disabled={busy !== null} onClick={() => void save("fav", { favorites: toggleFavorite(favorites, model.id) }, favorite ? `${model.id} saiu dos favoritos.` : `${model.id} entrou nos favoritos.`)} aria-label={favorite ? "Tirar dos favoritos" : "Favoritar"} title={favorite ? "Tirar dos favoritos" : "Favoritar"}>{favorite ? "★" : "☆"}</button>
            <div><b>{model.name}</b><code>{model.id}</code></div>
            <small title="Entrada / saída, por milhão de tokens">{formatPrice(model)}</small>
            <small title="Janela de contexto">{formatContext(model.contextLength)}</small>
            <button type="button" className="mind-button--ghost" disabled={busy !== null || !configured || openRouter?.model === model.id} onClick={() => use(model.id)}>{openRouter?.model === model.id ? "EM USO" : "USAR"}</button>
          </li>;
        })}
      </ul>
      {catalog && <small className="router-catalog__count">{catalog.length} modelos com ferramentas no catálogo{visible.length >= 60 ? " · refine a busca para ver outros" : ""}</small>}
    </div>
  </section>;
}
