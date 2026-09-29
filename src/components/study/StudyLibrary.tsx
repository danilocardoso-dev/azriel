import { useCallback, useEffect, useMemo, useState } from "react";
import type { StudyAICardDraft } from "../../ai/study/studyAIDrafts";
import { studyLabService } from "../../services/studyLabService";
import type { AddLinkStudyMaterialInput, ImportStudyMaterialInput, StudyCardInput, StudyMaterial, StudyMaterialRelationInput, StudyMaterialRelationType, StudyMaterialStorageKind, StudyMaterialSummary, StudyNotebook, StudyNoteInput, StudyNoteSummary, StudyRoadmap } from "../../types";
import { DeleteConfirmationDialog } from "../daily/DeleteConfirmationDialog";
import { StudyAIToolPanel } from "./StudyAIToolPanel";
import { StudyMarkdown } from "./StudyMarkdown";

export type StudyLibraryLaunch = { token: number; materialId?: string; relation?: StudyMaterialRelationInput };

type Props = { roadmaps: StudyRoadmap[]; launch: StudyLibraryLaunch | null; onChanged: () => Promise<void> };
type ImportMode = "file" | "link";
type ConfirmAction = "remove" | "delete-file" | null;

const emptyFile = (): Omit<ImportStudyMaterialInput, "id"> => ({ title: "", sourcePath: "", storageKind: "MANAGED_COPY", sourceAuthor: null, sourceTitle: null, sourceYear: null, description: null, relations: [] });
const emptyLink = (): Omit<AddLinkStudyMaterialInput, "id"> => ({ title: "", externalUrl: "", description: null, sourceAuthor: null, sourceTitle: null, sourceYear: null, relations: [] });
const filenameFromPath = (path: string) => path.split(/[\\/]/).pop()?.replace(/\.[^.]+$/, "") || "Material de estudo";
const formatBytes = (value: number | null) => value === null ? "—" : value >= 1024 * 1024 ? `${(value / 1024 / 1024).toFixed(1)} MB` : `${Math.max(1, Math.round(value / 1024))} KB`;

