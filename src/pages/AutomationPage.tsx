import { useState, type FormEvent } from "react";
import { RecordEditorDialog } from "../components/core/RecordEditorDialog";
import { DeleteConfirmationDialog } from "../components/daily/DeleteConfirmationDialog";
import { HudPanel } from "../components/hud/HudPanel";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { useAutomation } from "../contexts/useAutomation";
import { useSystem } from "../contexts/useSystem";
import { useAzrielData } from "../contexts/useAzrielData";
import type { Application, ApplicationInput, AutomationActionId, RegisteredAction, RegisteredUrl, RegisteredUrlInput, Routine, RoutineInput, RoutineStep } from "../types";

type Tab = "applications" | "urls" | "routines" | "history";
type DeleteTarget = { kind: "application"; value: Application } | { kind: "url"; value: RegisteredUrl } | { kind: "routine"; value: Routine };
const newApplication = (): ApplicationInput => ({ id: crypto.randomUUID(), name: "", path: "", enabled: true });
const newUrl = (): RegisteredUrlInput => ({ id: crypto.randomUUID(), name: "", url: "https://", enabled: true });
const newRoutine = (): RoutineInput => ({ id: crypto.randomUUID(), name: "", description: "", enabled: true, confirmationRequired: true, steps: [] });
const resultLabel = (success: boolean | null) => success === null ? "PENDENTE" : success ? "SUCESSO" : "RECUSADA / FALHA";

