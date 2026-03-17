import { useEffect, useState } from "react";
import {
  getIssue,
  startIssue,
  updateIssue,
  getIssueProgress,
  getActivity,
  listUserStories,
  listTestCases,
  listBugs,
  listDiscussions,
  type IssueFull,
  type IssueProgress,
  type ActivityEvent,
  type UserStoryInfo,
  type TestCaseInfo,
  type BugInfo,
  type DiscussionInfo,
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
  Workflow,
  BookOpen,
  FlaskConical,
  Bug,
  MessageCircle,
  ArrowRight,
  Users,
} from "lucide-react";
import type { Page } from "../App";

type Tab = "overview" | "stories" | "tests" | "bugs" | "discussions";

interface Props {
  issueKey: string;
  setPage: (p: Page) => void;
  initialTab?: Tab;
}

const typeColors: Record<string, string> = {
  product: "bg-blue-500/20 text-blue-300",
  technical: "bg-purple-500/20 text-purple-300",
  bug: "bg-red-500/20 text-red-300",
  idea: "bg-yellow-500/20 text-yellow-300",
};

const tabDefs: { key: Tab; label: string; icon: typeof BookOpen }[] = [
  { key: "overview", label: "Overview", icon: ClipboardList },
  { key: "stories", label: "Stories", icon: BookOpen },
  { key: "tests", label: "Tests", icon: FlaskConical },
  { key: "bugs", label: "Bugs", icon: Bug },
  { key: "discussions", label: "Discussions", icon: MessageCircle },
];

export function IssueDetailView({ issueKey, setPage, initialTab }: Props) {
  const [issue, setIssue] = useState<IssueFull | null>(null);
  const [progress, setProgress] = useState<IssueProgress | null>(null);
  const [activity, setActivity] = useState<ActivityEvent[]>([]);
  const [stories, setStories] = useState<UserStoryInfo[]>([]);
  const [testCases, setTestCases] = useState<TestCaseInfo[]>([]);
  const [bugs, setBugs] = useState<BugInfo[]>([]);
  const [discussions, setDiscussions] = useState<DiscussionInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [starting, setStarting] = useState(false);
  const [activeTab, setActiveTab] = useState<Tab>(initialTab ?? "overview");

  useEffect(() => {
    let issueData: IssueFull;
    getIssue(issueKey)
      .then((i) => {
        issueData = i;
        setIssue(i);
        return getIssueProgress(issueKey);
      })
      .then((p) => {
        setProgress(p);

        const runId = p.pipeline_run_id ?? undefined;

        const loads: Promise<void>[] = [];
        if (runId) {
          loads.push(getActivity(runId).then(setActivity));
          loads.push(
            listDiscussions({ runId }).then(setDiscussions)
          );
        }
        loads.push(
          listUserStories({ issueId: issueData.id }).then(setStories)
        );
        loads.push(
          listBugs({ issueId: issueData.id }).then(setBugs)
        );
        loads.push(
          listTestCases({ runId }).then(setTestCases)
        );

        return Promise.all(loads);
      })
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

  const counts = {
    stories: stories.length,
    tests: testCases.length,
    bugs: bugs.length,
    discussions: discussions.length,
  };

  return (
    <div className="space-y-6 max-w-6xl">
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
            {issue.pipeline && (
              <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-cyan-500/15 text-cyan-300">
                <Workflow size={10} />
                {issue.pipeline}
              </span>
            )}
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

      {/* Tab Navigation */}
      <div className="flex border-b border-[var(--border)]">
        {tabDefs.map((tab) => {
          const count = counts[tab.key as keyof typeof counts];
          const isActive = activeTab === tab.key;
          return (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key)}
              className={`flex items-center gap-2 px-4 py-2.5 text-sm font-medium border-b-2 transition-colors ${
                isActive
                  ? "border-[var(--accent)] text-[var(--accent)]"
                  : "border-transparent text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:border-[var(--border)]"
              }`}
            >
              <tab.icon size={15} />
              {tab.label}
              {count !== undefined && count > 0 && (
                <span
                  className={`inline-flex items-center justify-center min-w-[20px] h-5 px-1.5 rounded-full text-[11px] font-medium ${
                    isActive
                      ? "bg-[var(--accent)]/15 text-[var(--accent)]"
                      : "bg-[var(--bg-tertiary)] text-[var(--text-secondary)]"
                  }`}
                >
                  {count}
                </span>
              )}
            </button>
          );
        })}
      </div>

      {/* Tab Content */}
      {activeTab === "overview" && (
        <OverviewTab
          issue={issue}
          progress={progress}
          activity={activity}
          counts={counts}
          setPage={setPage}
          onStatusChange={handleStatusChange}
        />
      )}
      {activeTab === "stories" && (
        <StoriesTab stories={stories} setPage={setPage} issueKey={issueKey} />
      )}
      {activeTab === "tests" && (
        <TestsTab testCases={testCases} setPage={setPage} issueKey={issueKey} />
      )}
      {activeTab === "bugs" && (
        <BugsTab bugs={bugs} setPage={setPage} issueKey={issueKey} />
      )}
      {activeTab === "discussions" && (
        <DiscussionsTab
          discussions={discussions}
          setPage={setPage}
          issueKey={issueKey}
        />
      )}
    </div>
  );
}

