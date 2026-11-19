import { describe, expect, it } from "vitest";
import { finishedStep, lineCount, startedStep, summarizeSteps } from "./steps";

describe("passos do agente, como no Claude", () => {
  it("conta linhas sem a quebra final", () => {
    expect(lineCount("a\nb\n")).toBe(2);
    expect(lineCount("a\r\nb")).toBe(2);
    expect(lineCount("")).toBe(0);
  });

  it("mostra o que está rodando e o alvo", () => {
    expect(startedStep(1, "run_command", "Programação", { command: "npm test", cwd: "C:\\app" })).toMatchObject({ label: "Rodando", target: "npm test", status: "running" });
    expect(startedStep(2, "get_cpu_status", "Sistema", {})).toMatchObject({ label: "Consultando sistema" });
  });

  it("edição mostra o arquivo e o +/− de linhas", () => {
    const step = finishedStep(1, "edit_file", "Programação", { path: "C:\\app\\src\\a.ts", oldText: "x\ny", newText: "x\ny\nz\nw" }, { replacements: 1, backup: "C:\\bkp" });
    expect(step).toMatchObject({ label: "Editou", target: "C:\\app\\src\\a.ts", added: 4, removed: 2, status: "ok" });
  });

  it("diferencia criar de reescrever arquivo", () => {
    expect(finishedStep(1, "write_file", "Programação", { path: "C:\\a.js", content: "1\n2\n3\n" }, { created: true }).label).toBe("Criou");
    expect(finishedStep(1, "write_file", "Programação", { path: "C:\\a.js", content: "1" }, { created: false, backup: "C:\\b" })).toMatchObject({ label: "Reescreveu", added: 1, detail: "backup: C:\\b" });
  });

  it("comando com falha fica vermelho e guarda a saída", () => {
    const step = finishedStep(1, "run_command", "Programação", { command: "npm test" }, { codigoSaida: 1, sucesso: false, duracaoSegundos: 2.5, saida: "1 failed", erros: "" });
    expect(step).toMatchObject({ label: "Comando falhou", status: "error" });
    expect(step.detail).toContain("código 1 · 2,5 s");
    expect(step.detail).toContain("1 failed");
  });

  it("erro e permissão negada viram estados próprios", () => {
    expect(finishedStep(1, "read_file", "Programação", { path: "C:\\x" }, { error: "Arquivo não encontrado" })).toMatchObject({ status: "error", detail: "Arquivo não encontrado" });
    expect(finishedStep(1, "delete_path", "Programação", { path: "C:\\x" }, { negado: true, mensagem: "O Senhor negou." })).toMatchObject({ status: "denied", label: "Negado pelo Senhor" });
    expect(finishedStep(1, "read_file", "Programação", {}, null, "falha de rede")).toMatchObject({ status: "error", detail: "falha de rede" });
  });

  it("resume arquivos alterados e comandos", () => {
    const steps = [
      finishedStep(1, "read_file", "Programação", { path: "C:\\a" }, {}),
      finishedStep(2, "edit_file", "Programação", { path: "C:\\a", oldText: "a", newText: "b" }, {}),
      finishedStep(3, "edit_file", "Programação", { path: "C:\\a", oldText: "b", newText: "c" }, {}),
      finishedStep(4, "run_command", "Programação", { command: "npm test" }, { sucesso: true, codigoSaida: 0 }),
    ];
    expect(summarizeSteps(steps)).toBe("4 passos · 1 arquivo alterado · 1 comando");
  });
});
