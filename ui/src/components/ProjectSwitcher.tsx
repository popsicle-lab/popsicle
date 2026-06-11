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
  if (!ts) return "never";
  const diff = Math.max(0, Math.floor(Date.now() / 1000 - ts));
  if (diff < 60) return "now";
  if (diff < 3600) return `${Math.floor(diff / 60)}m`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h`;
  return `${Math.floor(diff / 86400)}d`;
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
        className="flex w-full items-center gap-2 rounded-[var(--radius-sm)] border border-[var(--border)] bg-[var(--bg-primary)] px-2.5 py-2 text-left transition-colors hover:border-[var(--border-strong)] hover:bg-[var(--bg-hover)]"
      >
        <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-[var(--radius-sm)] bg-[var(--accent-muted)] text-[var(--accent)]">
          <FolderOpen size={14} />
        </div>
        <div className="min-w-0 flex-1">
          <div className="truncate text-[13px] font-medium">{current.name}</div>
          <div className="truncate text-[11px] text-[var(--text-muted)]">
            {truncatePath(current.path)}
          </div>
        </div>
        <ChevronDown
          size={14}
          className={`shrink-0 text-[var(--text-muted)] transition-transform ${open ? "rotate-180" : ""}`}
        />
      </button>

      {open && (
        <div className="absolute left-0 right-0 top-[calc(100%+4px)] z-50 overflow-hidden rounded-[var(--radius-md)] border border-[var(--border)] bg-[var(--bg-elevated)] shadow-[var(--shadow-md)]">
          <div className="section-label border-b border-[var(--border)] px-3 py-2">
            Projects
          </div>
          <div className="max-h-60 overflow-y-auto p-1">
            {projects.length === 0 && (
              <p className="empty-state py-6">No saved projects</p>
            )}
            {projects.map((p) => {
              const active = p.path === current.path;
              return (
                <div
                  key={p.path}
                  className={`group flex items-center gap-1 rounded-[var(--radius-sm)] px-1.5 py-1.5 ${active ? "bg-[var(--accent-muted)]" : "hover:bg-[var(--bg-hover)]"}`}
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
                      <span className="truncate text-[13px] font-medium">
                        {p.name}
                      </span>
                      {p.is_default && (
                        <Star
                          size={11}
                          className="shrink-0 text-[var(--accent-yellow)]"
                        />
                      )}
                    </div>
                    <div className="flex items-center gap-1 text-[11px] text-[var(--text-muted)]">
                      <Clock size={10} />
                      {formatRelativeTime(p.last_opened_at)}
                      {!p.is_valid && (
                        <span className="text-[var(--accent-red)]">· missing</span>
                      )}
                    </div>
                  </button>
                  <button
                    type="button"
                    title="Remove from registry"
                    disabled={loading}
                    onClick={() => handleRemove(p.name)}
                    className="btn btn-ghost rounded-[var(--radius-sm)] p-1 opacity-0 group-hover:opacity-100 hover:text-[var(--accent-red)]"
                  >
                    <Trash2 size={13} />
                  </button>
                </div>
              );
            })}
          </div>
          <div className="space-y-0.5 border-t border-[var(--border)] p-1.5">
            <button
              type="button"
              onClick={handlePick}
              className="flex w-full items-center gap-2 rounded-[var(--radius-sm)] px-2.5 py-2 text-[13px] hover:bg-[var(--bg-hover)]"
            >
              <FolderOpen size={15} className="text-[var(--accent)]" />
              Browse folder…
            </button>
            <button
              type="button"
              onClick={() => {
                setOpen(false);
                onBrowseOther();
              }}
              className="flex w-full items-center gap-2 rounded-[var(--radius-sm)] px-2.5 py-2 text-[13px] hover:bg-[var(--bg-hover)]"
            >
              <Plus size={15} className="text-[var(--accent)]" />
              Switch project
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
