import type { DailyCounters, Task, TaskInput } from "../types";
import { invokeDatabase } from "./tauri";

export const taskRepository = {
  list: () => invokeDatabase<Task[]>("list_tasks"),
  get: (id: string) => invokeDatabase<Task | null>("get_task", { id }),
  save: (input: TaskInput) => invokeDatabase<Task>("save_task", { input }),
  complete: (id: string) => invokeDatabase<Task>("complete_task", { id }),
  remove: (id: string) => invokeDatabase<void>("delete_task", { id }),
  today: (today: string) => invokeDatabase<Task[]>("list_today_tasks", { today }),
  upcoming: (today: string) => invokeDatabase<Task[]>("list_upcoming_tasks", { today }),
  inbox: () => invokeDatabase<Task[]>("list_inbox_tasks"),
  completed: () => invokeDatabase<Task[]>("list_completed_tasks"),
  counters: (today: string) => invokeDatabase<DailyCounters>("daily_counters", { today }),
};
