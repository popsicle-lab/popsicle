import { useState } from "react";
import { FolderOpen } from "lucide-react";

interface Props {
  onSelect: (path: string) => Promise<void>;
  initialPath?: string;
}

export function ProjectPicker({ onSelect, initialPath }: Props) {
  const [path, setPath] = useState(initialPath ?? "");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!path.trim()) return;
    setLoading(true);
    setError(null);
    try {
      await onSelect(path.trim());
    } catch (err: any) {
      setError(err?.toString() || "Failed to open project");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex items-center justify-center h-screen">
      <div className="bg-[var(--bg-secondary)] rounded-xl p-8 w-[480px] border border-[var(--border)]">
        <div className="text-center mb-6">
          <span className="text-5xl">🐕</span>
          <h1 className="text-2xl font-bold mt-3">Popsicle</h1>
          <p className="text-[var(--text-secondary)] mt-1">
            Open a Popsicle project to get started
          </p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="flex gap-2">
            <div className="flex-1 relative">
              <FolderOpen
                size={18}
                className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-secondary)]"
              />
              <input
                type="text"
                value={path}
                onChange={(e) => setPath(e.target.value)}
                placeholder="/path/to/your/project"
                className="w-full pl-10 pr-3 py-2.5 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg text-sm focus:outline-none focus:border-[var(--accent)]"
              />
            </div>
            <button
              type="submit"
              disabled={loading || !path.trim()}
              className="px-5 py-2.5 bg-[var(--accent)] text-[var(--bg-primary)] rounded-lg text-sm font-medium disabled:opacity-50 hover:opacity-90 transition-opacity"
            >
              {loading ? "..." : "Open"}
            </button>
          </div>
          {error && (
            <p className="text-[var(--accent-red)] text-sm">{error}</p>
          )}
        </form>
      </div>
    </div>
  );
}
