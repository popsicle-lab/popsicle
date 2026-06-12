import { useEffect, useState } from "react";
import { MarkdownWithMermaid } from "./MarkdownWithMermaid";
import { ArrowLeft } from "lucide-react";
import { readTaskContent, type TaskFull } from "../hooks/useTauri";
import { FrontmatterSidebar } from "./FrontmatterSidebar";
import { LoadingState } from "./LoadingState";
import type { Page } from "../App";

interface Props {
  product: string;
  taskId: string;
  returnTo?: Page;
  setPage: (p: Page) => void;
  showBack?: boolean;
}

function backLabel(returnTo?: Page): string {
  if (returnTo?.kind === "issue") return `Back to ${returnTo.issueKey}`;
  if (returnTo?.kind === "issues" && returnTo.selectedKey) {
    return `Back to ${returnTo.selectedKey}`;
  }
  return "Back to Products";
}

export function TaskDetailPanel({
  product,
  taskId,
  returnTo,
  setPage,
  showBack,
}: Props) {
  const [task, setTask] = useState<TaskFull | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setError(null);
    readTaskContent(taskId, product)
      .then(setTask)
      .catch((e) => setError(String(e)));
  }, [taskId, product]);

  if (error) {
    return (
      <div className="card border-[rgba(239,68,68,0.25)] bg-[rgba(239,68,68,0.08)] p-4 text-[13px] text-[var(--accent-red)]">
        {error}
      </div>
    );
  }
  if (!task) {
    return <LoadingState label="Loading task…" />;
  }

  const rawIntents = task.frontmatter.related_intents;
  const relatedIntents = Array.isArray(rawIntents)
    ? rawIntents.map(String)
    : typeof rawIntents === "string"
      ? [rawIntents]
      : [];

  return (
    <div className="flex flex-col gap-3">
      {showBack && (
        <button
          type="button"
          onClick={() =>
            setPage(
              returnTo ?? {
                kind: "products",
                product,
                tab: "tasks",
                taskId,
              }
            )
          }
          className="btn btn-ghost w-fit gap-1.5 px-0"
        >
          <ArrowLeft size={15} /> {backLabel(returnTo)}
        </button>
      )}
      <div className="detail-grid">
        <div className="min-w-0">
          <p className="font-mono text-[12px] text-[#93c5fd]">{task.task_id}</p>
          <h2 className="mb-3 text-base font-semibold leading-snug">{task.title}</h2>
          <div className="card p-5 prose prose-invert max-w-none">
            <MarkdownWithMermaid content={task.body} />
          </div>
          {relatedIntents.length > 0 && (
            <div className="mt-3">
              <h3 className="section-label mb-2">Related intents</h3>
              <div className="flex flex-wrap gap-1.5">
                {relatedIntents.map((ref) => (
                  <button
                    key={ref}
                    type="button"
                    onClick={() => {
                      const [file, block] = ref.split("#");
                      setPage({
                        kind: "products",
                        product,
                        tab: "intents",
                        intentFile: file,
                        intentBlock: block,
                      });
                    }}
                    className="badge badge-accent font-mono transition-opacity hover:opacity-90"
                  >
                    {ref}
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>
        <FrontmatterSidebar data={task.frontmatter} />
      </div>
    </div>
  );
}
