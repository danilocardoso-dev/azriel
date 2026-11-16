import type { MoneyItem, MoneyItemInput, MoneyMonth, MoneyRecurrence } from "../../repositories/moneyRepository";
import type { SearchKind, SpotifyAction, SpotifyPlayer, SpotifySearch } from "../../repositories/spotifyRepository";
import type { AIToolInput, Note, NoteInput, Task, TaskInput, TaskPriority, TaskStatus } from "../../types";
import type { AzrielTool } from "./toolRegistry";

/**
 * Ferramentas de ação da Fase 2. Regra do Senhor: o Azriel cria, altera, conclui e organiza sozinho;
 * EXCLUIR qualquer coisa só com a permissão dele (cartão Permitir/Negar no AI Core).
 */
export interface ActionDependencies {
  tasks: { get(id: string): Promise<Task | null>; save(input: TaskInput): Promise<Task>; complete(id: string): Promise<Task>; remove(id: string): Promise<void> };
  notes: { get(id: string): Promise<Note | null>; save(input: NoteInput): Promise<Note>; archive(id: string): Promise<Note>; remove(id: string): Promise<void> };
  money: { month(month: string): Promise<MoneyMonth>; save(input: MoneyItemInput): Promise<MoneyItem>; setPaid(id: string, month: string, paid: boolean): Promise<void>; remove(id: string): Promise<void> };
  music: {
    search(query: string, kind: SearchKind): Promise<SpotifySearch>;
    play(uri: string, deviceId: string | null, context?: string | null): Promise<void>;
    player(): Promise<SpotifyPlayer | null>;
    control(action: SpotifyAction, context: { active: boolean; activeDevice?: string | null; deviceId: string | null }): Promise<void>;
    device(): string | null;
  };
  /** Avisa as telas abertas (Operações) que os dados mudaram. */
  changed(): void;
  newId(): string;
  today(): string;
}

const PRIORITIES: TaskPriority[] = ["low", "medium", "high", "critical"];
const STATUSES: TaskStatus[] = ["inbox", "pending", "in_progress", "completed", "cancelled"];
const RECURRENCES: MoneyRecurrence[] = ["once", "installments", "monthly"];
const str = (value: unknown) => typeof value === "string" ? value.trim() : "";
const has = (args: Record<string, unknown> | undefined, key: string) => Boolean(args && Object.prototype.hasOwnProperty.call(args, key));
const isDate = (value: string) => /^\d{4}-\d{2}-\d{2}$/.test(value);
const isMonth = (value: string) => /^\d{4}-\d{2}$/.test(value);
const reais = (cents: number) => Math.round(cents) / 100;
const fail = (error: string) => ({ error });

/** Prazo: "" ou null limpa; senão precisa ser AAAA-MM-DD. */
function dueDate(value: unknown): string | null | undefined {
  if (value === null || value === "") return null;
  const text = str(value);
  if (!text) return undefined;
  if (!isDate(text)) throw new Error("Prazo inválido: use AAAA-MM-DD.");
  return text;
}

/** O portão das exclusões: sem tela para perguntar (voz), não apaga; negado, não apaga. */
async function permitted(input: AIToolInput, title: string, detail: string): Promise<true | { negado: true; mensagem: string } | { error: string }> {
  if (!input.confirm) return fail("Excluir só pela tela do AI Core, com a permissão do Senhor. Nada foi apagado.");
  const allowed = await input.confirm({ title, detail });
  return allowed ? true : { negado: true, mensagem: "O Senhor negou a exclusão. Nada foi apagado." };
}

const taskView = (task: Task) => ({ id: task.id, titulo: task.title, status: task.status, prioridade: task.priority, prazo: task.dueDate });
const noteView = (note: Note) => ({ id: note.id, titulo: note.title, conteudo: note.content.slice(0, 300), status: note.status });

function moneyView(month: MoneyMonth) {
  const entry = (kind: string) => (item: MoneyMonth["expenses"][number]) => ({ id: item.item.id, tipo: kind, titulo: item.item.title, valor: reais(item.item.amountCents), data: item.date, parcela: item.label || null, pago: item.paid, projeto: item.item.project, repeticao: item.item.recurrence });
  return {
    mes: month.month,
    totais: { despesas: reais(month.totals.expensesCents), despesasPagas: reais(month.totals.expensesPaidCents), receitas: reais(month.totals.incomeCents), receitasRecebidas: reais(month.totals.incomeReceivedCents), saldo: reais(month.totals.balanceCents) },
    despesas: month.expenses.map(entry("despesa")),
    receitas: month.income.map(entry("receita")),
  };
}

