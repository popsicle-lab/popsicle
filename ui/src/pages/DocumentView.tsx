import { useEffect, useState } from "react";
import {
  getDocument,
  getCommitLinks,
  findIssueByRun,
  getPipelineStatus,
  completeStage,
  type DocFull,
  type CommitLinkInfo,
  type IssueInfo,
  type StageStatusInfo,
} from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeHighlight from "rehype-highlight";
import {
  FileText,
  Tag,
  Clock,
  GitBranch,
  GitCommit,
  Puzzle,
  ListChecks,
  ClipboardList,
  BookOpen,
  Hash,
  CheckCircle2,
  ShieldCheck,
  Loader2,
} from "lucide-react";
import type { Page } from "../App";

interface Props {
  docId: string;
  setPage: (p: Page) => void;
}

export function DocumentView({ docId, setPage }: Props) {
  const [doc, setDoc] = useState<DocFull | null>(null);
  const [linkedCommits, setLinkedCommits] = useState<CommitLinkInfo[]>([]);
  const [linkedIssue, setLinkedIssue] = useState<IssueInfo | null>(null);
  const [stage, setStage] = useState<StageStatusInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  const loadData = () => {
    getDocument(docId)
      .then((d) => {
        setDoc(d);
        return Promise.all([
          getCommitLinks({ docId }),
          findIssueByRun(d.pipeline_run_id).catch(() => null),
          getPipelineStatus(d.pipeline_run_id)
            .then((ps) =>
              ps.stages.find((s) =>
                s.skills.includes(d.skill_name)
              ) ?? null
            )
            .catch(() => null),
        ]);
      })
      .then(([commits, issue, stageInfo]) => {
        setLinkedCommits(commits);
        setLinkedIssue(issue);
        setStage(stageInfo);
      })
      .catch((e) => setError(e?.toString()));
  };

  useEffect(() => {
    loadData();
  }, [docId]);

  if (error)
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );
  if (!doc)
    return <div className="text-[var(--text-secondary)]">Loading...</div>;

  return (
    <div className="space-y-4">
      {/* Breadcrumb */}
      <div className="flex items-center gap-2 text-sm text-[var(--text-secondary)]">
        {linkedIssue && (
          <>
            <button
              onClick={() =>
                setPage({ kind: "issue", issueKey: linkedIssue.key })
              }
              className="hover:text-[var(--text-primary)] transition-colors"
            >
              {linkedIssue.key}
            </button>
            <span>/</span>
          </>
        )}
        <button
          onClick={() =>
            setPage({ kind: "pipeline", runId: doc.pipeline_run_id })
          }
          className="hover:text-[var(--text-primary)] transition-colors font-mono"
        >
          {doc.pipeline_run_id.slice(0, 8)}
        </button>
        <span>/</span>
        <span className="text-[var(--text-primary)]">{doc.title}</span>
      </div>

      <div className="flex gap-6 h-full">
        {/* Document Body */}
        <div className="flex-1 min-w-0">
          <h2 className="text-2xl font-bold mb-4">{doc.title}</h2>
          <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)] p-6 prose prose-invert max-w-none prose-pre:my-3 prose-headings:border-b prose-headings:border-[var(--border)] prose-headings:pb-2 prose-h1:text-xl prose-h2:text-lg prose-h3:text-base">
            <Markdown
              remarkPlugins={[remarkGfm]}
              rehypePlugins={[rehypeHighlight]}
            >
              {doc.body || "*No content yet*"}
            </Markdown>
          </div>
        </div>

        {/* Metadata Panel */}
        <aside className="w-72 shrink-0">
          <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)] sticky top-0">
            <div className="px-4 py-3 border-b border-[var(--border)]">
              <h3 className="font-medium text-sm">Metadata</h3>
            </div>
            <div className="p-4 space-y-4 text-sm">
              <MetaRow icon={<FileText size={14} />} label="Type">
                <code className="text-xs bg-[var(--bg-tertiary)] px-1.5 py-0.5 rounded">
                  {doc.doc_type}
                </code>
              </MetaRow>
              <MetaRow icon={<Tag size={14} />} label="Status">
                <StatusBadge status={doc.status} />
              </MetaRow>
              {stage && (
                <StageActions
                  stage={stage}
                  doc={doc}
                  onCompleted={loadData}
                />
              )}
              <ChecklistProgress body={doc.body} />
              <MetaRow icon={<Puzzle size={14} />} label="Skill">
                {doc.skill_name}
              </MetaRow>

              {linkedIssue && (
                <MetaRow icon={<ClipboardList size={14} />} label="Issue">
                  <button
                    onClick={() =>
                      setPage({ kind: "issue", issueKey: linkedIssue.key })
                    }
                    className="text-[var(--accent)] hover:underline text-xs font-mono"
                  >
                    {linkedIssue.key} – {linkedIssue.title}
                  </button>
                </MetaRow>
              )}

              <MetaRow icon={<GitBranch size={14} />} label="Pipeline Run">
                <button
                  onClick={() =>
                    setPage({
                      kind: "pipeline",
                      runId: doc.pipeline_run_id,
                    })
                  }
                  className="text-[var(--accent)] hover:underline font-mono text-xs"
                >
                  {doc.pipeline_run_id.slice(0, 8)}
                </button>
              </MetaRow>

              {doc.tags.length > 0 && (
                <MetaRow icon={<Tag size={14} />} label="Tags">
                  <div className="flex flex-wrap gap-1">
                    {doc.tags.map((t) => (
                      <span
                        key={t}
                        className="text-xs bg-[var(--bg-tertiary)] px-1.5 py-0.5 rounded"
                      >
                        {t}
                      </span>
                    ))}
                  </div>
                </MetaRow>
              )}

              {/* Document Index: Summary & Semantic Tags */}
              <div className="pt-2 border-t border-[var(--border)]">
                <MetaRow icon={<BookOpen size={14} />} label="Summary">
                  {doc.summary ? (
                    <p className="text-xs text-[var(--text-secondary)] leading-relaxed">
                      {doc.summary}
                    </p>
                  ) : (
                    <span className="text-xs text-[var(--text-secondary)] italic opacity-60">
                      Not summarized yet
                    </span>
                  )}
                </MetaRow>
              </div>
              {doc.doc_tags.length > 0 && (
                <MetaRow icon={<Hash size={14} />} label="Semantic Tags">
                  <div className="flex flex-wrap gap-1">
                    {doc.doc_tags.map((t) => (
                      <span
                        key={t}
                        className="text-xs bg-[var(--accent)]/10 text-[var(--accent)] px-1.5 py-0.5 rounded"
                      >
                        {t}
                      </span>
                    ))}
                  </div>
                </MetaRow>
              )}

              {doc.created_at && (
                <MetaRow icon={<Clock size={14} />} label="Created">
                  <span className="text-xs text-[var(--text-secondary)]">
                    {new Date(doc.created_at).toLocaleString()}
                  </span>
                </MetaRow>
              )}
              {doc.updated_at && (
                <MetaRow icon={<Clock size={14} />} label="Updated">
                  <span className="text-xs text-[var(--text-secondary)]">
                    {new Date(doc.updated_at).toLocaleString()}
                  </span>
                </MetaRow>
              )}

              <div className="pt-2 border-t border-[var(--border)]">
                <div className="text-xs text-[var(--text-secondary)] mb-1">
                  File
                </div>
                <code className="text-xs break-all text-[var(--accent)]">
                  {doc.file_path}
                </code>
              </div>

              <div className="pt-2 border-t border-[var(--border)]">
                <div className="text-xs text-[var(--text-secondary)] mb-1">
                  ID
                </div>
                <code className="text-xs break-all font-mono">{doc.id}</code>
              </div>

              {linkedCommits.length > 0 && (
                <div className="pt-2 border-t border-[var(--border)]">
                  <div className="flex items-center gap-1.5 text-[var(--text-secondary)] mb-2">
                    <GitCommit size={14} />
                    <span className="text-xs">
                      Linked Commits ({linkedCommits.length})
                    </span>
                  </div>
                  <div className="space-y-1.5">
                    {linkedCommits.map((c) => (
                      <div
                        key={c.sha}
                        className="flex items-center gap-1.5 text-xs"
                      >
                        <StatusBadge status={c.review_status} />
                        <code className="font-mono text-[var(--accent)]">
                          {c.short_sha}
                        </code>
                        <span className="truncate text-[var(--text-secondary)]">
                          {c.message}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        </aside>
      </div>
    </div>
  );
}

function StageActions({
  stage,
  doc,
  onCompleted,
}: {
  stage: StageStatusInfo;
  doc: DocFull;
  onCompleted: () => void;
}) {
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<string | null>(null);
  const [confirmOpen, setConfirmOpen] = useState(false);

  const canComplete =
    stage.state === "ready" || stage.state === "in_progress";

  const handleComplete = async () => {
    setLoading(true);
    setResult(null);
    try {
      const res = await completeStage(
        doc.pipeline_run_id,
        stage.name,
        stage.requires_approval
      );
      setResult(
        res.all_done
          ? "All stages completed! Topic lock released."
          : res.unblocked.length > 0
            ? `Stage completed. Unblocked: ${res.unblocked.join(", ")}`
            : "Stage completed."
      );
      setConfirmOpen(false);
      onCompleted();
    } catch (e: unknown) {
      setResult(`Error: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="pb-3 mb-1 border-b border-[var(--border)]">
      <div className="flex items-center gap-1.5 text-[var(--text-secondary)] mb-2">
        <CheckCircle2 size={14} />
        <span className="text-xs">Stage</span>
      </div>
      <div className="pl-5 space-y-2">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium">{stage.name}</span>
          <StatusBadge status={stage.state} />
        </div>

        {canComplete && !confirmOpen && (
          <button
            onClick={() =>
              stage.requires_approval ? setConfirmOpen(true) : handleComplete()
            }
            disabled={loading}
            className={`w-full flex items-center justify-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
              stage.requires_approval
                ? "bg-amber-500/15 text-amber-300 hover:bg-amber-500/25 border border-amber-500/30"
                : "bg-green-500/15 text-green-300 hover:bg-green-500/25 border border-green-500/30"
            }`}
          >
            {loading ? (
              <Loader2 size={14} className="animate-spin" />
            ) : stage.requires_approval ? (
              <ShieldCheck size={14} />
            ) : (
              <CheckCircle2 size={14} />
            )}
            {stage.requires_approval ? "Approve & Complete" : "Complete Stage"}
          </button>
        )}

        {confirmOpen && (
          <div className="bg-amber-500/5 border border-amber-500/20 rounded-lg p-3 space-y-2">
            <p className="text-xs text-amber-300">
              This stage requires your approval. Completing it will mark all
              documents as final. Confirm?
            </p>
            <div className="flex gap-2">
              <button
                onClick={handleComplete}
                disabled={loading}
                className="flex-1 flex items-center justify-center gap-1.5 px-2 py-1.5 rounded text-xs font-medium bg-amber-500/20 text-amber-300 hover:bg-amber-500/30 transition-colors"
              >
                {loading ? (
                  <Loader2 size={12} className="animate-spin" />
                ) : (
                  <ShieldCheck size={12} />
                )}
                Confirm
              </button>
              <button
                onClick={() => setConfirmOpen(false)}
                className="flex-1 px-2 py-1.5 rounded text-xs font-medium bg-[var(--bg-tertiary)] text-[var(--text-secondary)] hover:bg-[var(--bg-primary)] transition-colors"
              >
                Cancel
              </button>
            </div>
          </div>
        )}

        {stage.state === "completed" && (
          <div className="flex items-center gap-1.5 text-xs text-green-300">
            <ShieldCheck size={12} />
            <span>Stage completed</span>
          </div>
        )}

        {result && (
          <div
            className={`text-xs px-2 py-1.5 rounded ${
              result.startsWith("Error")
                ? "bg-red-500/10 text-red-300"
                : "bg-green-500/10 text-green-300"
            }`}
          >
            {result}
          </div>
        )}
      </div>
    </div>
  );
}

function MetaRow({
  icon,
  label,
  children,
}: {
  icon: React.ReactNode;
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <div className="flex items-center gap-1.5 text-[var(--text-secondary)] mb-1">
        {icon}
        <span className="text-xs">{label}</span>
      </div>
      <div className="pl-5">{children}</div>
    </div>
  );
}

export function parseCheckboxes(body: string): {
  checked: number;
  unchecked: number;
} {
  let checked = 0;
  let unchecked = 0;
  for (const line of body.split("\n")) {
    const trimmed = line.trimStart();
    if (trimmed.startsWith("- [x] ") || trimmed.startsWith("- [X] ")) {
      checked++;
    } else if (trimmed.startsWith("- [ ] ")) {
      unchecked++;
    }
  }
  return { checked, unchecked };
}

function ChecklistProgress({ body }: { body: string }) {
  const { checked, unchecked } = parseCheckboxes(body || "");
  const total = checked + unchecked;
  if (total === 0) return null;

  const pct = Math.round((checked / total) * 100);
  const color =
    pct === 100
      ? "var(--accent-green)"
      : pct >= 50
        ? "var(--accent-yellow)"
        : "var(--accent-red)";

  return (
    <div className="pb-3 mb-1 border-b border-[var(--border)]">
      <div className="flex items-center gap-1.5 text-[var(--text-secondary)] mb-2">
        <ListChecks size={14} />
        <span className="text-xs">Checklist</span>
      </div>
      <div className="pl-5">
        <div className="flex items-center justify-between mb-1.5">
          <span className="text-xs font-mono" style={{ color }}>
            {checked}/{total}
          </span>
          <span className="text-xs font-mono" style={{ color }}>
            {pct}%
          </span>
        </div>
        <div className="w-full h-1.5 bg-[var(--bg-tertiary)] rounded-full overflow-hidden">
          <div
            className="h-full rounded-full transition-all"
            style={{ width: `${pct}%`, background: color }}
          />
        </div>
      </div>
    </div>
  );
}
