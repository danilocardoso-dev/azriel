import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { automationService } from "../services/automationService";
import type { ActionHistory, ActionRequest, ActionResult, Application, AutomationState, RegisteredAction, RegisteredUrl } from "../types";
import { AutomationContext, type AutomationContextValue } from "./automation-context";

const messageOf = (error: unknown) => error instanceof Error ? error.message : String(error);

export function AutomationProvider({ children }: { children: ReactNode }) {
  const [actions, setActions] = useState<RegisteredAction[]>([]);
  const [applications, setApplications] = useState<Application[]>([]);
  const [urls, setUrls] = useState<RegisteredUrl[]>([]);
  const [history, setHistory] = useState<ActionHistory[]>([]);
  const [state, setState] = useState<AutomationState>("offline");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastResult, setLastResult] = useState<ActionResult | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [nextActions, nextApplications, nextUrls, nextHistory] = await Promise.all([
        automationService.listActions(), automationService.listApplications(), automationService.listUrls(), automationService.listHistory(),
      ]);
      setActions(nextActions); setApplications(nextApplications); setUrls(nextUrls); setHistory(nextHistory);
      setError(null); setState("safe");
    } catch (reason) { setError(messageOf(reason)); setState("error"); }
    finally { setLoading(false); }
  }, []);

  useEffect(() => {
    const task = window.setTimeout(() => void refresh(), 0);
    return () => window.clearTimeout(task);
  }, [refresh]);

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

  const value = useMemo<AutomationContextValue>(() => ({
    actions, applications, urls, history, state, loading, error, lastResult, refresh, execute,
    saveApplication: async (input) => { setApplications(await automationService.saveApplication(input)); setHistory(await automationService.listHistory()); },
    deleteApplication: async (id) => { setApplications(await automationService.deleteApplication(id)); },
    saveUrl: async (input) => { setUrls(await automationService.saveUrl(input)); },
    deleteUrl: async (id) => { setUrls(await automationService.deleteUrl(id)); },
    selectApplication: automationService.selectApplication,
  }), [actions, applications, urls, history, state, loading, error, lastResult, refresh, execute]);
  return <AutomationContext.Provider value={value}>{children}</AutomationContext.Provider>;
}
