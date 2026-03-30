import { useEffect, useState } from "react";
import { listSpecs, type SpecInfo } from "../hooks/useTauri";
import { Tags, ArrowRight, GitBranch, FileText, Tag } from "lucide-react";
import type { Page } from "../App";

interface Props {
  setPage: (p: Page) => void;
}

export function SpecsView({ setPage }: Props) {
  const [specs, setSpecs] = useState<SpecInfo[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    listSpecs()
      .then(setSpecs)
      .catch((e) => setError(e?.toString()));
  }, []);

  if (error)
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold flex items-center gap-3">
        <Tags size={24} />
        Specs
      </h2>

      <div className="grid grid-cols-3 gap-4">
        <StatCard label="Total Specs" value={specs.length} color="var(--accent)" />
        <StatCard
          label="Total Runs"
          value={specs.reduce((acc, t) => acc + t.run_count, 0)}
          color="var(--accent-purple)"
        />
        <StatCard
          label="Total Documents"
          value={specs.reduce((acc, t) => acc + t.doc_count, 0)}
          color="var(--accent-green)"
        />
      </div>

      <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
        {specs.length === 0 ? (
          <div className="p-6 text-center text-[var(--text-secondary)]">
            No specs found. Create one with{" "}
            <code className="text-[var(--accent)] bg-[var(--bg-tertiary)] px-1 py-0.5 rounded text-xs">
              popsicle spec create &lt;name&gt;
            </code>
          </div>
        ) : (
          <div className="divide-y divide-[var(--border)]">
            {specs.map((spec) => (
              <button
                key={spec.id}
                onClick={() => setPage({ kind: "spec", specName: spec.name })}
                className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors text-left"
              >
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-medium">{spec.name}</span>
                    <span className="text-xs font-mono text-[var(--text-secondary)]">
                      {spec.slug}
                    </span>
                  </div>
                  {spec.description && (
                    <div className="text-xs text-[var(--text-secondary)] mt-0.5 truncate">
                      {spec.description}
                    </div>
                  )}
                  <div className="flex items-center gap-3 mt-1 text-xs text-[var(--text-secondary)]">
                    <span className="flex items-center gap-1">
                      <GitBranch size={11} />
                      {spec.run_count} run{spec.run_count !== 1 ? "s" : ""}
                    </span>
                    <span className="flex items-center gap-1">
                      <FileText size={11} />
                      {spec.doc_count} doc{spec.doc_count !== 1 ? "s" : ""}
                    </span>
                    {spec.tags.length > 0 && (
                      <span className="flex items-center gap-1">
                        <Tag size={11} />
                        {spec.tags.join(", ")}
                      </span>
                    )}
                    <span>{new Date(spec.created_at).toLocaleDateString()}</span>
                  </div>
                </div>
                <ArrowRight
                  size={16}
                  className="text-[var(--text-secondary)] shrink-0 ml-2"
                />
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function StatCard({
  label,
  value,
  color,
}: {
  label: string;
  value: number;
  color: string;
}) {
  return (
    <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
      <div
        className="text-2xl font-bold"
        style={{ color: value > 0 ? color : undefined }}
      >
        {value}
      </div>
      <div className="text-xs text-[var(--text-secondary)]">{label}</div>
    </div>
  );
}
