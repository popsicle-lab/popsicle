import { useEffect, useState } from "react";
import {
  Clock,
  FolderOpen,
  FolderSearch,
  Star,
} from "lucide-react";
import type { ProjectInfo } from "../hooks/useTauri";
import {
  listRegisteredProjects,
  pickProjectDirectory,
} from "../hooks/useTauri";

interface Props {
  onSelect: (path: string) => Promise<void>;
  initialPath?: string;
}

function formatRelativeTime(ts: number | null): string {
  if (!ts) return "Not opened yet";
  const diff = Math.max(0, Math.floor(Date.now() / 1000 - ts));
  if (diff < 60) return "Opened just now";
  if (diff < 3600) return `Opened ${Math.floor(diff / 60)} minutes ago`;
  if (diff < 86400) return `Opened ${Math.floor(diff / 3600)} hours ago`;
  return `Opened ${Math.floor(diff / 86400)} days ago`;
}

export function ProjectPicker({ onSelect, initialPath }: Props) {
  const [path, setPath] = useState(initialPath ?? "");
  const [projects, setProjects] = useState<ProjectInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    listRegisteredProjects()
      .then((list) => setProjects(list.projects.filter((p) => p.is_valid)))
      .catch(console.error);
  }, []);

  const openPath = async (target: string) => {
    if (!target.trim()) return;
    setLoading(true);
    setError(null);
    try {
      await onSelect(target.trim());
    } catch (err: unknown) {
      setError(String(err) || "Failed to open project");
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await openPath(path);
  };

  const handleBrowse = async () => {
    const picked = await pickProjectDirectory();
    if (picked) {
      setPath(picked);
      await openPath(picked);
    }
  };

  const recents = projects.filter((p) => p.last_opened_at != null);

  return (
    <div className="welcome-shell flex min-h-screen items-center justify-center p-6">
      <div className="welcome-panel w-full max-w-2xl">
        <div className="mb-8 text-center">
          <div className="welcome-logo mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl text-3xl">
            🐕
          </div>
          <h1 className="text-3xl font-bold tracking-tight">Popsicle</h1>
          <p className="mt-2 text-[var(--text-secondary)]">
            Open a workspace to manage issues, pipelines, and product specs
          </p>
        </div>

        {recents.length > 0 && (
          <section className="mb-6">
            <h2 className="mb-3 flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-[var(--text-secondary)]">
              <Clock size={14} />
              Recent projects
            </h2>
            <div className="grid gap-2 sm:grid-cols-2">
              {recents.map((p) => (
                <button
                  key={p.path}
                  type="button"
                  disabled={loading}
                  onClick={() => openPath(p.path)}
                  className="project-card group flex flex-col items-start rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)]/80 p-4 text-left transition-all hover:border-[var(--accent)]/50 hover:bg-[var(--bg-secondary)] disabled:opacity-50"
                >
                  <div className="flex w-full items-center justify-between gap-2">
                    <span className="font-medium">{p.name}</span>
                    {p.is_default && (
                      <Star
                        size={14}
                        className="text-[var(--accent-yellow)]"
                      />
                    )}
                  </div>
                  <span className="mt-1 line-clamp-1 w-full text-xs text-[var(--text-secondary)]">
                    {p.path}
                  </span>
                  <span className="mt-2 text-[11px] text-[var(--text-secondary)]">
                    {formatRelativeTime(p.last_opened_at)}
                  </span>
                </button>
              ))}
            </div>
          </section>
        )}

        <form onSubmit={handleSubmit} className="space-y-3">
          <label className="text-xs font-semibold uppercase tracking-wide text-[var(--text-secondary)]">
            Open by path
          </label>
          <div className="flex gap-2">
            <div className="relative flex-1">
              <FolderOpen
                size={18}
                className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-secondary)]"
              />
              <input
                type="text"
                value={path}
                onChange={(e) => setPath(e.target.value)}
                placeholder="/path/to/your/project"
                className="w-full rounded-xl border border-[var(--border)] bg-[var(--bg-primary)] py-3 pl-10 pr-3 text-sm focus:border-[var(--accent)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/20"
              />
            </div>
            <button
              type="button"
              onClick={handleBrowse}
              disabled={loading}
              className="inline-flex items-center gap-2 rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] px-4 py-3 text-sm font-medium hover:border-[var(--accent)]/40 disabled:opacity-50"
            >
              <FolderSearch size={16} />
              Browse
            </button>
            <button
              type="submit"
              disabled={loading || !path.trim()}
              className="rounded-xl bg-[var(--accent)] px-5 py-3 text-sm font-semibold text-[var(--bg-primary)] disabled:opacity-50 hover:opacity-90"
            >
              {loading ? "Opening…" : "Open"}
            </button>
          </div>
          {error && (
            <p className="text-sm text-[var(--accent-red)]">{error}</p>
          )}
        </form>
      </div>
    </div>
  );
}
