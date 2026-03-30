import { useEffect, useState } from "react";
import {
  listPipelineRuns,
  listDocuments,
  listIssues,
  listSpecs,
  getIssueProgress,
  getGitStatus,
  getProjectContext,
  type PipelineRunInfo,
  type DocInfo,
  type GitStatusInfo,
  type IssueInfo,
  type IssueProgress,
  type ProjectContextInfo,
  type SpecInfo,
} from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import {
  GitBranch,
  FileText,
  ArrowRight,
  GitCommit,
  Copy,
  Check,
  Zap,
  Terminal,
  ClipboardList,
  Tags,
  ChevronDown,
  ChevronRight,
  Cpu,
  FolderTree,
  Wrench,
  Package,
  StickyNote,
  Layers,
  Search,
} from "lucide-react";
import type { Page } from "../App";

interface Props {
  setPage: (p: Page) => void;
}

export function Dashboard({ setPage }: Props) {
  const [runs, setRuns] = useState<PipelineRunInfo[]>([]);
  const [docs, setDocs] = useState<DocInfo[]>([]);
  const [issues, setIssues] = useState<IssueInfo[]>([]);
  const [specs, setSpecs] = useState<SpecInfo[]>([]);
  const [activeProgress, setActiveProgress] = useState<
    { issue: IssueInfo; progress: IssueProgress }[]
  >([]);
  const [gitStatus, setGitStatus] = useState<GitStatusInfo | null>(null);
  const [projectCtx, setProjectCtx] = useState<ProjectContextInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([
      listPipelineRuns(),
      listDocuments(),
      listIssues(),
      listSpecs().catch(() => []),
      getGitStatus().catch(() => null),
      getProjectContext().catch(() => null),
    ])
      .then(([r, d, iss, t, g, pc]) => {
        setRuns(r);
        setDocs(d);
        setIssues(iss);
        setSpecs(t);
        setGitStatus(g);
        setProjectCtx(pc);

        const active = iss.filter((i) => i.status === "in_progress");
        Promise.all(
          active.map((i) =>
            getIssueProgress(i.key)
              .then((p) => ({ issue: i, progress: p }))
              .catch(() => null),
          ),
        ).then((results) => {
          setActiveProgress(
            results.filter(
              (r): r is { issue: IssueInfo; progress: IssueProgress } =>
                r !== null,
            ),
          );
        });
      })
      .catch((e) => setError(e?.toString()));
  }, []);

  if (error) {
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );
  }

  const activeIssueCount = issues.filter(
    (i) => i.status === "in_progress",
  ).length;

  const statusCounts = docs.reduce(
    (acc, d) => {
      acc[d.status] = (acc[d.status] || 0) + 1;
      return acc;
    },
    {} as Record<string, number>,
  );

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">Dashboard</h2>

      {/* Stats */}
      <div className="grid grid-cols-5 gap-4">
        <StatCard
          icon={<ClipboardList size={20} />}
          label="Active Issues"
          value={activeIssueCount}
          color="var(--accent-purple)"
        />
        <StatCard
          icon={<Tags size={20} />}
          label="Specs"
          value={specs.length}
          color="var(--accent-cyan, #06b6d4)"
        />
        <StatCard
          icon={<GitBranch size={20} />}
          label="Pipeline Runs"
          value={runs.length}
          color="var(--accent)"
        />
        <StatCard
          icon={<FileText size={20} />}
          label="Documents"
          value={docs.length}
          color="var(--accent-green)"
        />
        <StatCard
          icon={<GitCommit size={20} />}
          label="Tracked Commits"
          value={gitStatus?.total_commits ?? 0}
          color="var(--accent-yellow)"
        />
      </div>

      {/* Project Context */}
      {projectCtx?.available && projectCtx.content && (
        <ProjectContextCard content={projectCtx.content} />
      )}

      {/* Active Issues in Progress */}
      {activeProgress.length > 0 && (
        <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
          <div className="px-4 py-3 border-b border-[var(--border)] flex items-center justify-between">
            <div className="flex items-center gap-2">
              <ClipboardList
                size={16}
                className="text-[var(--accent-purple)]"
              />
              <h3 className="font-medium text-sm">Issues in Progress</h3>
            </div>
            <button
              onClick={() => setPage({ kind: "issues" })}
              className="text-xs text-[var(--accent)] hover:underline"
            >
              View all →
            </button>
          </div>
          <div className="divide-y divide-[var(--border)]">
            {activeProgress.map(({ issue, progress }) => {
              const stagePct =
                progress.stages_total > 0
                  ? Math.round(
                      (progress.stages_completed / progress.stages_total) * 100,
                    )
                  : 0;
              const color =
                stagePct === 100
                  ? "var(--accent-green)"
                  : stagePct >= 50
                    ? "var(--accent-yellow)"
                    : "var(--accent)";

              return (
                <button
                  key={issue.id}
                  onClick={() =>
                    setPage({ kind: "issue", issueKey: issue.key })
                  }
                  className="w-full px-4 py-3 hover:bg-[var(--bg-tertiary)] transition-colors text-left"
                >
                  <div className="flex items-center justify-between mb-1.5">
                    <div className="flex items-center gap-2 min-w-0">
                      <span className="font-mono text-xs text-[var(--accent)]">
                        {issue.key}
                      </span>
                      <span className="font-medium text-sm truncate">
                        {issue.title}
                      </span>
                    </div>
                    <div className="flex items-center gap-2 shrink-0 ml-3">
                      {progress.current_stage && (
                        <span className="text-xs text-[var(--accent-purple)]">
                          {progress.current_stage}
                        </span>
                      )}
                      <span
                        className="text-xs font-mono"
                        style={{ color }}
                      >
                        {progress.stages_completed}/{progress.stages_total}
                      </span>
                    </div>
                  </div>
                  <div className="w-full h-1.5 bg-[var(--bg-tertiary)] rounded-full overflow-hidden">
                    <div
                      className="h-full rounded-full transition-all"
                      style={{
                        width: `${stagePct}%`,
                        background: color,
                      }}
                    />
                  </div>
                </button>
              );
            })}
          </div>
        </div>
      )}

      {/* Recent Specs */}
      {specs.length > 0 && (
        <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
          <div className="px-4 py-3 border-b border-[var(--border)] flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Tags size={16} className="text-cyan-400" />
              <h3 className="font-medium text-sm">Recent Specs</h3>
            </div>
            <button
              onClick={() => setPage({ kind: "specs" })}
              className="text-xs text-[var(--accent)] hover:underline"
            >
              View all →
            </button>
          </div>
          <div className="divide-y divide-[var(--border)]">
            {specs.slice(0, 5).map((spec) => (
              <button
                key={spec.id}
                onClick={() => setPage({ kind: "spec", specName: spec.name })}
                className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors text-left"
              >
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-medium text-sm">{spec.name}</span>
                    {spec.tags.length > 0 && spec.tags.slice(0, 3).map((tag) => (
                      <span
                        key={tag}
                        className="px-1.5 py-0.5 rounded bg-cyan-500/15 text-cyan-300 text-[10px] font-medium"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                  <div className="text-xs text-[var(--text-secondary)] mt-0.5 flex items-center gap-3">
                    <span>{spec.run_count} run{spec.run_count !== 1 ? "s" : ""}</span>
                    <span>{spec.doc_count} doc{spec.doc_count !== 1 ? "s" : ""}</span>
                    <span>{new Date(spec.created_at).toLocaleDateString()}</span>
                  </div>
                </div>
                <ArrowRight
                  size={16}
                  className="text-[var(--text-secondary)] shrink-0 ml-2"
                />
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Git Status Bar */}
      {gitStatus && (
        <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)] flex items-center gap-6">
          <div className="flex items-center gap-2 text-sm">
            <GitBranch size={14} className="text-[var(--accent)]" />
            <span className="font-mono">{gitStatus.branch}</span>
          </div>
          <div className="text-xs text-[var(--text-secondary)]">
            HEAD: {gitStatus.head}
          </div>
          <div className="text-xs">
            {gitStatus.uncommitted_changes ? (
              <span className="text-yellow-400">dirty</span>
            ) : (
              <span className="text-green-400">clean</span>
            )}
          </div>
          {gitStatus.total_commits > 0 && (
            <div className="flex items-center gap-3 ml-auto text-xs">
              <span className="text-green-400">
                {gitStatus.passed} passed
              </span>
              <span className="text-yellow-400">
                {gitStatus.pending_review} pending
              </span>
              {gitStatus.failed > 0 && (
                <span className="text-red-400">
                  {gitStatus.failed} failed
                </span>
              )}
            </div>
          )}
        </div>
      )}

      {/* Quick Actions */}
      <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
        <div className="px-4 py-3 border-b border-[var(--border)] flex items-center gap-2">
          <Terminal size={16} className="text-[var(--text-secondary)]" />
          <h3 className="font-medium text-sm">Quick Actions</h3>
        </div>
        <div className="p-4 grid grid-cols-2 gap-3">
          <QuickAction
            label="Start full pipeline"
            command='popsicle pipeline run full-sdlc --title "My Feature"'
          />
          <QuickAction
            label="Quick change (skip ceremony)"
            command='popsicle pipeline quick --title "Fix something"'
            icon={<Zap size={12} />}
          />
          <QuickAction
            label="Check next steps"
            command="popsicle pipeline next"
          />
          <QuickAction
            label="Search historical docs"
            command='popsicle context search "authentication"'
            icon={<Search size={12} />}
          />
        </div>
      </div>

      {/* Document Status Distribution */}
      {Object.keys(statusCounts).length > 0 && (
        <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
          <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-3">
            Document Status Distribution
          </h3>
          <div className="flex flex-wrap gap-3">
            {Object.entries(statusCounts).map(([status, count]) => (
              <div key={status} className="flex items-center gap-2">
                <StatusBadge status={status} />
                <span className="text-sm font-mono">{count}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Pipeline Runs */}
      <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
        <div className="px-4 py-3 border-b border-[var(--border)]">
          <h3 className="font-medium">Pipeline Runs</h3>
        </div>
        {runs.length === 0 ? (
          <div className="p-6 text-center text-[var(--text-secondary)]">
            No pipeline runs yet. Use the Quick Actions above to start one.
          </div>
        ) : (
          <div className="divide-y divide-[var(--border)]">
            {runs.map((run) => (
              <button
                key={run.id}
                onClick={() => setPage({ kind: "pipeline", runId: run.id })}
                className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors text-left"
              >
                <div>
                  <div className="flex items-center gap-2">
                    <span className="font-medium">{run.title}</span>
                    {run.pipeline_name === "quick" && (
                      <span className="inline-flex items-center gap-1 text-xs bg-yellow-500/15 text-yellow-300 px-1.5 py-0.5 rounded-full">
                        <Zap size={8} /> quick
                      </span>
                    )}
                  </div>
                  <div className="text-xs text-[var(--text-secondary)] mt-0.5">
                    {run.pipeline_name} &middot; {run.id.slice(0, 8)}
                  </div>
                </div>
                <ArrowRight
                  size={16}
                  className="text-[var(--text-secondary)]"
                />
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function StatCard({
  icon,
  label,
  value,
  color,
}: {
  icon: React.ReactNode;
  label: string;
  value: number;
  color: string;
}) {
  return (
    <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
      <div className="flex items-center gap-3">
        <div
          className="p-2 rounded-lg"
          style={{
            background: `color-mix(in srgb, ${color} 15%, transparent)`,
          }}
        >
          <span style={{ color }}>{icon}</span>
        </div>
        <div>
          <div className="text-2xl font-bold">{value}</div>
          <div className="text-xs text-[var(--text-secondary)]">{label}</div>
        </div>
      </div>
    </div>
  );
}

function QuickAction({
  label,
  command,
  icon,
}: {
  label: string;
  command: string;
  icon?: React.ReactNode;
}) {
  const [copied, setCopied] = useState(false);
  const copy = () => {
    navigator.clipboard.writeText(command);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <button
      onClick={copy}
      className="flex items-center gap-2 p-3 rounded-lg bg-[var(--bg-primary)]/50 hover:bg-[var(--bg-primary)] border border-[var(--border)] transition-colors text-left group"
    >
      <div className="flex-1 min-w-0">
        <div className="text-xs font-medium flex items-center gap-1">
          {icon}
          {label}
        </div>
        <code className="text-xs text-[var(--accent)] font-mono truncate block mt-0.5">
          {command}
        </code>
      </div>
      {copied ? (
        <Check size={14} className="text-[var(--accent-green)] shrink-0" />
      ) : (
        <Copy
          size={14}
          className="text-[var(--text-secondary)] shrink-0 opacity-0 group-hover:opacity-100 transition-opacity"
        />
      )}
    </button>
  );
}

// ── Project Context Card ──

interface ParsedSection {
  title: string;
  items: string[];
}

function parseProjectContext(content: string): ParsedSection[] {
  const sections: ParsedSection[] = [];
  let current: ParsedSection | null = null;

  for (const line of content.split("\n")) {
    const trimmed = line.trim();
    if (trimmed.startsWith("## ")) {
      if (current) sections.push(current);
      current = { title: trimmed.slice(3), items: [] };
    } else if (current && trimmed.startsWith("- ")) {
      current.items.push(trimmed.slice(2));
    }
  }
  if (current) sections.push(current);
  return sections;
}

const sectionIcons: Record<string, React.ReactNode> = {
  "Tech Stack": <Cpu size={14} />,
  "Project Structure": <FolderTree size={14} />,
  "Development Practices": <Wrench size={14} />,
  "Key Dependencies": <Package size={14} />,
  Notes: <StickyNote size={14} />,
};

function ProjectContextCard({ content }: { content: string }) {
  const [expanded, setExpanded] = useState(false);
  const sections = parseProjectContext(content);

  const summary = sections.find((s) => s.title === "Tech Stack");
  const summaryText = summary
    ? summary.items
        .filter((i) => i.startsWith("**Language**") || i.startsWith("**Build**"))
        .map((i) => i.replace(/\*\*/g, "").split(":").pop()?.trim())
        .filter(Boolean)
        .join(" · ")
    : "";

  return (
    <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors rounded-xl"
      >
        <div className="flex items-center gap-2">
          <Layers size={16} className="text-[var(--accent-purple)]" />
          <h3 className="font-medium text-sm">Project Context</h3>
          {!expanded && summaryText && (
            <span className="text-xs text-[var(--text-secondary)] ml-2">
              {summaryText}
            </span>
          )}
        </div>
        {expanded ? (
          <ChevronDown size={16} className="text-[var(--text-secondary)]" />
        ) : (
          <ChevronRight size={16} className="text-[var(--text-secondary)]" />
        )}
      </button>

      {expanded && (
        <div className="px-4 pb-4 grid grid-cols-2 gap-4">
          {sections.map((section) => {
            if (section.items.length === 0) return null;
            const icon = sectionIcons[section.title] ?? <FileText size={14} />;
            return (
              <div
                key={section.title}
                className="bg-[var(--bg-primary)]/50 rounded-lg border border-[var(--border)] p-3"
              >
                <div className="flex items-center gap-1.5 text-[var(--text-secondary)] mb-2">
                  {icon}
                  <span className="text-xs font-medium">{section.title}</span>
                </div>
                <ul className="space-y-1">
                  {section.items.map((item, i) => (
                    <li key={i} className="text-xs text-[var(--text-primary)]">
                      <ContextItem text={item} />
                    </li>
                  ))}
                </ul>
              </div>
            );
          })}
          <div className="col-span-2 text-xs text-[var(--text-secondary)] flex items-center gap-1">
            <Terminal size={12} />
            <span>
              Update with{" "}
              <code className="text-[var(--accent)] bg-[var(--bg-tertiary)] px-1 py-0.5 rounded">
                popsicle context scan --force
              </code>
            </span>
          </div>
        </div>
      )}
    </div>
  );
}

function ContextItem({ text }: { text: string }) {
  const boldRegex = /\*\*(.+?)\*\*:\s*(.*)/;
  const match = text.match(boldRegex);
  if (match) {
    return (
      <span>
        <span className="font-medium text-[var(--text-primary)]">
          {match[1]}
        </span>
        <span className="text-[var(--text-secondary)]">: {match[2]}</span>
      </span>
    );
  }
  return <span className="text-[var(--text-secondary)]">{text}</span>;
}
