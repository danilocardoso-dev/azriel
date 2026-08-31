import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { automationService, ROUTINE_RESULT_EVENT } from "../services/automationService";
import type { ActionHistory, ActionRequest, ActionResult, Application, AutomationState, RegisteredAction, RegisteredUrl, Routine, RoutineExecutionResult, RoutineHistory, RunRoutineRequest } from "../types";
import { AutomationContext, type AutomationContextValue } from "./automation-context";

const messageOf = (error: unknown) => error instanceof Error ? error.message : String(error);

export function AutomationProvider({ children }: { children: ReactNode }) {
  const [actions, setActions] = useState<RegisteredAction[]>([]);
  const [applications, setApplications] = useState<Application[]>([]);
  const [urls, setUrls] = useState<RegisteredUrl[]>([]);
  const [history, setHistory] = useState<ActionHistory[]>([]);
  const [routines, setRoutines] = useState<Routine[]>([]);
  const [routineHistory, setRoutineHistory] = useState<RoutineHistory[]>([]);
  const [pendingRoutine, setPendingRoutine] = useState<RoutineExecutionResult | null>(null);
  const [state, setState] = useState<AutomationState>("offline");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastResult, setLastResult] = useState<ActionResult | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [nextActions, nextApplications, nextUrls, nextHistory, nextRoutines, nextRoutineHistory] = await Promise.all([
        automationService.listActions(), automationService.listApplications(), automationService.listUrls(), automationService.listHistory(), automationService.listRoutines(), automationService.listRoutineHistory(),
      ]);
      setActions(nextActions); setApplications(nextApplications); setUrls(nextUrls); setHistory(nextHistory); setRoutines(nextRoutines); setRoutineHistory(nextRoutineHistory);
      setError(null); setState("safe");
    } catch (reason) { setError(messageOf(reason)); setState("error"); }
    finally { setLoading(false); }
  }, []);

  useEffect(() => {
    const task = window.setTimeout(() => void refresh(), 0);
    return () => window.clearTimeout(task);
  }, [refresh]);

  const applyRoutineResult = useCallback((result: RoutineExecutionResult) => {
    setPendingRoutine(result.confirmation ? result : null);
    setState(result.status === "waiting_confirmation" ? "waiting_confirmation" : result.success ? "completed" : "failed");
    window.setTimeout(() => setState((current) => current === "waiting_confirmation" ? current : "safe"), 1800);
  }, []);

  useEffect(() => {
    const receive = (event: Event) => applyRoutineResult((event as CustomEvent<RoutineExecutionResult>).detail);
    window.addEventListener(ROUTINE_RESULT_EVENT, receive);
    return () => window.removeEventListener(ROUTINE_RESULT_EVENT, receive);
  }, [applyRoutineResult]);

  const execute = useCallback(async (request: ActionRequest) => {
    setState("executing"); setError(null);
    try {
      const result = await automationService.execute(request);
      setLastResult(result);
      setHistory(await automationService.listHistory());
      setState(result.confirmation ? "waiting_confirmation" : result.success ? "safe" : result.errorCode?.includes("BLOCK") ? "blocked" : "error");
      window.setTimeout(() => setState("safe"), 1600);
      return result;
    } catch (reason) {
      const message = messageOf(reason); setError(message); setState("error"); throw reason;
    }
  }, []);

  const runRoutine = useCallback(async (request: RunRoutineRequest) => {
    setState("validating"); setError(null);
    try {
      const result = await automationService.runRoutine(request);
      setHistory(await automationService.listHistory());
      setRoutineHistory(await automationService.listRoutineHistory());
      return result;
    } catch (reason) {
      const message = messageOf(reason); setError(message); setState("failed"); throw reason;
    }
  }, []);

  const confirmRoutine = useCallback(async () => {
    if (!pendingRoutine) throw new Error("Nenhuma rotina aguarda confirmação");
    setState("executing"); setError(null);
    try {
      const result = await automationService.confirmRoutine(pendingRoutine.historyId);
      setHistory(await automationService.listHistory());
      setRoutineHistory(await automationService.listRoutineHistory());
      return result;
    } catch (reason) {
      const message = messageOf(reason); setError(message); setState("failed"); setPendingRoutine(null); throw reason;
    }
  }, [pendingRoutine]);

  const cancelRoutine = useCallback(async () => {
    if (!pendingRoutine) return;
    await automationService.cancelRoutine(pendingRoutine.historyId);
    setPendingRoutine(null); setState("cancelled");
    setRoutineHistory(await automationService.listRoutineHistory());
    window.setTimeout(() => setState("safe"), 1400);
  }, [pendingRoutine]);

  const value = useMemo<AutomationContextValue>(() => ({
    actions, applications, urls, history, routines, routineHistory, pendingRoutine, state, loading, error, lastResult, refresh, execute, runRoutine, confirmRoutine, cancelRoutine,
    saveApplication: async (input) => { setApplications(await automationService.saveApplication(input)); setHistory(await automationService.listHistory()); },
    deleteApplication: async (id) => { setApplications(await automationService.deleteApplication(id)); },
    saveUrl: async (input) => { setUrls(await automationService.saveUrl(input)); },
    deleteUrl: async (id) => { setUrls(await automationService.deleteUrl(id)); },
    saveRoutine: async (input) => { setRoutines(await automationService.saveRoutine(input)); },
    deleteRoutine: async (id) => { setRoutines(await automationService.deleteRoutine(id)); },
    selectApplication: automationService.selectApplication,
  }), [actions, applications, urls, history, routines, routineHistory, pendingRoutine, state, loading, error, lastResult, refresh, execute, runRoutine, confirmRoutine, cancelRoutine]);
  return <AutomationContext.Provider value={value}>{children}</AutomationContext.Provider>;
}
