import { beforeEach, describe, expect, it, vi } from "vitest";
import type { MoneyItem, MoneyMonth } from "../../repositories/moneyRepository";
import type { Note, Task, TaskInput } from "../../types";
import { actionTools, type ActionDependencies } from "./actionTools";

const task = (input: Partial<Task> & { id: string; title: string }): Task => ({ description: "", status: "pending", priority: "medium", dueDate: null, createdAt: "", updatedAt: "", completedAt: null, ...input });
const moneyItem: MoneyItem = { id: "m1", kind: "expense", title: "Internet", amountCents: 9990, firstDate: "2026-11-10", recurrence: "monthly", installments: null, project: null };
const month: MoneyMonth = { month: "2026-11", expenses: [{ item: moneyItem, date: "2026-11-10", label: "permanente", paid: false }], income: [], totals: { expensesCents: 9990, expensesPaidCents: 0, incomeCents: 0, incomeReceivedCents: 0, balanceCents: -9990 }, projects: [] };

let tasks: Map<string, Task>;
let notes: Map<string, Note>;
let deps: ActionDependencies;
const removedMoney: string[] = [];

beforeEach(() => {
  tasks = new Map([["t1", task({ id: "t1", title: "Pagar internet" })]]);
  notes = new Map([["n1", { id: "n1", title: "Ideias", content: "Azriel com voz", status: "active", createdAt: "", updatedAt: "" }]]);
  removedMoney.length = 0;
  deps = {
    tasks: {
      get: async (id) => tasks.get(id) ?? null,
      save: async (input: TaskInput) => { const saved = task(input); tasks.set(input.id, saved); return saved; },
      complete: async (id) => { const done = { ...tasks.get(id)!, status: "completed" as const }; tasks.set(id, done); return done; },
      remove: async (id) => { tasks.delete(id); },
    },
    notes: {
      get: async (id) => notes.get(id) ?? null,
      save: async (input) => { const saved = { ...input, createdAt: "", updatedAt: "" }; notes.set(input.id, saved); return saved; },
      archive: async (id) => { const archived = { ...notes.get(id)!, status: "archived" as const }; notes.set(id, archived); return archived; },
      remove: async (id) => { notes.delete(id); },
    },
    money: { month: async () => month, save: async (input) => input, setPaid: vi.fn(async () => undefined), remove: async (id) => { removedMoney.push(id); } },
    music: {
      search: async () => ({ tracks: [{ uri: "spotify:track:1", name: "Blinding Lights", subtitle: "The Weeknd", image: null, albumUri: "spotify:album:9" }], playlists: [], albums: [], artists: [], hasMore: false }),
      play: vi.fn(async () => undefined),
      player: async () => null,
      control: vi.fn(async () => undefined),
      device: () => "azriel-device",
    },
    changed: vi.fn(),
    newId: () => "novo",
    today: () => "2026-11-15",
  };
});

const run = (name: string, args: Record<string, unknown>, confirm?: () => Promise<boolean>) => {
  const tool = actionTools(deps).find((item) => item.name === name)!;
  return tool.execute({ query: "", args, confirm }) as Promise<Record<string, unknown>>;
};

describe("ações do Azriel: cria e altera sozinho", () => {
  it("cria tarefa com prioridade e prazo e avisa as telas", async () => {
    const result = await run("create_task", { title: "Revisar contrato", priority: "high", dueDate: "2026-11-20" });
    expect(result.criada).toBe(true);
    expect(tasks.get("novo")).toMatchObject({ title: "Revisar contrato", priority: "high", dueDate: "2026-11-20", status: "pending" });
    expect(deps.changed).toHaveBeenCalled();
    await expect(run("create_task", { title: "x", dueDate: "amanhã" })).rejects.toThrow("Prazo inválido");
  });

  it("conclui e altera tarefa sem pedir permissão", async () => {
    await run("update_task", { id: "t1", status: "completed", priority: "critical" });
    expect(tasks.get("t1")).toMatchObject({ status: "completed", priority: "critical", title: "Pagar internet" });
    expect((await run("update_task", { id: "nao-existe" })).error).toMatch(/não encontrada/);
  });

  it("arquiva nota sem pedir permissão", async () => {
    await run("archive_note", { id: "n1" });
    expect(notes.get("n1")?.status).toBe("archived");
  });

  it("cria lançamento em reais e marca como pago", async () => {
    const saved = await run("save_money_item", { kind: "income", title: "Freela", amount: 1500.5, firstDate: "2026-11-30" });
    expect(saved.lancamento).toMatchObject({ titulo: "Freela", valor: 1500.5, tipo: "income" });
    await run("set_money_paid", { id: "m1", month: "2026-11", paid: true });
    expect(deps.money.setPaid).toHaveBeenCalledWith("m1", "2026-11", true);
  });

  it("lê as finanças do mês com totais em reais", async () => {
    const view = await run("get_finances", {});
    expect(view.totais).toMatchObject({ despesas: 99.9, saldo: -99.9 });
  });

  it("toca música no player do Azriel", async () => {
    const result = await run("play_music", { query: "blinding lights" });
    expect(result).toMatchObject({ tocando: true, nome: "Blinding Lights" });
    expect(deps.music.play).toHaveBeenCalledWith("spotify:track:1", "azriel-device", "spotify:album:9");
  });
});

describe("ações do Azriel: excluir só com permissão", () => {
  it("apaga a tarefa quando o Senhor permite, com o título no pedido", async () => {
    const confirm = vi.fn(async () => true);
    const result = await run("delete_task", { id: "t1" }, confirm);
    expect(confirm).toHaveBeenCalledWith({ title: 'Apagar a tarefa "Pagar internet"', detail: expect.any(String) });
    expect(result.apagada).toBe(true);
    expect(tasks.has("t1")).toBe(false);
  });

  it("não apaga quando o Senhor nega", async () => {
    const result = await run("delete_task", { id: "t1" }, async () => false);
    expect(result.negado).toBe(true);
    expect(tasks.has("t1")).toBe(true);
  });

  it("sem tela para perguntar (voz), nunca apaga", async () => {
    expect((await run("delete_note", { id: "n1" })).error).toMatch(/Nada foi apagado/);
    expect(notes.has("n1")).toBe(true);
    expect((await run("delete_money_item", { id: "m1", month: "2026-11" })).error).toMatch(/Nada foi apagado/);
    expect(removedMoney).toEqual([]);
  });

  it("lançamento: pede com nome e valor, e só então apaga", async () => {
    const confirm = vi.fn(async (request: { title: string; detail: string }) => Boolean(request));
    await run("delete_money_item", { id: "m1", month: "2026-11" }, confirm as unknown as () => Promise<boolean>);
    expect(confirm.mock.calls[0]?.[0]?.title).toBe('Apagar o lançamento "Internet" (R$ 99,90)');
    expect(removedMoney).toEqual(["m1"]);
  });
});
