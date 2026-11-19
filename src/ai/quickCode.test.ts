import { describe, expect, it, vi } from "vitest";
import { InterruptedError } from "./interrupt";
import { buildPrompt, parseFiles, preferredQuickModel, runQuickCode, safeJoin, type QuickDependencies } from "./quickCode";

describe("programação rápida: leitura da resposta", () => {
  it("extrai vários arquivos, mesmo com variações de formato dos modelos pequenos", () => {
    const text = [
      "ARQUIVO: index.html", "```html", "<h1>Olá</h1>", "```", "",
      "**Arquivo:** `css/estilo.css`", "", "```css", "h1 { color: red; }", "```",
      "arquivo: app.js", "```", "alert(1);", "```",
    ].join("\n");
    expect(parseFiles(text)).toEqual([
      { path: "index.html", content: "<h1>Olá</h1>\n" },
      { path: "css/estilo.css", content: "h1 { color: red; }\n" },
      { path: "app.js", content: "alert(1);\n" },
    ]);
    expect(parseFiles("Aqui está:\n```html\n<p>sem nome</p>\n```")).toEqual([]);
  });

  it("nunca grava fora da pasta escolhida", () => {
    expect(safeJoin("C:\\Projetos\\teste", "index.html")).toBe("C:\\Projetos\\teste\\index.html");
    expect(safeJoin("C:\\Projetos\\teste\\", "css/estilo.css")).toBe("C:\\Projetos\\teste\\css\\estilo.css");
    expect(safeJoin("C:\\Projetos\\teste", "./a/../b.js")).toBe("C:\\Projetos\\teste\\b.js");
    expect(safeJoin("C:\\Projetos\\teste", "C:\\Projetos\\teste\\x.js")).toBe("C:\\Projetos\\teste\\x.js");
    expect(safeJoin("C:\\Projetos\\teste", "../outro/x.js")).toBeNull();
    expect(safeJoin("C:\\Projetos\\teste", "C:\\Windows\\x.dll")).toBeNull();
    expect(safeJoin("C:\\Projetos\\teste", "")).toBeNull();
  });

  it("pedido enxuto: pasta, arquivos pequenos existentes e o pedido", () => {
    const prompt = buildPrompt("adicione um botão", "C:\\p", [{ path: "index.html", content: "<p>oi</p>" }, { path: "logo.png" }]);
    expect(prompt).toContain("Pasta do projeto: C:\\p");
    expect(prompt).toContain("ARQUIVO: index.html\n```\n<p>oi</p>\n```");
    expect(prompt).toContain("Outros arquivos na pasta: logo.png");
    expect(prompt.endsWith("Pedido: adicione um botão")).toBe(true);
  });

  it("prefere o modelo medido como mais rápido", () => {
    expect(preferredQuickModel(["qwen3:4b", "qwen3:1.7b"])).toBe("qwen3:1.7b");
    expect(preferredQuickModel(["llama3"])).toBe("llama3");
  });
});

describe("programação rápida: fluxo completo", () => {
  const deps = (stream: QuickDependencies["stream"]): QuickDependencies & { written: string[] } => {
    const written: string[] = [];
    return {
      written,
      list: async () => ({ entries: [{ name: "index.html", kind: "arquivo", size: 20 }, { name: "foto.png", kind: "arquivo", size: 90_000 }] }),
      read: async () => ({ content: "<p>antigo</p>" }),
      write: async (path) => { written.push(path); return { created: !path.endsWith("index.html") }; },
      stream,
      cancel: vi.fn(async () => undefined),
    };
  };

  it("transmite o texto, grava os arquivos e mostra os passos", async () => {
    const tokens: string[] = [];
    let prompt = "";
    const d = deps(async (_model, _system, p, onToken) => { prompt = p; onToken("ARQUIVO: index.html\n"); onToken("```html\n<p>novo</p>\n```\nARQUIVO: ../fora.js\n```js\nx\n```"); return "ARQUIVO: index.html\n```html\n<p>novo</p>\n```\nARQUIVO: ../fora.js\n```js\nx\n```"; });
    const result = await runQuickCode({ deps: d, model: "qwen3:1.7b", folder: "C:\\Projetos\\teste", request: "mude o texto", onText: (t) => tokens.push(t) });
    expect(tokens.join("")).toContain("<p>novo</p>");
    expect(prompt).toContain("<p>antigo</p>");
    expect(prompt).toContain("Outros arquivos na pasta: foto.png");
    expect(d.written).toEqual(["C:\\Projetos\\teste\\index.html"]);
    expect(result.written).toBe(1);
    expect(result.steps.map((step) => [step.label, step.status])).toEqual([["Escreveu", "ok"], ["Reescreveu", "ok"], ["Caminho recusado (fora da pasta)", "denied"]]);
    expect(result.content.startsWith("⚡ Pronto, Senhor: 1 de 2 arquivo(s)")).toBe(true);
  });

  it("ESC durante a escrita para o modelo e não grava nada", async () => {
    const controller = new AbortController();
    const d = deps(async () => { controller.abort(); throw new Error("Interrompido pelo Senhor."); });
    await expect(runQuickCode({ deps: d, model: "m", folder: "C:\\p", request: "x", signal: controller.signal })).rejects.toBeInstanceOf(InterruptedError);
    expect(d.cancel).toHaveBeenCalled();
    expect(d.written).toEqual([]);
  });

  it("recusa pasta fora do disco C", async () => {
    await expect(runQuickCode({ deps: deps(async () => ""), model: "m", folder: "D:\\x", request: "x" })).rejects.toThrow("disco C");
  });
});
