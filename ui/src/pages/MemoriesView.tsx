import { useEffect, useState } from "react";
import {
  listMemories,
  getMemoryStats,
  type MemoryInfo,
  type MemoryStatsInfo,
} from "../hooks/useTauri";
import {
  Brain,
  Bug,
  Lightbulb,
  Repeat,
  AlertTriangle,
  ChevronDown,
  ChevronRight,
  Tag,
  FileCode,
  Clock,
  Archive,
  Zap,
} from "lucide-react";

const typeConfig: Record<
  string,
  { icon: React.ReactNode; color: string; bg: string }
> = {
  BUG: {
    icon: <Bug size={14} />,
    color: "text-red-400",
    bg: "bg-red-500/10",
  },
  DECISION: {
    icon: <Lightbulb size={14} />,
    color: "text-yellow-400",
    bg: "bg-yellow-500/10",
  },
  PATTERN: {
    icon: <Repeat size={14} />,
    color: "text-blue-400",
    bg: "bg-blue-500/10",
  },
  GOTCHA: {
    icon: <AlertTriangle size={14} />,
    color: "text-orange-400",
    bg: "bg-orange-500/10",
  },
};

function TypeBadge({ type: t }: { type: string }) {
  const cfg = typeConfig[t] || typeConfig.BUG;
  return (
    <span
      className={`inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded-full ${cfg.bg} ${cfg.color}`}
    >
      {cfg.icon}
      {t}
    </span>
  );
}

function LayerBadge({ layer }: { layer: string }) {
  const isLong = layer === "long-term";
  return (
    <span
      className={`inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded-full ${
        isLong
          ? "bg-green-500/10 text-green-400"
          : "bg-purple-500/10 text-purple-400"
      }`}
    >
      {isLong ? <Archive size={12} /> : <Clock size={12} />}
      {isLong ? "long-term" : "short-term"}
    </span>
  );
}

function capacityColor(pct: number): string {
  if (pct >= 90) return "var(--accent-red)";
  if (pct >= 70) return "var(--accent-yellow)";
  return "var(--accent-green)";
}