async function findMoneyItem(deps: ActionDependencies, id: string, month: string): Promise<MoneyItem | null> {
  if (!isMonth(month)) return null;
  const view = await deps.money.month(month);
  return [...view.expenses, ...view.income].find((entry) => entry.item.id === id)?.item ?? null;
}

export function actionTools(deps: ActionDependencies): AzrielTool[] {
  const write = { readonly: false, autonomous: true, permission: "safe_write" as const };
  const destroy = { readonly: false, autonomous: true, permission: "confirm_write" as const };
  return [
    // ---------- tarefas ----------
    { name: "create_task", domain: "Tarefas", ...write,
      description: "Cria uma tarefa. Use prioridade e prazo quando o Senhor indicar",
      parameters: { type: "object", properties: { title: { type: "string" }, description: { type: "string" }, priority: { type: "string", enum: PRIORITIES }, dueDate: { type: "string", description: "Prazo AAAA-MM-DD" } }, required: ["title"] },
      execute: async (input) => {
        const title = str(input.args?.title);
        if (!title) return fail("Informe o título da tarefa.");
        const priority = PRIORITIES.includes(input.args?.priority as TaskPriority) ? input.args?.priority as TaskPriority : "medium";
        const task = await deps.tasks.save({ id: deps.newId(), title, description: str(input.args?.description), status: "pending", priority, dueDate: dueDate(input.args?.dueDate) ?? null });
        deps.changed();
        return { criada: true, tarefa: taskView(task) };
      } },
    { name: "update_task", domain: "Tarefas", ...write,
      description: "Altera uma tarefa existente pelo id: título, descrição, prioridade, prazo ou status. Para concluir, use status completed",
      parameters: { type: "object", properties: { id: { type: "string" }, title: { type: "string" }, description: { type: "string" }, priority: { type: "string", enum: PRIORITIES }, dueDate: { type: "string", description: "AAAA-MM-DD, ou vazio para tirar o prazo" }, status: { type: "string", enum: STATUSES } }, required: ["id"] },
      execute: async (input) => {
        const current = await deps.tasks.get(str(input.args?.id));
        if (!current) return fail("Tarefa não encontrada. Liste as tarefas para obter o id.");
        const args = input.args ?? {};
        const status = STATUSES.includes(args.status as TaskStatus) ? args.status as TaskStatus : current.status;
        const next: TaskInput = {
          id: current.id,
          title: str(args.title) || current.title,
          description: has(args, "description") ? str(args.description) : current.description,
          priority: PRIORITIES.includes(args.priority as TaskPriority) ? args.priority as TaskPriority : current.priority,
          dueDate: has(args, "dueDate") ? dueDate(args.dueDate) ?? null : current.dueDate,
          status: status === "completed" ? (current.status === "completed" ? "completed" : current.status) : status,
        };
        let task = await deps.tasks.save(next);
        if (status === "completed" && current.status !== "completed") task = await deps.tasks.complete(current.id);
        deps.changed();
        return { alterada: true, tarefa: taskView(task) };
      } },
    { name: "delete_task", domain: "Tarefas", ...destroy,
      description: "Apaga uma tarefa pelo id. Exige a permissão do Senhor: o sistema pergunta a ele",
      parameters: { type: "object", properties: { id: { type: "string" } }, required: ["id"] },
      execute: async (input) => {
        const task = await deps.tasks.get(str(input.args?.id));
        if (!task) return fail("Tarefa não encontrada.");
        const gate = await permitted(input, `Apagar a tarefa "${task.title}"`, "A tarefa será removida definitivamente.");
        if (gate !== true) return gate;
        await deps.tasks.remove(task.id);
        deps.changed();
        return { apagada: true, titulo: task.title };
      } },
    // ---------- notas ----------
    { name: "create_note", domain: "Notas", ...write,
      description: "Cria uma nota com conteúdo e título opcional",
      parameters: { type: "object", properties: { content: { type: "string" }, title: { type: "string" } }, required: ["content"] },
      execute: async (input) => {
        const content = str(input.args?.content);
        if (!content) return fail("Informe o conteúdo da nota.");
        const note = await deps.notes.save({ id: deps.newId(), title: str(input.args?.title) || null, content, status: "active" });
        deps.changed();
        return { criada: true, nota: noteView(note) };
      } },
    { name: "update_note", domain: "Notas", ...write,
      description: "Altera o título ou o conteúdo de uma nota pelo id. O conteúdo enviado substitui o anterior por completo",
      parameters: { type: "object", properties: { id: { type: "string" }, title: { type: "string" }, content: { type: "string" } }, required: ["id"] },
      execute: async (input) => {
        const current = await deps.notes.get(str(input.args?.id));
        if (!current) return fail("Nota não encontrada. Liste as notas para obter o id.");
        const note = await deps.notes.save({ id: current.id, title: has(input.args, "title") ? str(input.args?.title) || null : current.title, content: str(input.args?.content) || current.content, status: current.status });
        deps.changed();
        return { alterada: true, nota: noteView(note) };
      } },
    { name: "archive_note", domain: "Notas", ...write,
      description: "Arquiva uma nota pelo id (pode ser restaurada depois)",
      parameters: { type: "object", properties: { id: { type: "string" } }, required: ["id"] },
      execute: async (input) => {
        const current = await deps.notes.get(str(input.args?.id));
        if (!current) return fail("Nota não encontrada.");
        const note = await deps.notes.archive(current.id);
        deps.changed();
        return { arquivada: true, nota: noteView(note) };
      } },
    { name: "delete_note", domain: "Notas", ...destroy,
      description: "Apaga uma nota pelo id. Exige a permissão do Senhor: o sistema pergunta a ele",
      parameters: { type: "object", properties: { id: { type: "string" } }, required: ["id"] },
      execute: async (input) => {
        const note = await deps.notes.get(str(input.args?.id));
        if (!note) return fail("Nota não encontrada.");
        const gate = await permitted(input, `Apagar a nota "${note.title || note.content.slice(0, 60)}"`, "A nota será removida definitivamente.");
        if (gate !== true) return gate;
        await deps.notes.remove(note.id);
        deps.changed();
        return { apagada: true };
      } },
    // ---------- finanças ----------
    { name: "get_finances", domain: "Finanças", readonly: true,
      description: "Contas do mês: despesas, receitas, o que foi pago e o saldo, com os ids dos lançamentos",
      parameters: { type: "object", properties: { month: { type: "string", description: "Mês AAAA-MM; padrão: mês atual" } } },
      execute: async (input) => {
        const month = str(input.args?.month) || deps.today().slice(0, 7);
        if (!isMonth(month)) return fail("Mês inválido: use AAAA-MM.");
        return moneyView(await deps.money.month(month));
      } },
    { name: "save_money_item", domain: "Finanças", ...write,
      description: "Cria um lançamento (despesa ou receita) ou altera um existente. Para alterar, informe id e o mês em que ele aparece. Valor em reais",
      parameters: { type: "object", properties: {
        id: { type: "string", description: "Só para alterar" }, month: { type: "string", description: "AAAA-MM onde o lançamento aparece (obrigatório ao alterar)" },
        kind: { type: "string", enum: ["expense", "income"] }, title: { type: "string" }, amount: { type: "number", description: "Valor em reais" },
        firstDate: { type: "string", description: "Data AAAA-MM-DD (da primeira parcela, se parcelado)" }, recurrence: { type: "string", enum: RECURRENCES },
        installments: { type: "number", description: "Número de parcelas, se parcelado" }, project: { type: "string" },
      } },
      execute: async (input) => {
        const args = input.args ?? {};
        const id = str(args.id);
        const current = id ? await findMoneyItem(deps, id, str(args.month)) : null;
        if (id && !current) return fail("Lançamento não encontrado. Consulte get_finances do mês em que ele aparece.");
        const amount = typeof args.amount === "number" ? args.amount : Number(args.amount);
        const recurrence = RECURRENCES.includes(args.recurrence as MoneyRecurrence) ? args.recurrence as MoneyRecurrence : current?.recurrence ?? "once";
        const item: MoneyItemInput = {
          id: current?.id ?? deps.newId(),
          kind: args.kind === "income" || args.kind === "expense" ? args.kind : current?.kind ?? "expense",
          title: str(args.title) || current?.title || "",
          amountCents: Number.isFinite(amount) && amount > 0 ? Math.round(amount * 100) : current?.amountCents ?? 0,
          firstDate: isDate(str(args.firstDate)) ? str(args.firstDate) : current?.firstDate ?? deps.today(),
          recurrence,
          installments: recurrence === "installments" ? Math.round(Number(args.installments ?? current?.installments ?? 0)) : null,
          project: has(args, "project") ? str(args.project) || null : current?.project ?? null,
        };
        const saved = await deps.money.save(item);
        return { salvo: true, lancamento: { id: saved.id, titulo: saved.title, valor: reais(saved.amountCents), tipo: saved.kind, data: saved.firstDate, repeticao: saved.recurrence } };
      } },
    { name: "set_money_paid", domain: "Finanças", ...write,
      description: "Marca um lançamento como pago/recebido (ou desmarca) num mês",
      parameters: { type: "object", properties: { id: { type: "string" }, month: { type: "string", description: "AAAA-MM" }, paid: { type: "boolean" } }, required: ["id", "month", "paid"] },
      execute: async (input) => {
        const month = str(input.args?.month);
        const item = await findMoneyItem(deps, str(input.args?.id), month);
        if (!item) return fail("Lançamento não encontrado nesse mês.");
        const paid = input.args?.paid !== false;
        await deps.money.setPaid(item.id, month, paid);
        return { atualizado: true, titulo: item.title, mes: month, pago: paid };
      } },
    { name: "delete_money_item", domain: "Finanças", ...destroy,
      description: "Apaga um lançamento (todas as parcelas e repetições). Exige a permissão do Senhor: o sistema pergunta a ele",
      parameters: { type: "object", properties: { id: { type: "string" }, month: { type: "string", description: "AAAA-MM onde ele aparece" } }, required: ["id", "month"] },
      execute: async (input) => {
        const item = await findMoneyItem(deps, str(input.args?.id), str(input.args?.month));
        if (!item) return fail("Lançamento não encontrado nesse mês.");
        const gate = await permitted(input, `Apagar o lançamento "${item.title}" (R$ ${reais(item.amountCents).toFixed(2).replace(".", ",")})`, item.recurrence === "once" ? "O lançamento será removido definitivamente." : "Todas as parcelas e repetições serão removidas definitivamente.");
        if (gate !== true) return gate;
        await deps.money.remove(item.id);
        return { apagado: true, titulo: item.title };
      } },
    // ---------- música ----------
    { name: "play_music", domain: "Música", ...write,
      description: "Busca e toca no Spotify uma música, artista, álbum ou playlist",
      parameters: { type: "object", properties: { query: { type: "string", description: "O que tocar" }, kind: { type: "string", enum: ["track", "artist", "album", "playlist"], description: "Padrão: track" } }, required: ["query"] },
      execute: async (input) => {
        const query = str(input.args?.query);
        if (!query) return fail("Diga o que tocar.");
        const kind = (["track", "artist", "album", "playlist"].includes(str(input.args?.kind)) ? str(input.args?.kind) : "track") as "track" | "artist" | "album" | "playlist";
        const found = await deps.music.search(query, kind);
        const item = { track: found.tracks, artist: found.artists, album: found.albums, playlist: found.playlists }[kind][0];
        if (!item) return fail(`Nada encontrado no Spotify para "${query}".`);
        await deps.music.play(item.uri, deps.music.device(), kind === "track" ? item.albumUri ?? null : null);
        return { tocando: true, nome: item.name, detalhe: item.subtitle };
      } },
    { name: "music_control", domain: "Música", ...write,
      description: "Controla a música: tocar, pausar, próxima ou anterior",
      parameters: { type: "object", properties: { action: { type: "string", enum: ["play", "pause", "next", "previous"] } }, required: ["action"] },
      execute: async (input) => {
        const action = str(input.args?.action) as SpotifyAction;
        if (!["play", "pause", "next", "previous"].includes(action)) return fail("Ação de música inválida.");
        const player = await deps.music.player().catch(() => null);
        await deps.music.control(action, { active: Boolean(player), activeDevice: player?.device?.name, deviceId: deps.music.device() });
        return { feito: true, acao: action };
      } },
  ];
}
