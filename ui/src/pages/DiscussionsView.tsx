import { useEffect, useState } from "react";
import { listDiscussions, type DiscussionInfo } from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import { MessageCircle, ArrowRight, Users } from "lucide-react";
import type { Page } from "../App";

interface Props {
  setPage: (p: Page) => void;
}

export function DiscussionsView({ setPage }: Props) {
  const [discussions, setDiscussions] = useState<DiscussionInfo[]>([]);
  const [filter, setFilter] = useState<"all" | "active" | "concluded">("all");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const statusFilter = filter === "all" ? undefined : filter;
    listDiscussions({ status: statusFilter })
      .then(setDiscussions)
      .catch((e) => setError(e?.toString()));
  }, [filter]);

  if (error)
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );

  const activeCount = discussions.filter((d) => d.status === "active").length;
  const concludedCount = discussions.filter(
    (d) => d.status === "concluded"
  ).length;

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold flex items-center gap-3">
        <MessageCircle size={24} />
        Discussions
      </h2>

      <div className="grid grid-cols-3 gap-4">
        <StatCard
          label="Total"
          value={discussions.length}
          color="var(--accent)"
        />
        <StatCard
          label="Active"
          value={activeCount}
          color="var(--accent-green)"
        />
        <StatCard
          label="Concluded"
          value={concludedCount}
          color="var(--accent-purple)"
        />
      </div>

      <div className="flex gap-2">
        {(["all", "active", "concluded"] as const).map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
              filter === f
                ? "bg-[var(--accent)]/15 text-[var(--accent)]"
                : "bg-[var(--bg-secondary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
            }`}
          >
            {f.charAt(0).toUpperCase() + f.slice(1)}
          </button>
        ))}
      </div>

      <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
        {discussions.length === 0 ? (
          <div className="p-6 text-center text-[var(--text-secondary)]">
            No discussions yet. Discussions are created during multi-role review
            processes.
          </div>
        ) : (
          <div className="divide-y divide-[var(--border)]">
            {discussions.map((disc) => (
              <button
                key={disc.id}
                onClick={() =>
                  setPage({ kind: "discussion", discussionId: disc.id })
                }
                className="w-full px-4 py-3 flex items-center justify-between hover:bg-[var(--bg-tertiary)] transition-colors text-left"
              >
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-medium truncate">{disc.topic}</span>
                    <StatusBadge status={disc.status} />
                  </div>
                  <div className="text-xs text-[var(--text-secondary)] mt-0.5 flex items-center gap-3">
                    <span className="font-mono">{disc.skill}</span>
                    <span className="flex items-center gap-1">
                      <Users size={10} />
                      {disc.message_count} messages
                    </span>
                    <span>
                      {new Date(disc.created_at).toLocaleDateString()}
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