export function MemoriesView() {
  const [memories, setMemories] = useState<MemoryInfo[]>([]);
  const [stats, setStats] = useState<MemoryStatsInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [layerFilter, setLayerFilter] = useState<string>("all");
  const [typeFilter, setTypeFilter] = useState<string>("all");
  const [expandedId, setExpandedId] = useState<number | null>(null);

  useEffect(() => {
    const filters: { layer?: string; memoryType?: string } = {};
    if (layerFilter !== "all") filters.layer = layerFilter;
    if (typeFilter !== "all") filters.memoryType = typeFilter;

    Promise.all([listMemories(filters), getMemoryStats()])
      .then(([m, s]) => {
        setMemories(m);
        setStats(s);
      })
      .catch((e) => setError(e?.toString()));
  }, [layerFilter, typeFilter]);

  if (error)
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );
  if (!stats)
    return <div className="text-[var(--text-secondary)]">Loading...</div>;

  const capacityPct = Math.round((stats.line_count / stats.max_lines) * 100);

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">Memories</h2>

      {/* Stats Cards */}
      <div className="grid grid-cols-4 gap-4">
        <StatCard
          icon={<Brain size={18} />}
          label="Total"
          value={stats.total.toString()}
          sub={`${stats.long_term} long / ${stats.short_term} short`}
          color="var(--accent)"
        />
        <StatCard
          icon={<Zap size={18} />}
          label="By Type"
          value={`${stats.bugs}B ${stats.decisions}D ${stats.patterns}P ${stats.gotchas}G`}
          sub="bug / decision / pattern / gotcha"
          color="var(--accent-purple)"
        />
        <StatCard
          icon={<Archive size={18} />}
          label="Capacity"
          value={`${stats.line_count} / ${stats.max_lines}`}
          sub={`${capacityPct}% used`}
          color={capacityColor(capacityPct)}
        />
        <StatCard
          icon={<AlertTriangle size={18} />}
          label="Stale"
          value={stats.stale.toString()}
          sub={stats.stale > 0 ? "Run `popsicle memory gc`" : "All fresh"}
          color={
            stats.stale > 0 ? "var(--accent-yellow)" : "var(--accent-green)"
          }
        />
      </div>

      {/* Capacity Progress Bar */}
      <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
        <div className="flex items-center justify-between mb-2">
          <span className="text-sm text-[var(--text-secondary)]">
            Memory Capacity
          </span>
          <span className="text-sm font-mono">
            {stats.line_count} / {stats.max_lines} lines
          </span>
        </div>
        <div className="w-full bg-[var(--bg-tertiary)] rounded-full h-2">
          <div
            className="h-2 rounded-full transition-all"
            style={{
              width: `${Math.min(capacityPct, 100)}%`,
              backgroundColor: capacityColor(capacityPct),
            }}
          />
        </div>
      </div>

      {/* Filters */}
      <div className="flex gap-3">
        <select
          value={layerFilter}
          onChange={(e) => setLayerFilter(e.target.value)}
          className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-lg px-3 py-1.5 text-sm"
        >
          <option value="all">All Layers</option>
          <option value="long-term">Long-term</option>
          <option value="short-term">Short-term</option>
        </select>

        <select
          value={typeFilter}
          onChange={(e) => setTypeFilter(e.target.value)}
          className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-lg px-3 py-1.5 text-sm"
        >
          <option value="all">All Types</option>
          <option value="bug">Bug</option>
          <option value="decision">Decision</option>
          <option value="pattern">Pattern</option>
          <option value="gotcha">Gotcha</option>
        </select>

        <span className="text-sm text-[var(--text-secondary)] self-center ml-auto">
          {memories.length} {memories.length === 1 ? "memory" : "memories"}
        </span>
      </div>

      {/* Memory List */}
      <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
        <div className="px-4 py-3 border-b border-[var(--border)] flex items-center gap-2">
          <Brain size={16} />
          <h3 className="font-medium">Memory Entries</h3>
        </div>

        {memories.length === 0 ? (
          <div className="p-6 text-center text-[var(--text-secondary)]">
            No memories found. Memories are saved via{" "}
            <code className="bg-[var(--bg-tertiary)] px-1.5 py-0.5 rounded text-xs">
              popsicle memory save
            </code>
          </div>
        ) : (
          <div className="divide-y divide-[var(--border)]">
            {memories.map((m) => {
              const isExpanded = expandedId === m.id;
              return (
                <div key={m.id}>
                  <button
                    onClick={() =>
                      setExpandedId(isExpanded ? null : m.id)
                    }
                    className="w-full px-4 py-3 flex items-start gap-3 hover:bg-[var(--bg-tertiary)] transition-colors text-left"
                  >
                    <div className="pt-0.5 text-[var(--text-secondary)]">
                      {isExpanded ? (
                        <ChevronDown size={14} />
                      ) : (
                        <ChevronRight size={14} />
                      )}
                    </div>

                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 flex-wrap">
                        <span className="text-xs font-mono text-[var(--text-secondary)]">
                          #{m.id}
                        </span>
                        <TypeBadge type={m.memory_type} />
                        <LayerBadge layer={m.layer} />
                        {m.stale && (
                          <span className="text-xs px-2 py-0.5 rounded-full bg-yellow-500/10 text-yellow-400">
                            STALE
                          </span>
                        )}
                      </div>

                      <div className="mt-1 text-sm font-medium">
                        {m.summary}
                      </div>

                      <div className="flex items-center gap-3 mt-1 text-xs text-[var(--text-secondary)]">
                        <span>{m.created}</span>
                        <span>refs: {m.refs}</span>
                        {m.tags.length > 0 && (
                          <span className="inline-flex items-center gap-1">
                            <Tag size={10} />
                            {m.tags.join(", ")}
                          </span>
                        )}
                      </div>
                    </div>
                  </button>

                  {isExpanded && (
                    <div className="px-4 pb-4 pl-11 space-y-2">
                      {m.detail && (
                        <div className="text-sm bg-[var(--bg-primary)] rounded p-3 whitespace-pre-wrap">
                          {m.detail}
                        </div>
                      )}

                      {m.files.length > 0 && (
                        <div className="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
                          <FileCode size={12} />
                          <span>Files: {m.files.join(", ")}</span>
                        </div>
                      )}

                      {m.run && (
                        <div className="text-xs text-[var(--text-secondary)]">
                          Run: {m.run}
                        </div>
                      )}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

function StatCard({
  icon,
  label,
  value,
  sub,
  color,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  sub: string;
  color: string;
}) {
  return (
    <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
      <div className="flex items-center gap-2 mb-2">
        <span style={{ color }}>{icon}</span>
        <span className="text-xs text-[var(--text-secondary)]">{label}</span>
      </div>
      <div className="text-xl font-bold">{value}</div>
      <div className="text-xs text-[var(--text-secondary)] mt-0.5">{sub}</div>
    </div>
  );
}
