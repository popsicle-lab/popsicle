import { useEffect, useState } from "react";
import { getTestCase, type TestCaseFull } from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import { FlaskConical, ArrowLeft } from "lucide-react";
import type { Page } from "../App";

interface Props {
  testCaseKey: string;
  setPage: (p: Page) => void;
}

const typeColors: Record<string, string> = {
  unit: "bg-blue-500/20 text-blue-300",
  api: "bg-purple-500/20 text-purple-300",
  e2e: "bg-green-500/20 text-green-300",
  ui: "bg-pink-500/20 text-pink-300",
};

const priorityColors: Record<string, string> = {
  p0: "bg-red-500/20 text-red-300",
  p1: "bg-orange-500/20 text-orange-300",
  p2: "bg-yellow-500/20 text-yellow-300",
};

export function TestCaseDetailView({ testCaseKey, setPage }: Props) {
  const [tc, setTc] = useState<TestCaseFull | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getTestCase(testCaseKey)
      .then(setTc)
      .catch((e) => setError(e?.toString()));
  }, [testCaseKey]);

  if (error)
    return (
      <div className="text-red-400 p-4 bg-red-500/10 rounded-lg">{error}</div>
    );
  if (!tc) return <div className="text-[var(--text-secondary)]">Loading...</div>;

  return (
    <div className="space-y-6 max-w-4xl">
      <button
        onClick={() => setPage({ kind: "testcases" })}
        className="flex items-center gap-2 text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
      >
        <ArrowLeft size={16} /> Back to Test Cases
      </button>

      <div className="flex items-start gap-4">
        <FlaskConical size={28} className="text-blue-400 mt-1 shrink-0" />
        <div className="min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <span className="font-mono text-sm text-[var(--accent)]">{tc.key}</span>
            <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${typeColors[tc.test_type] || "bg-gray-500/20 text-gray-300"}`}>
              {tc.test_type}
            </span>
            <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${priorityColors[tc.priority_level] || "bg-gray-500/20 text-gray-300"}`}>
              {tc.priority_level.toUpperCase()}
            </span>
            <StatusBadge status={tc.status} />
          </div>
          <h2 className="text-2xl font-bold mt-1">{tc.title}</h2>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <InfoCard label="Test Type" value={tc.test_type.toUpperCase()} />
        <InfoCard label="Priority" value={tc.priority_level.toUpperCase()} />
        <InfoCard label="Created" value={new Date(tc.created_at).toLocaleString()} />
        <InfoCard label="Updated" value={new Date(tc.updated_at).toLocaleString()} />
      </div>

      {tc.description && (
        <Section title="Description">
          <p className="text-sm text-[var(--text-secondary)] whitespace-pre-wrap">{tc.description}</p>
        </Section>
      )}

      {tc.preconditions.length > 0 && (
        <Section title="Preconditions">
          <ul className="list-disc list-inside space-y-1 text-sm text-[var(--text-secondary)]">
            {tc.preconditions.map((p, i) => (
              <li key={i}>{p}</li>
            ))}
          </ul>
        </Section>
      )}

      {tc.steps.length > 0 && (
        <Section title="Steps">
          <ol className="list-decimal list-inside space-y-1 text-sm text-[var(--text-secondary)]">
            {tc.steps.map((step, i) => (
              <li key={i}>{step}</li>
            ))}
          </ol>
        </Section>
      )}

      {tc.expected_result && (
        <Section title="Expected Result">
          <p className="text-sm text-green-400">{tc.expected_result}</p>
        </Section>
      )}

      {tc.labels.length > 0 && (
        <Section title="Labels">
          <div className="flex gap-2 flex-wrap">
            {tc.labels.map((l) => (
              <span key={l} className="px-2 py-0.5 rounded-full text-xs bg-[var(--bg-tertiary)] text-[var(--text-secondary)]">
                {l}
              </span>
            ))}
          </div>
        </Section>
      )}
    </div>
  );
}

function InfoCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-[var(--bg-secondary)] rounded-lg p-3 border border-[var(--border)]">
      <div className="text-xs text-[var(--text-secondary)]">{label}</div>
      <div className="text-sm font-medium mt-0.5">{value}</div>
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
      <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-2">{title}</h3>
      {children}
    </div>
  );
}
