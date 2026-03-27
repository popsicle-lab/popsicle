import { useEffect, useState } from "react";
import {
  getPipelineStatus,
  getNextSteps,
  getCommitLinks,
  verifyPipelineRun,
  searchDocuments,
  type PipelineStatusFull,
  type NextStepInfo,
  type CommitLinkInfo,
  type VerifyResult,
  type SearchDocResult,
} from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import {
  ChevronRight,
  ChevronDown,
  Copy,
  Check,
  FileText,
  Lightbulb,
  ArrowRight,
  GitCommit,
  ShieldCheck,
  ShieldAlert,
  Archive,
  Zap,
  BookOpen,
  Hash,
} from "lucide-react";
import type { Page } from "../App";

interface Props {
  runId: string;
  setPage: (p: Page) => void;
}

export function PipelineView({ runId, setPage }: Props) {
  const [status, setStatus] = useState<PipelineStatusFull | null>(null);
  const [steps, setSteps] = useState<NextStepInfo[]>([]);
  const [commits, setCommits] = useState<CommitLinkInfo[]>([]);
  const [verify, setVerify] = useState<VerifyResult | null>(null);
  const [relatedDocs, setRelatedDocs] = useState<SearchDocResult[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([
      getPipelineStatus(runId),
      getNextSteps(runId),
      getCommitLinks({ runId }),
      verifyPipelineRun(runId),
    ])
      .then(([s, ns, cl, v]) => {
        setStatus(s);
        setSteps(ns);
        setCommits(cl);
        setVerify(v);
        if (s.title) {
          searchDocuments({
            query: s.title,
            status: "approved",
            excludeRun: runId,
            limit: 5,
          })
            .then(setRelatedDocs)
            .catch(() => {});
        }
      })
      .catch((e) => setError(e?.toString()));
  }, [runId]);

  if (error)
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );
  if (!status)
    return <div className="text-[var(--text-secondary)]">Loading...</div>;

  const actionable = steps.filter((s) => s.action !== "blocked");
  const isQuick = status.pipeline_name === "quick";
  const allCompleted = status.stages.every((s) => s.state === "completed" || s.state === "skipped");

  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between">
        <div>
          <div className="flex items-center gap-2">
            <h2 className="text-2xl font-bold">{status.title}</h2>
            {isQuick && (
              <span className="inline-flex items-center gap-1 text-xs bg-yellow-500/15 text-yellow-300 px-2 py-0.5 rounded-full">
                <Zap size={10} /> Quick
              </span>
            )}
          </div>
          <p className="text-sm text-[var(--text-secondary)] mt-1">
            Pipeline: {status.pipeline_name} &middot; {status.id.slice(0, 8)}
          </p>
        </div>

        {/* Verify Badge */}
        {verify && (
          <div
            className={`flex items-center gap-2 px-3 py-2 rounded-lg text-sm ${
              verify.verified
                ? "bg-green-500/10 text-green-300 border border-green-500/30"
                : "bg-yellow-500/10 text-yellow-300 border border-yellow-500/30"
            }`}
          >
            {verify.verified ? (
              <>
                <ShieldCheck size={16} /> Verified
              </>
            ) : (
              <>
                <ShieldAlert size={16} /> {verify.issues.length} issue(s)
              </>
            )}
          </div>
        )}
      </div>

      {/* Verify Issues */}
      {verify && !verify.verified && (
        <div className="bg-yellow-500/5 border border-yellow-500/20 rounded-xl p-4">
          <h3 className="text-sm font-medium text-yellow-300 mb-2 flex items-center gap-2">
            <ShieldAlert size={14} /> Verification Issues
          </h3>
          <ul className="space-y-1 text-sm text-[var(--text-secondary)]">
            {verify.issues.map((issue, i) => (
              <li key={i} className="flex items-start gap-2">
                <span className="text-yellow-400 mt-0.5">-</span>
                {issue}
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Archive hint */}
      {allCompleted && (
        <div className="bg-green-500/5 border border-green-500/20 rounded-xl p-4 flex items-center justify-between">
          <div className="flex items-center gap-2 text-green-300 text-sm">
            <Archive size={16} />
            All stages completed. Ready to archive.
          </div>
          <CopyableCommand command={`popsicle pipeline archive --run ${status.id}`} />
        </div>
      )}

      {/* Pipeline DAG */}
      <div className="bg-[var(--bg-secondary)] rounded-xl p-5 border border-[var(--border)]">
        <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-4">
          Pipeline Stages
        </h3>
        <div className="flex items-start gap-2 overflow-x-auto pb-2">
          {status.stages.map((stage, i) => (
            <div key={stage.name} className="flex items-start">
              <div className="min-w-[180px]">
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
                    <span className="text-sm font-medium">{stage.name}</span>
                    <StatusBadge status={stage.state} />
                  </div>
                  <div className="text-xs text-[var(--text-secondary)] mb-2">
                    {stage.skills.join(", ")}
                  </div>
                  {stage.documents.map((doc) => (
                    <button
                      key={doc.id}
                      onClick={() =>
                        setPage({ kind: "document", docId: doc.id })
                      }
                      className="w-full text-xs py-1 px-2 rounded bg-[var(--bg-primary)]/50 hover:bg-[var(--bg-primary)] transition-colors mt-1 text-left"
                    >
                      <div className="flex items-center gap-1.5">
                        <FileText size={12} />
                        <span className="truncate flex-1">{doc.title}</span>
                        <StatusBadge status={doc.status} />
                      </div>
                      {doc.checklist_total > 0 && (
                        <MiniChecklistBar
                          checked={doc.checklist_checked}
                          total={doc.checklist_total}
                        />
                      )}
                    </button>
                  ))}
                  {stage.documents.length === 0 && (
                    <div className="text-xs text-[var(--text-secondary)] italic">
                      No documents yet
                    </div>
                  )}
                  {commits
                    .filter((c) => c.stage === stage.name)
                    .slice(0, 3)
                    .map((c) => (
                      <div
                        key={c.sha}
                        className="flex items-center gap-1.5 text-xs py-1 px-2 rounded bg-[var(--bg-primary)]/30 mt-1"
                      >
                        <GitCommit size={10} className="shrink-0" />
                        <code className="text-[var(--accent)] font-mono">
                          {c.short_sha}
                        </code>
                        <span className="truncate flex-1 text-[var(--text-secondary)]">
                          {c.message}
                        </span>
                        <StatusBadge status={c.review_status} />
                      </div>
                    ))}
                </div>
              </div>
              {i < status.stages.length - 1 && (
                <div className="flex items-center px-1 pt-5">
                  <ChevronRight
                    size={16}
                    className="text-[var(--text-secondary)]"
                  />
                </div>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Historical References */}
      {relatedDocs.length > 0 && (
        <HistoricalRefsCard docs={relatedDocs} setPage={setPage} />
      )}

      {/* Next Steps Advisor */}
      {actionable.length > 0 && (
        <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
          <div className="px-4 py-3 border-b border-[var(--border)] flex items-center gap-2">
            <Lightbulb size={16} className="text-[var(--accent-yellow)]" />
            <h3 className="font-medium">Next Steps</h3>
          </div>
          <div className="divide-y divide-[var(--border)]">
            {actionable.map((step, i) => (
              <NextStepCard key={i} step={step} />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function CopyableCommand({ command }: { command: string }) {
  const [copied, setCopied] = useState(false);
  const copy = () => {
    navigator.clipboard.writeText(command);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };
  return (
    <div className="flex items-center gap-2">
      <code className="text-xs bg-[var(--bg-primary)] px-3 py-1.5 rounded font-mono text-[var(--accent)]">
        $ {command}
      </code>
      <button
        onClick={copy}
        className="p-1.5 rounded hover:bg-[var(--bg-tertiary)] transition-colors"
      >
        {copied ? (
          <Check size={14} className="text-[var(--accent-green)]" />
        ) : (
          <Copy size={14} className="text-[var(--text-secondary)]" />
        )}
      </button>
    </div>
  );
}

function MiniChecklistBar({
  checked,
  total,
}: {
  checked: number;
  total: number;
}) {
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

function HistoricalRefsCard({
  docs,
  setPage,
}: {
  docs: SearchDocResult[];
  setPage: (p: Page) => void;
}) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors rounded-xl"
      >
        <div className="flex items-center gap-2">
          <BookOpen size={16} className="text-[var(--accent-purple)]" />
          <h3 className="font-medium text-sm">
            Historical References ({docs.length})
          </h3>
          <span className="text-xs text-[var(--text-secondary)]">
            Related documents from other runs
          </span>
        </div>
        {expanded ? (
          <ChevronDown size={16} className="text-[var(--text-secondary)]" />
        ) : (
          <ChevronRight size={16} className="text-[var(--text-secondary)]" />
        )}
      </button>

      {expanded && (
        <div className="divide-y divide-[var(--border)]">
          {docs.map((doc) => (
            <button
              key={doc.id}
              onClick={() => setPage({ kind: "document", docId: doc.id })}
              className="w-full px-4 py-3 hover:bg-[var(--bg-tertiary)] transition-colors text-left"
            >
              <div className="flex items-center gap-2 mb-1">
                <FileText size={12} />
                <span className="text-sm font-medium">{doc.title}</span>
                <StatusBadge status={doc.status} />
                <code className="text-xs bg-[var(--bg-tertiary)] px-1.5 py-0.5 rounded text-[var(--text-secondary)]">
                  {doc.doc_type}
                </code>
              </div>
              {doc.summary && (
                <p className="text-xs text-[var(--text-secondary)] leading-relaxed mb-1.5 line-clamp-2 pl-5">
                  {doc.summary}
                </p>
              )}
              <div className="flex items-center gap-3 text-xs text-[var(--text-secondary)] pl-5">
                <span>{doc.skill_name}</span>
                <span className="font-mono">
                  run:{doc.pipeline_run_id.slice(0, 8)}
                </span>
                {doc.doc_tags.length > 0 && (
                  <span className="inline-flex items-center gap-1">
                    <Hash size={10} />
                    {doc.doc_tags.slice(0, 4).join(", ")}
                  </span>
                )}
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function NextStepCard({ step }: { step: NextStepInfo }) {
  const [copied, setCopied] = useState(false);

  const cmdToShow = step.requires_approval
    ? `${step.cli_command} --confirm`
    : step.cli_command;

  const copyCommand = () => {
    navigator.clipboard.writeText(cmdToShow);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="px-4 py-3">
      <div className="flex items-center gap-2 mb-1">
        <StatusBadge status={step.stage} />
        <ArrowRight size={12} className="text-[var(--text-secondary)]" />
        <span className="text-sm font-medium">{step.skill}</span>
        <span className="text-xs text-[var(--text-secondary)]">
          ({step.action})
        </span>
        {step.requires_approval && (
          <span className="inline-flex items-center gap-1 text-xs px-1.5 py-0.5 rounded bg-amber-500/15 text-amber-400">
            <ShieldAlert size={10} />
            Approval
          </span>
        )}
      </div>
      <p className="text-sm text-[var(--text-secondary)] mb-2">
        {step.description}
      </p>
      {step.requires_approval && (
        <p className="text-xs text-amber-400/90 mb-2">
          需您本人审批：请先审阅/参与讨论，确认后由您本人在终端执行下方命令，勿让 AI 代执行。
        </p>
      )}
      {step.cli_command && (
        <div className="flex items-center gap-2">
          <code className="flex-1 text-xs bg-[var(--bg-primary)] px-3 py-1.5 rounded font-mono text-[var(--accent)] overflow-x-auto">
            $ {cmdToShow}
          </code>
          <button
            onClick={copyCommand}
            className="p-1.5 rounded hover:bg-[var(--bg-tertiary)] transition-colors"
            title="Copy command"
          >
            {copied ? (
              <Check size={14} className="text-[var(--accent-green)]" />
            ) : (
              <Copy size={14} className="text-[var(--text-secondary)]" />
            )}
          </button>
        </div>
      )}
    </div>
  );
}
