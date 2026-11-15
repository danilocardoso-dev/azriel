import { useCallback, useEffect, useMemo, useState, type FormEvent } from "react";
import { MEMORY_CATEGORIES } from "../../ai/tools/toolRegistry";
import { MEMORY_CHANGED, memoryRepository } from "../../repositories/memoryRepository";
import { formatTimestamp } from "../../services/dateService";
import type { Memory, MemoryCategory } from "../../types";
import { DeleteConfirmationDialog } from "../daily/DeleteConfirmationDialog";
import "../../styles/mind.css";
import { CATEGORY_NAMES, filterMemories } from "./memoryFilters";

const messageOf = (reason: unknown) => reason instanceof Error ? reason.message : String(reason);
const newId = () => globalThis.crypto?.randomUUID?.() ?? `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;

/** O que o Azriel lembra: o Senhor vê, acrescenta, corrige e apaga. */
export function MemoryPanel() {
  const [memories, setMemories] = useState<Memory[] | null>(null);
  const [category, setCategory] = useState<MemoryCategory | "all">("all");
  const [search, setSearch] = useState("");
  const [draft, setDraft] = useState({ content: "", category: "operador" as MemoryCategory });
  const [editing, setEditing] = useState<{ id: string; content: string; category: MemoryCategory } | null>(null);
  const [toDelete, setToDelete] = useState<Memory | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(() => { void memoryRepository.list().then(setMemories).catch((reason) => setError(messageOf(reason))); }, []);
  useEffect(() => {
    const first = window.setTimeout(load, 0);
    window.addEventListener(MEMORY_CHANGED, load);
    return () => { window.clearTimeout(first); window.removeEventListener(MEMORY_CHANGED, load); };
  }, [load]);

  const visible = useMemo(() => filterMemories(memories ?? [], category, search), [memories, category, search]);

  async function run(action: () => Promise<unknown>) {
    setBusy(true); setError(null);
    try { await action(); return true; } catch (reason) { setError(messageOf(reason)); return false; } finally { setBusy(false); }
  }

  async function add(event: FormEvent) {
    event.preventDefault();
    if (!draft.content.trim()) return;
    if (await run(() => memoryRepository.save({ id: newId(), content: draft.content, category: draft.category, source: "operador" }))) setDraft((current) => ({ ...current, content: "" }));
  }

  async function saveEdit(memory: Memory) {
    if (!editing) return;
    if (await run(() => memoryRepository.save({ id: memory.id, content: editing.content, category: editing.category, source: memory.source, conversationId: memory.conversationId }))) setEditing(null);
  }

  return <section className="ai-settings mind-panel">
    <header><span>MEMÓRIA DE LONGO PRAZO // O QUE O AZRIEL LEMBRA</span><i data-state="online">{memories ? `${memories.length} MEMÓRIA${memories.length === 1 ? "" : "S"}` : "CARREGANDO"}</i></header>
    <div className="mind-panel__body">
      <p className="mind-panel__hint">O Azriel registra e corrige memórias sozinho durante as conversas. Só o Senhor pode apagá-las.</p>
      <form className="mind-add" onSubmit={add}>
        <input value={draft.content} maxLength={1000} placeholder="Ensinar algo ao Azriel (ex.: O Senhor prefere reuniões pela manhã)" onChange={(event) => setDraft((current) => ({ ...current, content: event.target.value }))} aria-label="Nova memória" />
        <select value={draft.category} onChange={(event) => setDraft((current) => ({ ...current, category: event.target.value as MemoryCategory }))} aria-label="Categoria da nova memória">
          {MEMORY_CATEGORIES.map((item) => <option key={item} value={item}>{CATEGORY_NAMES[item]}</option>)}
        </select>
        <button disabled={busy || !draft.content.trim()}>LEMBRAR</button>
      </form>
      <div className="mind-filters">
        <input value={search} placeholder="Filtrar memórias" onChange={(event) => setSearch(event.target.value)} aria-label="Filtrar memórias" />
        <div role="group" aria-label="Categoria">
          {(["all", ...MEMORY_CATEGORIES] as const).map((item) => <button type="button" key={item} data-active={category === item} onClick={() => setCategory(item)}>{item === "all" ? "Todas" : CATEGORY_NAMES[item]}</button>)}
        </div>
      </div>
      {error && <p className="mind-panel__status" role="alert" data-kind="error">{error}</p>}
      <ul className="mind-list">
        {memories && !visible.length && <li className="mind-list__empty">{memories.length ? "Nenhuma memória com esse filtro." : "O Azriel ainda não tem memórias. Converse com ele ou ensine algo acima."}</li>}
        {visible.map((memory) => {
          const open = editing?.id === memory.id;
          return <li key={memory.id} className="mind-memory" data-category={memory.category}>
            <header>
              <b>{CATEGORY_NAMES[memory.category] ?? memory.category}</b>
              <small>{memory.source === "azriel" ? "aprendida pelo Azriel" : "ensinada pelo Senhor"} · {formatTimestamp(memory.updatedAt)}</small>
            </header>
            {open ? <div className="mind-memory__edit">
              <textarea value={editing.content} maxLength={1000} onChange={(event) => setEditing({ ...editing, content: event.target.value })} aria-label="Texto da memória" autoFocus />
              <select value={editing.category} onChange={(event) => setEditing({ ...editing, category: event.target.value as MemoryCategory })} aria-label="Categoria">
                {MEMORY_CATEGORIES.map((item) => <option key={item} value={item}>{CATEGORY_NAMES[item]}</option>)}
              </select>
              <div><button type="button" disabled={busy || !editing.content.trim()} onClick={() => void saveEdit(memory)}>SALVAR</button><button type="button" className="mind-button--ghost" onClick={() => setEditing(null)}>CANCELAR</button></div>
            </div> : <p>{memory.content}</p>}
            {!open && <div className="mind-memory__actions">
              <button type="button" className="mind-button--ghost" onClick={() => setEditing({ id: memory.id, content: memory.content, category: memory.category })}>CORRIGIR</button>
              <button type="button" className="mind-button--danger" onClick={() => setToDelete(memory)}>APAGAR</button>
            </div>}
          </li>;
        })}
      </ul>
    </div>
    {toDelete && <DeleteConfirmationDialog kind="memória" title={toDelete.content.length > 80 ? `${toDelete.content.slice(0, 80)}…` : toDelete.content} description="O Azriel vai esquecer isto definitivamente. Esta ação não poderá ser desfeita." busy={busy} onCancel={() => setToDelete(null)} onConfirm={() => void run(() => memoryRepository.remove(toDelete.id)).then((ok) => { if (ok) setToDelete(null); })} />}
  </section>;
}
