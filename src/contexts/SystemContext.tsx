import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { systemService } from "../services/systemService";
import type { ProcessSnapshot, SystemSnapshot, Workspace, WorkspaceStatus } from "../types";
import { SystemContext, type SystemContextValue } from "./system-context";
import { DATA_RELATIONS_CHANGED } from "./dataEvents";

const messageOf = (error: unknown) => error instanceof Error ? error.message : String(error);

export function SystemProvider({ children }: { children: ReactNode }) {
  const [snapshot, setSnapshot] = useState<SystemSnapshot | null>(null);
  const [processes, setProcesses] = useState<ProcessSnapshot[]>([]);
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [selectedWorkspace, setSelectedWorkspace] = useState<WorkspaceStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [processLoading, setProcessLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refreshSnapshot = useCallback(async () => {
    try { setSnapshot(await systemService.snapshot()); setError(null); }
    catch (reason) { setError(messageOf(reason)); }
  }, []);
  const refreshProcesses = useCallback(async () => {
    setProcessLoading(true);
    try { setProcesses(await systemService.processes()); setError(null); }
    catch (reason) { setError(messageOf(reason)); }
    finally { setProcessLoading(false); }
  }, []);
  const refreshWorkspaces = useCallback(async () => {
    try { setWorkspaces(await systemService.listWorkspaces()); setError(null); }
    catch (reason) { setError(messageOf(reason)); }
  }, []);
  const inspectWorkspace = useCallback(async (id: string) => {
    try { setSelectedWorkspace(await systemService.workspaceStatus(id)); setError(null); }
    catch (reason) { setSelectedWorkspace(null); setError(messageOf(reason)); }
  }, []);

  useEffect(() => {
    let active = true;
    const initialize = async () => {
      setLoading(true);
      await Promise.all([refreshSnapshot(), refreshWorkspaces()]);
      if (active) setLoading(false);
    };
    void initialize();
    const timer = window.setInterval(() => void refreshSnapshot(), 3000);
    return () => { active = false; window.clearInterval(timer); };
  }, [refreshSnapshot, refreshWorkspaces]);

  useEffect(() => {
    const refreshRelations = () => { setSelectedWorkspace(null); void refreshWorkspaces(); };
    window.addEventListener(DATA_RELATIONS_CHANGED, refreshRelations);
    return () => window.removeEventListener(DATA_RELATIONS_CHANGED, refreshRelations);
  }, [refreshWorkspaces]);

  const value = useMemo<SystemContextValue>(() => ({
    snapshot, processes, workspaces, selectedWorkspace, loading, processLoading, error,
    refreshSnapshot, refreshProcesses, refreshWorkspaces, inspectWorkspace,
    saveWorkspace: async (input) => { setWorkspaces(await systemService.saveWorkspace(input)); setSelectedWorkspace(null); },
    deleteWorkspace: async (id) => { setWorkspaces(await systemService.deleteWorkspace(id)); setSelectedWorkspace(null); },
    selectDirectory: systemService.selectDirectory,
  }), [snapshot, processes, workspaces, selectedWorkspace, loading, processLoading, error, refreshSnapshot, refreshProcesses, refreshWorkspaces, inspectWorkspace]);
  return <SystemContext.Provider value={value}>{children}</SystemContext.Provider>;
}
