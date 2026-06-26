import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Activity,
  AlertTriangle,
  Bot,
  ChevronDown,
  ChevronRight,
  Clock,
  FileCheck2,
  Layers,
  Sparkles,
} from "lucide-react";
import {
  getTelemetryRunDetail,
  useRefresh,
  type TelemetryRunDetail,
  type TelemetrySpan,
} from "../hooks/useTauri";
import { MermaidRenderer } from "./MermaidRenderer";
import { useLocale } from "../i18n/LocaleContext";
import {
  buildCoverageFlowMermaid,
  buildSpanSequenceMermaid,
  formatDurationMs,
  isAgentSpan,
  pickDisplayAttributes,
  spanKindLabel,
} from "../lib/telemetryDiagrams";

function spanExtra(span: TelemetrySpan): Record<string, string> {
  const out: Record<string, string> = {};
  for (const [k, v] of Object.entries(span)) {
    if (k !== "ts" && k !== "span") out[k] = v;
  }
  return out;
}

type Tab = "overview" | "timeline" | "sequence" | "spans";

interface Props {
  runId: string;
  /** Compact: stats row only (Issue detail run cards). */
  compact?: boolean;
  className?: string;
}

function spanIcon(span: string) {
  if (span === "gen_ai.chat") return Bot;
  if (span === "popsicle.run.score") return Sparkles;
  if (span === "popsicle.doc.check") return FileCheck2;
  if (span.startsWith("popsicle.stage")) return Layers;
  return Activity;
}

function SpanTimelineItem({
  span,
  index,
  expanded,
  onToggle,
}: {
  span: TelemetrySpan;
  index: number;
  expanded: boolean;
  onToggle: () => void;
}) {
  const Icon = spanIcon(span.span);
  const agent = isAgentSpan(span.span);
  const attrs = pickDisplayAttributes(span);

  return (
    <div className="telemetry-span-row">
      <div className="telemetry-span-rail">
        <span
          className={`telemetry-span-dot ${agent ? "telemetry-span-dot-agent" : "telemetry-span-dot-ops"}`}
        />
        {index > 0 && <span className="telemetry-span-line" aria-hidden />}
      </div>
      <button
        type="button"
        className="telemetry-span-card"
        onClick={onToggle}
        aria-expanded={expanded}
      >
        <div className="flex min-w-0 flex-1 items-start gap-2">
          <Icon size={14} className="mt-0.5 shrink-0 text-[var(--text-muted)]" />
          <div className="min-w-0 flex-1 text-left">
            <div className="flex flex-wrap items-center gap-1.5">
              <span className="font-mono text-[11px] text-[var(--accent)]">
                {span.span}
              </span>
              <span className="text-[10px] text-[var(--text-muted)]">
                {spanKindLabel(span.span)}
              </span>
            </div>
            <p className="mt-0.5 truncate text-[10px] text-[var(--text-muted)]">
              {span.ts}
              {spanExtra(span)["popsicle.duration_ms"]
                ? ` · ${formatDurationMs(Number(spanExtra(span)["popsicle.duration_ms"]))}`
                : ""}
            </p>
          </div>
          {expanded ? (
            <ChevronDown size={14} className="shrink-0 text-[var(--text-muted)]" />
          ) : (
            <ChevronRight size={14} className="shrink-0 text-[var(--text-muted)]" />
          )}
        </div>
        {expanded && attrs.length > 0 && (
          <dl className="telemetry-span-attrs">
            {attrs.map(([k, v]) => (
              <div key={k} className="telemetry-span-attr">
                <dt>{k.replace(/^popsicle\./, "")}</dt>
                <dd className="break-all">{v}</dd>
              </div>
            ))}
          </dl>
        )}
      </button>
    </div>
  );
}

