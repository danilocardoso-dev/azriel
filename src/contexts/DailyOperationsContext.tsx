import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { noteService } from "../services/noteService";
import { taskService } from "../services/taskService";
import type { DailyView, Note, Task } from "../types";
import { DailyContext, emptyDailyCounters, type DailyContextValue } from "./daily-context";
import { DATA_RELATIONS_CHANGED } from "./dataEvents";

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
      if (view === "notes" || view === "archived_notes") {
        const listed = await noteService.list(view === "archived_notes");
        setNotes(view === "archived_notes" ? listed.filter((note) => note.status === "archived") : listed); setTasks([]);
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

  useEffect(() => {
    const refreshRelations = () => void reload();
    window.addEventListener(DATA_RELATIONS_CHANGED, refreshRelations);
    return () => window.removeEventListener(DATA_RELATIONS_CHANGED, refreshRelations);
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
    restoreNote: (id) => refreshAfter(noteService.restore(id)),
    deleteNote: (id) => refreshAfter(noteService.remove(id)),
  }), [view, tasks, notes, counters, loading, error, reload, refreshAfter]);

  return <DailyContext.Provider value={value}>{children}</DailyContext.Provider>;
}
