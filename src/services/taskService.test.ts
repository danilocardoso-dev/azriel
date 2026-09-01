import { describe, expect, it, vi } from "vitest";
import { taskRepository } from "../repositories/taskRepository";
import type { Task, TaskPriority, TaskStatus } from "../types";
import { filterCounterTasks, taskService } from "./taskService";

const task = (id: string, status: TaskStatus, priority: TaskPriority, dueDate: string | null): Task => ({
  id, title: id, description: "", status, priority, dueDate, projectId: null, knowledgeAreaId: null, createdAt: "", updatedAt: "", completedAt: null,
});

describe("filterCounterTasks", () => {
  const tasks = [
    task("overdue", "pending", "high", "2026-08-31"),
    task("today", "pending", "medium", "2026-09-01"),
    task("priority", "in_progress", "critical", null),
    task("completed", "completed", "critical", "2026-08-30"),
  ];

  it("keeps only active tasks in pending", () => {
    expect(filterCounterTasks(tasks, "pending", "2026-09-01").map((item) => item.id)).toEqual(["overdue", "today", "priority"]);
  });

  it("separates overdue tasks from tasks due today", () => {
    expect(filterCounterTasks(tasks, "overdue", "2026-09-01").map((item) => item.id)).toEqual(["overdue"]);
  });

  it("keeps only active high and critical priorities", () => {
    expect(filterCounterTasks(tasks, "priority", "2026-09-01").map((item) => item.id)).toEqual(["overdue", "priority"]);
  });
});

describe("taskService", () => {
  it("cria captura rápida como item de caixa de entrada", async () => {
    const save = vi.spyOn(taskRepository, "save").mockImplementation(async (input) => ({ ...input, createdAt: "", updatedAt: "", completedAt: null }));
    const created = await taskService.createQuick("  Revisar PCR  ");
    expect(created.title).toBe("Revisar PCR");
    expect(created.status).toBe("inbox");
    save.mockRestore();
  });

  it("rejeita tarefa sem título", () => {
    expect(() => taskService.createQuick("   ")).toThrow(/título/);
  });
});
