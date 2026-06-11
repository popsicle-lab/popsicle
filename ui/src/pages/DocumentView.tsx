import { useCallback, useEffect, useState } from "react";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeHighlight from "rehype-highlight";
import { CheckCircle2, XCircle } from "lucide-react";
import { readDoc, type DocFull } from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import { MermaidRenderer } from "../components/MermaidRenderer";
import type { Page } from "../App";

interface Props {
  docId: string;
  setPage: (p: Page) => void;
}

export function DocumentView({ docId, setPage: _setPage }: Props) {
  const [doc, setDoc] = useState<DocFull | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(() => {
    readDoc(docId)
      .then(setDoc)
      .catch((e) => setError(String(e)));
  }, [docId]);

  useEffect(() => {
    load();
  }, [load]);

  if (error) {
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );
  }
  if (!doc) {
    return <div className="text-[var(--text-secondary)]">Loading…</div>;
  }

  return (
    <div className="flex gap-6">
      <div className="flex-1 min-w-0">
        <h2 className="text-2xl font-bold mb-2">{doc.title}</h2>
        <div className="flex items-center gap-2 text-sm text-[var(--text-secondary)] mb-4">
          <StatusBadge status={doc.status} />
          <span>{doc.doc_type}</span>
          <span className="font-mono text-xs">{doc.file_path}</span>
        </div>
        <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)] p-6 prose prose-invert max-w-none">
          <Markdown
            remarkPlugins={[remarkGfm]}
            rehypePlugins={[rehypeHighlight]}
            components={{
              code(props) {
                const { className, children, ...rest } = props;
                const match = /language-(\w+)/.exec(className || "");
                const lang = match?.[1];
                const text = String(children).replace(/\n$/, "");
                if (lang === "mermaid") {
                  return <MermaidRenderer chart={text} className="my-4" />;
                }
                return (
                  <code className={className} {...rest}>
                    {children}
                  </code>
                );
              },
            }}
          >
            {doc.body}
          </Markdown>
        </div>
      </div>
      <aside className="w-56 shrink-0 space-y-3">
        <div className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-xl p-4 text-sm">
          <h3 className="font-medium mb-2">Doc check</h3>
          <div className="flex items-center gap-2">
            {doc.check_passed ? (
              <>
                <CheckCircle2 size={16} className="text-[var(--accent-green)]" />
                <span className="text-[var(--accent-green)]">Passed</span>
              </>
            ) : (
              <>
                <XCircle size={16} className="text-[var(--accent-red)]" />
                <span className="text-[var(--accent-red)]">Failed</span>
              </>
            )}
          </div>
        </div>
      </aside>
    </div>
  );
}
