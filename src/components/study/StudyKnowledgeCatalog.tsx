import { useMemo, useState } from "react";
import type { KnowledgeArea, KnowledgeInput, StudyRoadmap } from "../../types";
import { DeleteConfirmationDialog } from "../daily/DeleteConfirmationDialog";
import { KnowledgeEditor } from "./KnowledgeEditor";
import { knowledgeReferenceCount } from "./knowledgeCatalog";

interface StudyKnowledgeCatalogProps {
  areas: KnowledgeArea[];
  roadmaps: StudyRoadmap[];
  onSave: (input: KnowledgeInput) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
  onError: (message: string | null) => void;
}

function catalogJson(areas: KnowledgeArea[]) {
  return JSON.stringify(areas.map((area) => ({
    id: area.id,
    name: area.name,
    category: area.category,
    description: area.description,
    priority: area.priority,
    nodeType: area.nodeType,
    parentId: area.parentId,
  })), null, 2);
}

export function StudyKnowledgeCatalog({ areas, roadmaps, onSave, onDelete, onError }: StudyKnowledgeCatalogProps) {
  const [query, setQuery] = useState("");
  const [editing, setEditing] = useState<KnowledgeArea | "new" | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<KnowledgeArea | null>(null);
  const [busy, setBusy] = useState(false);
  const [copyState, setCopyState] = useState("COPIAR CATÁLOGO JSON");
  const filtered = useMemo(() => {
    const normalized = query.trim().toLocaleLowerCase("pt-BR");
    if (!normalized) return areas;
    return areas.filter((area) => [area.id, area.name, area.category, area.description].some((value) => value.toLocaleLowerCase("pt-BR").includes(normalized)));
  }, [areas, query]);

  async function copyCatalog() {
    try {
      await navigator.clipboard.writeText(catalogJson(areas));
      setCopyState("CATÁLOGO COPIADO");
      window.setTimeout(() => setCopyState("COPIAR CATÁLOGO JSON"), 1800);
    } catch (reason) {
      onError(reason instanceof Error ? reason.message : "Não foi possível copiar o catálogo.");
    }
  }

  function exportCatalog() {
    const url = URL.createObjectURL(new Blob([catalogJson(areas)], { type: "application/json;charset=utf-8" }));
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = "azriel-knowledge-catalog.json";
    anchor.click();
    URL.revokeObjectURL(url);
  }

  async function remove() {
    if (!deleteTarget) return;
    setBusy(true);
    onError(null);
    try {
      await onDelete(deleteTarget.id);
      setDeleteTarget(null);
    } catch (reason) {
      onError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  return <section className="study-knowledge">
    <header className="study-knowledge__header"><div><span>KNOWLEDGE REGISTRY</span><h2>Catálogo de conhecimentos</h2><p>IDs autorizados para roadmaps, atividades, projetos e contexto local do Azriel.</p></div><div><button onClick={() => void copyCatalog()}>{copyState}</button><button onClick={exportCatalog}>EXPORTAR JSON</button><button className="primary" onClick={() => setEditing("new")}>＋ NOVO CONHECIMENTO</button></div></header>
    <div className="study-knowledge__summary"><article><small>REGISTROS</small><strong>{areas.length}</strong></article><article><small>CATEGORIAS</small><strong>{new Set(areas.map((area) => area.category)).size}</strong></article><article><small>VINCULADOS AO AZRIEL</small><strong>{areas.filter((area) => area.projectIds.includes("azriel")).length}</strong></article><label><span>BUSCAR NO CATÁLOGO</span><input value={query} placeholder="ID, nome, categoria..." onChange={(event) => setQuery(event.target.value)} /></label></div>
    <div className="study-knowledge__table-wrap"><table className="study-knowledge__table"><thead><tr><th>ID</th><th>Conhecimento</th><th>Categoria</th><th>Tipo</th><th>Vínculos</th><th /></tr></thead><tbody>{filtered.map((area) => { const references = knowledgeReferenceCount(area.id, roadmaps); const childCount = areas.filter((item) => item.parentId === area.id).length; return <tr key={area.id}><td><code>{area.id}</code></td><td><strong>{area.name}</strong><small>{area.description || "Sem descrição."}</small></td><td>{area.category}</td><td><span data-priority={area.priority}>{area.nodeType.toUpperCase()}</span></td><td><small>{area.projectIds.length} projeto(s) · {references.topics} tópico(s) · {references.activities} atividade(s) · {childCount} filho(s)</small></td><td><button onClick={() => setEditing(area)}>EDITAR</button><button className="danger-link" onClick={() => setDeleteTarget(area)}>EXCLUIR</button></td></tr>; })}</tbody></table>{!filtered.length && <div className="core-empty">Nenhum conhecimento corresponde à busca.</div>}</div>
    {editing && <KnowledgeEditor area={editing === "new" ? null : editing} areas={areas} onCancel={() => setEditing(null)} onSave={async (input) => { await onSave(input); setEditing(null); }} />}
    {deleteTarget && (() => { const references = knowledgeReferenceCount(deleteTarget.id, roadmaps); return <DeleteConfirmationDialog kind="conhecimento" title={`${deleteTarget.name} (${deleteTarget.id})`} description={`Esta ação remove o conhecimento e suas métricas. Vínculos atuais: ${deleteTarget.projectIds.length} projeto(s), ${references.topics} tópico(s) e ${references.activities} atividade(s). Atividades ou eventos protegidos impedirão a exclusão.`} busy={busy} onCancel={() => setDeleteTarget(null)} onConfirm={() => void remove()} />; })()}
  </section>;
}
