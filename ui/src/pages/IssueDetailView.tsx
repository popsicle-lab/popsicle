import { useEffect, useState } from "react";
import {
  getIssue,
  startIssue,
  updateIssue,
  getIssueProgress,
  getActivity,
  type IssueFull,
  type IssueProgress,
  type ActivityEvent,
} from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import {
  ClipboardList,
  Play,
  ArrowLeft,
  FileText,
  GitCommit,
  ChevronRight,
  ListChecks,
  Activity,
} from "lucide-react";
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

export function IssueDetailView({ issueKey, setPage }: Props) {
  const [issue, setIssue] = useState<IssueFull | null>(null);
  const [progress, setProgress] = useState<IssueProgress | null>(null);
  const [activity, setActivity] = useState<ActivityEvent[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [starting, setStarting] = useState(false);

  useEffect(() => {
    getIssue(issueKey)
      .then((i) => {
        setIssue(i);
        return getIssueProgress(issueKey);
      })
      .then((p) => {
        setProgress(p);
        if (p.pipeline_run_id) {
          return getActivity(p.pipeline_run_id);
        }
        return [];
      })
      .then(setActivity)
      .catch((e) => setError(e?.toString()));
  }, [issueKey]);

  const handleStart = async () => {
    setStarting(true);
    try {
      const updated = await startIssue(issueKey);
      setIssue((prev) => (prev ? { ...prev, ...updated } : prev));
      const p = await getIssueProgress(issueKey);
      setProgress(p);
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

  if (!issue)
    return <div className="text-[var(--text-secondary)]">Loading...</div>;

  const canStart = !issue.pipeline_run_id && issue.issue_type !== "idea";

  return (
    <div className="space-y-6 max-w-5xl">
      {/* Back */}
      <button
        onClick={() => setPage({ kind: "issues" })}
        className="flex items-center gap-1 text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
      >
        <ArrowLeft size={14} />
        Back to Issues
      </button>

      {/* Header */}
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

      {/* Progress Overview */}
      {progress && progress.pipeline_run_id && (
        <div className="bg-[var(--bg-secondary)] rounded-xl p-5 border border-[var(--border)]">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-sm font-medium text-[var(--text-secondary)]">
              Progress Overview
            </h3>
            {progress.current_stage && (
              <span className="text-xs text-[var(--accent)]">
                Current: {progress.current_stage}
              </span>
            )}
          </div>

          <div className="grid grid-cols-3 gap-6">
            <ProgressMetric
              label="Stages"
              current={progress.stages_completed}
              total={progress.stages_total}
            />
            <ProgressMetric
              label="Documents"
              current={progress.docs_final}
              total={progress.docs_total}
            />
            <ProgressMetric
              label="Checklist"
              current={progress.checklist_checked}
              total={progress.checklist_total}
            />
          </div>
        </div>
      )}

      {/* Pipeline Stages */}
      {progress &&
        progress.stage_summaries.length > 0 && (
          <div className="bg-[var(--bg-secondary)] rounded-xl p-5 border border-[var(--border)]">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-sm font-medium text-[var(--text-secondary)]">
                Pipeline Stages
              </h3>
              {progress.pipeline_run_id && (
                <button
                  onClick={() =>
                    setPage({
                      kind: "pipeline",
                      runId: progress.pipeline_run_id!,
                    })
                  }
                  className="text-xs text-[var(--accent)] hover:underline"
                >
                  View full pipeline →
                </button>
              )}
            </div>
            <div className="flex items-start gap-2 overflow-x-auto pb-2">
              {progress.stage_summaries.map((stage, i) => (
                <div key={stage.name} className="flex items-start">
                  <div className="min-w-[170px]">
                    <div
                      className={`rounded-lg p-3 border ${
                        stage.state === "completed"
                          ? "border-green-500/40 bg-green-500/5"
                          : stage.state === "in_progress"
                            ? "border-purple-500/40 bg-purple-500/5"
                            : stage.state === "ready"
                              ? "border-blue-500/40 bg-blue-500/5"
                              : "border-[var(--border)] bg-[var(--bg-tertiary)]/30"
                      }`}
                    >
                      <div className="flex items-center justify-between mb-1">
                        <span className="text-xs font-medium">
                          {stage.name}
                        </span>
                        <StatusBadge status={stage.state} />
                      </div>
                      {stage.docs.map((doc) => (
                        <button
                          key={doc.id}
                          onClick={() =>
                            setPage({ kind: "document", docId: doc.id })
                          }
                          className="w-full text-left text-xs py-1 px-2 rounded bg-[var(--bg-primary)]/50 hover:bg-[var(--bg-primary)] transition-colors mt-1"
                        >
                          <div className="flex items-center gap-1.5">
                            <FileText size={10} />
                            <span className="truncate flex-1">{doc.title}</span>
                            <StatusBadge status={doc.status} />
                          </div>
                          {doc.checklist_total > 0 && (
                            <MiniBar
                              checked={doc.checklist_checked}
                              total={doc.checklist_total}
                            />
                          )}
                        </button>
                      ))}
                      {stage.docs.length === 0 && (
                        <div className="text-xs text-[var(--text-secondary)] italic mt-1">
                          No documents
                        </div>
                      )}
                    </div>
                  </div>
                  {i < progress.stage_summaries.length - 1 && (
                    <div className="flex items-center px-1 pt-4">
                      <ChevronRight
                        size={14}
                        className="text-[var(--text-secondary)]"
                      />
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

      {/* Activity Timeline */}
      {activity.length > 0 && (
        <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
          <div className="px-4 py-3 border-b border-[var(--border)] flex items-center gap-2">
            <Activity size={14} className="text-[var(--text-secondary)]" />
            <h3 className="text-sm font-medium">Recent Activity</h3>
          </div>
          <div className="divide-y divide-[var(--border)]">
            {activity.slice(0, 15).map((evt, i) => (
              <div key={i} className="px-4 py-2.5 flex items-center gap-3">
                <EventIcon type={evt.event_type} />
                <div className="flex-1 min-w-0">
                  <span className="text-sm">{evt.title}</span>
                  {evt.detail && (
                    <span className="text-xs text-[var(--text-secondary)] ml-2">
                      {evt.detail}
                    </span>
                  )}
                </div>
                <span className="text-xs text-[var(--text-secondary)] shrink-0">
                  {formatTimestamp(evt.timestamp)}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Description */}
      {issue.description && (
        <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
          <h3 className="text-xs font-medium text-[var(--text-secondary)] mb-2">
            Description
          </h3>
          <p className="text-sm whitespace-pre-wrap">{issue.description}</p>
        </div>
      )}

      {/* Labels */}
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

      {/* Actions */}
      <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
        <h3 className="text-xs font-medium text-[var(--text-secondary)] mb-3">
          Actions
        </h3>
        <div className="flex gap-2 flex-wrap">
          {["backlog", "ready", "in_progress", "done", "cancelled"].map(
            (s) => (
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
                {s
                  .replace("_", " ")
                  .replace(/\b\w/g, (c) => c.toUpperCase())}
              </button>
            ),
          )}
        </div>
      </div>

      {/* Timestamps */}
      <div className="text-xs text-[var(--text-secondary)] flex gap-4">
        <span>Created: {new Date(issue.created_at).toLocaleString()}</span>
        <span>Updated: {new Date(issue.updated_at).toLocaleString()}</span>
      </div>
    </div>
  );
}

// ── Helper components ──

function ProgressMetric({
  label,
  current,
  total,
}: {
  label: string;
  current: number;
  total: number;
}) {
  const pct = total > 0 ? Math.round((current / total) * 100) : 0;
  const color =
    total === 0
      ? "var(--text-secondary)"
      : pct === 100
        ? "var(--accent-green)"
        : pct >= 50
          ? "var(--accent-yellow)"
          : "var(--accent-red)";

  return (
    <div>
      <div className="flex items-baseline justify-between mb-2">
        <span className="text-xs text-[var(--text-secondary)]">{label}</span>
        <span className="text-sm font-mono font-medium" style={{ color }}>
          {current}/{total}
        </span>
      </div>
      <div className="w-full h-2 bg-[var(--bg-tertiary)] rounded-full overflow-hidden">
        <div
          className="h-full rounded-full transition-all"
          style={{ width: total > 0 ? `${pct}%` : "0%", background: color }}
        />
      </div>
    </div>
  );
}

function MiniBar({ checked, total }: { checked: number; total: number }) {
  const pct = Math.round((checked / total) * 100);
  const color =
    pct === 100
      ? "var(--accent-green)"
      : pct >= 50
        ? "var(--accent-yellow)"
        : "var(--accent-red)";
  return (
    <div className="flex items-center gap-1.5 mt-1">
      <div className="flex-1 h-1 bg-[var(--bg-tertiary)] rounded-full overflow-hidden">
        <div
          className="h-full rounded-full transition-all"
          style={{ width: `${pct}%`, background: color }}
        />
      </div>
      <span className="text-[10px] font-mono shrink-0" style={{ color }}>
        {checked}/{total}
      </span>
    </div>
  );
}

function EventIcon({ type }: { type: string }) {
  switch (type) {
    case "doc_created":
      return <FileText size={12} className="text-[var(--accent-green)]" />;
    case "doc_updated":
      return <FileText size={12} className="text-[var(--accent-yellow)]" />;
    case "commit_linked":
      return <GitCommit size={12} className="text-[var(--accent)]" />;
    default:
      return <ListChecks size={12} className="text-[var(--text-secondary)]" />;
  }
}

function formatTimestamp(ts: string): string {
  try {
    const d = new Date(ts);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffMin = Math.floor(diffMs / 60000);
    if (diffMin < 1) return "just now";
    if (diffMin < 60) return `${diffMin}m ago`;
    const diffHr = Math.floor(diffMin / 60);
    if (diffHr < 24) return `${diffHr}h ago`;
    const diffDay = Math.floor(diffHr / 24);
    if (diffDay < 7) return `${diffDay}d ago`;
    return d.toLocaleDateString();
  } catch {
    return ts;
  }
}
