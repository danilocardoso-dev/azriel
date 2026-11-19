import type { AgentStep } from "../../types";

/** Linhas de um texto (sem contar a quebra final), para o +/− das edições. */
export function lineCount(text: unknown): number {
  if (typeof text !== "string" || !text) return 0;
  const lines = text.replace(/\r\n/g, "\n").split("\n");
  return text.endsWith("\n") ? lines.length - 1 : lines.length;
}

const str = (value: unknown) => typeof value === "string" ? value.trim() : "";
const obj = (value: unknown): Record<string, unknown> => value && typeof value === "object" ? value as Record<string, unknown> : {};
const tail = (text: string, max = 1_500) => text.length > max ? `…${text.slice(-max)}` : text;
const short = (text: string, max = 90) => text.length > max ? `${text.slice(0, max - 1)}…` : text;

/** Como cada ferramenta aparece em andamento (antes do resultado). */
const RUNNING: Record<string, string> = {
  read_file: "Lendo", list_folder: "Listando", search_code: "Buscando", write_file: "Gravando", edit_file: "Editando",
  create_folder: "Criando pasta", delete_path: "Apagando", run_command: "Rodando", search_web: "Pesquisando na web",
  remember: "Guardando na memória", update_memory: "Corrigindo memória", search_memories: "Procurando na memória",
  play_music: "Procurando música", music_control: "Controlando a música",
};

function target(tool: string, args: Record<string, unknown>): string | undefined {
  switch (tool) {
    case "read_file": case "list_folder": case "write_file": case "edit_file": case "create_folder": case "delete_path": return str(args.path) || undefined;
    case "search_code": return `"${str(args.query)}" em ${str(args.root)}`;
    case "run_command": return str(args.command) || undefined;
    case "search_web": case "search_memories": return str(args.query) || undefined;
    case "remember": case "update_memory": return short(str(args.content));
    case "create_task": case "create_note": return short(str(args.title) || str(args.content));
    case "play_music": return str(args.query) || undefined;
    case "music_control": return str(args.action) || undefined;
    default: return undefined;
  }
}

/** Passo em andamento, mostrado enquanto a ferramenta roda. */
export function startedStep(id: number, tool: string, domain: string, args: unknown): AgentStep {
  return { id, tool, label: RUNNING[tool] ?? `Consultando ${domain.toLowerCase()}`, target: target(tool, obj(args)), status: "running" };
}

/** Passo concluído, com o que de fato aconteceu (arquivo criado ou reescrito, linhas, código de saída...). */
export function finishedStep(id: number, tool: string, domain: string, args: unknown, data: unknown, failure?: string): AgentStep {
  const a = obj(args);
  const d = obj(data);
  const base: AgentStep = { id, tool, label: "", target: target(tool, a), status: "ok" };
  if (failure || typeof d.error === "string") return { ...base, label: `Falhou: ${(RUNNING[tool] ?? domain).toLowerCase()}`, detail: failure ?? String(d.error), status: "error" };
  if (d.negado) return { ...base, label: "Negado pelo Senhor", detail: str(d.mensagem), status: "denied" };
  switch (tool) {
    case "read_file": {
      const range = typeof d.startLine === "number" && typeof d.endLine === "number" ? `linhas ${d.startLine}–${d.endLine} de ${d.totalLines}` : undefined;
      return { ...base, label: "Leu", detail: range };
    }
    case "list_folder": return { ...base, label: "Listou", detail: Array.isArray(d.entries) ? `${d.entries.length} itens` : undefined };
    case "search_code": return { ...base, label: "Buscou", detail: Array.isArray(data) ? `${data.length} resultado(s)` : undefined };
    case "write_file": return { ...base, label: d.created === false ? "Reescreveu" : "Criou", added: lineCount(a.content), detail: str(d.backup) ? `backup: ${str(d.backup)}` : undefined };
    case "edit_file": {
      const times = typeof d.replacements === "number" && d.replacements > 0 ? d.replacements : 1;
      return { ...base, label: "Editou", added: lineCount(a.newText) * times, removed: lineCount(a.oldText) * times };
    }
    case "create_folder": return { ...base, label: "Criou pasta" };
    case "delete_path": return { ...base, label: "Mandou para a Lixeira" };
    case "run_command": {
      const ok = d.sucesso === true;
      const seconds = typeof d.duracaoSegundos === "number" ? `${String(d.duracaoSegundos).replace(".", ",")} s` : "";
      const code = d.tempoEsgotado ? "tempo esgotado" : `código ${d.codigoSaida ?? "?"}`;
      const output = [str(d.saida), str(d.erros)].filter(Boolean).join("\n");
      return { ...base, label: ok ? "Rodou" : "Comando falhou", detail: [`${code}${seconds ? ` · ${seconds}` : ""}`, output && tail(output)].filter(Boolean).join("\n"), status: ok ? "ok" : "error" };
    }
    case "search_web": return { ...base, label: "Pesquisou na web" };
    case "remember": return { ...base, label: "Guardou na memória" };
    case "update_memory": return { ...base, label: "Corrigiu memória" };
    case "search_memories": return { ...base, label: "Procurou na memória" };
    case "create_task": return { ...base, label: "Criou tarefa" };
    case "update_task": return { ...base, label: "Alterou tarefa", target: str(obj(d.tarefa).titulo) || base.target };
    case "delete_task": return { ...base, label: "Apagou tarefa", target: str(d.titulo) || base.target };
    case "create_note": return { ...base, label: "Criou nota" };
    case "update_note": return { ...base, label: "Alterou nota" };
    case "archive_note": return { ...base, label: "Arquivou nota" };
    case "delete_note": return { ...base, label: "Apagou nota" };
    case "save_money_item": return { ...base, label: "Salvou lançamento", target: str(obj(d.lancamento).titulo) || base.target };
    case "set_money_paid": return { ...base, label: d.pago ? "Marcou como pago" : "Desmarcou pagamento", target: str(d.titulo) || base.target };
    case "delete_money_item": return { ...base, label: "Apagou lançamento", target: str(d.titulo) || base.target };
    case "play_music": return { ...base, label: "Tocando", target: [str(d.nome), str(d.detalhe)].filter(Boolean).join(" · ") || base.target };
    case "music_control": return { ...base, label: "Música" };
    default: return { ...base, label: `Consultou ${domain.toLowerCase()}` };
  }
}

/** Resumo de uma lista de passos, para o título recolhido ("3 arquivos · 2 comandos"). */
export function summarizeSteps(steps: AgentStep[]): string {
  const files = new Set(steps.filter((step) => ["write_file", "edit_file"].includes(step.tool) && step.status === "ok").map((step) => step.target));
  const commands = steps.filter((step) => step.tool === "run_command").length;
  const parts = [
    files.size ? `${files.size} arquivo${files.size === 1 ? "" : "s"} alterado${files.size === 1 ? "" : "s"}` : "",
    commands ? `${commands} comando${commands === 1 ? "" : "s"}` : "",
  ].filter(Boolean);
  const total = `${steps.length} ${steps.length === 1 ? "passo" : "passos"}`;
  return parts.length ? `${total} · ${parts.join(" · ")}` : total;
}
