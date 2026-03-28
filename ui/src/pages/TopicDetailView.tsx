import { useEffect, useState } from "react";
import { getTopic, type TopicDetailInfo } from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import {
  Tags,
  ArrowLeft,
  GitBranch,
  FileText,
  Tag,
  Zap,
  RefreshCw,
  ArrowRight,
} from "lucide-react";
import type { Page } from "../App";

interface Props {
  topicName: string;
  setPage: (p: Page) => void;
}

const runTypeColors: Record<string, string> = {
  New: "bg-blue-500/20 text-blue-300",
  Revision: "bg-amber-500/20 text-amber-300",
  Continuation: "bg-purple-500/20 text-purple-300",
};

export function TopicDetailView({ topicName, setPage }: Props) {
  const [topic, setTopic] = useState<TopicDetailInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getTopic(topicName)
      .then(setTopic)
      .catch((e) => setError(e?.toString()));
  }, [topicName]);

  if (error)
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );
  if (!topic) return <div className="text-[var(--text-secondary)]">Loading…</div>;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <button
          onClick={() => setPage({ kind: "topics" })}
          className="flex items-center gap-1 text-sm text-[var(--text-secondary)] hover:text-[var(--accent)] transition-colors mb-3"
        >
          <ArrowLeft size={14} /> Back to Topics
        </button>
        <div className="flex items-center gap-3">
          <Tags size={24} className="text-[var(--accent)]" />
          <div>
            <h2 className="text-2xl font-bold">{topic.name}</h2>
            {topic.description && (
              <p className="text-sm text-[var(--text-secondary)] mt-0.5">
                {topic.description}
              </p>
            )}
          </div>
        </div>
        <div className="flex items-center gap-4 mt-2 text-xs text-[var(--text-secondary)]">
          <span className="font-mono">{topic.slug}</span>
          <span>{new Date(topic.created_at).toLocaleDateString()}</span>
          {topic.tags.length > 0 && (
            <span className="flex items-center gap-1">
              <Tag size={11} />
              {topic.tags.map((tag) => (
                <span
                  key={tag}
                  className="px-1.5 py-0.5 rounded bg-cyan-500/15 text-cyan-300 text-[10px] font-medium"
                >
                  {tag}
                </span>
              ))}
            </span>
          )}
        </div>
      </div>

      {/* Stats row */}
      <div className="grid grid-cols-2 gap-4">
        <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)] flex items-center gap-3">
          <div className="p-2 rounded-lg bg-[var(--accent-purple)]/15">
            <GitBranch size={20} className="text-[var(--accent-purple)]" />
          </div>
          <div>
            <div className="text-2xl font-bold">{topic.runs.length}</div>
            <div className="text-xs text-[var(--text-secondary)]">Pipeline Runs</div>
          </div>
        </div>
        <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)] flex items-center gap-3">
          <div className="p-2 rounded-lg bg-[var(--accent-green)]/15">
            <FileText size={20} className="text-[var(--accent-green)]" />
          </div>
          <div>
            <div className="text-2xl font-bold">{topic.documents.length}</div>
            <div className="text-xs text-[var(--text-secondary)]">Documents (latest)</div>
          </div>
        </div>
      </div>

      {/* Pipeline Runs */}
      <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
        <div className="px-4 py-3 border-b border-[var(--border)] flex items-center gap-2">
          <GitBranch size={16} className="text-[var(--accent-purple)]" />
          <h3 className="font-medium text-sm">Pipeline Runs</h3>
        </div>
        {topic.runs.length === 0 ? (
          <div className="p-6 text-center text-[var(--text-secondary)]">
            No pipeline runs in this topic yet.
          </div>
        ) : (
          <div className="divide-y divide-[var(--border)]">
            {topic.runs.map((run) => (
              <button
                key={run.id}
                onClick={() => setPage({ kind: "pipeline", runId: run.id })}
                className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors text-left"
              >
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-medium">{run.title}</span>
                    <RunTypeBadge runType={run.run_type} />
                    {run.pipeline_name === "quick" && (
                      <span className="inline-flex items-center gap-1 text-xs bg-yellow-500/15 text-yellow-300 px-1.5 py-0.5 rounded-full">
                        <Zap size={8} /> quick
                      </span>
                    )}
                  </div>
                  <div className="text-xs text-[var(--text-secondary)] mt-0.5">
                    {run.pipeline_name} &middot; {run.id.slice(0, 8)} &middot;{" "}
                    {new Date(run.created_at).toLocaleDateString()}
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

      {/* Documents */}
      <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
        <div className="px-4 py-3 border-b border-[var(--border)] flex items-center gap-2">
          <FileText size={16} className="text-[var(--accent-green)]" />
          <h3 className="font-medium text-sm">Documents (Latest Versions)</h3>
        </div>
        {topic.documents.length === 0 ? (
          <div className="p-6 text-center text-[var(--text-secondary)]">
            No documents in this topic yet.
          </div>
        ) : (
          <div className="divide-y divide-[var(--border)]">
            {topic.documents.map((doc) => (
              <button
                key={doc.id}
                onClick={() => setPage({ kind: "document", docId: doc.id })}
                className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors text-left"
              >
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-medium">{doc.title}</span>
                    <StatusBadge status={doc.status} />
                  </div>
                  <div className="text-xs text-[var(--text-secondary)] mt-0.5">
                    {doc.doc_type} &middot; {doc.skill_name}
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

function RunTypeBadge({ runType }: { runType: string }) {
  if (runType === "New") return null;
  const color = runTypeColors[runType] || "bg-gray-500/20 text-gray-300";
  const icon = runType === "Revision" ? <RefreshCw size={10} /> : null;
  return (
    <span
      className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium ${color}`}
    >
      {icon}
      {runType}
    </span>
  );
}
