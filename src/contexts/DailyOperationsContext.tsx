import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { noteService } from "../services/noteService";
import { taskService } from "../services/taskService";
import type { DailyView, Note, Task } from "../types";
import { DailyContext, emptyDailyCounters, type DailyContextValue } from "./daily-context";

const messageOf = (error: unknown) => error instanceof Error ? error.message : String(error);

export function DailyOperationsProvider({ children }: { children: ReactNode }) {
  const [view, setView] = useState<DailyView>("today");
  const [tasks, setTasks] = useState<Task[]>([]);
  const [notes, setNotes] = useState<Note[]>([]);
  const [counters, setCounters] = useState(emptyDailyCounters);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true); setError(null);
    try {
      const counts = await taskService.counters();
      setCounters(counts);
      if (view === "notes") {
        setNotes(await noteService.list(false)); setTasks([]);
      } else {
        const loaders = { today: taskService.today, inbox: taskService.inbox, upcoming: taskService.upcoming, completed: taskService.completed };
        setTasks(await loaders[view]()); setNotes([]);
      }
    } catch (reason) { setError(messageOf(reason)); }
    finally { setLoading(false); }
  }, [view]);

  useEffect(() => {
    const initialization = window.setTimeout(() => void reload(), 0);
    return () => window.clearTimeout(initialization);
  }, [reload]);

  const refreshAfter = useCallback(async <T,>(operation: Promise<T>) => {
    try { const result = await operation; await reload(); return result; }
    catch (reason) { setError(messageOf(reason)); throw reason; }
  }, [reload]);

  const value = useMemo<DailyContextValue>(() => ({
    view, setView, tasks, notes, counters, loading, error, reload,
    createQuickTask: (title) => refreshAfter(taskService.createQuick(title)),
    createQuickNote: (content) => refreshAfter(noteService.createQuick(content)),
    saveTask: (input) => refreshAfter(taskService.save(input)),
    completeTask: (id) => refreshAfter(taskService.complete(id)),
    deleteTask: (id) => refreshAfter(taskService.remove(id)),
    saveNote: (input) => refreshAfter(noteService.save(input)),
    archiveNote: (id) => refreshAfter(noteService.archive(id)),
    deleteNote: (id) => refreshAfter(noteService.remove(id)),
  }), [view, tasks, notes, counters, loading, error, reload, refreshAfter]);

  return <DailyContext.Provider value={value}>{children}</DailyContext.Provider>;
}
