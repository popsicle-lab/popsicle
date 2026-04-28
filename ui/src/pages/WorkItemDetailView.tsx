import { useEffect, useState } from "react";
import { ArrowLeft, Bug, BookOpen, FlaskConical } from "lucide-react";
import { getWorkItem, type WorkItemFull } from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import type { Page } from "../App";

interface Props {
  itemKey: string;
  setPage: (p: Page) => void;
  fromIssue?: string;
}

const kindMeta: Record<string, { label: string; icon: typeof Bug; color: string }> = {
  bug: { label: "Bug", icon: Bug, color: "text-red-300" },
  story: { label: "User Story", icon: BookOpen, color: "text-blue-300" },
  testcase: { label: "Test Case", icon: FlaskConical, color: "text-purple-300" },
};

export function WorkItemDetailView({ itemKey, setPage, fromIssue }: Props) {
  const [item, setItem] = useState<WorkItemFull | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getWorkItem(itemKey)
      .then(setItem)
      .catch((e) => setError(e?.toString()));
  }, [itemKey]);

  if (error)
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );
  if (!item)
    return <div className="text-[var(--text-secondary)]">Loading…</div>;

  const meta = kindMeta[item.kind] ?? {
    label: item.kind,
    icon: BookOpen,
    color: "text-gray-300",
  };
  const Icon = meta.icon;

  return (
    <div className="max-w-4xl space-y-5">
      <button
        onClick={() =>
          setPage(
            fromIssue
              ? { kind: "issue", issueKey: fromIssue }
              : { kind: "issues" }
          )
        }
        className="text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] flex items-center gap-1.5"
      >
        <ArrowLeft size={14} /> Back
      </button>

      <div>
        <div className="flex items-center gap-2 mb-1">
          <Icon size={16} className={meta.color} />
          <span className="text-xs uppercase tracking-wide text-[var(--text-secondary)]">
            {meta.label}
          </span>
          <span className="font-mono text-xs text-[var(--accent)]">{item.key}</span>
          <StatusBadge status={item.status || "—"} />
        </div>
        <h1 className="text-2xl font-bold">{item.title}</h1>
      </div>

      <div className="grid grid-cols-2 gap-3">
        <InfoCard label="Priority" value={item.priority} />
        {item.issue_id && <InfoCard label="Issue" value={item.issue_id} />}
        {item.pipeline_run_id && (
          <InfoCard label="Pipeline Run" value={item.pipeline_run_id} />
        )}
        {item.source_doc_id && (
          <InfoCard label="Source Doc" value={item.source_doc_id} />
        )}
      </div>

      {item.labels.length > 0 && (
        <div className="flex flex-wrap gap-1.5">
          {item.labels.map((l) => (
            <span
              key={l}
              className="px-2 py-0.5 rounded-full text-xs bg-[var(--bg-tertiary)] text-[var(--text-secondary)]"
            >
              {l}
            </span>
          ))}
        </div>
      )}

      {item.description && (
        <Section title="Description">
          <p className="whitespace-pre-wrap text-sm">{item.description}</p>
        </Section>
      )}

      {Object.keys(item.fields).length > 0 && (
        <Section title="Fields">
          <pre className="text-xs bg-[var(--bg-tertiary)] rounded-lg p-3 overflow-auto">
            {JSON.stringify(item.fields, null, 2)}
          </pre>
        </Section>
      )}

      <div className="text-xs text-[var(--text-secondary)] pt-2 border-t border-[var(--border)]">
        Created {new Date(item.created_at).toLocaleString()} · Updated{" "}
        {new Date(item.updated_at).toLocaleString()}
      </div>
    </div>
  );
}

function InfoCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-[var(--bg-secondary)] rounded-lg border border-[var(--border)] p-3">
      <div className="text-[10px] uppercase text-[var(--text-secondary)] mb-1">
        {label}
      </div>
      <div className="text-sm font-mono break-all">{value}</div>
    </div>
  );
}

function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <h2 className="text-sm font-semibold uppercase tracking-wide text-[var(--text-secondary)] mb-2">
        {title}
      </h2>
      <div className="bg-[var(--bg-secondary)] rounded-lg border border-[var(--border)] p-4">
        {children}
      </div>
    </div>
  );
}
