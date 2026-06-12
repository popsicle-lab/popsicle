import { useState } from "react";
import { AlertTriangle, CheckCircle2, ChevronDown, ChevronUp, FileText } from "lucide-react";
import type { ProductHealthReport } from "../hooks/useTauri";

interface Props {
  health: ProductHealthReport;
}

function healthIcon(level: string) {
  if (level === "critical") {
    return <AlertTriangle size={16} className="text-[var(--accent-red)]" />;
  }
  if (level === "warn") {
    return <AlertTriangle size={16} className="text-[var(--accent-yellow)]" />;
  }
  return <CheckCircle2 size={16} className="text-[var(--accent-green)]" />;
}

export function ProductHealthPanel({ health }: Props) {
  const [expanded, setExpanded] = useState(true);

  return (
    <section className="card overflow-hidden">
      <button
        type="button"
        className="flex w-full items-center justify-between gap-2 p-4 text-left transition-colors hover:bg-[var(--bg-primary)]"
        onClick={() => setExpanded((v) => !v)}
        aria-expanded={expanded}
        aria-controls="product-health-details"
      >
        <h3 className="text-[13px] font-semibold">文档健康度</h3>
        <span className="flex items-center gap-2 text-[12px] capitalize text-[var(--text-muted)]">
          <span className="flex items-center gap-1.5">
            {healthIcon(health.health)}
            {health.health}
          </span>
          {!expanded && (
            <span className="hidden text-[11px] normal-case sm:inline">
              {health.task_count} tasks · {health.intent_block_count} intents
            </span>
          )}
          {expanded ? (
            <ChevronUp size={16} aria-hidden />
          ) : (
            <ChevronDown size={16} aria-hidden />
          )}
        </span>
      </button>
      {expanded && (
        <div
          id="product-health-details"
          className="space-y-3 border-t border-[var(--border)] px-4 pb-4 pt-3"
        >
          <div className="grid grid-cols-2 gap-2 text-[12px] sm:grid-cols-4">
            <Stat label="Tasks" value={health.task_count} />
            <Stat label="Intent blocks" value={health.intent_block_count} />
            <Stat
              label="未验证 task"
              value={health.unverified_tasks}
              warn={health.unverified_tasks > 0}
            />
            <Stat
              label="断链引用"
              value={health.broken_refs}
              warn={health.broken_refs > 0}
            />
          </div>
          <div className="flex flex-wrap gap-1.5">
            {health.journey_stages.map((s) => (
              <span key={s} className="badge">
                {s}
              </span>
            ))}
          </div>
          <div className="flex flex-wrap gap-3 text-[11px] text-[var(--text-muted)]">
            <span className="flex items-center gap-1">
              <FileText size={12} />
              PRODUCT.md {health.has_product_md ? "✓" : "缺失"}
            </span>
            <span className="flex items-center gap-1">
              <FileText size={12} />
              ARCHITECTURE.md {health.has_architecture_md ? "✓" : "缺失"}
            </span>
          </div>
          {health.hints.length > 0 && (
            <ul className="space-y-1 text-[12px] text-[var(--text-secondary)]">
              {health.hints.map((h) => (
                <li key={h} className="flex gap-2">
                  <span className="text-[var(--accent-yellow)]">·</span>
                  {h}
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </section>
  );
}

function Stat({
  label,
  value,
  warn,
}: {
  label: string;
  value: number;
  warn?: boolean;
}) {
  return (
    <div className="rounded-[var(--radius-sm)] border border-[var(--border)] bg-[var(--bg-primary)] px-2.5 py-2">
      <div className="text-[10px] uppercase tracking-wide text-[var(--text-muted)]">
        {label}
      </div>
      <div
        className={`text-[15px] font-semibold ${warn ? "text-[var(--accent-yellow)]" : ""}`}
      >
        {value}
      </div>
    </div>
  );
}
