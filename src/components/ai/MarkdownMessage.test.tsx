import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { MarkdownMessage } from "./MarkdownMessage";

const html = (content: string) => renderToStaticMarkup(<MarkdownMessage content={content} />);

describe("respostas do Azriel com markdown", () => {
  it("renderiza negrito, listas, títulos e código em vez de mostrar os símbolos", () => {
    const out = html("## Resumo\n\n* **O que faço hoje:** consulto tarefas\n* item dois\n\n1. primeiro\n\nUse `npm test`.\n\n```\nnpm run build\n```");
    expect(out).toContain("<h2>Resumo</h2>");
    expect(out).toContain("<strong>O que faço hoje:</strong>");
    expect(out).toContain("<ul>");
    expect(out).toContain("<ol>");
    expect(out).toContain("<code>npm test</code>");
    expect(out).toContain("<pre><code>npm run build");
    expect(out).not.toContain("**");
  });

  it("ignora HTML cru e neutraliza links perigosos", () => {
    const out = html("<script>alert(1)</script>\n\nTexto com <b>tag</b> e [clique](javascript:alert(1)).\n\nFontes: [Reuters](https://www.reuters.com)");
    expect(out).not.toContain("<b>");
    expect(out).not.toContain("<script");
    expect(out).not.toContain("javascript:");
    expect(out).toContain('href="https://www.reuters.com"');
  });

  it("imagens não são carregadas, viram link", () => {
    expect(html("![gráfico](https://exemplo.com/a.png)")).not.toContain("<img");
  });
});
