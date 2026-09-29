import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { StudyMarkdown } from "./StudyMarkdown";
import { safeStudyMarkdownUrl } from "./studyMarkdownSecurity";

describe("StudyMarkdown", () => {
  it("renderiza estrutura Markdown e preserva a linguagem do bloco de código", () => {
    const html = renderToStaticMarkup(<StudyMarkdown content={'# Título\n\n- item\n\n```ts\nconst value = 1;\n```'} />);
    expect(html).toContain("<h1>Título</h1>");
    expect(html).toContain("<li>item</li>");
    expect(html).toContain('data-language="ts"');
    expect(html).toContain("const value = 1;");
  });

  it("remove HTML arbitrário e bloqueia protocolos inseguros", () => {
    const html = renderToStaticMarkup(<StudyMarkdown content={'<script>alert(1)</script>\n\n[perigo](javascript:alert(1))'} />);
    expect(html).not.toContain("<script");
    expect(html).not.toContain("javascript:");
    expect(safeStudyMarkdownUrl("https://example.com")).toBe("https://example.com");
    expect(safeStudyMarkdownUrl("javascript:alert(1)")).toBe("");
  });
});