// ── Overview Tab ──

function OverviewTab({
  issue,
  progress,
  activity,
  counts,
  setPage,
  onStatusChange,
}: {
  issue: IssueFull;
  progress: IssueProgress | null;
  activity: ActivityEvent[];
  counts: { stories: number; tests: number; bugs: number; discussions: number };
  setPage: (p: Page) => void;
  onStatusChange: (s: string) => void;
}) {
  return (
    <div className="space-y-6">
      {/* Entity Summary Cards */}
      <div className="grid grid-cols-4 gap-4">
        <SummaryCard
          icon={BookOpen}
          label="Stories"
          count={counts.stories}
          color="var(--accent)"
        />
        <SummaryCard
          icon={FlaskConical}
          label="Tests"
          count={counts.tests}
          color="var(--accent-green)"
        />
        <SummaryCard
          icon={Bug}
          label="Bugs"
          count={counts.bugs}
          color="var(--accent-red)"
        />
        <SummaryCard
          icon={MessageCircle}
          label="Discussions"
          count={counts.discussions}
          color="var(--accent-purple)"
        />
      </div>

      {/* Progress Overview */}
      {progress && progress.pipeline_run_id && (
        <div className="bg-[var(--bg-secondary)] rounded-xl p-5 border border-[var(--border)]">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-sm font-medium text-[var(--text-secondary)]">
              Progress
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
      {progress && progress.stage_summaries.length > 0 && (
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
            {progress.stage_summaries.map((stage, idx) => (
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
                      <span className="text-xs font-medium">{stage.name}</span>
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
                {idx < progress.stage_summaries.length - 1 && (
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

      {/* Description */}
      {issue.description && (
        <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
          <h3 className="text-xs font-medium text-[var(--text-secondary)] mb-2">
            Description
          </h3>
          <p className="text-sm whitespace-pre-wrap">{issue.description}</p>
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
            {activity.slice(0, 10).map((evt, idx) => (
              <div key={idx} className="px-4 py-2.5 flex items-center gap-3">
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
                onClick={() => onStatusChange(s)}
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
            )
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

// ── Stories Tab ──

const storyPriorityColors: Record<string, string> = {
  critical: "text-red-400",
  high: "text-orange-400",
  medium: "text-yellow-400",
  low: "text-gray-400",
};

function StoriesTab({
  stories,
  setPage,
  issueKey,
}: {
  stories: UserStoryInfo[];
  setPage: (p: Page) => void;
  issueKey: string;
}) {
  if (stories.length === 0) {
    return (
      <EmptyState
        icon={BookOpen}
        title="No User Stories"
        description="User stories linked to this issue will appear here. Extract them from PRD documents with the CLI."
      />
    );
  }

  const totalAc = stories.reduce((sum, s) => sum + s.ac_count, 0);
  const verifiedAc = stories.reduce((sum, s) => sum + s.ac_verified, 0);

  return (
    <div className="space-y-4">
      {/* Mini stats */}
      <div className="flex gap-4">
        <MiniStat label="Total" value={stories.length} />
        <MiniStat label="Accepted" value={stories.filter((s) => s.status === "accepted").length} />
        <MiniStat label="Verified" value={stories.filter((s) => s.status === "verified").length} />
        <MiniStat
          label="AC Verified"
          value={`${verifiedAc}/${totalAc}`}
          highlight={totalAc > 0 && verifiedAc === totalAc}
        />
      </div>

      <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
        <div className="divide-y divide-[var(--border)]">
          {stories.map((story) => (
            <button
              key={story.id}
              onClick={() =>
                setPage({ kind: "story", storyKey: story.key, fromIssue: issueKey } as Page)
              }
              className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors text-left"
            >
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <span className="font-mono text-xs text-[var(--accent)]">
                    {story.key}
                  </span>
                  <span className="font-medium truncate">{story.title}</span>
                  <StatusBadge status={story.status} />
                </div>
                <div className="text-xs text-[var(--text-secondary)] mt-0.5 flex items-center gap-3">
                  <span className={storyPriorityColors[story.priority] || ""}>
                    {story.priority}
                  </span>
                  {story.persona && (
                    <span className="italic">as {story.persona}</span>
                  )}
                  <span>
                    AC: {story.ac_verified}/{story.ac_count}
                  </span>
                </div>
                {story.ac_count > 0 && (
                  <div className="flex items-center gap-2 mt-1.5">
                    <div className="flex-1 h-1 bg-[var(--bg-tertiary)] rounded-full overflow-hidden max-w-[200px]">
                      <div
                        className="h-full rounded-full transition-all"
                        style={{
                          width: `${(story.ac_verified / story.ac_count) * 100}%`,
                          background:
                            story.ac_verified === story.ac_count
                              ? "var(--accent-green)"
                              : "var(--accent-yellow)",
                        }}
                      />
                    </div>
                    <span className="text-[10px] font-mono text-[var(--text-secondary)]">
                      {story.ac_verified}/{story.ac_count} verified
                    </span>
                  </div>
                )}
              </div>
              <ArrowRight
                size={16}
                className="text-[var(--text-secondary)] shrink-0 ml-2"
              />
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

// ── Tests Tab ──

const testTypeColors: Record<string, string> = {
  unit: "bg-blue-500/20 text-blue-300",
  api: "bg-purple-500/20 text-purple-300",
  e2e: "bg-green-500/20 text-green-300",
  ui: "bg-pink-500/20 text-pink-300",
};

const testPriorityColors: Record<string, string> = {
  p0: "text-red-400 font-bold",
  p1: "text-orange-400",
  p2: "text-yellow-400",
};

function TestsTab({
  testCases,
  setPage,
  issueKey,
}: {
  testCases: TestCaseInfo[];
  setPage: (p: Page) => void;
  issueKey: string;
}) {
  const [typeFilter, setTypeFilter] = useState("all");

  const filtered =
    typeFilter === "all"
      ? testCases
      : testCases.filter((tc) => tc.test_type === typeFilter);

  if (testCases.length === 0) {
    return (
      <EmptyState
        icon={FlaskConical}
        title="No Test Cases"
        description="Test cases linked to this issue will appear here. Extract them from test spec documents with the CLI."
      />
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex gap-4 items-center justify-between">
        <div className="flex gap-4">
          <MiniStat label="Total" value={testCases.length} />
          <MiniStat label="Automated" value={testCases.filter((t) => t.status === "automated").length} />
          <MiniStat label="Ready" value={testCases.filter((t) => t.status === "ready").length} />
        </div>
        <div className="flex gap-1.5">
          {["all", "unit", "api", "e2e", "ui"].map((t) => (
            <button
              key={t}
              onClick={() => setTypeFilter(t)}
              className={`px-2.5 py-1 rounded-md text-xs font-medium transition-colors ${
                typeFilter === t
                  ? "bg-[var(--accent)]/15 text-[var(--accent)]"
                  : "text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
              }`}
            >
              {t.toUpperCase()}
            </button>
          ))}
        </div>
      </div>

      <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
        <div className="divide-y divide-[var(--border)]">
          {filtered.map((tc) => (
            <button
              key={tc.id}
              onClick={() =>
                setPage({ kind: "testcase", testCaseKey: tc.key, fromIssue: issueKey } as Page)
              }
              className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors text-left"
            >
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <span className="font-mono text-xs text-[var(--accent)]">
                    {tc.key}
                  </span>
                  <span className="font-medium truncate">{tc.title}</span>
                  <span
                    className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${
                      testTypeColors[tc.test_type] ||
                      "bg-gray-500/20 text-gray-300"
                    }`}
                  >
                    {tc.test_type}
                  </span>
                  <StatusBadge status={tc.status} />
                </div>
                <div className="text-xs text-[var(--text-secondary)] mt-0.5 flex items-center gap-3">
                  <span
                    className={testPriorityColors[tc.priority_level] || ""}
                  >
                    {tc.priority_level.toUpperCase()}
                  </span>
                  <span>
                    {new Date(tc.created_at).toLocaleDateString()}
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
      </div>
    </div>
  );
}

// ── Bugs Tab ──

const bugSeverityColors: Record<string, string> = {
  blocker: "text-red-400 font-bold",
  critical: "text-red-400",
  major: "text-orange-400",
  minor: "text-yellow-400",
  trivial: "text-gray-400",
};

const bugSourceColors: Record<string, string> = {
  manual: "bg-gray-500/20 text-gray-300",
  test_failure: "bg-red-500/20 text-red-300",
  doc_extracted: "bg-blue-500/20 text-blue-300",
};

function BugsTab({
  bugs,
  setPage,
  issueKey,
}: {
  bugs: BugInfo[];
  setPage: (p: Page) => void;
  issueKey: string;
}) {
  if (bugs.length === 0) {
    return (
      <EmptyState
        icon={Bug}
        title="No Bugs"
        description="Bugs linked to this issue will appear here. They can be created manually or extracted from test failures."
      />
    );
  }

  const open = bugs.filter(
    (b) =>
      b.status === "open" ||
      b.status === "confirmed" ||
      b.status === "in_progress"
  ).length;
  const fixed = bugs.filter(
    (b) => b.status === "fixed" || b.status === "verified"
  ).length;

  return (
    <div className="space-y-4">
      <div className="flex gap-4">
        <MiniStat label="Total" value={bugs.length} />
        <MiniStat label="Open" value={open} warn />
        <MiniStat label="Fixed" value={fixed} />
      </div>

      <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
        <div className="divide-y divide-[var(--border)]">
          {bugs.map((bug) => (
            <button
              key={bug.id}
              onClick={() =>
                setPage({ kind: "bug", bugKey: bug.key, fromIssue: issueKey } as Page)
              }
              className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors text-left"
            >
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <span className="font-mono text-xs text-[var(--accent)]">
                    {bug.key}
                  </span>
                  <span className="font-medium truncate">{bug.title}</span>
                  <StatusBadge status={bug.status} />
                  <span
                    className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${
                      bugSourceColors[bug.source] ||
                      "bg-gray-500/20 text-gray-300"
                    }`}
                  >
                    {bug.source.replace("_", " ")}
                  </span>
                </div>
                <div className="text-xs text-[var(--text-secondary)] mt-0.5 flex items-center gap-3">
                  <span className={bugSeverityColors[bug.severity] || ""}>
                    {bug.severity}
                  </span>
                  <span>{bug.priority}</span>
                  <span>
                    {new Date(bug.created_at).toLocaleDateString()}
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
      </div>
    </div>
  );
}

// ── Discussions Tab ──

function DiscussionsTab({
  discussions,
  setPage,
  issueKey,
}: {
  discussions: DiscussionInfo[];
  setPage: (p: Page) => void;
  issueKey: string;
}) {
  if (discussions.length === 0) {
    return (
      <EmptyState
        icon={MessageCircle}
        title="No Discussions"
        description="Discussions from multi-role review sessions will appear here when the pipeline runs."
      />
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex gap-4">
        <MiniStat label="Total" value={discussions.length} />
        <MiniStat
          label="Active"
          value={discussions.filter((d) => d.status === "active").length}
        />
        <MiniStat
          label="Concluded"
          value={discussions.filter((d) => d.status === "concluded").length}
        />
      </div>

      <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
        <div className="divide-y divide-[var(--border)]">
          {discussions.map((disc) => (
            <button
              key={disc.id}
              onClick={() =>
                setPage({
                  kind: "discussion",
                  discussionId: disc.id,
                  fromIssue: issueKey,
                } as Page)
              }
              className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors text-left"
            >
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <span className="font-medium truncate">{disc.topic}</span>
                  <StatusBadge status={disc.status} />
                </div>
                <div className="text-xs text-[var(--text-secondary)] mt-0.5 flex items-center gap-3">
                  <span className="font-mono">{disc.skill}</span>
                  <span className="flex items-center gap-1">
                    <Users size={10} />
                    {disc.message_count} messages
                  </span>
                  <span>
                    {new Date(disc.created_at).toLocaleDateString()}
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
      </div>
    </div>
  );
}

// ── Shared Components ──

function EmptyState({
  icon: Icon,
  title,
  description,
}: {
  icon: typeof BookOpen;
  title: string;
  description: string;
}) {
  return (
    <div className="flex flex-col items-center justify-center py-16 text-center">
      <div className="w-12 h-12 rounded-full bg-[var(--bg-secondary)] border border-[var(--border)] flex items-center justify-center mb-4">
        <Icon size={20} className="text-[var(--text-secondary)]" />
      </div>
      <h3 className="text-sm font-medium mb-1">{title}</h3>
      <p className="text-xs text-[var(--text-secondary)] max-w-sm">
        {description}
      </p>
    </div>
  );
}

function SummaryCard({
  icon: Icon,
  label,
  count,
  color,
}: {
  icon: typeof BookOpen;
  label: string;
  count: number;
  color: string;
}) {
  return (
    <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)] flex items-center gap-3">
      <div
        className="w-8 h-8 rounded-lg flex items-center justify-center"
        style={{ background: `color-mix(in srgb, ${color} 15%, transparent)` }}
      >
        <Icon size={16} style={{ color }} />
      </div>
      <div>
        <div className="text-xl font-bold" style={{ color: count > 0 ? color : undefined }}>
          {count}
        </div>
        <div className="text-xs text-[var(--text-secondary)]">{label}</div>
      </div>
    </div>
  );
}

function MiniStat({
  label,
  value,
  warn,
  highlight,
}: {
  label: string;
  value: number | string;
  warn?: boolean;
  highlight?: boolean;
}) {
  return (
    <div className="flex items-center gap-1.5 text-xs">
      <span className="text-[var(--text-secondary)]">{label}:</span>
      <span
        className={`font-medium ${
          highlight
            ? "text-[var(--accent-green)]"
            : warn
              ? "text-[var(--accent-red)]"
              : "text-[var(--text-primary)]"
        }`}
      >
        {value}
      </span>
    </div>
  );
}

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
