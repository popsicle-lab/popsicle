import { useEffect, useState, useCallback } from "react";
import {
  listIssues,
  createIssue,
  type IssueInfo,
} from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import { ClipboardList, ArrowRight, Plus, X } from "lucide-react";
import type { Page } from "../App";

interface Props {
  setPage: (p: Page) => void;
}

const typeColors: Record<string, string> = {
  product: "bg-blue-500/20 text-blue-300",
  technical: "bg-purple-500/20 text-purple-300",
  bug: "bg-red-500/20 text-red-300",
  idea: "bg-yellow-500/20 text-yellow-300",
};

const priorityColors: Record<string, string> = {
  critical: "text-red-400",
  high: "text-orange-400",
  medium: "text-yellow-400",
  low: "text-gray-400",
};

export function IssuesView({ setPage }: Props) {
  const [issues, setIssues] = useState<IssueInfo[]>([]);
  const [typeFilter, setTypeFilter] = useState<string>("all");
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [showCreate, setShowCreate] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(() => {
    listIssues({
      issueType: typeFilter === "all" ? undefined : typeFilter,
      status: statusFilter === "all" ? undefined : statusFilter,
    })
      .then(setIssues)
      .catch((e) => setError(e?.toString()));
  }, [typeFilter, statusFilter]);

  useEffect(() => {
    load();
  }, [load]);

  if (error)
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );

  const counts = {
    total: issues.length,
    backlog: issues.filter((i) => i.status === "backlog").length,
    in_progress: issues.filter((i) => i.status === "in_progress").length,
    done: issues.filter((i) => i.status === "done").length,
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold flex items-center gap-3">
          <ClipboardList size={24} />
          Issues
        </h2>
        <button
          onClick={() => setShowCreate(true)}
          className="flex items-center gap-2 px-3 py-2 rounded-lg bg-[var(--accent)]/15 text-[var(--accent)] text-sm font-medium hover:bg-[var(--accent)]/25 transition-colors"
        >
          <Plus size={16} />
          New Issue
        </button>
      </div>

      {showCreate && (
        <CreateIssueForm
          onCreated={() => {
            setShowCreate(false);
            load();
          }}
          onCancel={() => setShowCreate(false)}
        />
      )}

      <div className="grid grid-cols-4 gap-4">
        <StatCard label="Total" value={counts.total} color="var(--accent)" />
        <StatCard
          label="Backlog"
          value={counts.backlog}
          color="var(--text-secondary)"
        />
        <StatCard
          label="In Progress"
          value={counts.in_progress}
          color="var(--accent-purple)"
        />
        <StatCard label="Done" value={counts.done} color="var(--accent-green)" />
      </div>

      <div className="flex gap-4">
        <div className="flex gap-2 items-center">
          <span className="text-xs text-[var(--text-secondary)]">Type:</span>
          {["all", "product", "technical", "bug", "idea"].map((t) => (
            <button
              key={t}
              onClick={() => setTypeFilter(t)}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                typeFilter === t
                  ? "bg-[var(--accent)]/15 text-[var(--accent)]"
                  : "bg-[var(--bg-secondary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
              }`}
            >
              {t.charAt(0).toUpperCase() + t.slice(1)}
            </button>
          ))}
        </div>
        <div className="flex gap-2 items-center">
          <span className="text-xs text-[var(--text-secondary)]">Status:</span>
          {["all", "backlog", "ready", "in_progress", "done"].map((s) => (
            <button
              key={s}
              onClick={() => setStatusFilter(s)}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                statusFilter === s
                  ? "bg-[var(--accent)]/15 text-[var(--accent)]"
                  : "bg-[var(--bg-secondary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
              }`}
            >
              {s.replace("_", " ").replace(/\b\w/g, (c) => c.toUpperCase())}
            </button>
          ))}
        </div>
      </div>

      <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
        {issues.length === 0 ? (
          <div className="p-6 text-center text-[var(--text-secondary)]">
            No issues found. Click "New Issue" to create one.
          </div>
        ) : (
          <div className="divide-y divide-[var(--border)]">
            {issues.map((issue) => (
              <button
                key={issue.id}
                onClick={() => setPage({ kind: "issue", issueKey: issue.key })}
                className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors text-left"
              >
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-mono text-xs text-[var(--accent)]">
                      {issue.key}
                    </span>
                    <span className="font-medium truncate">{issue.title}</span>
                    <TypeBadge type={issue.issue_type} />
                    <StatusBadge status={issue.status} />
                  </div>
                  <div className="text-xs text-[var(--text-secondary)] mt-0.5 flex items-center gap-3">
                    <span className={priorityColors[issue.priority] || ""}>
                      {issue.priority}
                    </span>
                    {issue.labels.length > 0 && (
                      <span>{issue.labels.join(", ")}</span>
                    )}
                    <span>
                      {new Date(issue.created_at).toLocaleDateString()}
                    </span>
                  </div>
                </div>
                <ArrowRight
                  size={16}
                  className="text-[var(--text-secondary)] shrink-0 ml-2"
                />
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function TypeBadge({ type }: { type: string }) {
  const color = typeColors[type] || "bg-gray-500/20 text-gray-300";
  return (
    <span
      className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${color}`}
    >
      {type}
    </span>
  );
}

function StatCard({
  label,
  value,
  color,
}: {
  label: string;
  value: number;
  color: string;
}) {
  return (
    <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
      <div
        className="text-2xl font-bold"
        style={{ color: value > 0 ? color : undefined }}
      >
        {value}
      </div>
      <div className="text-xs text-[var(--text-secondary)]">{label}</div>
    </div>
  );
}

function CreateIssueForm({
  onCreated,
  onCancel,
}: {
  onCreated: () => void;
  onCancel: () => void;
}) {
  const [title, setTitle] = useState("");
  const [issueType, setIssueType] = useState("product");
  const [priority, setPriority] = useState("medium");
  const [description, setDescription] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const submit = async () => {
    if (!title.trim()) return;
    setSubmitting(true);
    setError(null);
    try {
      await createIssue({
        issueType,
        title: title.trim(),
        description: description.trim() || undefined,
        priority,
      });
      onCreated();
    } catch (e: unknown) {
      setError(e?.toString() ?? "Unknown error");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)] p-4 space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="font-medium">New Issue</h3>
        <button
          onClick={onCancel}
          className="text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
        >
          <X size={18} />
        </button>
      </div>

      {error && (
        <div className="text-red-400 text-sm bg-red-500/10 p-2 rounded">
          {error}
        </div>
      )}

      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="block text-xs text-[var(--text-secondary)] mb-1">
            Title
          </label>
          <input
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="Issue title..."
            className="w-full px-3 py-2 rounded-lg bg-[var(--bg-primary)] border border-[var(--border)] text-sm focus:outline-none focus:border-[var(--accent)]"
          />
        </div>
        <div className="flex gap-4">
          <div className="flex-1">
            <label className="block text-xs text-[var(--text-secondary)] mb-1">
              Type
            </label>
            <select
              value={issueType}
              onChange={(e) => setIssueType(e.target.value)}
              className="w-full px-3 py-2 rounded-lg bg-[var(--bg-primary)] border border-[var(--border)] text-sm focus:outline-none focus:border-[var(--accent)]"
            >
              <option value="product">Product</option>
              <option value="technical">Technical</option>
              <option value="bug">Bug</option>
              <option value="idea">Idea</option>
            </select>
          </div>
          <div className="flex-1">
            <label className="block text-xs text-[var(--text-secondary)] mb-1">
              Priority
            </label>
            <select
              value={priority}
              onChange={(e) => setPriority(e.target.value)}
              className="w-full px-3 py-2 rounded-lg bg-[var(--bg-primary)] border border-[var(--border)] text-sm focus:outline-none focus:border-[var(--accent)]"
            >
              <option value="critical">Critical</option>
              <option value="high">High</option>
              <option value="medium">Medium</option>
              <option value="low">Low</option>
            </select>
          </div>
        </div>
      </div>

      <div>
        <label className="block text-xs text-[var(--text-secondary)] mb-1">
          Description (optional)
        </label>
        <textarea
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          rows={3}
          placeholder="Describe the issue..."
          className="w-full px-3 py-2 rounded-lg bg-[var(--bg-primary)] border border-[var(--border)] text-sm focus:outline-none focus:border-[var(--accent)] resize-none"
        />
      </div>

      <div className="flex justify-end">
        <button
          onClick={submit}
          disabled={!title.trim() || submitting}
          className="px-4 py-2 rounded-lg bg-[var(--accent)] text-white text-sm font-medium hover:opacity-90 transition-opacity disabled:opacity-50"
        >
          {submitting ? "Creating..." : "Create Issue"}
        </button>
      </div>
    </div>
  );
}
