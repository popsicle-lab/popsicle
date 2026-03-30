import { useEffect, useState } from "react";
import {
  getProjectEntity,
  type ProjectEntityDetail,
} from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import {
  FolderOpen,
  ArrowLeft,
  Tags,
  Tag,
  GitBranch,
  FileText,
  ArrowRight,
} from "lucide-react";
import type { Page } from "../App";

interface Props {
  projectId: string;
  setPage: (p: Page) => void;
}

export function ProjectDetailView({ projectId, setPage }: Props) {
  const [project, setProject] = useState<ProjectEntityDetail | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getProjectEntity(projectId)
      .then(setProject)
      .catch((e) => setError(e?.toString()));
  }, [projectId]);

  if (error)
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );
  if (!project)
    return (
      <div className="text-[var(--text-secondary)]">Loading…</div>
    );

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <button
          onClick={() => setPage({ kind: "projects" })}
          className="flex items-center gap-1 text-sm text-[var(--text-secondary)] hover:text-[var(--accent)] transition-colors mb-3"
        >
          <ArrowLeft size={14} /> Back to Projects
        </button>
        <div className="flex items-center gap-3">
          <FolderOpen size={24} className="text-[var(--accent)]" />
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-2xl font-bold">{project.name}</h2>
              <StatusBadge status={project.status} />
            </div>
            {project.description && (
              <p className="text-sm text-[var(--text-secondary)] mt-0.5">
                {project.description}
              </p>
            )}
          </div>
        </div>
        <div className="flex items-center gap-4 mt-2 text-xs text-[var(--text-secondary)]">
          <span className="font-mono">{project.slug}</span>
          <span>
            {new Date(project.created_at).toLocaleDateString()}
          </span>
          {project.tags.length > 0 && (
            <span className="flex items-center gap-1">
              <Tag size={11} />
              {project.tags.map((tag) => (
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

      {/* Stats */}
      <div className="grid grid-cols-3 gap-4">
        <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)] flex items-center gap-3">
          <div className="p-2 rounded-lg bg-[var(--accent)]/15">
            <Tags size={20} className="text-[var(--accent)]" />
          </div>
          <div>
            <div className="text-2xl font-bold">{project.topics.length}</div>
            <div className="text-xs text-[var(--text-secondary)]">Topics</div>
          </div>
        </div>
        <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)] flex items-center gap-3">
          <div className="p-2 rounded-lg bg-[var(--accent-purple)]/15">
            <GitBranch size={20} className="text-[var(--accent-purple)]" />
          </div>
          <div>
            <div className="text-2xl font-bold">
              {project.topics.reduce((acc, t) => acc + t.run_count, 0)}
            </div>
            <div className="text-xs text-[var(--text-secondary)]">
              Total Runs
            </div>
          </div>
        </div>
        <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)] flex items-center gap-3">
          <div className="p-2 rounded-lg bg-[var(--accent-green)]/15">
            <FileText size={20} className="text-[var(--accent-green)]" />
          </div>
          <div>
            <div className="text-2xl font-bold">
              {project.topics.reduce((acc, t) => acc + t.doc_count, 0)}
            </div>
            <div className="text-xs text-[var(--text-secondary)]">
              Total Documents
            </div>
          </div>
        </div>
      </div>

      {/* Topics */}
      <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
        <div className="px-4 py-3 border-b border-[var(--border)] flex items-center gap-2">
          <Tags size={16} className="text-[var(--accent)]" />
          <h3 className="font-medium text-sm">Topics</h3>
        </div>
        {project.topics.length === 0 ? (
          <div className="p-6 text-center text-[var(--text-secondary)]">
            No topics in this project yet.
          </div>
        ) : (
          <div className="divide-y divide-[var(--border)]">
            {project.topics.map((topic) => (
              <button
                key={topic.id}
                onClick={() =>
                  setPage({ kind: "topic", topicName: topic.name })
                }
                className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors text-left"
              >
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-medium">{topic.name}</span>
                    <span className="text-xs font-mono text-[var(--text-secondary)]">
                      {topic.slug}
                    </span>
                  </div>
                  {topic.description && (
                    <div className="text-xs text-[var(--text-secondary)] mt-0.5 truncate">
                      {topic.description}
                    </div>
                  )}
                  <div className="flex items-center gap-3 mt-1 text-xs text-[var(--text-secondary)]">
                    <span className="flex items-center gap-1">
                      <GitBranch size={11} />
                      {topic.run_count} run
                      {topic.run_count !== 1 ? "s" : ""}
                    </span>
                    <span className="flex items-center gap-1">
                      <FileText size={11} />
                      {topic.doc_count} doc
                      {topic.doc_count !== 1 ? "s" : ""}
                    </span>
                    {topic.tags.length > 0 && (
                      <span className="flex items-center gap-1">
                        <Tag size={11} />
                        {topic.tags.join(", ")}
                      </span>
                    )}
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
