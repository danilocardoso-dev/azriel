import { describe, expect, it, vi } from "vitest";
import { noteRepository } from "../repositories/noteRepository";
import { noteService } from "./noteService";

describe("noteService", () => {
  it("cria captura rápida como nota ativa", async () => {
    const save = vi.spyOn(noteRepository, "save").mockImplementation(async (input) => ({ ...input, createdAt: "", updatedAt: "" }));
    const created = await noteService.createQuick("  Ideia sobre MQTT  ");
    expect(created.content).toBe("Ideia sobre MQTT");
    expect(created.status).toBe("active");
    save.mockRestore();
  });

  it("rejeita nota vazia", () => {
    expect(() => noteService.createQuick(" ")).toThrow(/conteúdo/);
  });
});
