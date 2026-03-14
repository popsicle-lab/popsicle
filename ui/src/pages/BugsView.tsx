import { useEffect, useState, useCallback } from "react";
import { listBugs, type BugInfo } from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import { Bug, ArrowRight } from "lucide-react";
import type { Page } from "../App";

interface Props {
  setPage: (p: Page) => void;
}

const severityColors: Record<string, string> = {
  blocker: "text-red-400 font-bold",
  critical: "text-red-400",
  major: "text-orange-400",
  minor: "text-yellow-400",
  trivial: "text-gray-400",
};

const sourceColors: Record<string, string> = {
  manual: "bg-gray-500/20 text-gray-300",
  test_failure: "bg-red-500/20 text-red-300",
  doc_extracted: "bg-blue-500/20 text-blue-300",
};

export function BugsView({ setPage }: Props) {
  const [bugs, setBugs] = useState<BugInfo[]>([]);
  const [severityFilter, setSeverityFilter] = useState("all");
  const [statusFilter, setStatusFilter] = useState("all");
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(() => {
    listBugs({
      severity: severityFilter === "all" ? undefined : severityFilter,
      status: statusFilter === "all" ? undefined : statusFilter,
    })
      .then(setBugs)
      .catch((e) => setError(e?.toString()));
  }, [severityFilter, statusFilter]);

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
    total: bugs.length,
    open: bugs.filter((b) => b.status === "open" || b.status === "confirmed" || b.status === "in_progress").length,
    fixed: bugs.filter((b) => b.status === "fixed" || b.status === "verified").length,
    closed: bugs.filter((b) => b.status === "closed" || b.status === "wont_fix").length,
  };

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold flex items-center gap-3">
        <Bug size={24} />
        Bugs
      </h2>

      <div className="grid grid-cols-4 gap-4">
        <StatCard label="Total" value={counts.total} color="var(--accent)" />
        <StatCard label="Open" value={counts.open} color="var(--accent-red, #ef4444)" />
        <StatCard label="Fixed" value={counts.fixed} color="var(--accent-green)" />
        <StatCard label="Closed" value={counts.closed} color="var(--text-secondary)" />
      </div>

      <div className="flex gap-4">
        <div className="flex gap-2 items-center">
          <span className="text-xs text-[var(--text-secondary)]">Severity:</span>
          {["all", "blocker", "critical", "major", "minor", "trivial"].map((s) => (
            <button
              key={s}
              onClick={() => setSeverityFilter(s)}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                severityFilter === s
                  ? "bg-[var(--accent)]/15 text-[var(--accent)]"
                  : "bg-[var(--bg-secondary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
              }`}
            >
              {s.charAt(0).toUpperCase() + s.slice(1)}
            </button>
          ))}
        </div>
        <div className="flex gap-2 items-center">
          <span className="text-xs text-[var(--text-secondary)]">Status:</span>
          {["all", "open", "confirmed", "in_progress", "fixed", "verified", "closed", "wont_fix"].map((s) => (
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
        {bugs.length === 0 ? (
          <div className="p-6 text-center text-[var(--text-secondary)]">
            No bugs found.
          </div>
        ) : (
          <div className="divide-y divide-[var(--border)]">
            {bugs.map((bug) => (
              <button
                key={bug.id}
                onClick={() => setPage({ kind: "bug", bugKey: bug.key })}
                className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors text-left"
              >
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-mono text-xs text-[var(--accent)]">{bug.key}</span>
                    <span className="font-medium truncate">{bug.title}</span>
                    <StatusBadge status={bug.status} />
                    <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${sourceColors[bug.source] || "bg-gray-500/20 text-gray-300"}`}>
                      {bug.source.replace("_", " ")}
                    </span>
                  </div>
                  <div className="text-xs text-[var(--text-secondary)] mt-0.5 flex items-center gap-3">
                    <span className={severityColors[bug.severity] || ""}>{bug.severity}</span>
                    <span>{bug.priority}</span>
                    <span>{new Date(bug.created_at).toLocaleDateString()}</span>
                  </div>
                </div>
                <ArrowRight size={16} className="text-[var(--text-secondary)] shrink-0 ml-2" />
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function StatCard({ label, value, color }: { label: string; value: number; color: string }) {
  return (
    <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
      <div className="text-2xl font-bold" style={{ color: value > 0 ? color : undefined }}>
        {value}
      </div>
      <div className="text-xs text-[var(--text-secondary)]">{label}</div>
    </div>
  );
}
