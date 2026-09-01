import { createContext } from "react";
import type { DailyCounters, DailyView, Note, NoteInput, Task, TaskInput } from "../types";

export interface DailyContextValue {
  view: DailyView; setView: (view: DailyView) => void;
  tasks: Task[]; notes: Note[]; counters: DailyCounters;
  loading: boolean; error: string | null; reload: () => Promise<void>;
  createQuickTask: (title: string) => Promise<Task>;
  createQuickNote: (content: string) => Promise<Note>;
  saveTask: (input: TaskInput) => Promise<Task>;
  completeTask: (id: string) => Promise<Task>;
  deleteTask: (id: string) => Promise<void>;
  saveNote: (input: NoteInput) => Promise<Note>;
  archiveNote: (id: string) => Promise<Note>;
  restoreNote: (id: string) => Promise<Note>;
  deleteNote: (id: string) => Promise<void>;
}

export const emptyDailyCounters: DailyCounters = { pending: 0, today: 0, overdue: 0, priority: 0, notes: 0, completed: 0 };
export const DailyContext = createContext<DailyContextValue | null>(null);
