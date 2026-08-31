import { taskRepository } from "../repositories/taskRepository";
import type { TaskInput } from "../types";
import { localDateKey } from "./dateService";

function validate(input: TaskInput) {
  if (!input.title.trim()) throw new Error("Informe o título da tarefa.");
  if (input.dueDate && !/^\d{4}-\d{2}-\d{2}$/.test(input.dueDate)) throw new Error("Prazo inválido.");
  return { ...input, title: input.title.trim(), description: input.description.trim() };
}

export const taskService = {
  list: taskRepository.list,
  get: taskRepository.get,
  save: (input: TaskInput) => taskRepository.save(validate(input)),
  createQuick: (title: string) => taskRepository.save(validate({ id: crypto.randomUUID(), title, description: "", status: "inbox", priority: "medium", dueDate: null, projectId: null, knowledgeAreaId: null })),
  complete: taskRepository.complete,
  remove: taskRepository.remove,
  today: () => taskRepository.today(localDateKey()),
  upcoming: () => taskRepository.upcoming(localDateKey()),
  inbox: taskRepository.inbox,
  completed: taskRepository.completed,
  counters: () => taskRepository.counters(localDateKey()),
};
