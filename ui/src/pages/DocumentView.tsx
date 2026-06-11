import { useCallback, useEffect, useState } from "react";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeHighlight from "rehype-highlight";
import { CheckCircle2, XCircle } from "lucide-react";
import { readDoc, type DocFull } from "../hooks/useTauri";
import { LoadingState } from "../components/LoadingState";
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
      <div className="page-frame">
        <div className="card border-[rgba(239,68,68,0.25)] bg-[rgba(239,68,68,0.08)] p-4 text-[13px] text-[var(--accent-red)]">
          {error}
        </div>
      </div>
    );
  }
  if (!doc) {
    return (
      <div className="page-frame">
        <LoadingState label="Loading document…" />
      </div>
    );
  }

  return (
    <div className="page-frame mx-auto max-w-5xl">
      <div className="detail-grid">
        <div className="min-w-0 space-y-3">
          <div>
            <h2 className="text-lg font-semibold leading-snug">{doc.title}</h2>
            <div className="mt-2 flex flex-wrap items-center gap-2 text-[12px] text-[var(--text-muted)]">
              <StatusBadge status={doc.status} />
              <span>{doc.doc_type}</span>
              <span className="font-mono text-[11px]">{doc.file_path}</span>
            </div>
          </div>
          <div className="card p-5 prose prose-invert max-w-none">
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
        <aside className="detail-rail card p-3.5 text-[12px]">
          <h3 className="section-label mb-2">Doc check</h3>
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
        </aside>
      </div>
    </div>
  );
}
