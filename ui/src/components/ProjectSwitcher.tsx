import { useEffect, useRef, useState } from "react";
import {
  ChevronDown,
  Clock,
  FolderOpen,
  Plus,
  Star,
  Trash2,
} from "lucide-react";
import type { ProjectInfo } from "../hooks/useTauri";
import {
  listRegisteredProjects,
  pickProjectDirectory,
  removeRegisteredProject,
} from "../hooks/useTauri";

interface Props {
  current: ProjectInfo;
  onSwitch: (path: string) => Promise<void>;
  onBrowseOther: () => void;
}

function formatRelativeTime(ts: number | null): string {
  if (!ts) return "never opened";
  const diff = Math.max(0, Math.floor(Date.now() / 1000 - ts));
  if (diff < 60) return "just now";
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  return `${Math.floor(diff / 86400)}d ago`;
}

function truncatePath(path: string, max = 28): string {
  if (path.length <= max) return path;
  const parts = path.split("/");
  const name = parts.pop() ?? path;
  if (name.length >= max - 3) return `…/${name.slice(-(max - 4))}`;
  return `…/${name}`;
}

export function ProjectSwitcher({ current, onSwitch, onBrowseOther }: Props) {
  const [open, setOpen] = useState(false);
  const [projects, setProjects] = useState<ProjectInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);

  const refresh = async () => {
    const list = await listRegisteredProjects();
    setProjects(list.projects);
  };

  useEffect(() => {
    if (open) refresh().catch(console.error);
  }, [open]);

  useEffect(() => {
    const onDocClick = (e: MouseEvent) => {
      if (!rootRef.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", onDocClick);
    return () => document.removeEventListener("mousedown", onDocClick);
  }, []);

  const handlePick = async () => {
    setOpen(false);
    const picked = await pickProjectDirectory();
    if (picked) await onSwitch(picked);
  };

  const handleRemove = async (name: string) => {
    setLoading(true);
    try {
      await removeRegisteredProject(name);
      await refresh();
    } finally {
      setLoading(false);
    }
  };

  return (
    <div ref={rootRef} className="relative">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="w-full flex items-center gap-2 rounded-lg border border-[var(--border)] bg-[var(--bg-primary)]/60 px-3 py-2.5 text-left transition-colors hover:border-[var(--accent)]/40 hover:bg-[var(--bg-primary)]"
      >
        <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-[var(--accent)]/15 text-[var(--accent)]">
          <FolderOpen size={16} />
        </div>
        <div className="min-w-0 flex-1">
          <div className="truncate text-sm font-medium">{current.name}</div>
          <div className="truncate text-[11px] text-[var(--text-secondary)]">
            {truncatePath(current.path)}
          </div>
        </div>
        <ChevronDown
          size={16}
          className={`shrink-0 text-[var(--text-secondary)] transition-transform ${open ? "rotate-180" : ""}`}
        />
      </button>

      {open && (
        <div className="absolute left-0 right-0 top-[calc(100%+6px)] z-50 overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] shadow-2xl shadow-black/40">
          <div className="border-b border-[var(--border)] px-3 py-2 text-[11px] font-medium uppercase tracking-wide text-[var(--text-secondary)]">
            Recent & registered
          </div>
          <div className="max-h-64 overflow-y-auto p-1">
            {projects.length === 0 && (
              <p className="px-3 py-4 text-center text-xs text-[var(--text-secondary)]">
                No saved projects yet
              </p>
            )}
            {projects.map((p) => {
              const active = p.path === current.path;
              return (
                <div
                  key={p.path}
                  className={`group flex items-center gap-2 rounded-lg px-2 py-2 ${active ? "bg-[var(--accent)]/10" : "hover:bg-[var(--bg-tertiary)]/60"}`}
                >
                  <button
                    type="button"
                    disabled={!p.is_valid || loading}
                    onClick={async () => {
                      setOpen(false);
                      if (!active) await onSwitch(p.path);
                    }}
                    className="min-w-0 flex-1 text-left disabled:opacity-40"
                  >
                    <div className="flex items-center gap-1.5">
                      <span className="truncate text-sm font-medium">
                        {p.name}
                      </span>
                      {p.is_default && (
                        <Star
                          size={12}
                          className="shrink-0 text-[var(--accent-yellow)]"
                        />
                      )}
                    </div>
                    <div className="flex items-center gap-1 text-[11px] text-[var(--text-secondary)]">
                      <Clock size={10} />
                      {formatRelativeTime(p.last_opened_at)}
                      {!p.is_valid && (
                        <span className="text-[var(--accent-red)]">
                          · missing
                        </span>
                      )}
                    </div>
                  </button>
                  <button
                    type="button"
                    title="Remove from registry"
                    disabled={loading}
                    onClick={() => handleRemove(p.name)}
                    className="rounded p-1 text-[var(--text-secondary)] opacity-0 transition-opacity hover:bg-[var(--bg-primary)] hover:text-[var(--accent-red)] group-hover:opacity-100"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              );
            })}
          </div>
          <div className="space-y-1 border-t border-[var(--border)] p-2">
            <button
              type="button"
              onClick={handlePick}
              className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]/60"
            >
              <FolderOpen size={16} className="text-[var(--accent)]" />
              Browse folder…
            </button>
            <button
              type="button"
              onClick={() => {
                setOpen(false);
                onBrowseOther();
              }}
              className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]/60"
            >
              <Plus size={16} className="text-[var(--accent)]" />
              Open another project
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
