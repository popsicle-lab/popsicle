import { useEffect, useState } from "react";
import { BookOpen, GitBranch, Play } from "lucide-react";
import {
  getIssue,
  getIssueGuidance,
  listDocsForRun,
  startIssue,
  type DocInfo,
  type IssueGuidance,
  type IssueInfo,
} from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import type { Page } from "../App";

interface Props {
  issueKey: string;
  setPage: (p: Page) => void;
}

export function IssueDetailView({ issueKey, setPage }: Props) {
  const [issue, setIssue] = useState<IssueInfo | null>(null);
  const [docsByRun, setDocsByRun] = useState<Record<string, DocInfo[]>>({});
  const [guidance, setGuidance] = useState<IssueGuidance | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [starting, setStarting] = useState(false);

  const load = () => {
    getIssue(issueKey)
      .then(async (i) => {
        setIssue(i);
        getIssueGuidance(issueKey)
          .then(setGuidance)
          .catch(() => setGuidance(null));
        const entries = await Promise.all(
          i.run_ids.map(async (runId) => {
            const docs = await listDocsForRun(runId).catch(() => []);
            return [runId, docs] as const;
          })
        );
        setDocsByRun(Object.fromEntries(entries));
      })
      .catch((e) => setError(String(e)));
  };

  useEffect(() => {
    load();
  }, [issueKey]);

  const handleStart = async () => {
    setStarting(true);
    try {
      const runId = await startIssue(issueKey);
      setPage({ kind: "pipeline", runId });
    } catch (e: unknown) {
      setError(String(e));
    } finally {
      setStarting(false);
    }
  };

  if (error) {
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );
  }
  if (!issue) {
    return <div className="text-[var(--text-secondary)]">Loading…</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <p className="font-mono text-sm text-[var(--accent)]">{issue.key}</p>
          <h2 className="text-2xl font-bold mt-1">{issue.title}</h2>
          <div className="flex items-center gap-2 mt-2">
            <StatusBadge status={issue.status} />
            <span className="text-xs text-[var(--text-secondary)]">
              {issue.issue_type} · {issue.priority} · {issue.spec_id}
            </span>
          </div>
        </div>
        {!issue.active_run_id && issue.status !== "done" && (
          <button
            onClick={handleStart}
            disabled={starting}
            className="flex items-center gap-2 px-4 py-2 bg-[var(--accent)] text-[var(--bg-primary)] rounded-lg text-sm font-medium disabled:opacity-50"
          >
            <Play size={16} />
            {starting ? "Starting…" : "Start pipeline"}
          </button>
        )}
      </div>

      {issue.description && (
        <div className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-xl p-4 text-sm whitespace-pre-wrap">
          {issue.description}
        </div>
      )}

      {guidance && (guidance.recommended_tasks.length > 0 || guidance.related_intents.length > 0) && (
        <div className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-xl p-4 space-y-4">
          <div className="flex items-center gap-2">
            <BookOpen size={18} className="text-[var(--accent)]" />
            <h3 className="font-medium">工作流引导</h3>
            {guidance.product && (
              <button
                onClick={() =>
                  setPage({
                    kind: "products",
                    product: guidance.product!,
                    tab: "tasks",
                  })
                }
                className="text-xs text-[var(--accent)] hover:underline ml-auto"
              >
                打开 {guidance.product}
              </button>
            )}
          </div>
          <p className="text-xs text-[var(--text-secondary)]">{guidance.hint}</p>
          {guidance.recommended_tasks.length > 0 && (
            <div>
              <p className="text-xs font-medium text-[var(--text-secondary)] mb-2">
                推荐 Tasks
              </p>
              <div className="space-y-1">
                {guidance.recommended_tasks.map((t) => (
                  <button
                    key={t.task_id}
                    onClick={() =>
                      setPage({
                        kind: "task",
                        taskId: t.task_id,
                        product: t.product,
                      })
                    }
                    className="w-full text-left px-3 py-2 rounded-lg hover:bg-[var(--bg-tertiary)] text-sm flex items-center gap-2"
                  >
                    <span className="font-mono text-xs text-[var(--accent)] shrink-0">
                      {t.task_id}
                    </span>
                    <span className="truncate flex-1">{t.title}</span>
                    <span className="text-[10px] px-1.5 py-0.5 rounded bg-[var(--bg-primary)] text-[var(--text-secondary)] shrink-0">
                      {t.journey_stage}
                    </span>
                  </button>
                ))}
              </div>
            </div>
          )}
          {guidance.related_intents.length > 0 && (
            <div>
              <p className="text-xs font-medium text-[var(--text-secondary)] mb-2">
                相关 Intents
              </p>
              <div className="flex flex-wrap gap-2">
                {guidance.related_intents.map((ri) => (
                  <button
                    key={ri.reference}
                    onClick={() =>
                      setPage({
                        kind: "intent",
                        product: ri.product,
                        file: ri.file,
                        block: ri.block,
                      })
                    }
                    className="px-2 py-1 rounded bg-[var(--bg-tertiary)] text-[var(--accent)] text-xs font-mono hover:opacity-90"
                  >
                    {ri.reference}
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {issue.active_run_id && (
        <div className="bg-blue-500/10 border border-blue-500/30 rounded-xl p-4 flex items-center justify-between">
          <div className="flex items-center gap-2 text-sm">
            <GitBranch size={16} />
            Active run:{" "}
            <code className="font-mono text-[var(--accent)]">
              {issue.active_run_id.slice(0, 8)}…
            </code>
          </div>
          <button
            onClick={() =>
              setPage({ kind: "pipeline", runId: issue.active_run_id! })
            }
            className="text-sm text-[var(--accent)] hover:underline"
          >
            Open pipeline
          </button>
        </div>
      )}

      <div>
        <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-3">
          Pipeline runs ({issue.run_ids.length})
        </h3>
        <div className="space-y-3">
          {issue.run_ids.map((runId) => (
            <div
              key={runId}
              className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-xl p-4"
            >
              <button
                onClick={() => setPage({ kind: "pipeline", runId })}
                className="font-mono text-sm text-[var(--accent)] hover:underline"
              >
                {runId}
              </button>
              <div className="mt-2 space-y-1">
                {(docsByRun[runId] ?? []).map((doc) => (
                  <button
                    key={doc.id}
                    onClick={() => setPage({ kind: "document", docId: doc.id })}
                    className="block w-full text-left text-xs px-2 py-1 rounded hover:bg-[var(--bg-tertiary)]"
                  >
                    {doc.title}{" "}
                    <StatusBadge status={doc.status} />
                  </button>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
