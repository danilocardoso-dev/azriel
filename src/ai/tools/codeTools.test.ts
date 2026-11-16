import { beforeEach, describe, expect, it, vi } from "vitest";
import type { CommandResult } from "../../repositories/workspaceRepository";
import { codeTools, deletionReason, type CodeDependencies } from "./codeTools";

describe("detector de exclusão em comandos", () => {
  it.each([
    "Remove-Item .\\dist -Recurse -Force",
    "rm -rf node_modules",
    "del /s /q *.log",
    "cd src; rmdir old",
    "npx rimraf build",
    "[IO.File]::Delete('C:\\a.txt')",
    "git clean -fdx",
    "git reset --hard HEAD~1",
    "git checkout -- .",
    "git push origin main --force",
    "git branch -D feature",
    "robocopy C:\\a C:\\b /MIR",
    "reg delete HKCU\\Software\\X /f",
    "winget uninstall Spotify",
    "sqlite3 app.db \"DROP TABLE users\"",
    "Format-Volume -DriveLetter D",
  ])("pede permissão para: %s", (command) => {
    expect(deletionReason(command)).not.toBeNull();
  });

  it.each([
    "npm test",
    "npm run build",
    "git status",
    "git commit -m \"remove o botão antigo\"",
    "git push origin main",
    "cargo test",
    "Get-ChildItem -Recurse",
    "mkdir nova-pasta",
    "node scripts/format.js",
    "npm install react-markdown",
    "echo deletar depois",
  ])("roda sozinho: %s", (command) => {
    expect(deletionReason(command)).toBeNull();
  });
});

describe("ferramentas de programação", () => {
  const ok: CommandResult = { exitCode: 0, stdout: "tudo certo", stderr: "", timedOut: false, durationMs: 1234, cwd: "C:\\Projetos\\app" };
  let deps: CodeDependencies;
  beforeEach(() => {
    deps = {
      list: vi.fn(), read: vi.fn(), search: vi.fn(), mkdir: vi.fn(async (path: string) => path),
      write: vi.fn(async (path: string) => ({ path, created: true, backup: null, replacements: 0 })),
      edit: vi.fn(async (path: string) => ({ path, created: false, backup: "C:\\bkp", replacements: 1 })),
      remove: vi.fn(async (path: string) => `${path} foi para a Lixeira.`),
      run: vi.fn(async () => ok),
    };
  });
  const tool = (name: string) => codeTools(deps).find((item) => item.name === name)!;

  it("cria, edita e roda comandos comuns sem pedir permissão", async () => {
    const confirm = vi.fn(async () => true);
    await tool("write_file").execute({ query: "", confirm, args: { path: "C:\\Projetos\\app\\a.ts", content: "x" } });
    await tool("edit_file").execute({ query: "", confirm, args: { path: "C:\\Projetos\\app\\a.ts", oldText: "x", newText: "y" } });
    const result = await tool("run_command").execute({ query: "", confirm, args: { command: "npm test", cwd: "C:\\Projetos\\app" } });
    expect(result).toMatchObject({ codigoSaida: 0, sucesso: true, saida: "tudo certo", duracaoSegundos: 1.2 });
    expect(confirm).not.toHaveBeenCalled();
    expect(deps.write).toHaveBeenCalledWith("C:\\Projetos\\app\\a.ts", "x", false);
  });

  it("comando que apaga: pergunta com o comando exato e só roda se permitido", async () => {
    const confirm = vi.fn(async () => false);
    const denied = await tool("run_command").execute({ query: "", confirm, args: { command: "Remove-Item dist -Recurse", cwd: "C:\\Projetos\\app" } });
    expect(denied).toMatchObject({ negado: true });
    expect(deps.run).not.toHaveBeenCalled();
    expect(confirm).toHaveBeenCalledWith({ title: "Rodar um comando que apaga arquivos ou pastas", detail: "C:\\Projetos\\app> Remove-Item dist -Recurse" });
    confirm.mockResolvedValueOnce(true);
    await tool("run_command").execute({ query: "", confirm, args: { command: "Remove-Item dist -Recurse", cwd: "C:\\Projetos\\app" } });
    expect(deps.run).toHaveBeenCalledTimes(1);
  });

  it("apagar arquivo: só com permissão, e nunca pela voz", async () => {
    expect(await tool("delete_path").execute({ query: "", args: { path: "C:\\Projetos\\app\\velho.ts" } })).toMatchObject({ error: expect.stringMatching(/Nada foi apagado/) });
    expect(await tool("delete_path").execute({ query: "", confirm: async () => false, args: { path: "C:\\Projetos\\app\\velho.ts" } })).toMatchObject({ negado: true });
    expect(deps.remove).not.toHaveBeenCalled();
    await tool("delete_path").execute({ query: "", confirm: async () => true, args: { path: "C:\\Projetos\\app\\velho.ts" } });
    expect(deps.remove).toHaveBeenCalledWith("C:\\Projetos\\app\\velho.ts");
  });

  it("pela voz (sem tela), um comando que apaga não roda", async () => {
    const result = await tool("run_command").execute({ query: "", args: { command: "git clean -fd", cwd: "C:\\Projetos\\app" } });
    expect(result).toMatchObject({ error: expect.stringMatching(/Nada foi apagado nem executado/) });
    expect(deps.run).not.toHaveBeenCalled();
  });
});
