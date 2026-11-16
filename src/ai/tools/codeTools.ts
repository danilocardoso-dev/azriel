import type { CommandResult, FileText, Listing, SearchHit, WriteResult } from "../../repositories/workspaceRepository";
import type { AIToolInput } from "../../types";
import type { AzrielTool } from "./toolRegistry";

/**
 * Programação: o Azriel lê, busca, cria e edita arquivos e roda comandos em todo o disco C.
 * Regra do Senhor: tudo sozinho, exceto excluir. Apagar arquivo/pasta e comandos que apagam
 * passam pelo cartão Permitir/Negar.
 */
export interface CodeDependencies {
  list(path: string): Promise<Listing>;
  read(path: string, startLine?: number | null, maxLines?: number | null): Promise<FileText>;
  search(root: string, query: string, extension?: string | null): Promise<SearchHit[]>;
  write(path: string, content: string, overwrite: boolean): Promise<WriteResult>;
  edit(path: string, oldText: string, newText: string, replaceAll: boolean): Promise<WriteResult>;
  mkdir(path: string): Promise<string>;
  remove(path: string): Promise<string>;
  run(command: string, cwd: string, timeoutSeconds?: number | null): Promise<CommandResult>;
}

// Cada regra: o que procurar no comando e como explicar ao Senhor. Detector de boa-fé: um script pode
// apagar sem parecer (o Senhor sabe disso); por isso também há backup e Lixeira nas ferramentas de arquivo.
const DELETION_RULES: Array<[RegExp, string]> = [
  [/(^|[\s;|&({`])(remove-item|ri|rm|rmdir|rd|del|erase|rimraf|clear-content|clear-recyclebin)(?=[\s;|&)`]|$)/i, "apaga arquivos ou pastas"],
  [/\[(system\.)?io\.(file|directory)\]::delete\b|\.delete\s*\(/i, "apaga arquivos ou pastas"],
  [/\bformat(-volume)?\s+[a-z]:|\bformat-volume\b|\bdiskpart\b|\bcipher\s+\/w\b|\bclear-disk\b/i, "formata ou limpa um disco"],
  [/\bgit\s+clean\b/i, "apaga arquivos não versionados (git clean)"],
  [/\bgit\s+reset\b[^;|&]*--hard\b/i, "descarta alterações (git reset --hard)"],
  [/\bgit\s+(checkout|restore)\b[^;|&]*\s(--\s+)?\.(\s|$)/i, "descarta alterações nos arquivos"],
  [/\bgit\s+branch\b[^;|&]*\s-(d|D)\b|\bgit\s+push\b[^;|&]*(\s-f\b|--force)/i, "apaga branch ou sobrescreve histórico"],
  [/\brobocopy\b[^;|&]*\/(mir|purge)\b/i, "espelha pastas apagando o que sobra (robocopy)"],
  [/\breg(\.exe)?\s+delete\b|\bremove-itemproperty\b|\bsc(\.exe)?\s+delete\b/i, "apaga do registro ou serviço do Windows"],
  [/\b(winget|choco|scoop)\s+uninstall\b|\buninstall-(package|module)\b/i, "desinstala programas"],
  [/\b(drop\s+(table|database|schema)|truncate\s+table|delete\s+from)\b/i, "apaga dados de um banco"],
];

/** Por que um comando parece apagar algo, ou null. */
export function deletionReason(command: string): string | null {
  for (const [pattern, reason] of DELETION_RULES) if (pattern.test(command)) return reason;
  return null;
}

const str = (value: unknown) => typeof value === "string" ? value : "";
const num = (value: unknown) => typeof value === "number" && Number.isFinite(value) ? value : typeof value === "string" && value.trim() && Number.isFinite(Number(value)) ? Number(value) : null;

async function permitted(input: AIToolInput, title: string, detail: string) {
  if (!input.confirm) return { error: "Excluir só pela tela do AI Core, com a permissão do Senhor. Nada foi apagado nem executado." };
  return await input.confirm({ title, detail }) ? true : { negado: true, mensagem: "O Senhor negou. Nada foi apagado nem executado." };
}

const commandView = (result: CommandResult) => ({
  codigoSaida: result.exitCode,
  sucesso: result.exitCode === 0 && !result.timedOut,
  tempoEsgotado: result.timedOut,
  duracaoSegundos: Math.round(result.durationMs / 100) / 10,
  saida: result.stdout,
  erros: result.stderr || undefined,
});

export function codeTools(deps: CodeDependencies): AzrielTool[] {
  const write = { domain: "Programação", readonly: false, autonomous: true, permission: "safe_write" as const };
  const read = { domain: "Programação", readonly: true };
  const path = { type: "string", description: "Caminho completo no disco C, ex.: C:\\Projetos\\app\\src\\main.ts" };
  return [
    { name: "list_folder", ...read,
      description: "Lista arquivos e pastas de uma pasta no disco C",
      parameters: { type: "object", properties: { path }, required: ["path"] },
      execute: (input) => deps.list(str(input.args?.path)) },
    { name: "read_file", ...read,
      description: "Lê um arquivo de texto. Para arquivos grandes, leia por partes com startLine e maxLines",
      parameters: { type: "object", properties: { path, startLine: { type: "number", description: "Primeira linha (1 = início)" }, maxLines: { type: "number" } }, required: ["path"] },
      execute: (input) => deps.read(str(input.args?.path), num(input.args?.startLine), num(input.args?.maxLines)) },
    { name: "search_code", ...read,
      description: "Procura um texto dentro dos arquivos de uma pasta (e nos nomes). Pula node_modules, .git, target e dist",
      parameters: { type: "object", properties: { root: path, query: { type: "string" }, extension: { type: "string", description: "Opcional, ex.: ts" } }, required: ["root", "query"] },
      execute: (input) => deps.search(str(input.args?.root), str(input.args?.query), str(input.args?.extension) || null) },
    { name: "write_file", ...write,
      description: "Cria um arquivo com o conteúdo completo (cria as pastas que faltarem). Para substituir um arquivo existente use overwrite=true; um backup é feito antes",
      parameters: { type: "object", properties: { path, content: { type: "string" }, overwrite: { type: "boolean" } }, required: ["path", "content"] },
      execute: (input) => deps.write(str(input.args?.path), str(input.args?.content), input.args?.overwrite === true) },
    { name: "edit_file", ...write,
      description: "Edita um arquivo trocando um trecho exato (copie do read_file) por outro. O trecho precisa ser único, ou use replaceAll. Um backup é feito antes",
      parameters: { type: "object", properties: { path, oldText: { type: "string" }, newText: { type: "string" }, replaceAll: { type: "boolean" } }, required: ["path", "oldText", "newText"] },
      execute: (input) => deps.edit(str(input.args?.path), str(input.args?.oldText), str(input.args?.newText), input.args?.replaceAll === true) },
    { name: "create_folder", ...write,
      description: "Cria uma pasta (e as intermediárias)",
      parameters: { type: "object", properties: { path }, required: ["path"] },
      execute: async (input) => ({ criada: await deps.mkdir(str(input.args?.path)) }) },
    { name: "delete_path", domain: "Programação", readonly: false, autonomous: true, permission: "confirm_write",
      description: "Apaga um arquivo ou pasta, mandando para a Lixeira. Exige a permissão do Senhor: o sistema pergunta a ele",
      parameters: { type: "object", properties: { path }, required: ["path"] },
      execute: async (input) => {
        const target = str(input.args?.path).trim();
        if (!target) return { error: "Informe o caminho." };
        const gate = await permitted(input, `Apagar "${target}"`, "Vai para a Lixeira do Windows (dá para restaurar de lá).");
        if (gate !== true) return gate;
        return { feito: await deps.remove(target) };
      } },
    { name: "run_command", domain: "Programação", readonly: false, autonomous: true, permission: "safe_write",
      description: "Roda um comando no PowerShell numa pasta do disco C e devolve a saída e o código de saída (0 = sucesso). Tempo limite padrão de 120 s (máx. 600). Não use para servidores que ficam rodando",
      parameters: { type: "object", properties: { command: { type: "string" }, cwd: { ...path, description: "Pasta onde rodar" }, timeoutSeconds: { type: "number" } }, required: ["command", "cwd"] },
      execute: async (input) => {
        const command = str(input.args?.command).trim();
        const cwd = str(input.args?.cwd).trim();
        if (!command || !cwd) return { error: "Informe o comando e a pasta (cwd)." };
        const reason = deletionReason(command);
        if (reason) {
          const gate = await permitted(input, `Rodar um comando que ${reason}`, `${cwd}> ${command}`);
          if (gate !== true) return gate;
        }
        return commandView(await deps.run(command, cwd, num(input.args?.timeoutSeconds)));
      } },
  ];
}