export function StudyLibrary({ roadmaps, launch, onChanged }: Props) {
  const [materials, setMaterials] = useState<StudyMaterialSummary[]>([]);
  const [selected, setSelected] = useState<StudyMaterial | null>(null);
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<"ALL" | "PDF" | "IMAGE" | "TEXT" | "LINK">("ALL");
  const [importMode, setImportMode] = useState<ImportMode | null>(null);
  const [fileDraft, setFileDraft] = useState(emptyFile);
  const [linkDraft, setLinkDraft] = useState(emptyLink);
  const [notebooks, setNotebooks] = useState<StudyNotebook[]>([]);
  const [notes, setNotes] = useState<StudyNoteSummary[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [confirmAction, setConfirmAction] = useState<ConfirmAction>(null);

  const load = useCallback(async (preferredId?: string) => {
    setError(null);
    try {
      const [nextMaterials, nextNotebooks, nextNotes] = await Promise.all([
        studyLabService.materials({ query: query || null, materialType: filter === "ALL" ? null : filter, limit: 100, offset: 0 }),
        studyLabService.notebooks(false),
        studyLabService.notes({ limit: 100, offset: 0 }),
      ]);
      setMaterials(nextMaterials);
      setNotebooks(nextNotebooks);
      setNotes(nextNotes);
      if (preferredId) setSelected(await studyLabService.material(preferredId));
    } catch (reason) { setError(reason instanceof Error ? reason.message : String(reason)); }
  }, [filter, query]);

  useEffect(() => { const timer = window.setTimeout(() => void load(launch?.materialId), 0); return () => window.clearTimeout(timer); }, [launch?.materialId, launch?.token, load]);

  async function chooseFile() {
    const path = await studyLabService.selectMaterialFile();
    if (path) setFileDraft((current) => ({ ...current, sourcePath: path, title: current.title || filenameFromPath(path), relations: launch?.relation ? [launch.relation] : current.relations }));
  }

  async function importMaterial() {
    setBusy(true); setError(null); setNotice(null);
    try {
      let openedId: string | undefined;
      if (importMode === "file") {
        if (!fileDraft.sourcePath) throw new Error("Selecione um arquivo.");
        const result = await studyLabService.importMaterial({ id: crypto.randomUUID(), ...fileDraft });
        setNotice(result.duplicate ? "Este conteúdo já existia. O material existente foi aberto e a nova relação foi preservada." : "Material importado com sucesso.");
        setSelected(result.material);
        openedId = result.material.id;
        setFileDraft(emptyFile());
      } else if (importMode === "link") {
        const material = await studyLabService.addLinkMaterial({ id: crypto.randomUUID(), ...linkDraft, relations: launch?.relation ? [launch.relation] : linkDraft.relations });
        setSelected(material); openedId = material.id; setLinkDraft(emptyLink()); setNotice("Link adicionado à biblioteca.");
      }
      setImportMode(null);
      await load(openedId);
      await onChanged();
    } catch (reason) { setError(reason instanceof Error ? reason.message : String(reason)); }
    finally { setBusy(false); }
  }

  async function refreshSelected() {
    if (!selected) return;
    await load(selected.id);
    await onChanged();
  }

  async function confirmRemoval() {
    if (!selected || !confirmAction) return;
    setBusy(true); setError(null);
    try {
      if (confirmAction === "delete-file") setSelected(await studyLabService.deleteManagedMaterialFile(selected.id));
      else { await studyLabService.removeMaterial(selected.id); setSelected(null); }
      setConfirmAction(null); await load(); await onChanged();
    } catch (reason) { setError(reason instanceof Error ? reason.message : String(reason)); }
    finally { setBusy(false); }
  }

  return <section className="study-library">
    <aside className="study-library__catalog">
      <header><div><span>STUDY LIBRARY</span><strong>BIBLIOTECA LOCAL</strong></div><button onClick={() => { setImportMode("file"); setFileDraft((current) => ({ ...current, relations: launch?.relation ? [launch.relation] : [] })); }}>＋ ADICIONAR</button></header>
      <input aria-label="Buscar materiais" value={query} onChange={(event) => setQuery(event.target.value)} onKeyDown={(event) => { if (event.key === "Enter") void load(); }} placeholder="Buscar título, autor ou texto extraído..." />
      <nav>{(["ALL", "PDF", "IMAGE", "TEXT", "LINK"] as const).map((item) => <button key={item} className={filter === item ? "active" : ""} onClick={() => setFilter(item)}>{item === "ALL" ? "TODOS" : item === "IMAGE" ? "IMAGEM" : item === "TEXT" ? "TEXTO" : item}</button>)}</nav>
      <div className="study-library__list">{materials.map((material) => <button key={material.id} className={selected?.id === material.id ? "active" : ""} onClick={() => void studyLabService.material(material.id).then(setSelected).catch((reason: unknown) => setError(reason instanceof Error ? reason.message : String(reason)))}><b>{material.materialType}</b><span><strong>{material.title}</strong><small>{material.sourceAuthor || material.originalFilename || material.storageKind}</small><i>{material.relationLabels.join(" · ") || "SEM CONTEXTO"}</i></span><em data-status={material.status}>{material.extractionStale ? "STALE" : material.extractionStatus}</em></button>)}{!materials.length && <p>Nenhum material encontrado.</p>}</div>
    </aside>
    <main className="study-library__workspace">
      {error && <div className="form-error stark-error">{error}<button onClick={() => setError(null)}>FECHAR</button></div>}
      {notice && <div className="study-material-notice">{notice}<button onClick={() => setNotice(null)}>×</button></div>}
      {selected ? <MaterialDetail material={selected} roadmaps={roadmaps} notebooks={notebooks} notes={notes} busy={busy} onRefresh={refreshSelected} onError={setError} onRemove={() => setConfirmAction("remove")} onDeleteFile={() => setConfirmAction("delete-file")} /> : <div className="study-library__empty"><i>◇</i><strong>NENHUM MATERIAL SELECIONADO</strong><p>Adicione um PDF, imagem, texto, Markdown ou link para construir sua biblioteca local.</p><button onClick={() => setImportMode("file")}>＋ ADICIONAR MATERIAL</button><button onClick={() => setImportMode("link")}>＋ ADICIONAR LINK</button></div>}
    </main>
    {importMode && <ImportDialog mode={importMode} fileDraft={fileDraft} linkDraft={linkDraft} busy={busy} onFileDraft={setFileDraft} onLinkDraft={setLinkDraft} onChooseFile={chooseFile} onMode={setImportMode} onCancel={() => setImportMode(null)} onSave={importMaterial} />}
    {selected && confirmAction && <DeleteConfirmationDialog kind="material" title={selected.title} description={confirmAction === "delete-file" ? "A cópia física gerenciada será excluída. Metadata, texto e relações históricas serão preservados, mas o arquivo não poderá ser recuperado pelo Azriel." : "O material será removido da Biblioteca sem excluir automaticamente o arquivo. Metadata e relações históricas serão preservadas."} busy={busy} onCancel={() => setConfirmAction(null)} onConfirm={() => void confirmRemoval()} />}
  </section>;
}

type ImportProps = { mode: ImportMode; fileDraft: Omit<ImportStudyMaterialInput, "id">; linkDraft: Omit<AddLinkStudyMaterialInput, "id">; busy: boolean; onFileDraft: (value: Omit<ImportStudyMaterialInput, "id">) => void; onLinkDraft: (value: Omit<AddLinkStudyMaterialInput, "id">) => void; onChooseFile: () => Promise<void>; onMode: (value: ImportMode) => void; onCancel: () => void; onSave: () => Promise<void> };
function ImportDialog({ mode, fileDraft, linkDraft, busy, onFileDraft, onLinkDraft, onChooseFile, onMode, onCancel, onSave }: ImportProps) {
  const draft = mode === "file" ? fileDraft : linkDraft;
  const update = (patch: Record<string, unknown>) => mode === "file" ? onFileDraft({ ...fileDraft, ...patch }) : onLinkDraft({ ...linkDraft, ...patch });
  return <div className="study-material-modal"><section role="dialog" aria-modal="true" aria-label="Adicionar material"><header><div><span>STUDY LIBRARY</span><h2>Adicionar material</h2></div><button onClick={onCancel}>×</button></header><nav><button className={mode === "file" ? "active" : ""} onClick={() => onMode("file")}>ARQUIVO LOCAL</button><button className={mode === "link" ? "active" : ""} onClick={() => onMode("link")}>LINK EXTERNO</button></nav><div className="study-material-form">
    {mode === "file" ? <><label>ARQUIVO<div className="study-material-picker"><input readOnly value={fileDraft.sourcePath} placeholder="PDF, imagem, TXT ou Markdown" /><button onClick={() => void onChooseFile()}>SELECIONAR</button></div></label><label>ARMAZENAMENTO<select value={fileDraft.storageKind} onChange={(event) => update({ storageKind: event.target.value as StudyMaterialStorageKind })}><option value="MANAGED_COPY">COPIAR PARA O AZRIEL</option><option value="LINKED_LOCAL_FILE">VINCULAR ARQUIVO ORIGINAL</option></select></label></> : <label>URL<input value={linkDraft.externalUrl} onChange={(event) => update({ externalUrl: event.target.value })} placeholder="https://..." /></label>}
    <label>TÍTULO<input maxLength={240} value={draft.title} onChange={(event) => update({ title: event.target.value })} /></label><div className="study-material-form__row"><label>AUTOR<input value={draft.sourceAuthor ?? ""} onChange={(event) => update({ sourceAuthor: event.target.value || null })} /></label><label>ANO<input type="number" min="1000" max="9999" value={draft.sourceYear ?? ""} onChange={(event) => update({ sourceYear: event.target.value ? Number(event.target.value) : null })} /></label></div><label>TÍTULO DA FONTE<input value={draft.sourceTitle ?? ""} onChange={(event) => update({ sourceTitle: event.target.value || null })} /></label><label>DESCRIÇÃO<textarea value={draft.description ?? ""} onChange={(event) => update({ description: event.target.value || null })} /></label>
  </div><footer><button onClick={onCancel} disabled={busy}>CANCELAR</button><button className="primary" onClick={() => void onSave()} disabled={busy || !draft.title.trim()}>{busy ? "PROCESSANDO..." : mode === "file" ? "IMPORTAR" : "ADICIONAR LINK"}</button></footer></section></div>;
}

type DetailProps = { material: StudyMaterial; roadmaps: StudyRoadmap[]; notebooks: StudyNotebook[]; notes: StudyNoteSummary[]; busy: boolean; onRefresh: () => Promise<void>; onError: (value: string | null) => void; onRemove: () => void; onDeleteFile: () => void };
function MaterialDetail({ material, roadmaps, notebooks, notes, busy, onRefresh, onError, onRemove, onDeleteFile }: DetailProps) {
  const [preview, setPreview] = useState<{ materialId: string; url: string } | null>(null);
  const [selectedText, setSelectedText] = useState("");
  const [pageFrom, setPageFrom] = useState(1);
  const [pageTo, setPageTo] = useState(1);
  const [showAI, setShowAI] = useState(false);
  const [relationType, setRelationType] = useState<StudyMaterialRelationType>("ROADMAP");
  const [relationId, setRelationId] = useState("");
  const [notebookId, setNotebookId] = useState(notebooks[0]?.id ?? "");

  useEffect(() => {
    let url: string | null = null;
    let cancelled = false;
    if (material.storageKind !== "EXTERNAL_URL" && ["PDF", "IMAGE"].includes(material.materialType) && material.managedFileAvailable) {
      void studyLabService.previewMaterial(material.id).then((value) => {
        const bytes = value instanceof Uint8Array ? value : new Uint8Array(value);
        url = URL.createObjectURL(new Blob([bytes.slice().buffer], { type: material.mimeType || "application/octet-stream" }));
        if (cancelled) URL.revokeObjectURL(url);
        else setPreview({ materialId: material.id, url });
      }).catch(() => undefined);
    }
    return () => { cancelled = true; if (url) URL.revokeObjectURL(url); };
  }, [material.id, material.managedFileAvailable, material.materialType, material.mimeType, material.storageKind]);
  const previewUrl = preview?.materialId === material.id ? preview.url : null;

  const relationOptions = useMemo(() => {
    if (relationType === "ROADMAP") return roadmaps.map((item) => ({ id: item.id, label: item.name }));
    if (relationType === "STAGE") return roadmaps.flatMap((roadmap) => roadmap.stages.map((item) => ({ id: item.id, label: `${roadmap.name} / ${item.name}` })));
    if (relationType === "TOPIC") return roadmaps.flatMap((roadmap) => roadmap.stages.flatMap((stage) => stage.topics.map((item) => ({ id: item.id, label: `${roadmap.name} / ${stage.name} / ${item.name}` }))));
    if (relationType === "ACTIVITY") return roadmaps.flatMap((roadmap) =>
      roadmap.stages.flatMap((stage) =>
        stage.topics.flatMap((topic) =>
          topic.activities.map((item) => ({ id: item.id, label: `${topic.name} / ${item.title}` })),
        ),
      ),
    );
    if (relationType === "NOTEBOOK") return notebooks.map((item) => ({ id: item.id, label: item.title }));
    if (relationType === "NOTE") return notes.map((item) => ({ id: item.id, label: `${item.notebookTitle} / ${item.title}` }));
    return [];
  }, [notebooks, notes, relationType, roadmaps]);

  function usePages() {
    const pages = material.textContent.pages.slice(Math.max(0, pageFrom - 1), Math.min(material.textContent.pages.length, pageTo));
    setSelectedText(pages.join("\n\n").trim());
  }
  async function associate() {
    const target = relationId || relationOptions[0]?.id;
    if (!target) return;
    try { await studyLabService.associateMaterial(material.id, { relationType, relationId: target }); await onRefresh(); }
    catch (reason) { onError(reason instanceof Error ? reason.message : String(reason)); }
  }
  async function createNote() {
    const targetNotebook = notebookId || notebooks[0]?.id;
    if (!targetNotebook) { onError("Crie um caderno antes de gerar uma nota a partir do material."); return; }
    try {
      const input: StudyNoteInput = { id: crypto.randomUUID(), notebookId: targetNotebook, title: `Notas — ${material.title}`.slice(0, 200), content: "", roadmapId: null, stageId: null, topicId: null, activityId: null, studySessionId: null };
      await studyLabService.createNoteFromMaterial(material.id, input);
      await onRefresh();
    } catch (reason) { onError(reason instanceof Error ? reason.message : String(reason)); }
  }
  async function saveGeneratedCards(drafts: StudyAICardDraft[]) {
    const inputs: StudyCardInput[] = drafts.map((draft) => ({ id: draft.id, front: draft.front, back: draft.back, roadmapId: null, stageId: null, topicId: null, activityId: null, notebookId: null, noteId: null, sourceMaterialIds: [material.id] }));
    await studyLabService.saveCards(inputs); await onRefresh();
  }
  const aiContext = selectedText ? { materials: [{ id: material.id, title: material.title, materialType: material.materialType, selectedText, pageRange: material.materialType === "PDF" ? { from: pageFrom, to: pageTo } : null }] } : null;
  return <article className="study-material-detail">
    <header><div><span>{material.materialType} · {material.storageKind}</span><h2>{material.title}</h2><small data-status={material.status}>{material.status} · {material.textContent.isStale ? "EXTRAÇÃO DESATUALIZADA" : material.textContent.status}</small></div><nav><button onClick={() => void studyLabService.openMaterial(material.id)}>ABRIR</button><button onClick={() => setShowAI((value) => !value)} disabled={!selectedText}>USAR COM AZRIEL</button></nav></header>
    <div className="study-material-detail__metadata"><span><b>ARQUIVO</b>{material.originalFilename || "LINK EXTERNO"}</span><span><b>TAMANHO</b>{formatBytes(material.fileSize)}</span><span><b>AUTOR</b>{material.sourceAuthor || "—"}</span><span><b>ANO</b>{material.sourceYear || "—"}</span><span><b>PÁGINAS</b>{material.pageCount ?? "—"}</span><span><b>CHECKSUM</b>{material.checksumSha256?.slice(0, 16) || "—"}</span></div>
    {material.description && <p className="study-material-detail__description">{material.description}</p>}
    <section className="study-material-preview"><header><strong>PREVIEW / TEXTO DA FONTE</strong><span>PROCESSAMENTO LOCAL</span></header>{material.materialType === "PDF" && previewUrl ? <object data={previewUrl} type="application/pdf"><p>Preview indisponível. Use ABRIR.</p></object> : material.materialType === "IMAGE" && previewUrl ? <img src={previewUrl} alt={material.title} /> : material.materialType === "MARKDOWN" && material.textContent.text ? <div onMouseUp={() => setSelectedText(window.getSelection()?.toString().trim() || "")}><StudyMarkdown content={material.textContent.text} /></div> : material.textContent.text ? <textarea readOnly value={material.textContent.text} onSelect={(event) => { const target = event.currentTarget; setSelectedText(target.value.slice(target.selectionStart, target.selectionEnd).trim()); }} /> : <p>{material.materialType === "LINK" ? "Links não são baixados ou processados automaticamente." : material.textContent.status === "TEXT_UNAVAILABLE" ? "PDF válido sem camada textual. OCR não faz parte desta versão." : "Preview textual indisponível."}</p>}</section>
    {material.textContent.pages.length > 1 && <div className="study-material-pages"><label>PÁGINA INICIAL<input type="number" min="1" max={material.textContent.pages.length} value={pageFrom} onChange={(event) => setPageFrom(Number(event.target.value))} /></label><label>PÁGINA FINAL<input type="number" min={pageFrom} max={material.textContent.pages.length} value={pageTo} onChange={(event) => setPageTo(Number(event.target.value))} /></label><button onClick={usePages}>USAR INTERVALO</button></div>}
    {selectedText && <div className="study-material-selection"><span>{selectedText.length.toLocaleString("pt-BR")} caracteres selecionados{selectedText.length > 10_000 ? " · o contexto poderá ser truncado" : ""}</span><button onClick={() => setSelectedText("")}>LIMPAR</button></div>}
    {showAI && aiContext && <StudyAIToolPanel context={aiContext} onSaveCards={saveGeneratedCards} onClose={() => setShowAI(false)} />}
    <section className="study-material-relations"><header><div><strong>RELAÇÕES</strong><span>CONTEXTO EXPLÍCITO</span></div></header><div>{material.relations.map((relation) => <span key={relation.id}><b>{relation.relationType}</b>{relation.label}<button aria-label={`Remover relação ${relation.label}`} onClick={() => void studyLabService.dissociateMaterial(material.id, relation.relationType, relation.relationId).then(onRefresh).catch((reason: unknown) => onError(reason instanceof Error ? reason.message : String(reason)))}>×</button></span>)}{!material.relations.length && <small>Material ainda sem relações.</small>}</div><footer><select value={relationType} onChange={(event) => { setRelationType(event.target.value as StudyMaterialRelationType); setRelationId(""); }}>{(["ROADMAP", "STAGE", "TOPIC", "ACTIVITY", "NOTEBOOK", "NOTE"] as const).map((item) => <option key={item} value={item}>{item}</option>)}</select><select value={relationId} onChange={(event) => setRelationId(event.target.value)}>{relationOptions.map((option) => <option key={option.id} value={option.id}>{option.label}</option>)}</select><button onClick={() => void associate()} disabled={!relationOptions.length}>ASSOCIAR</button></footer></section>
    <footer className="study-material-actions"><label>CRIAR NOTA EM<select value={notebookId} onChange={(event) => setNotebookId(event.target.value)}>{notebooks.map((item) => <option key={item.id} value={item.id}>{item.title}</option>)}</select></label><button onClick={() => void createNote()}>CRIAR NOTA VAZIA</button>{["PDF", "TEXT", "MARKDOWN"].includes(material.materialType) && <button disabled={busy} onClick={() => void studyLabService.reprocessMaterialText(material.id).then(onRefresh).catch((reason: unknown) => onError(reason instanceof Error ? reason.message : String(reason)))}>REPROCESSAR TEXTO</button>}<button onClick={() => void studyLabService.setMaterialStatus(material.id, material.status === "ARCHIVED" ? "ACTIVE" : "ARCHIVED").then(onRefresh).catch((reason: unknown) => onError(reason instanceof Error ? reason.message : String(reason)))}>{material.status === "ARCHIVED" ? "RESTAURAR" : "ARQUIVAR"}</button><button className="danger-link" onClick={onRemove}>REMOVER DA BIBLIOTECA</button>{material.storageKind === "MANAGED_COPY" && material.managedFileAvailable && <button className="danger-link" onClick={onDeleteFile}>EXCLUIR CÓPIA</button>}</footer>
  </article>;
}
