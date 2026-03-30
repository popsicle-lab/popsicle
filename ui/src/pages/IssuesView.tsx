import { useEffect, useState, useCallback } from "react";
import {
  listIssues,
  listPipelines,
  createIssue,
  getIssueProgress,
  listUserStories,
  listTestCases,
  listBugs,
  type IssueInfo,
  type IssueProgress,
  type PipelineInfo,
} from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import {
  ClipboardList,
  ArrowRight,
  Plus,
  X,
  BookOpen,
  FlaskConical,
  Bug,
  MessageCircle,
} from "lucide-react";
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

interface IssueCounts {
  stories: number;
  tests: number;
  bugs: number;
  discussions: number;
}

export function IssuesView({ setPage }: Props) {
  const [issues, setIssues] = useState<IssueInfo[]>([]);
  const [progressMap, setProgressMap] = useState<
    Record<string, IssueProgress>
  >({});
  const [countsMap, setCountsMap] = useState<Record<string, IssueCounts>>({});
  const [typeFilter, setTypeFilter] = useState<string>("all");
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [showCreate, setShowCreate] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(() => {
    listIssues({
      issueType: typeFilter === "all" ? undefined : typeFilter,
      status: statusFilter === "all" ? undefined : statusFilter,
    })
      .then((list) => {
        setIssues(list);

        const active = list.filter((i) => i.status === "in_progress" || i.status === "ready");
        Promise.all(
          active.map((i) =>
            getIssueProgress(i.key)
              .then((p) => [i.key, p] as const)
              .catch(() => null)
          )
        ).then((results) => {
          const map: Record<string, IssueProgress> = {};
          for (const r of results) {
            if (r) map[r[0]] = r[1];
          }
          setProgressMap(map);
        });

        Promise.all(
          list.map(async (issue) => {
            const [stories, tests, bugs, discussions] = await Promise.all([
              listUserStories({ issueId: issue.id }).catch(() => []),
              listTestCases({}).catch(() => []),
              listBugs({ issueId: issue.id }).catch(() => []),
              Promise.resolve([]),
            ]);
            return [
              issue.key,
              {
                stories: stories.length,
                tests: tests.length,
                bugs: bugs.length,
                discussions: discussions.length,
              },
            ] as const;
          })
        ).then((results) => {
          const map: Record<string, IssueCounts> = {};
          for (const [key, counts] of results) {
            map[key] = counts;
          }
          setCountsMap(map);
        });
      })
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
            {issues.map((issue) => {
              const prog = progressMap[issue.key];
              const entityCounts = countsMap[issue.key];
              return (
                <button
                  key={issue.id}
                  onClick={() =>
                    setPage({ kind: "issue", issueKey: issue.key })
                  }
                  className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors text-left"
                >
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span className="font-mono text-xs text-[var(--accent)]">
                        {issue.key}
                      </span>
                      <span className="font-medium truncate">
                        {issue.title}
                      </span>
                      <TypeBadge type={issue.issue_type} />
                      <StatusBadge status={issue.status} />
                    </div>
                    <div className="text-xs text-[var(--text-secondary)] mt-1 flex items-center gap-3">
                      <span className={priorityColors[issue.priority] || ""}>
                        {issue.priority}
                      </span>
                      {issue.pipeline && (
                        <span className="px-1.5 py-0.5 rounded bg-cyan-500/15 text-cyan-300 text-[10px] font-medium">
                          {issue.pipeline}
                        </span>
                      )}
                      {prog && prog.current_stage && (
                        <span className="text-[var(--accent-purple)]">
                          {prog.current_stage}
                        </span>
                      )}
                      <span>
                        {new Date(issue.created_at).toLocaleDateString()}
                      </span>
                    </div>

                    {/* Entity counts + progress */}
                    <div className="flex items-center gap-3 mt-1.5">
                      {entityCounts && (
                        <div className="flex items-center gap-2">
                          <EntityBadge
                            icon={BookOpen}
                            count={entityCounts.stories}
                            color="text-blue-400"
                          />
                          <EntityBadge
                            icon={FlaskConical}
                            count={entityCounts.tests}
                            color="text-green-400"
                          />
                          <EntityBadge
                            icon={Bug}
                            count={entityCounts.bugs}
                            color="text-red-400"
                          />
                          <EntityBadge
                            icon={MessageCircle}
                            count={entityCounts.discussions}
                            color="text-purple-400"
                          />
                        </div>
                      )}
                      {prog && prog.stages_total > 0 && (
                        <IssueProgressBar progress={prog} />
                      )}
                    </div>
                  </div>
                  <ArrowRight
                    size={16}
                    className="text-[var(--text-secondary)] shrink-0 ml-2"
                  />
                </button>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

function EntityBadge({
  icon: Icon,
  count,
  color,
}: {
  icon: typeof BookOpen;
  count: number;
  color: string;
}) {
  if (count === 0) return null;
  return (
    <span className={`inline-flex items-center gap-0.5 text-[11px] ${color}`}>
      <Icon size={11} />
      {count}
    </span>
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

function IssueProgressBar({ progress }: { progress: IssueProgress }) {
  const stagePct =
    progress.stages_total > 0
      ? Math.round(
          (progress.stages_completed / progress.stages_total) * 100
        )
      : 0;
  const color =
    stagePct === 100
      ? "var(--accent-green)"
      : stagePct >= 50
        ? "var(--accent-yellow)"
        : "var(--accent)";

  return (
    <div className="flex items-center gap-2">
      <div className="flex-1 h-1 bg-[var(--bg-tertiary)] rounded-full overflow-hidden max-w-[140px]">
        <div
          className="h-full rounded-full transition-all"
          style={{ width: `${stagePct}%`, background: color }}
        />
      </div>
      <span className="text-[10px] font-mono text-[var(--text-secondary)]">
        {progress.stages_completed}/{progress.stages_total}
      </span>
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
  const [topicName, setTopicName] = useState("");
  const [issueType, setIssueType] = useState("product");
  const [priority, setPriority] = useState("medium");
  const [pipeline, setPipeline] = useState("");
  const [description, setDescription] = useState("");
  const [pipelines, setPipelines] = useState<PipelineInfo[]>([]);
  const [submitting, setSubmitting] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);

  useEffect(() => {
    listPipelines().then(setPipelines).catch(() => {});
  }, []);

  const submit = async () => {
    if (!title.trim() || !topicName.trim()) return;
    setSubmitting(true);
    setFormError(null);
    try {
      await createIssue({
        issueType,
        title: title.trim(),
        topicName: topicName.trim(),
        description: description.trim() || undefined,
        priority,
        pipeline: pipeline || undefined,
      });
      onCreated();
    } catch (e: unknown) {
      setFormError(e?.toString() ?? "Unknown error");
    } finally {
      setSubmitting(false);
    }
  };

  const defaultPipelines: Record<string, string> = {
    product: "full-sdlc",
    technical: "tech-sdlc",
    bug: "test-only",
    idea: "design-only",
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

      {formError && (
        <div className="text-red-400 text-sm bg-red-500/10 p-2 rounded">
          {formError}
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
        <div>
          <label className="block text-xs text-[var(--text-secondary)] mb-1">
            Topic
          </label>
          <input
            value={topicName}
            onChange={(e) => setTopicName(e.target.value)}
            placeholder="Topic name (e.g. jwt-migration)"
            className="w-full px-3 py-2 rounded-lg bg-[var(--bg-primary)] border border-[var(--border)] text-sm focus:outline-none focus:border-[var(--accent)]"
          />
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4">
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
          Pipeline
          <span className="ml-1 text-[var(--text-secondary)]/60">
            (optional — overrides auto-recommendation)
          </span>
        </label>
        <select
          value={pipeline}
          onChange={(e) => setPipeline(e.target.value)}
          className="w-full px-3 py-2 rounded-lg bg-[var(--bg-primary)] border border-[var(--border)] text-sm focus:outline-none focus:border-[var(--accent)]"
        >
          <option value="">
            Auto — {defaultPipelines[issueType] || "recommender"}
          </option>
          {pipelines.map((p) => (
            <option key={p.name} value={p.name}>
              {p.name} — {p.description} ({p.stages.length} stages)
            </option>
          ))}
        </select>
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
          disabled={!title.trim() || !topicName.trim() || submitting}
          className="px-4 py-2 rounded-lg bg-[var(--accent)] text-white text-sm font-medium hover:opacity-90 transition-opacity disabled:opacity-50"
        >
          {submitting ? "Creating..." : "Create Issue"}
        </button>
      </div>
    </div>
  );
}
