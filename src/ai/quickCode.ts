import type { AgentStep } from "../types";
import { lineCount } from "./agent/steps";
import { InterruptedError } from "./interrupt";

/**
 * Programação rápida: só programar. Sem identidade, memórias, ferramentas nem histórico do Azriel.
 * O modelo local escreve os arquivos de uma vez (em tempo real) e o Azriel grava na pasta escolhida.
 */
export const QUICK_SYSTEM = `Você é um programador. Responda SOMENTE com os arquivos, sem explicações antes ou depois.
Para cada arquivo escreva uma linha "ARQUIVO: caminho/relativo" e logo abaixo um bloco de código com o conteúdo COMPLETO do arquivo.
Use caminhos relativos à pasta do projeto. Para alterar um arquivo existente, reescreva o arquivo inteiro.
Código simples, correto e em português nos textos visíveis.
Exemplo:
ARQUIVO: index.html
\`\`\`html
<!doctype html>
\`\`\``;

/** Modelos locais em ordem de preferência (medido no PC do Senhor: o 1.7b gera ~11 tokens/s). */
export const QUICK_MODELS = ["qwen3:1.7b", "qwen3:4b"];
export const preferredQuickModel = (installed: string[]) => QUICK_MODELS.find((model) => installed.includes(model)) ?? installed[0] ?? QUICK_MODELS[0];

const TEXT_FILE = /\.(html?|css|m?js|jsx|tsx?|json|md|txt|py|svg|xml|ya?ml|toml|sh|ps1|bat|sql|java|cs|go|rs|php|rb|c|cpp|h)$/i;
const MAX_FILE_CHARS = 6_000;
const MAX_CONTEXT_CHARS = 12_000;

export interface ExistingFile { path: string; content?: string }
export interface GeneratedFile { path: string; content: string }