export function TelemetryRunPanel({ runId, compact, className }: Props) {
  const { m } = useLocale();
  const [detail, setDetail] = useState<TelemetryRunDetail | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [tab, setTab] = useState<Tab>("overview");
  const [expandedSpan, setExpandedSpan] = useState<number | null>(null);
  const [collapsed, setCollapsed] = useState(false);

  const load = useCallback(() => {
    getTelemetryRunDetail(runId)
      .then(setDetail)
      .catch((e) => setError(String(e)));
  }, [runId]);

  useEffect(() => {
    load();
    setTab("overview");
    setExpandedSpan(null);
  }, [load]);

  useRefresh(load);

  const report = detail?.report;
  const spans = detail?.spans ?? [];

  const maxStageMs = useMemo(() => {
    const durs = (report?.stages ?? [])
      .map((s) => s.duration_ms ?? 0)
      .filter((n) => n > 0);
    return durs.length ? Math.max(...durs) : 1;
  }, [report?.stages]);

  const sequenceChart = useMemo(
    () => buildSpanSequenceMermaid(spans),
    [spans]
  );

  const coverageChart = useMemo(
    () =>
      report
        ? buildCoverageFlowMermaid(report.agent_coverage)
        : null,
    [report]
  );

  if (error) {
    return (
      <div className={`telemetry-panel telemetry-panel-error ${className ?? ""}`}>
        <p className="text-[12px] text-[var(--accent-red)]">{error}</p>
      </div>
    );
  }

  if (!report) {
    return (
      <div className={`telemetry-panel ${className ?? ""}`}>
        <p className="text-[12px] text-[var(--text-muted)]">{m.telemetry.loading}</p>
      </div>
    );
  }

  const degraded = report.status === "degraded" || report.span_count === 0;
  const gaps = report.agent_coverage.gaps;

  if (compact) {
    return (
      <div className={`telemetry-compact ${className ?? ""}`}>
        <div className="flex flex-wrap items-center gap-2 text-[11px]">
          <Activity size={12} className="text-[var(--text-muted)]" />
          <span className="text-[var(--text-secondary)]">
            {m.telemetry.spans}: {report.span_count}
          </span>
          <span className="text-[var(--text-muted)]">·</span>
          <span
            className={
              degraded ? "text-[var(--accent-yellow)]" : "text-[var(--accent-green)]"
            }
          >
            {degraded ? m.telemetry.noData : m.telemetry.statusOk}
          </span>
          {!degraded && (
            <>
              <span className="text-[var(--text-muted)]">·</span>
              <span className="text-[var(--text-muted)]">
                Agent{" "}
                {[
                  report.agent_coverage.gen_ai_chat && "LLM",
                  report.agent_coverage.run_score && "score",
                ]
                  .filter(Boolean)
                  .join("+") || m.telemetry.agentMissing}
              </span>
            </>
          )}
          {gaps.length > 0 && (
            <span className="badge badge-warn">{gaps.length} gap</span>
          )}
        </div>
      </div>
    );
  }

  const tabs: { id: Tab; label: string }[] = [
    { id: "overview", label: m.telemetry.tabOverview },
    { id: "timeline", label: m.telemetry.tabTimeline },
    { id: "sequence", label: m.telemetry.tabSequence },
    { id: "spans", label: m.telemetry.tabSpans },
  ];

  return (
    <section className={`telemetry-panel ${className ?? ""}`}>
      <header className="telemetry-panel-header">
        <button
          type="button"
          className="telemetry-panel-title"
          onClick={() => setCollapsed((v) => !v)}
          aria-expanded={!collapsed}
        >
          {collapsed ? (
            <ChevronRight size={16} />
          ) : (
            <ChevronDown size={16} />
          )}
          <Activity size={16} className="text-[var(--accent)]" />
          <span>{m.telemetry.title}</span>
          <span className="telemetry-panel-meta">
            {report.span_count} spans
            {report.pipeline ? ` · ${report.pipeline}` : ""}
          </span>
          <span
            className={`telemetry-status-pill ${degraded ? "telemetry-status-degraded" : "telemetry-status-ok"}`}
          >
            {degraded ? m.telemetry.noData : report.status}
          </span>
        </button>
      </header>

      {!collapsed && (
        <div className="telemetry-panel-body">
          {degraded ? (
            <p className="telemetry-empty">{m.telemetry.emptyHint}</p>
          ) : (
            <>
              <div className="telemetry-tabs" role="tablist">
                {tabs.map((t) => (
                  <button
                    key={t.id}
                    type="button"
                    role="tab"
                    aria-selected={tab === t.id}
                    className={`telemetry-tab ${tab === t.id ? "telemetry-tab-active" : ""}`}
                    onClick={() => setTab(t.id)}
                  >
                    {t.label}
                  </button>
                ))}
              </div>

              {tab === "overview" && (
                <div className="telemetry-overview">
                  <div className="telemetry-stat-grid">
                    <div className="telemetry-stat">
                      <span className="telemetry-stat-value">{report.span_count}</span>
                      <span className="telemetry-stat-label">{m.telemetry.spans}</span>
                    </div>
                    <div className="telemetry-stat">
                      <span className="telemetry-stat-value">
                        {report.stages.length}
                      </span>
                      <span className="telemetry-stat-label">{m.telemetry.stages}</span>
                    </div>
                    <div className="telemetry-stat">
                      <span className="telemetry-stat-value">
                        {report.doc_checks.passed}/{report.doc_checks.total}
                      </span>
                      <span className="telemetry-stat-label">{m.telemetry.docChecks}</span>
                    </div>
                    <div className="telemetry-stat">
                      <span className="telemetry-stat-value">{gaps.length}</span>
                      <span className="telemetry-stat-label">{m.telemetry.gaps}</span>
                    </div>
                  </div>

                  <div className="telemetry-overview-split">
                    <div className="telemetry-card">
                      <h4 className="telemetry-card-title">{m.telemetry.agentCoverage}</h4>
                      {coverageChart && (
                        <MermaidRenderer
                          chart={coverageChart}
                          className="telemetry-mermaid-sm"
                        />
                      )}
                      <ul className="telemetry-checklist">
                        <li className={report.agent_coverage.gen_ai_chat ? "ok" : "miss"}>
                          gen_ai.chat
                        </li>
                        <li className={report.agent_coverage.run_score ? "ok" : "miss"}>
                          popsicle.run.score
                        </li>
                        <li className={report.agent_coverage.decision ? "ok" : "miss"}>
                          popsicle.decision
                        </li>
                      </ul>
                    </div>

                    <div className="telemetry-card">
                      <h4 className="telemetry-card-title">{m.telemetry.docBySkill}</h4>
                      {Object.keys(report.doc_checks.by_skill).length === 0 ? (
                        <p className="text-[11px] text-[var(--text-muted)]">—</p>
                      ) : (
                        <ul className="telemetry-skill-list">
                          {Object.entries(report.doc_checks.by_skill).map(
                            ([skill, counts]) => (
                              <li key={skill}>
                                <span className="font-mono text-[11px]">{skill}</span>
                                <span className="text-[var(--accent-green)]">
                                  ✓{counts.passed}
                                </span>
                                {counts.failed > 0 && (
                                  <span className="text-[var(--accent-red)]">
                                    ✗{counts.failed}
                                  </span>
                                )}
                              </li>
                            )
                          )}
                        </ul>
                      )}
                    </div>
                  </div>

                  {gaps.length > 0 && (
                    <div className="telemetry-gaps">
                      <h4 className="flex items-center gap-1.5 text-[12px] font-semibold text-[var(--accent-yellow)]">
                        <AlertTriangle size={14} />
                        {m.telemetry.gapsTitle}
                      </h4>
                      <ul className="space-y-1.5">
                        {gaps.map((g) => (
                          <li key={g.doc_id} className="telemetry-gap-item">
                            <code className="text-[11px]">{g.doc_id}</code>
                            {g.skill && (
                              <span className="text-[10px] text-[var(--text-muted)]">
                                {g.skill}
                              </span>
                            )}
                            <span className="text-[10px] text-[var(--accent-yellow)]">
                              {g.missing.join(", ")}
                            </span>
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>
              )}

              {tab === "timeline" && (
                <div className="telemetry-timeline">
                  {report.stages.length === 0 ? (
                    <p className="telemetry-empty">{m.telemetry.noStages}</p>
                  ) : (
                    report.stages.map((stage) => {
                      const pct = stage.duration_ms
                        ? Math.max(8, (stage.duration_ms / maxStageMs) * 100)
                        : 8;
                      return (
                        <div key={stage.name} className="telemetry-gantt-row">
                          <div className="telemetry-gantt-label">
                            <span className="font-medium">{stage.name}</span>
                            {stage.skill && (
                              <span className="text-[10px] text-[var(--text-muted)]">
                                {stage.skill}
                              </span>
                            )}
                          </div>
                          <div className="telemetry-gantt-track">
                            <div
                              className={`telemetry-gantt-bar ${stage.completed ? "telemetry-gantt-bar-done" : "telemetry-gantt-bar-pending"}`}
                              style={{ width: `${pct}%` }}
                              title={formatDurationMs(stage.duration_ms)}
                            />
                          </div>
                          <div className="telemetry-gantt-meta">
                            <Clock size={11} />
                            {formatDurationMs(stage.duration_ms)}
                            {stage.completed ? " ✓" : ""}
                          </div>
                        </div>
                      );
                    })
                  )}
                </div>
              )}

              {tab === "sequence" && (
                <div className="telemetry-sequence">
                  {sequenceChart ? (
                    <MermaidRenderer
                      chart={sequenceChart}
                      className="telemetry-mermaid"
                    />
                  ) : (
                    <p className="telemetry-empty">{m.telemetry.emptyHint}</p>
                  )}
                </div>
              )}

              {tab === "spans" && (
                <div className="telemetry-span-list">
                  {spans.map((span, i) => (
                    <SpanTimelineItem
                      key={`${span.ts}-${span.span}-${i}`}
                      span={span}
                      index={i}
                      expanded={expandedSpan === i}
                      onToggle={() =>
                        setExpandedSpan((prev) => (prev === i ? null : i))
                      }
                    />
                  ))}
                </div>
              )}
            </>
          )}
        </div>
      )}
    </section>
  );
}
