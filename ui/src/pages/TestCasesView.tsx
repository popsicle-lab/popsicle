import { useEffect, useState, useCallback } from "react";
import { listTestCases, getTestCoverage, type TestCaseInfo, type TestCoverageSummary } from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import { FlaskConical, ArrowRight } from "lucide-react";
import type { Page } from "../App";

interface Props {
  setPage: (p: Page) => void;
}

const typeColors: Record<string, string> = {
  unit: "bg-blue-500/20 text-blue-300",
  api: "bg-purple-500/20 text-purple-300",
  e2e: "bg-green-500/20 text-green-300",
  ui: "bg-pink-500/20 text-pink-300",
};

const priorityColors: Record<string, string> = {
  p0: "text-red-400 font-bold",
  p1: "text-orange-400",
  p2: "text-yellow-400",
};

export function TestCasesView({ setPage }: Props) {
  const [cases, setCases] = useState<TestCaseInfo[]>([]);
  const [coverage, setCoverage] = useState<TestCoverageSummary | null>(null);
  const [typeFilter, setTypeFilter] = useState("all");
  const [priorityFilter, setPriorityFilter] = useState("all");
  const [statusFilter, setStatusFilter] = useState("all");
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(() => {
    listTestCases({
      testType: typeFilter === "all" ? undefined : typeFilter,
      priority: priorityFilter === "all" ? undefined : priorityFilter,
      status: statusFilter === "all" ? undefined : statusFilter,
    })
      .then(setCases)
      .catch((e) => setError(e?.toString()));

    getTestCoverage({}).then(setCoverage).catch(() => {});
  }, [typeFilter, priorityFilter, statusFilter]);

  useEffect(() => {
    load();
  }, [load]);

  if (error)
    return (
      <div className="text-red-400 p-4 bg-red-500/10 rounded-lg">{error}</div>
    );

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold flex items-center gap-3">
        <FlaskConical size={24} />
        Test Cases
      </h2>

      {coverage && (
        <div className="grid grid-cols-5 gap-4">
          <StatCard label="Total" value={coverage.total} color="var(--accent)" />
          <StatCard label="Passed" value={coverage.passed} color="var(--accent-green)" />
          <StatCard label="Failed" value={coverage.failed} color="var(--accent-red, #ef4444)" />
          <StatCard label="No Runs" value={coverage.no_runs} color="var(--text-secondary)" />
          <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
            <div
              className="text-2xl font-bold"
              style={{
                color:
                  coverage.pass_rate >= 80
                    ? "var(--accent-green)"
                    : coverage.pass_rate >= 50
                      ? "var(--accent-yellow)"
                      : "var(--accent-red, #ef4444)",
              }}
            >
              {coverage.pass_rate.toFixed(1)}%
            </div>
            <div className="text-xs text-[var(--text-secondary)]">Pass Rate</div>
          </div>
        </div>
      )}

      <div className="flex gap-4 flex-wrap">
        <div className="flex gap-2 items-center">
          <span className="text-xs text-[var(--text-secondary)]">Type:</span>
          {["all", "unit", "api", "e2e", "ui"].map((t) => (
            <button
              key={t}
              onClick={() => setTypeFilter(t)}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                typeFilter === t
                  ? "bg-[var(--accent)]/15 text-[var(--accent)]"
                  : "bg-[var(--bg-secondary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
              }`}
            >
              {t.toUpperCase()}
            </button>
          ))}
        </div>
        <div className="flex gap-2 items-center">
          <span className="text-xs text-[var(--text-secondary)]">Priority:</span>
          {["all", "p0", "p1", "p2"].map((p) => (
            <button
              key={p}
              onClick={() => setPriorityFilter(p)}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                priorityFilter === p
                  ? "bg-[var(--accent)]/15 text-[var(--accent)]"
                  : "bg-[var(--bg-secondary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
              }`}
            >
              {p.toUpperCase()}
            </button>
          ))}
        </div>
        <div className="flex gap-2 items-center">
          <span className="text-xs text-[var(--text-secondary)]">Status:</span>
          {["all", "draft", "ready", "automated", "deprecated"].map((s) => (
            <button
              key={s}
              onClick={() => setStatusFilter(s)}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                statusFilter === s
                  ? "bg-[var(--accent)]/15 text-[var(--accent)]"
                  : "bg-[var(--bg-secondary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
              }`}
            >
              {s.charAt(0).toUpperCase() + s.slice(1)}
            </button>
          ))}
        </div>
      </div>

      <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
        {cases.length === 0 ? (
          <div className="p-6 text-center text-[var(--text-secondary)]">
            No test cases found.
          </div>
        ) : (
          <div className="divide-y divide-[var(--border)]">
            {cases.map((tc) => (
              <button
                key={tc.id}
                onClick={() => setPage({ kind: "testcase", testCaseKey: tc.key })}
                className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors text-left"
              >
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-mono text-xs text-[var(--accent)]">{tc.key}</span>
                    <span className="font-medium truncate">{tc.title}</span>
                    <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${typeColors[tc.test_type] || "bg-gray-500/20 text-gray-300"}`}>
                      {tc.test_type}
                    </span>
                    <StatusBadge status={tc.status} />
                  </div>
                  <div className="text-xs text-[var(--text-secondary)] mt-0.5 flex items-center gap-3">
                    <span className={priorityColors[tc.priority_level] || ""}>{tc.priority_level.toUpperCase()}</span>
                    <span>{new Date(tc.created_at).toLocaleDateString()}</span>
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