/** Junta o caminho relativo à pasta, sem deixar sair dela (nada de ".." ou outro disco). */
export function safeJoin(folder: string, relative: string): string | null {
  const base = folder.trim().replace(/[\\/]+$/, "");
  const clean = relative.trim().replace(/^[`"'*]+|[`"'*]+$/g, "").replace(/\//g, "\\");
  if (!clean) return null;
  const absolute = /^[a-z]:\\/i.test(clean) ? clean : `${base}\\${clean.replace(/^\.?\\+/, "")}`;
  // (um caminho absoluto em outro disco cai fora da pasta logo abaixo e é recusado)
  const parts: string[] = [];
  for (const part of absolute.split("\\")) {
    if (part === "" || part === ".") continue;
    if (part === "..") { if (parts.length <= 1) return null; parts.pop(); continue; }
    parts.push(part);
  }
  const resolved = parts.join("\\");
  const inside = resolved.toLowerCase() === base.toLowerCase() ? false : resolved.toLowerCase().startsWith(`${base.toLowerCase()}\\`);
  return inside ? resolved : null;
}

/** Extrai os arquivos da resposta: linha "ARQUIVO: caminho" seguida de um bloco de código. */
export function parseFiles(text: string): GeneratedFile[] {
  const files: GeneratedFile[] = [];
  const pattern = /^[\s>*#-]*\**\s*arquivo\s*\**\s*:\s*\**\s*(.+?)\s*\**\s*\r?\n+\s*```[^\n]*\r?\n([\s\S]*?)\r?\n?```/gim;
  for (const match of text.matchAll(pattern)) {
    const path = match[1].replace(/^[`"']+|[`"']+$/g, "").trim();
    if (path) files.push({ path, content: match[2].endsWith("\n") ? match[2] : `${match[2]}\n` });
  }
  return files;
}

/** Pedido enxuto: a pasta, os arquivos pequenos que já existem nela e o que o Senhor quer. */
export function buildPrompt(request: string, folder: string, existing: ExistingFile[]): string {
  const parts = [`Pasta do projeto: ${folder}`];
  const withContent = existing.filter((file) => file.content !== undefined);
  const namesOnly = existing.filter((file) => file.content === undefined);
  if (withContent.length) parts.push(`Arquivos que já existem:\n${withContent.map((file) => `ARQUIVO: ${file.path}\n\`\`\`\n${file.content}\n\`\`\``).join("\n")}`);
  if (namesOnly.length) parts.push(`Outros arquivos na pasta: ${namesOnly.map((file) => file.path).join(", ")}`);
  parts.push(`Pedido: ${request.trim()}`);
  return parts.join("\n\n");
}

export interface QuickDependencies {
  list(path: string): Promise<{ entries: Array<{ name: string; kind: string; size: number | null }> }>;
  read(path: string): Promise<{ content: string }>;
  write(path: string, content: string, overwrite: boolean): Promise<{ created: boolean }>;
  stream(model: string, system: string, prompt: string, onToken: (text: string) => void): Promise<string>;
  cancel(): Promise<unknown>;
}

/** Lê os arquivos de texto pequenos da pasta (sem subpastas), dentro do teto de contexto. */
export async function folderContext(deps: QuickDependencies, folder: string): Promise<ExistingFile[]> {
  const listing = await deps.list(folder).catch(() => null);
  if (!listing) return [];
  const files: ExistingFile[] = [];
  let used = 0;
  for (const entry of listing.entries.filter((item) => item.kind === "arquivo")) {
    const small = TEXT_FILE.test(entry.name) && (entry.size ?? 0) <= MAX_FILE_CHARS && used + (entry.size ?? 0) <= MAX_CONTEXT_CHARS;
    if (!small) { files.push({ path: entry.name }); continue; }
    const read = await deps.read(`${folder.replace(/[\\/]+$/, "")}\\${entry.name}`).catch(() => null);
    if (read) { files.push({ path: entry.name, content: read.content.trimEnd() }); used += read.content.length; }
    else files.push({ path: entry.name });
  }
  return files;
}

export interface QuickResult { content: string; steps: AgentStep[]; written: number }

export async function runQuickCode(input: {
  deps: QuickDependencies;
  model: string;
  folder: string;
  request: string;
  signal?: AbortSignal;
  onText?: (text: string) => void;
  onStep?: (step: AgentStep) => void;
}): Promise<QuickResult> {
  const { deps, signal } = input;
  const folder = input.folder.trim().replace(/[\\/]+$/, "");
  if (!/^c:\\/i.test(`${folder}\\`)) throw new Error("Escolha uma pasta no disco C, como C:\\Projetos\\teste.");
  const steps: AgentStep[] = [];
  const report = (step: AgentStep) => {
    const index = steps.findIndex((item) => item.id === step.id);
    if (index >= 0) steps[index] = step; else steps.push(step);
    input.onStep?.(step);
  };
  const onAbort = () => { void deps.cancel(); };
  signal?.addEventListener("abort", onAbort, { once: true });
  try {
    const existing = await folderContext(deps, folder);
    if (signal?.aborted) throw new InterruptedError();
    report({ id: 1, tool: "quick_code", label: "Escrevendo", target: input.model, status: "running" });
    let text: string;
    try {
      text = await deps.stream(input.model, QUICK_SYSTEM, buildPrompt(input.request, folder, existing), (token) => input.onText?.(token));
    } catch (error) {
      if (signal?.aborted) throw new InterruptedError();
      throw error;
    }
    if (signal?.aborted) throw new InterruptedError();
    const files = parseFiles(text);
    report({ id: 1, tool: "quick_code", label: "Escreveu", target: input.model, detail: `${files.length} arquivo(s)`, status: "ok" });
    let written = 0;
    for (const [index, file] of files.entries()) {
      const path = safeJoin(folder, file.path);
      const id = index + 2;
      if (!path) { report({ id, tool: "write_file", label: "Caminho recusado (fora da pasta)", target: file.path, status: "denied" }); continue; }
      try {
        const result = await deps.write(path, file.content, true);
        report({ id, tool: "write_file", label: result.created ? "Criou" : "Reescreveu", target: path, added: lineCount(file.content), status: "ok" });
        written += 1;
      } catch (error) {
        report({ id, tool: "write_file", label: "Falhou ao gravar", target: path, detail: error instanceof Error ? error.message : String(error), status: "error" });
      }
    }
    const summary = files.length
      ? `⚡ Pronto, Senhor: ${written} de ${files.length} arquivo(s) em \`${folder}\`.`
      : "⚡ O modelo não devolveu arquivos no formato esperado. Tente reformular o pedido.";
    return { content: `${summary}\n\n${text.trim()}`, steps, written };
  } finally {
    signal?.removeEventListener("abort", onAbort);
  }
}
