import { useEffect, useMemo, useState, type FormEvent } from "react";
import { DeleteConfirmationDialog } from "../components/daily/DeleteConfirmationDialog";
import { HudPanel } from "../components/hud/HudPanel";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { useAI } from "../contexts/useAI";
import { useAzrielData } from "../contexts/useAzrielData";
import { useSystem } from "../contexts/useSystem";
import { filterAndSortProcesses, formatBytes, type ProcessSort } from "../services/systemService";
import type { AzrielState, Workspace, WorkspaceInput } from "../types";

interface SystemPageProps { coreState: AzrielState; onOpenAI: () => void; }

const percent = (used: number, total: number) => total > 0 ? Math.min(100, Math.round((used / total) * 100)) : 0;
const formatUptime = (seconds: number) => {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  return `${days}d ${hours}h ${minutes}m`;
};
const blankWorkspace = (): WorkspaceInput => ({ id: crypto.randomUUID(), name: "", path: "", projectId: null, enabled: true });

export function SystemPage({ coreState, onOpenAI }: SystemPageProps) {
  const { projects } = useAzrielData();
  const { status: ollamaStatus, settings } = useAI();
  const { snapshot, processes, workspaces, selectedWorkspace, loading, processLoading, error, refreshProcesses, inspectWorkspace, saveWorkspace, deleteWorkspace, selectDirectory } = useSystem();
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<ProcessSort>("memory");
  const [descending, setDescending] = useState(true);
  const [workspaceDraft, setWorkspaceDraft] = useState<WorkspaceInput | null>(null);
  const [workspaceError, setWorkspaceError] = useState<string | null>(null);
  const [savingWorkspace, setSavingWorkspace] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Workspace | null>(null);

  useEffect(() => { void refreshProcesses(); }, [refreshProcesses]);
  const visibleProcesses = useMemo(() => filterAndSortProcesses(processes, query, sort, descending).slice(0, 100), [processes, query, sort, descending]);
  const memoryPercent = snapshot ? percent(snapshot.memory.usedBytes, snapshot.memory.totalBytes) : 0;
  const storageTotal = snapshot?.storage.reduce((sum, disk) => sum + disk.totalBytes, 0) ?? 0;
  const storageUsed = snapshot?.storage.reduce((sum, disk) => sum + disk.totalBytes - disk.availableBytes, 0) ?? 0;
  const networkDown = snapshot?.network.reduce((sum, item) => sum + item.receivedBytes, 0) ?? 0;
  const networkUp = snapshot?.network.reduce((sum, item) => sum + item.transmittedBytes, 0) ?? 0;

  const chooseDirectory = async () => {
    const path = await selectDirectory();
    if (!path) return;
    const fallbackName = path.split(/[\\/]/).filter(Boolean).at(-1) ?? "Workspace";
    setWorkspaceDraft((current) => ({ ...(current ?? blankWorkspace()), path, name: current?.name || fallbackName }));
  };
  const submitWorkspace = async (event: FormEvent) => {
    event.preventDefault();
    if (!workspaceDraft) return;
    setSavingWorkspace(true); setWorkspaceError(null);
    try { await saveWorkspace(workspaceDraft); setWorkspaceDraft(null); }
    catch (reason) { setWorkspaceError(reason instanceof Error ? reason.message : String(reason)); }
    finally { setSavingWorkspace(false); }
  };
  const toggleWorkspace = async (workspace: Workspace) => {
    await saveWorkspace({ id: workspace.id, name: workspace.name, path: workspace.path, projectId: workspace.projectId, enabled: !workspace.enabled });
  };

  return (
    <>
      <ModuleIntro code="SYS-08" title="System Core" description="Telemetria local, processos e workspaces autorizados em modo somente leitura." metric="V0.7 // WINDOWS NATIVE" />
      {(error || snapshot?.errors.length) ? <div className="system-warning"><strong>LEITURA PARCIAL</strong>{error ?? snapshot?.errors.join(" · ")}</div> : null}
      <div className="telemetry-grid">
        <article><span>CPU</span><strong>{snapshot ? `${snapshot.cpu.usagePercent.toFixed(1)}%` : "—"}</strong><small>{snapshot?.details.logicalCores ?? 0} processadores lógicos</small></article>
        <article><span>MEMÓRIA</span><strong>{snapshot ? `${memoryPercent}%` : "—"}</strong><small>{snapshot ? `${formatBytes(snapshot.memory.usedBytes)} / ${formatBytes(snapshot.memory.totalBytes)}` : "Coletando"}</small></article>
        <article><span>ARMAZENAMENTO</span><strong>{snapshot ? `${percent(storageUsed, storageTotal)}%` : "—"}</strong><small>{formatBytes(storageTotal - storageUsed)} disponíveis</small></article>
        <article><span>REDE / AMOSTRA</span><strong>↓ {formatBytes(networkDown)}</strong><small>↑ {formatBytes(networkUp)}</small></article>
        <article><span>UPTIME</span><strong>{snapshot ? formatUptime(snapshot.details.uptimeSeconds) : "—"}</strong><small>{snapshot?.details.hostname ?? "Host indisponível"}</small></article>
      </div>

      <div className="system-dashboard">
        <HudPanel title="Ambiente local" code={loading ? "COLETANDO" : "READ ONLY"}>
          <dl className="system-facts">
            <div><dt>Sistema</dt><dd>{snapshot?.details.osName ?? "Indisponível"} {snapshot?.details.osVersion}</dd></div>
            <div><dt>Kernel</dt><dd>{snapshot?.details.kernelVersion ?? "Indisponível"}</dd></div>
            <div><dt>Arquitetura</dt><dd>{snapshot?.details.architecture ?? "—"}</dd></div>
            <div><dt>Núcleos físicos</dt><dd>{snapshot?.details.physicalCores ?? "Indisponível"}</dd></div>
            <div><dt>AI Core</dt><dd><button className="system-link" onClick={onOpenAI}>{coreState === "offline" ? "OFFLINE" : "ONLINE"} / {settings?.model ?? "Ollama"}</button></dd></div>
            <div><dt>Modelos locais</dt><dd>{ollamaStatus?.models.length ? ollamaStatus.models.join(", ") : "Nenhum detectado"}</dd></div>
          </dl>
        </HudPanel>

        <HudPanel title="Volumes" code={`${snapshot?.storage.length ?? 0} DETECTADOS`}>
          <div className="storage-list">{snapshot?.storage.length ? snapshot.storage.map((disk) => {
            const used = disk.totalBytes - disk.availableBytes;
            return <article key={`${disk.mountPoint}-${disk.name}`}><header><strong>{disk.name || disk.mountPoint}</strong><span>{disk.fileSystem || "FS"}</span></header><div><i style={{ width: `${percent(used, disk.totalBytes)}%` }} /></div><p>{formatBytes(used)} usados de {formatBytes(disk.totalBytes)} · {disk.mountPoint}</p></article>;
          }) : <p className="system-empty">Armazenamento indisponível.</p>}</div>
        </HudPanel>
      </div>

      <HudPanel title="Monitor de processos" code={`${visibleProcesses.length}/${processes.length} VISÍVEIS`} className="process-panel">
        <div className="process-toolbar">
          <input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Pesquisar por processo ou PID" aria-label="Pesquisar processos" />
          <select value={sort} onChange={(event) => setSort(event.target.value as ProcessSort)}><option value="memory">Memória</option><option value="cpu">CPU</option><option value="name">Nome</option><option value="pid">PID</option></select>
          <button onClick={() => setDescending((value) => !value)}>{descending ? "MAIOR → MENOR" : "MENOR → MAIOR"}</button>
          <button onClick={() => void refreshProcesses()} disabled={processLoading}>{processLoading ? "ATUALIZANDO..." : "ATUALIZAR"}</button>
        </div>
        <div className="process-table-wrap"><table className="process-table"><thead><tr><th>PROCESSO</th><th>PID</th><th>CPU</th><th>MEMÓRIA</th></tr></thead><tbody>{visibleProcesses.map((process) => <tr key={process.pid}><td>{process.name || "Processo sem nome"}</td><td>{process.pid}</td><td>{process.cpuPercent.toFixed(1)}%</td><td>{formatBytes(process.memoryBytes)}</td></tr>)}</tbody></table></div>
        <footer className="process-privacy">Somente nome, PID, CPU e memória são expostos. Nenhuma ação sobre processos está disponível.</footer>
      </HudPanel>

      <div className="workspace-grid">
        <HudPanel title="Workspaces autorizados" code={`${workspaces.filter((item) => item.enabled).length} ATIVOS`}>
          <div className="workspace-toolbar"><button onClick={() => { setWorkspaceDraft(blankWorkspace()); setWorkspaceError(null); }}>+ AUTORIZAR WORKSPACE</button></div>
          <div className="workspace-list">{workspaces.map((workspace) => <article key={workspace.id} data-disabled={!workspace.enabled}>
            <button className="workspace-main" onClick={() => workspace.enabled && void inspectWorkspace(workspace.id)} disabled={!workspace.enabled}><strong>{workspace.name}</strong><span>{workspace.path}</span><small>{workspace.enabled ? "ATIVO" : "DESABILITADO"}</small></button>
            <div><button onClick={() => setWorkspaceDraft({ id: workspace.id, name: workspace.name, path: workspace.path, projectId: workspace.projectId, enabled: workspace.enabled })}>EDITAR</button><button onClick={() => void toggleWorkspace(workspace)}>{workspace.enabled ? "DESABILITAR" : "HABILITAR"}</button><button className="danger-link" onClick={() => setDeleteTarget(workspace)}>REMOVER</button></div>
          </article>)}</div>
          {!workspaces.length && <p className="system-empty">Nenhuma pasta autorizada. O Azriel não inspeciona o computador inteiro.</p>}
        </HudPanel>

        <HudPanel title="Workspace / Git" code={selectedWorkspace?.git?.repository ? "REPOSITÓRIO" : "AGUARDANDO SELEÇÃO"}>
          {selectedWorkspace ? <div className="workspace-status">
            <h3>{selectedWorkspace.workspace.name}</h3><p>{selectedWorkspace.workspace.path}</p>
            <dl><div><dt>Pasta</dt><dd>{selectedWorkspace.pathAvailable ? "DISPONÍVEL" : "INDISPONÍVEL"}</dd></div><div><dt>Itens na raiz</dt><dd>{selectedWorkspace.entryCount ?? "—"}</dd></div><div><dt>Branch</dt><dd>{selectedWorkspace.git?.branch ?? "—"}</dd></div><div><dt>Estado Git</dt><dd>{selectedWorkspace.git?.repository ? selectedWorkspace.git.clean ? "LIMPO" : "ALTERAÇÕES LOCAIS" : selectedWorkspace.git?.error ?? "SEM GIT"}</dd></div></dl>
            {selectedWorkspace.git?.lastCommit && <div className="git-commit"><small>ÚLTIMO COMMIT · {selectedWorkspace.git.lastCommit.shortHash}</small><strong>{selectedWorkspace.git.lastCommit.subject}</strong><span>{selectedWorkspace.git.lastCommit.author} · {new Date(selectedWorkspace.git.lastCommit.date).toLocaleString("pt-BR")}</span></div>}
            {selectedWorkspace.git && !selectedWorkspace.git.clean && <div className="git-changes"><span>M {selectedWorkspace.git.modified.length}</span><span>A {selectedWorkspace.git.added.length}</span><span>D {selectedWorkspace.git.removed.length}</span><span>? {selectedWorkspace.git.untracked.length}</span></div>}
          </div> : <p className="system-empty">Selecione um workspace ativo para consultar sua pasta e o estado Git.</p>}
        </HudPanel>
      </div>

      {workspaceDraft && <div className="workspace-editor"><form onSubmit={submitWorkspace}><header><span>REGISTRO DE ACESSO</span><strong>WORKSPACE AUTORIZADO</strong></header><label>Nome<input value={workspaceDraft.name} maxLength={100} required onChange={(event) => setWorkspaceDraft({ ...workspaceDraft, name: event.target.value })} /></label><label>Pasta<div className="workspace-path"><input value={workspaceDraft.path} readOnly required placeholder="Selecione uma pasta" /><button type="button" onClick={() => void chooseDirectory()}>SELECIONAR</button></div></label><label>Projeto relacionado<select value={workspaceDraft.projectId ?? ""} onChange={(event) => setWorkspaceDraft({ ...workspaceDraft, projectId: event.target.value || null })}><option value="">Nenhum projeto</option>{projects.map((project) => <option value={project.id} key={project.id}>{project.name}</option>)}</select></label><label className="workspace-check"><input type="checkbox" checked={workspaceDraft.enabled} onChange={(event) => setWorkspaceDraft({ ...workspaceDraft, enabled: event.target.checked })} /> Workspace habilitado</label>{workspaceError && <p className="form-error">{workspaceError}</p>}<footer><button type="button" onClick={() => setWorkspaceDraft(null)} disabled={savingWorkspace}>CANCELAR</button><button type="submit" disabled={savingWorkspace || !workspaceDraft.path}>{savingWorkspace ? "SALVANDO..." : "SALVAR AUTORIZAÇÃO"}</button></footer></form></div>}
      {deleteTarget && <DeleteConfirmationDialog kind="workspace" title={deleteTarget.name} description="Remove apenas a autorização e os metadados locais. Nenhuma pasta ou arquivo será apagado do computador." busy={false} onCancel={() => setDeleteTarget(null)} onConfirm={() => void deleteWorkspace(deleteTarget.id).then(() => setDeleteTarget(null))} />}
    </>
  );
}
