import { taskRepository } from "../repositories/taskRepository";
import type { Task, TaskInput } from "../types";
import { localDateKey } from "./dateService";

function validate(input: TaskInput) {
  if (!input.title.trim()) throw new Error("Informe o título da tarefa.");
  if (input.dueDate && !/^\d{4}-\d{2}-\d{2}$/.test(input.dueDate)) throw new Error("Prazo inválido.");
  return { ...input, title: input.title.trim(), description: input.description.trim() };
}

export function filterCounterTasks(tasks: Task[], view: "pending" | "overdue" | "priority", today = localDateKey()) {
  const active = tasks.filter((task) => !["completed", "cancelled"].includes(task.status));
  if (view === "overdue") return active.filter((task) => Boolean(task.dueDate && task.dueDate < today));
  if (view === "priority") return active.filter((task) => ["high", "critical"].includes(task.priority));
  return active;
}

export const taskService = {
  list: taskRepository.list,
  get: taskRepository.get,
  save: (input: TaskInput) => taskRepository.save(validate(input)),
  createQuick: (title: string) => taskRepository.save(validate({ id: crypto.randomUUID(), title, description: "", status: "inbox", priority: "medium", dueDate: null, projectId: null, knowledgeAreaId: null })),
  complete: taskRepository.complete,
  remove: taskRepository.remove,
  today: () => taskRepository.today(localDateKey()),
  pending: async () => filterCounterTasks(await taskRepository.list(), "pending"),
  overdue: async () => filterCounterTasks(await taskRepository.list(), "overdue"),
  priority: async () => filterCounterTasks(await taskRepository.list(), "priority"),
  upcoming: () => taskRepository.upcoming(localDateKey()),
  inbox: taskRepository.inbox,
  completed: taskRepository.completed,
  counters: () => taskRepository.counters(localDateKey()),
};
