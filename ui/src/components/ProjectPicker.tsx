import { useEffect, useState } from "react";
import {
  Clock,
  FolderOpen,
  FolderSearch,
  Layers,
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
  if (diff < 60) return "Just now";
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  return `${Math.floor(diff / 86400)}d ago`;
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
      <div className="welcome-panel w-full max-w-xl">
        <div className="mb-8 text-center">
          <div className="welcome-logo mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-[var(--radius-md)]">
            <Layers size={22} strokeWidth={2.25} />
          </div>
          <h1 className="text-xl font-semibold tracking-tight">Open workspace</h1>
          <p className="mt-1.5 text-[13px] text-[var(--text-secondary)]">
            Manage issues, pipelines, and product specs
          </p>
        </div>

        {recents.length > 0 && (
          <section className="mb-6">
            <h2 className="section-label mb-3 flex items-center gap-1.5">
              <Clock size={12} />
              Recent
            </h2>
            <div className="grid gap-2">
              {recents.map((p) => (
                <button
                  key={p.path}
                  type="button"
                  disabled={loading}
                  onClick={() => openPath(p.path)}
                  className="card card-interactive flex flex-col items-start p-3.5 text-left disabled:opacity-50"
                >
                  <div className="flex w-full items-center justify-between gap-2">
                    <span className="text-[13px] font-medium">{p.name}</span>
                    {p.is_default && (
                      <Star size={12} className="text-[var(--accent-yellow)]" />
                    )}
                  </div>
                  <span className="mt-0.5 line-clamp-1 w-full text-[11px] text-[var(--text-muted)]">
                    {p.path}
                  </span>
                  <span className="mt-1.5 text-[11px] text-[var(--text-muted)]">
                    {formatRelativeTime(p.last_opened_at)}
                  </span>
                </button>
              ))}
            </div>
          </section>
        )}

        <form onSubmit={handleSubmit} className="space-y-3">
          <label className="section-label">Path</label>
          <div className="flex flex-col gap-2 sm:flex-row">
            <div className="relative flex-1">
              <FolderOpen
                size={15}
                className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)]"
              />
              <input
                type="text"
                value={path}
                onChange={(e) => setPath(e.target.value)}
                placeholder="/path/to/project"
                className="input py-2.5 pl-9"
              />
            </div>
            <div className="flex gap-2">
              <button
                type="button"
                onClick={handleBrowse}
                disabled={loading}
                className="btn btn-secondary shrink-0"
              >
                <FolderSearch size={15} />
                Browse
              </button>
              <button
                type="submit"
                disabled={loading || !path.trim()}
                className="btn btn-primary shrink-0"
              >
                {loading ? "Opening…" : "Open"}
              </button>
            </div>
          </div>
          {error && (
            <p className="text-[13px] text-[var(--accent-red)]">{error}</p>
          )}
        </form>
      </div>
    </div>
  );
}
