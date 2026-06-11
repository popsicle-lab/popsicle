import { useCallback, useEffect, useState } from "react";
import { ArrowRight, ClipboardList, Plus, X } from "lucide-react";
import {
  createIssue,
  listIssues,
  type IssueInfo,
} from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
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

export function IssuesView({ setPage }: Props) {
  const [issues, setIssues] = useState<IssueInfo[]>([]);
  const [typeFilter, setTypeFilter] = useState("all");
  const [statusFilter, setStatusFilter] = useState("all");
  const [showCreate, setShowCreate] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(() => {
    listIssues()
      .then((list) => {
        setIssues(
          list.filter((i) => {
            if (typeFilter !== "all" && i.issue_type !== typeFilter) return false;
            if (statusFilter !== "all" && i.status !== statusFilter) return false;
            return true;
          })
        );
      })
      .catch((e) => setError(String(e)));
  }, [typeFilter, statusFilter]);

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

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <ClipboardList size={24} className="text-[var(--accent)]" />
          <h2 className="text-2xl font-bold">Issues</h2>
        </div>
        <button
          onClick={() => setShowCreate(true)}
          className="flex items-center gap-2 px-4 py-2 bg-[var(--accent)] text-[var(--bg-primary)] rounded-lg text-sm font-medium"
        >
          <Plus size={16} /> New Issue
        </button>
      </div>

      <div className="flex gap-3">
        <select
          value={typeFilter}
          onChange={(e) => setTypeFilter(e.target.value)}
          className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-lg px-3 py-1.5 text-sm"
        >
          <option value="all">All types</option>
          <option value="product">Product</option>
          <option value="technical">Technical</option>
          <option value="bug">Bug</option>
          <option value="idea">Idea</option>
        </select>
        <select
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
          className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-lg px-3 py-1.5 text-sm"
        >
          <option value="all">All statuses</option>
          <option value="backlog">Backlog</option>
          <option value="ready">Ready</option>
          <option value="in_progress">In progress</option>
          <option value="done">Done</option>
        </select>
      </div>

      {showCreate && (
        <CreateIssueForm
          onClose={() => setShowCreate(false)}
          onCreated={(issue) => {
            setShowCreate(false);
            setPage({ kind: "issue", issueKey: issue.key });
          }}
        />
      )}

      <div className="space-y-2">
        {issues.map((issue) => (
          <button
            key={issue.key}
            onClick={() => setPage({ kind: "issue", issueKey: issue.key })}
            className="w-full text-left bg-[var(--bg-secondary)] border border-[var(--border)] rounded-xl p-4 hover:border-[var(--accent)]/40 transition-colors"
          >
            <div className="flex items-start justify-between gap-4">
              <div className="min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <span className="font-mono text-sm text-[var(--accent)]">
                    {issue.key}
                  </span>
                  <span
                    className={`text-xs px-2 py-0.5 rounded-full ${typeColors[issue.issue_type] ?? "bg-gray-500/20 text-gray-300"}`}
                  >
                    {issue.issue_type}
                  </span>
                  <StatusBadge status={issue.status} />
                </div>
                <h3 className="font-medium truncate">{issue.title}</h3>
                <p className="text-xs text-[var(--text-secondary)] mt-1">
                  spec: {issue.spec_id}
                  {issue.pipeline ? ` · ${issue.pipeline}` : ""}
                </p>
              </div>
              <ArrowRight size={18} className="text-[var(--text-secondary)] shrink-0 mt-1" />
            </div>
          </button>
        ))}
        {issues.length === 0 && (
          <p className="text-[var(--text-secondary)] text-sm">No issues found.</p>
        )}
      </div>
    </div>
  );
}

function CreateIssueForm({
  onClose,
  onCreated,
}: {
  onClose: () => void;
  onCreated: (issue: IssueInfo) => void;
}) {
  const [issueType, setIssueType] = useState("product");
  const [title, setTitle] = useState("");
  const [specId, setSpecId] = useState("slice-3-cli-ux");
  const [pipeline, setPipeline] = useState("");
  const [description, setDescription] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      const issue = await createIssue({
        issueType,
        title,
        specId,
        pipeline: pipeline || undefined,
        description,
      });
      onCreated(issue);
    } catch (err: unknown) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-xl p-5">
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-medium">Create Issue</h3>
        <button onClick={onClose} className="text-[var(--text-secondary)]">
          <X size={18} />
        </button>
      </div>
      <form onSubmit={submit} className="grid gap-3">
        <div className="grid grid-cols-2 gap-3">
          <label className="text-sm">
            Type
            <select
              value={issueType}
              onChange={(e) => setIssueType(e.target.value)}
              className="mt-1 w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm"
            >
              <option value="product">Product</option>
              <option value="technical">Technical</option>
              <option value="bug">Bug</option>
              <option value="idea">Idea</option>
            </select>
          </label>
          <label className="text-sm">
            Spec ID
            <input
              value={specId}
              onChange={(e) => setSpecId(e.target.value)}
              className="mt-1 w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm"
            />
          </label>
        </div>
        <label className="text-sm">
          Title
          <input
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            required
            className="mt-1 w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm"
          />
        </label>
        <label className="text-sm">
          Pipeline (optional)
          <input
            value={pipeline}
            onChange={(e) => setPipeline(e.target.value)}
            placeholder="slice-delivery"
            className="mt-1 w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm"
          />
        </label>
        <label className="text-sm">
          Description
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={3}
            className="mt-1 w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm"
          />
        </label>
        {error && <p className="text-[var(--accent-red)] text-sm">{error}</p>}
        <button
          type="submit"
          disabled={loading || !title.trim()}
          className="px-4 py-2 bg-[var(--accent)] text-[var(--bg-primary)] rounded-lg text-sm font-medium disabled:opacity-50"
        >
          {loading ? "Creating…" : "Create"}
        </button>
      </form>
    </div>
  );
}
