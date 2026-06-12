import type { Components } from "react-markdown";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { MermaidRenderer } from "./MermaidRenderer";

interface Props {
  content: string;
  className?: string;
}

const components: Components = {
  code({ className, children, ...props }) {
    const text = String(children).replace(/\n$/, "");
    const lang = className?.replace("language-", "") ?? "";
    if (lang === "mermaid") {
      return (
        <div className="my-4 overflow-x-auto rounded-[var(--radius-sm)] border border-[var(--border)] bg-[var(--bg-primary)] p-3">
          <MermaidRenderer chart={text} />
        </div>
      );
    }
    return (
      <code className={className} {...props}>
        {children}
      </code>
    );
  },
};

export function MarkdownWithMermaid({ content, className }: Props) {
  return (
    <div className={className}>
      <Markdown remarkPlugins={[remarkGfm]} components={components}>
        {content}
      </Markdown>
    </div>
  );
}
