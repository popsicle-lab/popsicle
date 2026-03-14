import { useEffect, useState, useCallback } from "react";
import { listUserStories, type UserStoryInfo } from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import { BookOpen, ArrowRight } from "lucide-react";
import type { Page } from "../App";

interface Props {
  setPage: (p: Page) => void;
}

const priorityColors: Record<string, string> = {
  critical: "text-red-400",
  high: "text-orange-400",
  medium: "text-yellow-400",
  low: "text-gray-400",
};

export function StoriesView({ setPage }: Props) {
  const [stories, setStories] = useState<UserStoryInfo[]>([]);
  const [statusFilter, setStatusFilter] = useState("all");
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(() => {
    listUserStories({
      status: statusFilter === "all" ? undefined : statusFilter,
    })
      .then(setStories)
      .catch((e) => setError(e?.toString()));
  }, [statusFilter]);

  useEffect(() => {
    load();
  }, [load]);

  if (error)
    return (
      <div className="text-red-400 p-4 bg-red-500/10 rounded-lg">{error}</div>
    );

  const counts = {
    total: stories.length,
    draft: stories.filter((s) => s.status === "draft").length,
    accepted: stories.filter((s) => s.status === "accepted").length,
    verified: stories.filter((s) => s.status === "verified").length,
  };

  const totalAc = stories.reduce((sum, s) => sum + s.ac_count, 0);
  const verifiedAc = stories.reduce((sum, s) => sum + s.ac_verified, 0);

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold flex items-center gap-3">
        <BookOpen size={24} />
        User Stories
      </h2>

      <div className="grid grid-cols-5 gap-4">
        <StatCard label="Total" value={counts.total} color="var(--accent)" />
        <StatCard label="Draft" value={counts.draft} color="var(--text-secondary)" />
        <StatCard label="Accepted" value={counts.accepted} color="var(--accent-green)" />
        <StatCard label="Verified" value={counts.verified} color="var(--accent-purple)" />
        <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
          <div
            className="text-2xl font-bold"
            style={{ color: totalAc > 0 ? (verifiedAc === totalAc ? "var(--accent-green)" : "var(--accent-yellow)") : undefined }}
          >
            {verifiedAc}/{totalAc}
          </div>
          <div className="text-xs text-[var(--text-secondary)]">AC Verified</div>
        </div>
      </div>

      <div className="flex gap-2 items-center">
        <span className="text-xs text-[var(--text-secondary)]">Status:</span>
        {["all", "draft", "accepted", "implemented", "verified"].map((s) => (
          <button
            key={s}
            onClick={() => setStatusFilter(s)}
            className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
              statusFilter === s
                ? "bg-[var(--accent)]/15 text-[var(--accent)]"
                : "bg-[var(--bg-secondary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
            }`}
          >
            {s.charAt(0).toUpperCase() + s.slice(1)}
          </button>
        ))}
      </div>

      <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
        {stories.length === 0 ? (
          <div className="p-6 text-center text-[var(--text-secondary)]">
            No user stories found.
          </div>
        ) : (
          <div className="divide-y divide-[var(--border)]">
            {stories.map((story) => (
              <button
                key={story.id}
                onClick={() => setPage({ kind: "story", storyKey: story.key })}
                className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors text-left"
              >
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-mono text-xs text-[var(--accent)]">{story.key}</span>
                    <span className="font-medium truncate">{story.title}</span>
                    <StatusBadge status={story.status} />
                  </div>
                  <div className="text-xs text-[var(--text-secondary)] mt-0.5 flex items-center gap-3">
                    <span className={priorityColors[story.priority] || ""}>{story.priority}</span>
                    {story.persona && <span className="italic">as {story.persona}</span>}
                    <span>
                      AC: {story.ac_verified}/{story.ac_count}
                    </span>
                    <span>{new Date(story.created_at).toLocaleDateString()}</span>
                  </div>
                  {story.ac_count > 0 && (
                    <div className="flex items-center gap-2 mt-1.5">
                      <div className="flex-1 h-1 bg-[var(--bg-tertiary)] rounded-full overflow-hidden max-w-[200px]">
                        <div
                          className="h-full rounded-full transition-all"
                          style={{
                            width: `${(story.ac_verified / story.ac_count) * 100}%`,
                            background: story.ac_verified === story.ac_count ? "var(--accent-green)" : "var(--accent-yellow)",
                          }}
                        />
                      </div>
                      <span className="text-[10px] font-mono text-[var(--text-secondary)]">
                        {story.ac_verified}/{story.ac_count} verified
                      </span>
                    </div>
                  )}
                </div>
                <ArrowRight size={16} className="text-[var(--text-secondary)] shrink-0 ml-2" />
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function StatCard({ label, value, color }: { label: string; value: number; color: string }) {
  return (
    <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
      <div className="text-2xl font-bold" style={{ color: value > 0 ? color : undefined }}>
        {value}
      </div>
      <div className="text-xs text-[var(--text-secondary)]">{label}</div>
    </div>
  );
}
