import { useEffect, useState } from "react";
import {
  getIssue,
  startIssue,
  updateIssue,
  type IssueFull,
} from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import { ClipboardList, Play, ArrowLeft } from "lucide-react";
import type { Page } from "../App";

interface Props {
  issueKey: string;
  setPage: (p: Page) => void;
}

const typeColors: Record<string, string> = {
  product: "bg-blue-500/20 text-blue-300",
  technical: "bg-purple-500/20 text-purple-300",
  bug: "bg-red-500/20 text-red-300",
  idea: "bg-yellow-500/20 text-yellow-300",
};

const pipelineMap: Record<string, string> = {
  product: "full-sdlc",
  technical: "tech-sdlc",
  bug: "test-only",
};

export function IssueDetailView({ issueKey, setPage }: Props) {
  const [issue, setIssue] = useState<IssueFull | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [starting, setStarting] = useState(false);

  useEffect(() => {
    getIssue(issueKey)
      .then(setIssue)
      .catch((e) => setError(e?.toString()));
  }, [issueKey]);

  const handleStart = async () => {
    setStarting(true);
    try {
      const updated = await startIssue(issueKey);
      setIssue((prev) => (prev ? { ...prev, ...updated } : prev));
    } catch (e: unknown) {
      setError(e?.toString() ?? "Failed to start");
    } finally {
      setStarting(false);
    }
  };

  const handleStatusChange = async (newStatus: string) => {
    try {
      const updated = await updateIssue({ key: issueKey, status: newStatus });
      setIssue((prev) => (prev ? { ...prev, ...updated } : prev));
    } catch (e: unknown) {
      setError(e?.toString() ?? "Failed to update");
    }
  };

  if (error)
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );

  if (!issue) return <div className="text-[var(--text-secondary)]">Loading...</div>;

  const canStart = !issue.pipeline_run_id && issue.issue_type !== "idea";
  const pipeline = pipelineMap[issue.issue_type];

  return (
    <div className="space-y-6 max-w-3xl">
      <button
        onClick={() => setPage({ kind: "issues" })}
        className="flex items-center gap-1 text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
      >
        <ArrowLeft size={14} />
        Back to Issues
      </button>

      <div className="flex items-start justify-between">
        <div>
          <div className="flex items-center gap-3 mb-1">
            <ClipboardList size={20} className="text-[var(--accent)]" />
            <span className="font-mono text-sm text-[var(--accent)]">
              {issue.key}
            </span>
            <span
              className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${
                typeColors[issue.issue_type] || "bg-gray-500/20 text-gray-300"
              }`}
            >
              {issue.issue_type}
            </span>
            <StatusBadge status={issue.status} />
          </div>
          <h2 className="text-2xl font-bold">{issue.title}</h2>
        </div>

        {canStart && (
          <button
            onClick={handleStart}
            disabled={starting}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-[var(--accent-green)]/15 text-[var(--accent-green)] text-sm font-medium hover:bg-[var(--accent-green)]/25 transition-colors disabled:opacity-50"
          >
            <Play size={16} />
            {starting ? "Starting..." : "Start"}
          </button>
        )}
      </div>

      <div className="grid grid-cols-2 gap-4">
        <InfoCard label="Priority" value={issue.priority} />
        <InfoCard label="Status" value={issue.status} />
        <InfoCard label="Pipeline" value={pipeline || "N/A"} />
        <InfoCard
          label="Pipeline Run"
          value={issue.pipeline_run_id?.slice(0, 8) || "Not started"}
          onClick={
            issue.pipeline_run_id
              ? () =>
                  setPage({
                    kind: "pipeline",
                    runId: issue.pipeline_run_id!,
                  })
              : undefined
          }
        />
      </div>

      {issue.labels.length > 0 && (
        <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
          <h3 className="text-xs font-medium text-[var(--text-secondary)] mb-2">
            Labels
          </h3>
          <div className="flex flex-wrap gap-2">
            {issue.labels.map((l) => (
              <span
                key={l}
                className="px-2 py-0.5 bg-[var(--bg-tertiary)] rounded text-xs"
              >
                {l}
              </span>
            ))}
          </div>
        </div>
      )}

      {issue.description && (
        <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
          <h3 className="text-xs font-medium text-[var(--text-secondary)] mb-2">
            Description
          </h3>
          <p className="text-sm whitespace-pre-wrap">{issue.description}</p>
        </div>
      )}

      <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
        <h3 className="text-xs font-medium text-[var(--text-secondary)] mb-3">
          Actions
        </h3>
        <div className="flex gap-2 flex-wrap">
          {["backlog", "ready", "in_progress", "done", "cancelled"].map((s) => (
            <button
              key={s}
              onClick={() => handleStatusChange(s)}
              disabled={issue.status === s}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                issue.status === s
                  ? "bg-[var(--accent)]/15 text-[var(--accent)]"
                  : "bg-[var(--bg-tertiary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
              }`}
            >
              {s.replace("_", " ").replace(/\b\w/g, (c) => c.toUpperCase())}
            </button>
          ))}
        </div>
      </div>

      <div className="text-xs text-[var(--text-secondary)] flex gap-4">
        <span>Created: {new Date(issue.created_at).toLocaleString()}</span>
        <span>Updated: {new Date(issue.updated_at).toLocaleString()}</span>
      </div>
    </div>
  );
}

function InfoCard({
  label,
  value,
  onClick,
}: {
  label: string;
  value: string;
  onClick?: () => void;
}) {
  const Component = onClick ? "button" : "div";
  return (
    <Component
      onClick={onClick}
      className={`bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)] text-left ${
        onClick ? "hover:bg-[var(--bg-tertiary)] cursor-pointer transition-colors" : ""
      }`}
    >
      <div className="text-xs text-[var(--text-secondary)]">{label}</div>
      <div className="text-sm font-medium mt-1 capitalize">{value}</div>
    </Component>
  );
}
