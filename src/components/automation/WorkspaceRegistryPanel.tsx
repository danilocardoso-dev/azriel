import { useState, type FormEvent } from "react";
import { useAutomation } from "../../contexts/useAutomation";
import { useAzrielData } from "../../contexts/useAzrielData";
import { useSystem } from "../../contexts/useSystem";
import type { Workspace, WorkspaceInput } from "../../types";
import { RecordEditorDialog } from "../core/RecordEditorDialog";
import { DeleteConfirmationDialog } from "../daily/DeleteConfirmationDialog";

const blankWorkspace = (): WorkspaceInput => ({ id: crypto.randomUUID(), name: "", path: "", projectId: null, applicationId: null, enabled: true });

export function WorkspaceRegistryPanel() {
  const { projects } = useAzrielData();
  const { applications } = useAutomation();
  const { workspaces, saveWorkspace, deleteWorkspace, selectDirectory } = useSystem();
  const [draft, setDraft] = useState<WorkspaceInput | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<Workspace | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const chooseDirectory = async () => {
    const path = await selectDirectory();
    if (!path) return;
    const fallbackName = path.split(/[\\/]/).filter(Boolean).at(-1) ?? "Workspace";
    setDraft((current) => ({ ...(current ?? blankWorkspace()), path, name: current?.name || fallbackName }));
  };

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!draft) return;
    setBusy(true);
    setError(null);
    try {
      await saveWorkspace(draft);
      setDraft(null);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  };

  const toggle = async (workspace: Workspace) => {
    await saveWorkspace({ id: workspace.id, name: workspace.name, path: workspace.path, projectId: workspace.projectId, applicationId: workspace.applicationId, enabled: !workspace.enabled });
  };

  return <section className="automation-section">
    <header><p>Pastas explicitamente autorizadas para projetos, rotinas e ações seguras.</p><button onClick={() => { setError(null); setDraft(blankWorkspace()); }}>+ AUTORIZAR WORKSPACE</button></header>
    <div className="automation-records workspace-registry-records">{workspaces.map((workspace) => <article key={workspace.id} data-disabled={!workspace.enabled}>
      <div><small>WORKSPACE AUTORIZADO</small><strong>{workspace.name}</strong><span>{workspace.path}</span></div><i>{workspace.enabled ? "ATIVO" : "DESATIVADO"}</i>
      <button onClick={() => setDraft({ id: workspace.id, name: workspace.name, path: workspace.path, projectId: workspace.projectId, applicationId: workspace.applicationId, enabled: workspace.enabled })}>EDITAR</button>
      <button onClick={() => void toggle(workspace)}>{workspace.enabled ? "DESATIVAR" : "ATIVAR"}</button>
      <button className="danger-link" onClick={() => setDeleteTarget(workspace)}>REMOVER</button>
    </article>)}</div>
    {!workspaces.length && <p className="system-empty">Nenhum workspace autorizado.</p>}

    {draft && <RecordEditorDialog eyebrow="WORKSPACE REGISTRY" title="Workspace autorizado" busy={busy} error={error} onCancel={() => setDraft(null)} onSubmit={submit}>
      <label>Nome<input required maxLength={100} value={draft.name} onChange={(event) => setDraft({ ...draft, name: event.target.value })} /></label>
      <label className="record-editor__wide">Pasta<div className="automation-path"><input required readOnly value={draft.path} placeholder="Selecione uma pasta" /><button type="button" onClick={() => void chooseDirectory()}>SELECIONAR</button></div></label>
      <label>Projeto relacionado<select value={draft.projectId ?? ""} onChange={(event) => setDraft({ ...draft, projectId: event.target.value || null })}><option value="">Nenhum projeto</option>{projects.map((project) => <option value={project.id} key={project.id}>{project.name}</option>)}</select></label>
      <label>Aplicativo de abertura<select value={draft.applicationId ?? ""} onChange={(event) => setDraft({ ...draft, applicationId: event.target.value || null })}><option value="">Nenhum aplicativo</option>{applications.map((application) => <option value={application.id} key={application.id} disabled={!application.enabled}>{application.name}{application.enabled ? "" : " (desativado)"}</option>)}</select></label>
      <label className="automation-check record-editor__wide"><input type="checkbox" checked={draft.enabled} onChange={(event) => setDraft({ ...draft, enabled: event.target.checked })} /> Workspace habilitado</label>
    </RecordEditorDialog>}
    {deleteTarget && <DeleteConfirmationDialog kind="workspace" title={deleteTarget.name} description="Remove somente a autorização e os metadados locais. Nenhuma pasta ou arquivo será apagado." busy={busy} onCancel={() => setDeleteTarget(null)} onConfirm={() => void deleteWorkspace(deleteTarget.id).then(() => setDeleteTarget(null))} />}
  </section>;
}
