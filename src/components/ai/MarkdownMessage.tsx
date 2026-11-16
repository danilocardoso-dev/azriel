import Markdown, { type Components } from "react-markdown";
import { invokeDatabase } from "../../repositories/tauri";

/** Links abrem no navegador padrão; dentro do app um <a> trocaria a janela inteira. */
function openLink(href: string | undefined) {
  if (!href || !/^https?:\/\//i.test(href)) return;
  void invokeDatabase("open_external", { url: href }).catch(() => undefined);
}

const components: Components = {
  a: ({ href, children }) => <a href={href} title={href} onClick={(event) => { event.preventDefault(); openLink(href); }}>{children}</a>,
  // Imagens externas não carregam: viram link, sem buscar nada na rede sozinho.
  img: ({ src, alt }) => typeof src === "string" ? <a href={src} onClick={(event) => { event.preventDefault(); openLink(src); }}>{alt || "imagem"}</a> : null,
  table: ({ children }) => <div className="ai-markdown__table"><table>{children}</table></div>,
};

/** Resposta do Azriel com formatação: negrito, listas, títulos, citações e código. HTML cru é ignorado. */
export function MarkdownMessage({ content }: { content: string }) {
  return <div className="ai-markdown"><Markdown skipHtml components={components}>{content}</Markdown></div>;
}
