import { describe, expect, it } from "vitest";
import { groupUpcoming, localDateKey, parseLocalDate } from "./dateService";
import type { Task } from "../types";

const task = (id: string, dueDate: string): Task => ({ id, dueDate, title: id, description: "", status: "pending", priority: "medium", projectId: null, knowledgeAreaId: null, createdAt: "", updatedAt: "", completedAt: null });

describe("dateService", () => {
  it("gera a chave usando a data local, sem conversão UTC", () => {
    expect(localDateKey(new Date(2026, 7, 31, 23, 59))).toBe("2026-08-31");
    expect(parseLocalDate("2026-08-31").getDate()).toBe(31);
  });

  it("agrupa amanhã, restante da semana e datas posteriores", () => {
    const groups = groupUpcoming([task("tomorrow", "2026-09-01"), task("week", "2026-09-04"), task("later", "2026-09-10")], new Date(2026, 7, 31));
    expect(groups.tomorrow.map(({ id }) => id)).toEqual(["tomorrow"]);
    expect(groups.thisWeek.map(({ id }) => id)).toEqual(["week"]);
    expect(groups.later.map(({ id }) => id)).toEqual(["later"]);
  });

  it("não duplica amanhã quando hoje é domingo", () => {
    const groups = groupUpcoming([task("monday", "2026-09-07")], new Date(2026, 8, 6));
    expect(groups.tomorrow).toHaveLength(1);
    expect(groups.later).toHaveLength(0);
  });
});
