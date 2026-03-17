import { useEffect, useState } from "react";
import { getBug, type BugFull } from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import { Bug, ArrowLeft } from "lucide-react";
import type { Page } from "../App";

interface Props {
  bugKey: string;
  setPage: (p: Page) => void;
  fromIssue?: string;
}

const severityColors: Record<string, string> = {
  blocker: "bg-red-600/20 text-red-300 border-red-500/30",
  critical: "bg-red-500/20 text-red-300",
  major: "bg-orange-500/20 text-orange-300",
  minor: "bg-yellow-500/20 text-yellow-300",
  trivial: "bg-gray-500/20 text-gray-300",
};

export function BugDetailView({ bugKey, setPage, fromIssue }: Props) {
  const [bug, setBug] = useState<BugFull | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getBug(bugKey)
      .then(setBug)
      .catch((e) => setError(e?.toString()));
  }, [bugKey]);

  if (error)
    return (
      <div className="text-red-400 p-4 bg-red-500/10 rounded-lg">{error}</div>
    );
  if (!bug) return <div className="text-[var(--text-secondary)]">Loading...</div>;

  const handleBack = () => {
    if (fromIssue) {
      setPage({ kind: "issue", issueKey: fromIssue, tab: "bugs" });
    } else {
      setPage({ kind: "issues" });
    }
  };

  return (
    <div className="space-y-6 max-w-4xl">
      <button
        onClick={handleBack}
        className="flex items-center gap-2 text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
      >
        <ArrowLeft size={16} />
        {fromIssue ? `Back to ${fromIssue}` : "Back to Issues"}
      </button>

      <div className="flex items-start gap-4">
        <Bug size={28} className="text-red-400 mt-1 shrink-0" />
        <div className="min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <span className="font-mono text-sm text-[var(--accent)]">{bug.key}</span>
            <StatusBadge status={bug.status} />
            <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${severityColors[bug.severity] || "bg-gray-500/20 text-gray-300"}`}>
              {bug.severity}
            </span>
            <span className="text-xs text-[var(--text-secondary)]">
              {bug.source.replace("_", " ")}
            </span>
          </div>
          <h2 className="text-2xl font-bold mt-1">{bug.title}</h2>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <InfoCard label="Priority" value={bug.priority} />
        <InfoCard label="Source" value={bug.source.replace("_", " ")} />
        <InfoCard label="Created" value={new Date(bug.created_at).toLocaleString()} />
        <InfoCard label="Updated" value={new Date(bug.updated_at).toLocaleString()} />
        {bug.related_test_case_id && <InfoCard label="Related TestCase" value={bug.related_test_case_id} />}
        {bug.fix_commit_sha && <InfoCard label="Fix Commit" value={bug.fix_commit_sha.slice(0, 8)} />}
        {bug.related_commit_sha && <InfoCard label="Related Commit" value={bug.related_commit_sha.slice(0, 8)} />}
      </div>

      {bug.description && (
        <Section title="Description">
          <p className="text-sm text-[var(--text-secondary)] whitespace-pre-wrap">{bug.description}</p>
        </Section>
      )}

      {bug.steps_to_reproduce.length > 0 && (
        <Section title="Steps to Reproduce">
          <ol className="list-decimal list-inside space-y-1 text-sm text-[var(--text-secondary)]">
            {bug.steps_to_reproduce.map((step, i) => (
              <li key={i}>{step}</li>
            ))}
          </ol>
        </Section>
      )}

      {(bug.expected_behavior || bug.actual_behavior) && (
        <div className="grid grid-cols-2 gap-4">
          {bug.expected_behavior && (
            <Section title="Expected Behavior">
              <p className="text-sm text-green-400">{bug.expected_behavior}</p>
            </Section>
          )}
          {bug.actual_behavior && (
            <Section title="Actual Behavior">
              <p className="text-sm text-red-400">{bug.actual_behavior}</p>
            </Section>
          )}
        </div>
      )}

      {bug.environment && (
        <Section title="Environment">
          <p className="text-sm text-[var(--text-secondary)]">{bug.environment}</p>
        </Section>
      )}

      {bug.stack_trace && (
        <Section title="Stack Trace">
          <pre className="text-xs text-[var(--text-secondary)] bg-[var(--bg-primary)] p-3 rounded-lg overflow-x-auto font-mono">
            {bug.stack_trace}
          </pre>
        </Section>
      )}

      {bug.labels.length > 0 && (
        <Section title="Labels">
          <div className="flex gap-2 flex-wrap">
            {bug.labels.map((l) => (
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
