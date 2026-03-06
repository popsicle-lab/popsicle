import { useEffect, useState } from "react";
import {
  getPipelineStatus,
  getNextSteps,
  getCommitLinks,
  verifyPipelineRun,
  type PipelineStatusFull,
  type NextStepInfo,
  type CommitLinkInfo,
  type VerifyResult,
} from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import {
  ChevronRight,
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
                      className="w-full flex items-center gap-1.5 text-xs py-1 px-2 rounded bg-[var(--bg-primary)]/50 hover:bg-[var(--bg-primary)] transition-colors mt-1 text-left"
                    >
                      <FileText size={12} />
                      <span className="truncate flex-1">{doc.title}</span>
                      <StatusBadge status={doc.status} />
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

function NextStepCard({ step }: { step: NextStepInfo }) {
  const [copied, setCopied] = useState(false);

  const copyCommand = () => {
    navigator.clipboard.writeText(step.cli_command);
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
      </div>
      <p className="text-sm text-[var(--text-secondary)] mb-2">
        {step.description}
      </p>
      {step.cli_command && (
        <div className="flex items-center gap-2">
          <code className="flex-1 text-xs bg-[var(--bg-primary)] px-3 py-1.5 rounded font-mono text-[var(--accent)] overflow-x-auto">
            $ {step.cli_command}
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