export function AutomationPage() {
  const automation = useAutomation();
  const { workspaces } = useSystem();
  const { projects } = useAzrielData();
  const [tab, setTab] = useState<Tab>("applications");
  const [applicationDraft, setApplicationDraft] = useState<ApplicationInput | null>(null);
  const [urlDraft, setUrlDraft] = useState<RegisteredUrlInput | null>(null);
  const [routineDraft, setRoutineDraft] = useState<RoutineInput | null>(null);
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
  const saveRoutine = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault(); if (!routineDraft) return; setBusy(true); setFormError(null);
    try { await automation.saveRoutine(routineDraft); setRoutineDraft(null); }
    catch (reason) { setFormError(reason instanceof Error ? reason.message : String(reason)); }
    finally { setBusy(false); }
  };
  const confirmDelete = async () => {
    if (!deleteTarget) return; setBusy(true);
    try {
      if (deleteTarget.kind === "application") await automation.deleteApplication(deleteTarget.value.id);
      else if (deleteTarget.kind === "url") await automation.deleteUrl(deleteTarget.value.id);
      else await automation.deleteRoutine(deleteTarget.value.id);
      setDeleteTarget(null);
    } finally { setBusy(false); }
  };

  const targetsFor = (action: RegisteredAction) => {
    if (action.targetType === "application") return automation.applications.filter((item) => item.enabled).map((item) => ({ id: item.id, name: item.name }));
    if (action.targetType === "workspace") return workspaces.filter((item) => item.enabled).map((item) => ({ id: item.id, name: item.name }));
    if (action.targetType === "project") return projects.map((item) => ({ id: item.id, name: item.name }));
    return automation.urls.filter((item) => item.enabled).map((item) => ({ id: item.id, name: item.name }));
  };
  const updateStep = (index: number, patch: Partial<RoutineStep>) => setRoutineDraft((draft) => draft ? ({ ...draft, steps: draft.steps.map((step, position) => position === index ? { ...step, ...patch } : step) }) : null);
  const addStep = () => setRoutineDraft((draft) => {
    if (!draft) return null;
    const action = automation.actions[0];
    if (!action) return draft;
    const target = targetsFor(action)[0];
    return { ...draft, steps: [...draft.steps, { id: crypto.randomUUID(), order: draft.steps.length + 1, actionId: action.id, targetType: action.targetType, targetId: target?.id ?? "", delayMs: 0, enabled: true }] };
  });
  const removeStep = (index: number) => setRoutineDraft((draft) => draft ? ({ ...draft, steps: draft.steps.filter((_, position) => position !== index).map((step, position) => ({ ...step, order: position + 1 })) }) : null);
  const moveStep = (index: number, direction: -1 | 1) => setRoutineDraft((draft) => {
    if (!draft) return null;
    const destination = index + direction;
    if (destination < 0 || destination >= draft.steps.length) return draft;
    const steps = [...draft.steps]; [steps[index], steps[destination]] = [steps[destination], steps[index]];
    return { ...draft, steps: steps.map((step, position) => ({ ...step, order: position + 1 })) };
  });

  return <>
    <ModuleIntro code="AUT-10" title="Automation Core" description="Ações autorizadas organizadas em rotinas explícitas, persistentes e auditadas." metric="V0.8.1 // ROTINAS" />
    {automation.error && <div className="system-warning"><strong>AUTOMATION CORE</strong>{automation.error}</div>}
    <div className="automation-summary">
      <article><span>STATUS</span><strong>{automation.state.toUpperCase()}</strong><small>POLICY ENGINE</small></article>
      <article><span>AÇÕES DISPONÍVEIS</span><strong>{automation.actions.length.toString().padStart(2, "0")}</strong><small>SAFE MODE</small></article>
      <article><span>APLICATIVOS</span><strong>{automation.applications.filter((item) => item.enabled).length.toString().padStart(2, "0")}</strong><small>AUTORIZADOS</small></article>
      <article><span>ROTINAS</span><strong>{automation.routines.filter((item) => item.enabled).length.toString().padStart(2, "0")}</strong><small>ATIVAS</small></article>
    </div>
    <HudPanel title="Registros autorizados" code="CAPACIDADE SEM ACESSO IRRESTRITO">
      <nav className="automation-tabs" aria-label="Seções do Automation Core">
        <button className={tab === "applications" ? "active" : ""} onClick={() => setTab("applications")}>APLICATIVOS</button>
        <button className={tab === "urls" ? "active" : ""} onClick={() => setTab("urls")}>URLs</button>
        <button className={tab === "routines" ? "active" : ""} onClick={() => setTab("routines")}>ROTINAS</button>
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
      {tab === "routines" && <section className="automation-section">
        <header><p>Combine somente ações e alvos já autorizados. A execução segue a ordem e para no primeiro erro.</p><button onClick={() => { setFormError(null); setRoutineDraft(newRoutine()); }}>+ CRIAR ROTINA</button></header>
        <div className="automation-records routine-records">{automation.routines.map((routine) => <article key={routine.id} data-disabled={!routine.enabled}>
          <div><small>ROTINA // REV {routine.revision}</small><strong>{routine.name}</strong><span>{routine.description || "Sem descrição"} // {routine.steps.length} PASSOS</span></div><i>{routine.enabled ? "ATIVA" : "DESATIVADA"}</i>
          <button onClick={() => setRoutineDraft({ id: routine.id, name: routine.name, description: routine.description, enabled: routine.enabled, confirmationRequired: routine.confirmationRequired, steps: routine.steps })}>EDITAR</button>
          <button disabled={!routine.enabled} onClick={() => void automation.runRoutine({ routineId: routine.id, source: "ui" })}>EXECUTAR</button>
          <button className="danger-link" onClick={() => setDeleteTarget({ kind: "routine", value: routine })}>REMOVER</button>
        </article>)}</div>
        {!automation.routines.length && <p className="system-empty">Nenhuma rotina cadastrada.</p>}
      </section>}
      {tab === "history" && <section className="automation-section">
        <header><p>Toda tentativa é auditada, inclusive recusas e falhas.</p><button onClick={() => void automation.refresh()}>ATUALIZAR</button></header>
        <h3 className="automation-history-title">EXECUÇÕES DE ROTINAS</h3>
        <div className="automation-history"><table><thead><tr><th>DATA</th><th>ROTINA</th><th>ORIGEM</th><th>PROGRESSO</th><th>RESULTADO</th></tr></thead><tbody>{automation.routineHistory.map((item) => <tr key={item.id}><td>{new Date(item.startedAt).toLocaleString("pt-BR")}</td><td>{item.routineName}</td><td>{item.source.toUpperCase()}</td><td>{item.completedSteps} / {item.totalSteps}{item.failedStep ? ` // FALHA NO PASSO ${item.failedStep}` : ""}</td><td data-success={item.status === "completed"}>{item.status.toUpperCase()}{item.error && <small>{item.error}</small>}</td></tr>)}</tbody></table></div>
        {!automation.routineHistory.length && <p className="system-empty">Nenhuma rotina executada.</p>}
        <h3 className="automation-history-title">AÇÕES INDIVIDUAIS</h3>
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
    {routineDraft && <RecordEditorDialog eyebrow="ROUTINE REGISTRY" title="Rotina autorizada" busy={busy} error={formError} onCancel={() => setRoutineDraft(null)} onSubmit={saveRoutine}>
      <label>Nome<input required maxLength={120} value={routineDraft.name} onChange={(event) => setRoutineDraft({ ...routineDraft, name: event.target.value })} /></label>
      <label className="automation-check"><input type="checkbox" checked={routineDraft.enabled} onChange={(event) => setRoutineDraft({ ...routineDraft, enabled: event.target.checked })} /> Rotina habilitada</label>
      <label className="record-editor__wide">Descrição<textarea maxLength={500} rows={2} value={routineDraft.description} onChange={(event) => setRoutineDraft({ ...routineDraft, description: event.target.value })} /></label>
      <label className="automation-check record-editor__wide"><input type="checkbox" checked={routineDraft.confirmationRequired} onChange={(event) => setRoutineDraft({ ...routineDraft, confirmationRequired: event.target.checked })} /> Exigir confirmação antes de executar</label>
      <fieldset className="record-editor__wide routine-editor"><legend>PASSOS AUTORIZADOS</legend>
        <div className="routine-editor__toolbar"><small>Máximo de 20 passos. Intervalo de 0 a 10000 ms.</small><button type="button" onClick={addStep}>+ ADICIONAR PASSO</button></div>
        {routineDraft.steps.map((step, index) => {
          const action = automation.actions.find((item) => item.id === step.actionId) ?? automation.actions[0];
          const targets = action ? targetsFor(action) : [];
          return <article className="routine-step" key={step.id}>
            <header><strong>PASSO {String(index + 1).padStart(2, "0")}</strong><div><button type="button" disabled={index === 0} onClick={() => moveStep(index, -1)}>↑</button><button type="button" disabled={index === routineDraft.steps.length - 1} onClick={() => moveStep(index, 1)}>↓</button><button type="button" className="danger-link" onClick={() => removeStep(index)}>REMOVER</button></div></header>
            <label>Ação<select value={step.actionId} onChange={(event) => {
              const selected = automation.actions.find((item) => item.id === event.target.value)!;
              updateStep(index, { actionId: selected.id as AutomationActionId, targetType: selected.targetType, targetId: targetsFor(selected)[0]?.id ?? "" });
            }}>{automation.actions.map((item) => <option key={item.id} value={item.id}>{item.name}</option>)}</select></label>
            <label>Alvo autorizado<select required value={step.targetId} onChange={(event) => updateStep(index, { targetId: event.target.value })}><option value="">SELECIONE</option>{targets.map((target) => <option key={target.id} value={target.id}>{target.name}</option>)}</select></label>
            <label>Intervalo após o passo (ms)<input type="number" min={0} max={10000} step={100} value={step.delayMs} onChange={(event) => updateStep(index, { delayMs: Number(event.target.value) })} /></label>
            <label className="automation-check"><input type="checkbox" checked={step.enabled} onChange={(event) => updateStep(index, { enabled: event.target.checked })} /> Passo ativo</label>
          </article>;
        })}
        {!routineDraft.steps.length && <p className="system-empty">Adicione ao menos um passo para salvar.</p>}
      </fieldset>
    </RecordEditorDialog>}
    {deleteTarget && <DeleteConfirmationDialog kind={deleteTarget.kind === "application" ? "aplicativo" : deleteTarget.kind === "url" ? "URL autorizada" : "rotina"} title={deleteTarget.value.name} description={deleteTarget.kind === "routine" ? "Remove a rotina e seus passos persistidos. Os históricos de execução permanecem para auditoria." : "Remove somente a autorização persistida no Azriel. O aplicativo e os dados externos não serão apagados."} busy={busy} onCancel={() => setDeleteTarget(null)} onConfirm={() => void confirmDelete()} />}
  </>;
}
