import { useEffect, useState } from "react";
import {
  listProjectEntities,
  type ProjectEntityInfo,
} from "../hooks/useTauri";
import { FolderOpen, ArrowRight, Tags, Tag } from "lucide-react";
import { StatusBadge } from "../components/StatusBadge";
import type { Page } from "../App";

interface Props {
  setPage: (p: Page) => void;
}

export function ProjectsView({ setPage }: Props) {
  const [projects, setProjects] = useState<ProjectEntityInfo[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    listProjectEntities()
      .then(setProjects)
      .catch((e) => setError(e?.toString()));
  }, []);

  if (error)
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );

  const counts = {
    total: projects.length,
    active: projects.filter((p) => p.status === "active").length,
    completed: projects.filter((p) => p.status === "completed").length,
    archived: projects.filter((p) => p.status === "archived").length,
  };

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold flex items-center gap-3">
        <FolderOpen size={24} />
        Projects
      </h2>

      <div className="grid grid-cols-4 gap-4">
        <StatCard label="Total" value={counts.total} color="var(--accent)" />
        <StatCard
          label="Active"
          value={counts.active}
          color="var(--accent-green)"
        />
        <StatCard
          label="Completed"
          value={counts.completed}
          color="var(--accent-purple)"
        />
        <StatCard
          label="Archived"
          value={counts.archived}
          color="var(--text-secondary)"
        />
      </div>

      <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
        {projects.length === 0 ? (
          <div className="p-6 text-center text-[var(--text-secondary)]">
            No projects found. Create one with{" "}
            <code className="text-[var(--accent)] bg-[var(--bg-tertiary)] px-1 py-0.5 rounded text-xs">
              popsicle project create &lt;name&gt;
            </code>
          </div>
        ) : (
          <div className="divide-y divide-[var(--border)]">
            {projects.map((project) => (
              <button
                key={project.id}
                onClick={() =>
                  setPage({ kind: "project", projectId: project.id })
                }
                className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors text-left"
              >
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-medium">{project.name}</span>
                    <StatusBadge status={project.status} />
                    <span className="text-xs font-mono text-[var(--text-secondary)]">
                      {project.slug}
                    </span>
                  </div>
                  {project.description && (
                    <div className="text-xs text-[var(--text-secondary)] mt-0.5 truncate">
                      {project.description}
                    </div>
                  )}
                  <div className="flex items-center gap-3 mt-1 text-xs text-[var(--text-secondary)]">
                    <span className="flex items-center gap-1">
                      <Tags size={11} />
                      {project.topic_count} topic
                      {project.topic_count !== 1 ? "s" : ""}
                    </span>
                    {project.tags.length > 0 && (
                      <span className="flex items-center gap-1">
                        <Tag size={11} />
                        {project.tags.join(", ")}
                      </span>
                    )}
                    <span>
                      {new Date(project.created_at).toLocaleDateString()}
                    </span>
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
