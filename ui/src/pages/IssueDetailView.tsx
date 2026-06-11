import { useEffect, useState } from "react";
import { GitBranch, Play } from "lucide-react";
import {
  getIssue,
  listDocsForRun,
  startIssue,
  type DocInfo,
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
  const [error, setError] = useState<string | null>(null);
  const [starting, setStarting] = useState(false);

  const load = () => {
    getIssue(issueKey)
      .then(async (i) => {
        setIssue(i);
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
