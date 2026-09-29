import ReactMarkdown from "react-markdown";
import { safeStudyMarkdownUrl } from "./studyMarkdownSecurity";

export function StudyMarkdown({ content }: { content: string }) {
  return <div className="study-markdown">
    <ReactMarkdown
      skipHtml
      urlTransform={safeStudyMarkdownUrl}
      components={{
        a: ({ children, href, title }) => <a href={href} title={title} target="_blank" rel="noreferrer">{children}</a>,
        code: ({ children, className }) => <code className={className} data-language={className?.replace(/^language-/, "") || undefined}>{children}</code>,
      }}
    >{content}</ReactMarkdown>
  </div>;
}
