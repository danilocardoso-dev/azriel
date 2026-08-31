import { useState, type FormEvent } from "react";
import { RecordEditorDialog } from "../components/core/RecordEditorDialog";
import { DeleteConfirmationDialog } from "../components/daily/DeleteConfirmationDialog";
import { HudPanel } from "../components/hud/HudPanel";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { useAutomation } from "../contexts/useAutomation";
import { useSystem } from "../contexts/useSystem";
import type { Application, ApplicationInput, RegisteredUrl, RegisteredUrlInput } from "../types";

type Tab = "applications" | "urls" | "history";
type DeleteTarget = { kind: "application"; value: Application } | { kind: "url"; value: RegisteredUrl };
const newApplication = (): ApplicationInput => ({ id: crypto.randomUUID(), name: "", path: "", enabled: true });
const newUrl = (): RegisteredUrlInput => ({ id: crypto.randomUUID(), name: "", url: "https://", enabled: true });
const resultLabel = (success: boolean | null) => success === null ? "PENDENTE" : success ? "SUCESSO" : "RECUSADA / FALHA";

export function AutomationPage() {
  const automation = useAutomation();
  const { workspaces } = useSystem();
  const [tab, setTab] = useState<Tab>("applications");
  const [applicationDraft, setApplicationDraft] = useState<ApplicationInput | null>(null);
  const [urlDraft, setUrlDraft] = useState<RegisteredUrlInput | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<DeleteTarget | null>(null);
  const [busy, setBusy] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);

  const chooseApplication = async () => {
    const path = await automation.selectApplication();
    if (!path) return;
    const name = path.split(/[\\/]/).at(-1)?.replace(/\.exe$/i, "") ?? "Aplicativo";
    setApplicationDraft((draft) => ({ ...(draft ?? newApplication()), path, name: draft?.name || name }));
  };
  const saveApplication = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault(); if (!applicationDraft) return; setBusy(true); setFormError(null);
    try { await automation.saveApplication(applicationDraft); setApplicationDraft(null); }
    catch (reason) { setFormError(reason instanceof Error ? reason.message : String(reason)); }
    finally { setBusy(false); }
  };
  const saveUrl = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault(); if (!urlDraft) return; setBusy(true); setFormError(null);
    try { await automation.saveUrl(urlDraft); setUrlDraft(null); }
    catch (reason) { setFormError(reason instanceof Error ? reason.message : String(reason)); }
    finally { setBusy(false); }
  };
  const confirmDelete = async () => {
    if (!deleteTarget) return; setBusy(true);
    try {
      if (deleteTarget.kind === "application") await automation.deleteApplication(deleteTarget.value.id);
      else await automation.deleteUrl(deleteTarget.value.id);
      setDeleteTarget(null);
    } finally { setBusy(false); }
  };

  return <>
    <ModuleIntro code="AUT-10" title="Automation Core" description="Autoridade local limitada a aplicativos, workspaces e URLs previamente registrados." metric="V0.8.0 // SAFE ACTIONS" />
    {automation.error && <div className="system-warning"><strong>AUTOMATION CORE</strong>{automation.error}</div>}
    <div className="automation-summary">
      <article><span>STATUS</span><strong>{automation.state.toUpperCase()}</strong><small>POLICY ENGINE</small></article>
      <article><span>AÇÕES DISPONÍVEIS</span><strong>{automation.actions.length.toString().padStart(2, "0")}</strong><small>SAFE MODE</small></article>
      <article><span>APLICATIVOS</span><strong>{automation.applications.filter((item) => item.enabled).length.toString().padStart(2, "0")}</strong><small>AUTORIZADOS</small></article>
      <article><span>WORKSPACES</span><strong>{workspaces.filter((item) => item.enabled).length.toString().padStart(2, "0")}</strong><small>ATIVOS</small></article>
    </div>
    <HudPanel title="Registros autorizados" code="CAPACIDADE SEM ACESSO IRRESTRITO">
      <nav className="automation-tabs" aria-label="Seções do Automation Core">
        <button className={tab === "applications" ? "active" : ""} onClick={() => setTab("applications")}>APLICATIVOS</button>
        <button className={tab === "urls" ? "active" : ""} onClick={() => setTab("urls")}>URLs</button>
        <button className={tab === "history" ? "active" : ""} onClick={() => setTab("history")}>HISTÓRICO</button>
      </nav>
      {tab === "applications" && <section className="automation-section">
        <header><p>Somente executáveis escolhidos manualmente podem ser abertos.</p><button onClick={() => { setFormError(null); setApplicationDraft(newApplication()); }}>+ AUTORIZAR APLICATIVO</button></header>
        <div className="automation-records">{automation.applications.map((app) => <article key={app.id} data-disabled={!app.enabled}>
          <div><small>APLICATIVO</small><strong>{app.name}</strong><span>{app.path}</span></div><i>{app.enabled ? "ATIVO" : "DESATIVADO"}</i>
          <button onClick={() => setApplicationDraft({ id: app.id, name: app.name, path: app.path, enabled: app.enabled })}>EDITAR</button>
          <button onClick={() => void automation.execute({ actionId: "open_application", source: "ui", targetId: app.id })}>TESTAR</button>
          <button className="danger-link" onClick={() => setDeleteTarget({ kind: "application", value: app })}>REMOVER</button>
        </article>)}</div>
        {!automation.applications.length && <p className="system-empty">Nenhum aplicativo autorizado.</p>}
      </section>}
      {tab === "urls" && <section className="automation-section">
        <header><p>São aceitas apenas URLs HTTP/HTTPS cadastradas, sem credenciais embutidas.</p><button onClick={() => { setFormError(null); setUrlDraft(newUrl()); }}>+ CADASTRAR URL</button></header>
        <div className="automation-records">{automation.urls.map((url) => <article key={url.id} data-disabled={!url.enabled}>
          <div><small>URL REGISTRADA</small><strong>{url.name}</strong><span>{url.url}</span></div><i>{url.enabled ? "ATIVA" : "DESATIVADA"}</i>
          <button onClick={() => setUrlDraft({ id: url.id, name: url.name, url: url.url, enabled: url.enabled })}>EDITAR</button>
          <button onClick={() => void automation.execute({ actionId: "open_registered_url", source: "ui", targetId: url.id })}>TESTAR</button>
          <button className="danger-link" onClick={() => setDeleteTarget({ kind: "url", value: url })}>REMOVER</button>
        </article>)}</div>
        {!automation.urls.length && <p className="system-empty">Nenhuma URL autorizada.</p>}
      </section>}
      {tab === "history" && <section className="automation-section">
        <header><p>Toda tentativa é auditada, inclusive recusas e falhas.</p><button onClick={() => void automation.refresh()}>ATUALIZAR</button></header>
        <div className="automation-history"><table><thead><tr><th>DATA</th><th>AÇÃO</th><th>ORIGEM</th><th>ALVO</th><th>RESULTADO</th></tr></thead><tbody>{automation.history.map((item) => <tr key={item.id}><td>{new Date(item.createdAt).toLocaleString("pt-BR")}</td><td>{item.actionId}</td><td>{item.source.toUpperCase()}</td><td>{item.targetName ?? item.targetId ?? "—"}</td><td data-success={item.success}>{resultLabel(item.success)}{item.error && <small>{item.error}</small>}</td></tr>)}</tbody></table></div>
        {!automation.history.length && <p className="system-empty">Nenhuma tentativa registrada.</p>}
      </section>}
    </HudPanel>
    {applicationDraft && <RecordEditorDialog eyebrow="APPLICATION REGISTRY" title="Aplicativo autorizado" busy={busy} error={formError} onCancel={() => setApplicationDraft(null)} onSubmit={saveApplication}>
      <label>Nome<input required maxLength={100} value={applicationDraft.name} onChange={(event) => setApplicationDraft({ ...applicationDraft, name: event.target.value })} /></label>
      <label className="record-editor__wide">Executável<div className="automation-path"><input required readOnly value={applicationDraft.path} placeholder="Selecione um arquivo .exe" /><button type="button" onClick={() => void chooseApplication()}>SELECIONAR</button></div></label>
      <label className="automation-check"><input type="checkbox" checked={applicationDraft.enabled} onChange={(event) => setApplicationDraft({ ...applicationDraft, enabled: event.target.checked })} /> Registro habilitado</label>
    </RecordEditorDialog>}
    {urlDraft && <RecordEditorDialog eyebrow="URL REGISTRY" title="URL autorizada" busy={busy} error={formError} onCancel={() => setUrlDraft(null)} onSubmit={saveUrl}>
      <label>Nome<input required maxLength={100} value={urlDraft.name} onChange={(event) => setUrlDraft({ ...urlDraft, name: event.target.value })} /></label>
      <label className="record-editor__wide">URL<input required type="url" value={urlDraft.url} onChange={(event) => setUrlDraft({ ...urlDraft, url: event.target.value })} /></label>
      <label className="automation-check"><input type="checkbox" checked={urlDraft.enabled} onChange={(event) => setUrlDraft({ ...urlDraft, enabled: event.target.checked })} /> Registro habilitado</label>
    </RecordEditorDialog>}
    {deleteTarget && <DeleteConfirmationDialog kind={deleteTarget.kind === "application" ? "aplicativo" : "URL autorizada"} title={deleteTarget.value.name} description="Remove somente a autorização persistida no Azriel. O aplicativo e os dados externos não serão apagados." busy={busy} onCancel={() => setDeleteTarget(null)} onConfirm={() => void confirmDelete()} />}
  </>;
}
