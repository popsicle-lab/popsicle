import { useCallback, useEffect, useState } from "react";
import { BookOpen, GitBranch, Play, Radio } from "lucide-react";
import {
  agentRuntimeServerStatus,
  dispatchIssueRemote,
  getIssue,
  getIssueGuidance,
  getPipelineStatus,
  getRemoteRunMirror,
  listDocsForRun,
  startIssue,
  useRefresh,
  type DocInfo,
  type IssueGuidance,
  type IssueInfo,
} from "../hooks/useTauri";
import { LoadingState } from "../components/LoadingState";
import { RetroDocBanner } from "../components/RetroDocBanner";
import { TelemetryRunPanel } from "../components/TelemetryRunPanel";
import { IssueTypeBadge } from "../components/IssueTypeBadge";
import { StatusBadge } from "../components/StatusBadge";
import type { Page } from "../App";
import { useLocale } from "../i18n/LocaleContext";

interface Props {
  issueKey: string;
  setPage: (p: Page) => void;
  variant?: "page" | "panel" | "modal";
}

export function IssueDetailView({
  issueKey,
  setPage,
  variant = "page",
}: Props) {
  const { m } = useLocale();
  const ar = m.agentRuntime;
  const [issue, setIssue] = useState<IssueInfo | null>(null);
  const [docsByRun, setDocsByRun] = useState<Record<string, DocInfo[]>>({});
  const [guidance, setGuidance] = useState<IssueGuidance | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [starting, setStarting] = useState(false);
  const [dispatching, setDispatching] = useState(false);
  const [dispatchMsg, setDispatchMsg] = useState<string | null>(null);
  const [runtimeOnline, setRuntimeOnline] = useState(false);
  const [serverConfigured, setServerConfigured] = useState(false);
  const [syncHint, setSyncHint] = useState<string | null>(null);

  const load = useCallback(() => {
    getIssue(issueKey)
      .then(async (i) => {
        setIssue(i);
        getIssueGuidance(issueKey)
          .then(setGuidance)
          .catch(() => setGuidance(null));
        const entries = await Promise.all(
          i.run_ids.map(async (runId) => {
            const docs = await listDocsForRun(runId).catch(() => []);
            return [runId, docs] as const;
          })
        );
        setDocsByRun(Object.fromEntries(entries));
      })
      .catch((e) => setError(String(e)));
  }, [issueKey]);

  useEffect(() => {
    load();
  }, [load]);

  useRefresh(load);

  useEffect(() => {
    agentRuntimeServerStatus()
      .then((s) => {
        setServerConfigured(Boolean(s.server_url));
        setRuntimeOnline(s.runtime_state === "online");
      })
      .catch(() => {
        setServerConfigured(false);
        setRuntimeOnline(false);
      });
  }, [issueKey]);

  useEffect(() => {
    const latestRunId = issue?.run_ids.at(-1);
    if (!latestRunId || !serverConfigured) {
      setSyncHint(null);
      return;
    }
    let cancelled = false;
    Promise.all([
      getPipelineStatus(latestRunId).catch(() => null),
      getRemoteRunMirror(latestRunId).catch(() => null),
    ]).then(([local, remote]) => {
      if (cancelled || !local || !remote) {
        if (!cancelled) setSyncHint(null);
        return;
      }
      const mismatch =
        local.run_status !== remote.run_status ||
        local.current_stage !== remote.current_stage;
      setSyncHint(
        mismatch
          ? `${ar.syncMismatch}：远程 ${remote.run_status}/${remote.current_stage} · 本地 ${local.run_status}/${local.current_stage}`
          : null
      );
    });
    return () => {
      cancelled = true;
    };
  }, [issue?.run_ids, serverConfigured, ar.syncMismatch]);

  const handleStart = async () => {
    setStarting(true);
    try {
      const runId = await startIssue(issueKey);
      setPage({ kind: "pipeline", runId });
    } catch (e: unknown) {
      setError(String(e));
    } finally {
      setStarting(false);
    }
  };

  const handleRemoteDispatch = async () => {
    setDispatching(true);
    setDispatchMsg(null);
    setError(null);
    try {
      const result = await dispatchIssueRemote(issueKey);
      if (result.accepted) {
        setDispatchMsg(ar.dispatchQueued);
        load();
      } else {
        const reason = result.reason ?? "unknown";
        setDispatchMsg(`${ar.dispatchRejected}: ${reason}`);
      }
    } catch (e: unknown) {
      setError(String(e));
    } finally {
      setDispatching(false);
    }
  };

  const guidanceReturnTo: Page =
    variant === "panel" || variant === "modal"
      ? { kind: "issues", selectedKey: issueKey }
      : { kind: "issue", issueKey };

  const shell =
    variant === "panel"
      ? "detail-panel-scroll"
      : variant === "modal"
        ? "issue-modal-detail"
        : "page-frame mx-auto max-w-5xl";

  if (error) {
    return (
      <div className={shell}>
        <div className="card border-[rgba(239,68,68,0.25)] bg-[rgba(239,68,68,0.08)] p-4 text-[13px] text-[var(--accent-red)]">
          {error}
        </div>
      </div>
    );
  }
  if (!issue) {
    return (
      <div className={shell}>
        <LoadingState label="Loading issue…" />
      </div>
    );
  }

  const content = (
    <div className="detail-grid">
      <div className="space-y-4 min-w-0">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="min-w-0">
            {variant === "page" && (
              <p className="font-mono text-[12px] text-[#93c5fd]">{issue.key}</p>
            )}
            <h2
              className={`font-semibold leading-snug ${
                variant === "panel"
                  ? "text-[15px]"
                  : variant === "modal"
                    ? "text-[17px]"
                    : "mt-1 text-xl"
              }`}
            >
              {issue.title}
            </h2>
            <div className="mt-2 flex flex-wrap items-center gap-2">
              <IssueTypeBadge type={issue.issue_type} />
              <StatusBadge status={issue.status} />
              <span className="text-[11px] text-[var(--text-muted)]">
                {issue.priority} · {issue.product_id}
                {issue.task_links
                  .filter((l) => l.role === "linked" && l.task_id)
                  .map((l) => l.task_id)
                  .join(", ")
                  ? ` · tasks ${issue.task_links
                      .filter((l) => l.role === "linked" && l.task_id)
                      .map((l) => l.task_id)
                      .join(", ")}`
                  : ""}
              </span>
            </div>
          </div>
          {!issue.active_run_id && issue.status !== "done" && (
            <div className="flex shrink-0 flex-col items-end gap-1.5">
              <div className="flex flex-wrap gap-2">
                <button
                  type="button"
                  onClick={handleStart}
                  disabled={starting}
                  className="btn btn-primary shrink-0"
                  title={ar.localStartHint}
                >
                  <Play size={15} />
                  {starting ? "Starting…" : "Start pipeline"}
                </button>
                {issue.pipeline && (
                  <button
                    type="button"
                    onClick={handleRemoteDispatch}
                    disabled={
                      dispatching || !serverConfigured || !runtimeOnline
                    }
                    className="btn shrink-0 border border-[var(--border-strong)] bg-[var(--bg-elevated)]"
                    title={ar.dispatchHint}
                  >
                    <Radio size={15} />
                    {dispatching ? ar.dispatching : ar.dispatchRemote}
                  </button>
                )}
              </div>
              {issue.pipeline && !serverConfigured && (
                <p className="text-[10px] text-[var(--text-muted)]">
                  {ar.dispatchHint}
                </p>
              )}
              {dispatchMsg && (
                <p className="text-[11px] text-[var(--accent-green)]">
                  {dispatchMsg}
                </p>
              )}
            </div>
          )}
        </div>

        {syncHint && (
          <div className="rounded-[10px] border border-[var(--accent-orange)]/30 bg-[var(--accent-orange)]/10 px-3.5 py-2.5 text-[12px] leading-relaxed text-[var(--text-secondary)]">
            {syncHint}
            <span className="mt-1 block text-[11px] text-[var(--text-muted)]">
              Daemon 心跳时会自动 reconcile 镜像；若仍不一致，确认 Daemon 在线并重启一次。
            </span>
          </div>
        )}

        {issue.description && (
          <div className="card p-3.5 text-[13px] leading-relaxed whitespace-pre-wrap text-[var(--text-secondary)]">
            {issue.description}
          </div>
        )}

        {!issue.pipeline && issue.run_ids.length === 0 && (
          <RetroDocBanner productId={issue.product_id} />
        )}

        {guidance &&
          (guidance.linked_tasks.length > 0 ||
            guidance.proposed_tasks.length > 0 ||
            guidance.recommended_tasks.length > 0 ||
            guidance.related_intents.length > 0) && (
            <div className="card space-y-3 p-3.5">
              <div className="flex items-center gap-2">
                <BookOpen size={16} className="text-[var(--accent)]" />
                <h3 className="text-[13px] font-semibold">Guidance</h3>
                {guidance.product && (
                  <button
                    type="button"
                    onClick={() =>
                      setPage({
                        kind: "products",
                        product: guidance.product!,
                        tab: "tasks",
                      })
                    }
                    className="btn btn-ghost ml-auto text-[11px]"
                  >
                    Browse {guidance.product}
                  </button>
                )}
              </div>
              <p className="text-[12px] text-[var(--text-muted)]">{guidance.hint}</p>
              {guidance.linked_tasks.length > 0 && (
                <div className="space-y-1">
                  <p className="text-[11px] font-medium text-[var(--text-secondary)]">
                    Linked tasks
                  </p>
                  {guidance.linked_tasks.map((t) => (
                    <button
                      key={t.task_id}
                      type="button"
                      onClick={() =>
                        setPage({
                          kind: "task",
                          taskId: t.task_id,
                          product: t.product,
                          returnTo: guidanceReturnTo,
                        })
                      }
                      className="flex w-full items-center gap-2 rounded-[var(--radius-sm)] px-2.5 py-2 text-left text-[13px] transition-colors hover:bg-[var(--bg-hover)]"
                    >
                      <span className="shrink-0 font-mono text-[11px] text-[#93c5fd]">
                        {t.task_id}
                      </span>
                      <span className="min-w-0 flex-1 truncate">{t.title}</span>
                      <span className="badge badge-neutral shrink-0">
                        {t.journey_stage}
                      </span>
                    </button>
                  ))}
                </div>
              )}
              {guidance.proposed_tasks.length > 0 && (
                <div className="space-y-1">
                  <p className="text-[11px] font-medium text-[var(--text-secondary)]">
                    Proposed tasks
                  </p>
                  {guidance.proposed_tasks.map((p) => (
                    <div
                      key={`${p.title}-${p.source}`}
                      className="flex items-center gap-2 rounded-[var(--radius-sm)] px-2.5 py-2 text-[13px] text-[var(--text-secondary)]"
                    >
                      <span className="min-w-0 flex-1 truncate">{p.title}</span>
                      {p.journey_stage && (
                        <span className="badge badge-neutral shrink-0">
                          {p.journey_stage}
                        </span>
                      )}
                    </div>
                  ))}
                </div>
              )}
              {guidance.recommended_tasks.length > 0 && (
                <div className="space-y-1">
                  <p className="text-[11px] font-medium text-[var(--text-secondary)]">
                    Suggested (heuristic)
                  </p>
                  {guidance.recommended_tasks.map((t) => (
                    <button
                      key={t.task_id}
                      type="button"
                      onClick={() =>
                        setPage({
                          kind: "task",
                          taskId: t.task_id,
                          product: t.product,
                          returnTo: guidanceReturnTo,
                        })
                      }
                      className="flex w-full items-center gap-2 rounded-[var(--radius-sm)] px-2.5 py-2 text-left text-[13px] transition-colors hover:bg-[var(--bg-hover)]"
                    >
                      <span className="shrink-0 font-mono text-[11px] text-[#93c5fd]">
                        {t.task_id}
                      </span>
                      <span className="min-w-0 flex-1 truncate">{t.title}</span>
                      <span className="badge badge-neutral shrink-0">
                        {t.journey_stage}
                      </span>
                    </button>
                  ))}
                </div>
              )}
              {guidance.related_intents.length > 0 && (
                <div className="flex flex-wrap gap-1.5">
                  {guidance.related_intents.map((ri) => (
                    <button
                      key={ri.reference}
                      type="button"
                      onClick={() =>
                        setPage({
                          kind: "intent",
                          product: ri.product,
                          file: ri.file,
                          block: ri.block,
                          returnTo: guidanceReturnTo,
                        })
                      }
                      className="badge badge-accent font-mono transition-opacity hover:opacity-90"
                    >
                      {ri.reference}
                    </button>
                  ))}
                </div>
              )}
            </div>
          )}

        {issue.active_run_id && (
          <div className="card flex flex-wrap items-center justify-between gap-3 border-[rgba(59,130,246,0.25)] bg-[var(--accent-muted)] p-3.5">
            <div className="flex items-center gap-2 text-[13px]">
              <GitBranch size={15} />
              <span className="text-[var(--text-secondary)]">Active run</span>
              <code className="font-mono text-[12px] text-[#93c5fd]">
                {issue.active_run_id.slice(0, 8)}…
              </code>
            </div>
            <div className="flex flex-wrap gap-2">
              <button
                type="button"
                onClick={() =>
                  setPage({
                    kind: "workflows",
                    tab: "pipelines",
                    pipeline: issue.pipeline ?? undefined,
                    contextRunId: issue.active_run_id ?? undefined,
                    contextIssueKey: issue.key,
                  })
                }
                className="btn btn-secondary text-[12px]"
              >
                <BookOpen size={14} className="mr-1 inline" />
                {m.issues.openWorkflowHelp}
              </button>
              <button
                type="button"
                onClick={() =>
                  setPage({ kind: "pipeline", runId: issue.active_run_id! })
                }
                className="btn btn-secondary text-[12px]"
              >
                Open pipeline
              </button>
            </div>
          </div>
        )}

        {!issue.active_run_id && issue.pipeline && (
          <button
            type="button"
            onClick={() =>
              setPage({
                kind: "workflows",
                tab: "pipelines",
                pipeline: issue.pipeline ?? undefined,
                contextIssueKey: issue.key,
              })
            }
            className="btn btn-secondary text-[12px]"
          >
            <BookOpen size={14} className="mr-1 inline" />
            {m.issues.openWorkflowHelp}
          </button>
        )}

        <div>
          <h3 className="section-label mb-2">
            Runs ({issue.run_ids.length})
          </h3>
          <div className="space-y-2">
            {issue.run_ids.map((runId) => (
              <div key={runId} className="card p-3 space-y-2">
                <button
                  type="button"
                  onClick={() => setPage({ kind: "pipeline", runId })}
                  className="font-mono text-[12px] text-[#93c5fd] transition-opacity hover:opacity-80"
                >
                  {runId}
                </button>
                <TelemetryRunPanel runId={runId} compact />
                {(docsByRun[runId] ?? []).length > 0 && (
                  <div className="mt-2 space-y-0.5 border-t border-[var(--border)] pt-2">
                    {(docsByRun[runId] ?? []).map((doc) => (
                      <button
                        key={doc.id}
                        type="button"
                        onClick={() =>
                          setPage({ kind: "document", docId: doc.id })
                        }
                        className="flex w-full items-center gap-2 rounded-[var(--radius-sm)] px-2 py-1.5 text-left text-[12px] transition-colors hover:bg-[var(--bg-hover)]"
                      >
                        <span className="min-w-0 flex-1 truncate">{doc.title}</span>
                        <StatusBadge status={doc.status} />
                      </button>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      </div>

      <aside className="detail-rail card space-y-3 p-3 text-[12px]">
        <div>
          <p className="section-label mb-1">Issue</p>
          <p className="font-mono text-[#93c5fd]">{issue.key}</p>
        </div>
        <div>
          <p className="section-label mb-1">Pipeline</p>
          <p className="text-[var(--text-secondary)]">
            {issue.pipeline ?? "—"}
          </p>
        </div>
        <div>
          <p className="section-label mb-1">Runs</p>
          <p className="text-[var(--text-secondary)]">{issue.run_ids.length}</p>
        </div>
        {issue.active_run_id && (
          <div>
            <p className="section-label mb-1">Active</p>
            <p className="font-mono text-[11px] text-[var(--text-muted)] break-all">
              {issue.active_run_id}
            </p>
          </div>
        )}
      </aside>
    </div>
  );

  return <div className={shell}>{content}</div>;
}
