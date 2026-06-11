import { useEffect, useState } from "react";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { ArrowLeft } from "lucide-react";
import { readTaskContent, type TaskFull } from "../hooks/useTauri";
import { FrontmatterSidebar } from "./FrontmatterSidebar";
import type { Page } from "../App";

interface Props {
  product: string;
  taskId: string;
  setPage: (p: Page) => void;
  showBack?: boolean;
}

export function TaskDetailPanel({
  product,
  taskId,
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
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );
  }
  if (!task) {
    return <div className="text-[var(--text-secondary)]">Loading…</div>;
  }

  const rawIntents = task.frontmatter.related_intents;
  const relatedIntents = Array.isArray(rawIntents)
    ? rawIntents.map(String)
    : typeof rawIntents === "string"
      ? [rawIntents]
      : [];

  return (
    <div className="space-y-4 h-full flex flex-col">
      {showBack && (
        <button
          onClick={() =>
            setPage({
              kind: "products",
              product,
              tab: "tasks",
              taskId,
            })
          }
          className="flex items-center gap-2 text-sm text-[var(--text-secondary)] hover:text-[var(--accent)]"
        >
          <ArrowLeft size={16} /> Back to Products
        </button>
      )}
      <div className="flex gap-6 min-h-0 flex-1">
        <div className="flex-1 min-w-0 overflow-auto">
          <p className="font-mono text-sm text-[var(--accent)]">{task.task_id}</p>
          <h2 className="text-xl font-bold mb-4">{task.title}</h2>
          <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)] p-6 prose prose-invert max-w-none">
            <Markdown remarkPlugins={[remarkGfm]}>{task.body}</Markdown>
          </div>
          {relatedIntents.length > 0 && (
            <div className="mt-4 text-sm">
              <h3 className="font-medium text-[var(--text-secondary)] mb-2">
                Related intents
              </h3>
              <div className="flex flex-wrap gap-2">
                {relatedIntents.map((ref) => (
                  <button
                    key={ref}
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
                    className="px-2 py-1 rounded bg-[var(--bg-tertiary)] text-[var(--accent)] text-xs font-mono hover:opacity-90"
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
