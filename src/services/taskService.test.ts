import { describe, expect, it, vi } from "vitest";
import { taskRepository } from "../repositories/taskRepository";
import { taskService } from "./taskService";

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
