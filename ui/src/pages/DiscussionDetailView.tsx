import { useEffect, useState } from "react";
import {
  getDiscussion,
  type DiscussionFull,
  type DiscussionMessageInfo,
  type DiscussionRoleInfo,
} from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import {
  MessageCircle,
  Users,
  Clock,
  Puzzle,
  GitBranch,
  UserCircle,
  Bot,
  Bookmark,
  Pause,
  CheckCircle,
  AlertCircle,
  ChevronDown,
  ChevronRight,
  ArrowLeft,
} from "lucide-react";
import type { Page } from "../App";

interface Props {
  discussionId: string;
  setPage: (p: Page) => void;
  fromIssue?: string;
}

const roleColors = [
  { bg: "rgba(56, 189, 248, 0.12)", border: "#38bdf8", text: "#7dd3fc" },
  { bg: "rgba(167, 139, 250, 0.12)", border: "#a78bfa", text: "#c4b5fd" },
  { bg: "rgba(74, 222, 128, 0.12)", border: "#4ade80", text: "#86efac" },
  { bg: "rgba(251, 191, 36, 0.12)", border: "#fbbf24", text: "#fde68a" },
  { bg: "rgba(248, 113, 113, 0.12)", border: "#f87171", text: "#fca5a5" },
  { bg: "rgba(45, 212, 191, 0.12)", border: "#2dd4bf", text: "#5eead4" },
  { bg: "rgba(244, 114, 182, 0.12)", border: "#f472b6", text: "#f9a8d4" },
  { bg: "rgba(251, 146, 60, 0.12)", border: "#fb923c", text: "#fdba74" },
];

function getRoleColor(roleId: string, roles: DiscussionRoleInfo[]) {
  const idx = roles.findIndex((r) => r.role_id === roleId);
  return roleColors[idx % roleColors.length];
}

function formatTime(ts: string) {
  return new Date(ts).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });
}

function groupByPhase(messages: DiscussionMessageInfo[]) {
  const phases: { phase: string; messages: DiscussionMessageInfo[] }[] = [];
  for (const msg of messages) {
    const last = phases[phases.length - 1];
    if (last && last.phase === msg.phase) {
      last.messages.push(msg);
    } else {
      phases.push({ phase: msg.phase, messages: [msg] });
    }
  }
  return phases;
}

export function DiscussionDetailView({ discussionId, setPage, fromIssue }: Props) {
  const [disc, setDisc] = useState<DiscussionFull | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [collapsedPhases, setCollapsedPhases] = useState<Set<string>>(
    new Set()
  );

  useEffect(() => {
    getDiscussion(discussionId)
      .then(setDisc)
      .catch((e) => setError(e?.toString()));
  }, [discussionId]);

  if (error)
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );
  if (!disc)
    return <div className="text-[var(--text-secondary)]">Loading...</div>;

  const phases = groupByPhase(disc.messages);

  const togglePhase = (phase: string) => {
    setCollapsedPhases((prev) => {
      const next = new Set(prev);
      if (next.has(phase)) next.delete(phase);
      else next.add(phase);
      return next;
    });
  };

  const handleBack = () => {
    if (fromIssue) {
      setPage({ kind: "issue", issueKey: fromIssue, tab: "discussions" });
    } else {
      setPage({ kind: "issues" });
    }
  };

  return (
    <div className="flex gap-6 h-full">
      <div className="flex-1 min-w-0 space-y-4">
        <button
          onClick={handleBack}
          className="flex items-center gap-2 text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
        >
          <ArrowLeft size={16} />
          {fromIssue ? `Back to ${fromIssue}` : "Back to Issues"}
        </button>

        <div className="flex items-center gap-3">
          <MessageCircle size={24} className="text-[var(--accent)]" />
          <h2 className="text-2xl font-bold">{disc.topic}</h2>
          <StatusBadge status={disc.status} />
        </div>

        {disc.user_confidence != null && (
          <div className="flex items-center gap-2 text-sm">
            <span className="text-[var(--text-secondary)]">Confidence:</span>
            <ConfidenceBar value={disc.user_confidence} />
          </div>
        )}

        <div className="space-y-6">
          {phases.map(({ phase, messages }) => {
            const isCollapsed = collapsedPhases.has(phase);
            return (
              <div key={phase}>
                <button
                  onClick={() => togglePhase(phase)}
                  className="flex items-center gap-2 mb-3 group"
                >
                  {isCollapsed ? (
                    <ChevronRight
                      size={16}
                      className="text-[var(--text-secondary)]"
                    />
                  ) : (
                    <ChevronDown
                      size={16}
                      className="text-[var(--text-secondary)]"
                    />
                  )}
                  <h3 className="text-sm font-semibold text-[var(--accent)] uppercase tracking-wider">
                    {phase}
                  </h3>
                  <span className="text-xs text-[var(--text-secondary)]">
                    ({messages.length} messages)
                  </span>
                </button>

                {!isCollapsed && (
                  <div className="space-y-3 ml-2 border-l-2 border-[var(--border)] pl-4">
                    {messages.map((msg) => (
                      <MessageBubble
                        key={msg.id}
                        msg={msg}
                        roles={disc.roles}
                      />
                    ))}
                  </div>
                )}
              </div>
            );
          })}
        </div>

        {disc.messages.length === 0 && (
          <div className="text-center text-[var(--text-secondary)] py-12 bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
            <MessageCircle
              size={40}
              className="mx-auto mb-3 opacity-30"
            />
            No messages in this discussion yet.
          </div>
        )}
      </div>

      <aside className="w-64 shrink-0">
        <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)] sticky top-0">
          <div className="px-4 py-3 border-b border-[var(--border)]">
            <h3 className="font-medium text-sm flex items-center gap-2">
              <Users size={14} />
              Participants ({disc.roles.length})
            </h3>
          </div>
          <div className="p-3 space-y-2">
            {disc.roles.map((role) => {
              const color = getRoleColor(role.role_id, disc.roles);
              const msgCount = disc.messages.filter(
                (m) => m.role_id === role.role_id
              ).length;
              return (
                <div
                  key={role.role_id}
                  className="p-2.5 rounded-lg text-sm"
                  style={{ background: color.bg }}
                >
                  <div
                    className="font-medium text-xs"
                    style={{ color: color.text }}
                  >
                    {role.role_name}
                  </div>
                  {role.perspective && (
                    <div className="text-xs text-[var(--text-secondary)] mt-0.5 line-clamp-2">
                      {role.perspective}
                    </div>
                  )}
                  <div className="flex items-center justify-between mt-1">
                    <span className="text-xs text-[var(--text-secondary)]">
                      {role.source}
                    </span>
                    <span className="text-xs text-[var(--text-secondary)]">
                      {msgCount} msgs
                    </span>
                  </div>
                </div>
              );
            })}
          </div>

          <div className="px-4 py-3 border-t border-[var(--border)] space-y-3 text-sm">
            <MetaRow icon={<Puzzle size={14} />} label="Skill">
              {disc.skill}
            </MetaRow>
            <MetaRow icon={<GitBranch size={14} />} label="Pipeline Run">
              <span className="font-mono text-xs">
                {disc.pipeline_run_id.slice(0, 8)}
              </span>
            </MetaRow>
            {disc.document_id && (
              <MetaRow icon={<Bookmark size={14} />} label="Document">
                <span className="font-mono text-xs">
                  {disc.document_id.slice(0, 8)}
                </span>
              </MetaRow>
            )}
            <MetaRow icon={<Clock size={14} />} label="Created">
              <span className="text-xs text-[var(--text-secondary)]">
                {new Date(disc.created_at).toLocaleString()}
              </span>
            </MetaRow>
            {disc.concluded_at && (
              <MetaRow icon={<CheckCircle size={14} />} label="Concluded">
                <span className="text-xs text-[var(--text-secondary)]">
                  {new Date(disc.concluded_at).toLocaleString()}
                </span>
              </MetaRow>
            )}
          </div>
        </div>
      </aside>
    </div>
  );
}

function MessageBubble({
  msg,
  roles,
}: {
  msg: DiscussionMessageInfo;
  roles: DiscussionRoleInfo[];
}) {
  const color = getRoleColor(msg.role_id, roles);

  if (msg.message_type === "user_input") {
    return (
      <div className="flex gap-3">
        <div className="shrink-0 mt-1">
          <div className="w-7 h-7 rounded-full bg-[var(--accent)]/20 flex items-center justify-center">
            <UserCircle size={16} className="text-[var(--accent)]" />
          </div>
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-xs font-medium text-[var(--accent)]">
              User
            </span>
            <span className="text-xs text-[var(--text-secondary)]">
              {formatTime(msg.timestamp)}
            </span>
          </div>
          <div className="bg-[var(--accent)]/10 border border-[var(--accent)]/20 rounded-lg rounded-tl-none px-3 py-2 text-sm">
            <Markdown remarkPlugins={[remarkGfm]}>{msg.content}</Markdown>
          </div>
        </div>
      </div>
    );
  }

  if (msg.message_type === "pause_point") {
    return (
      <div className="flex items-center gap-3 py-2">
        <Pause size={14} className="text-[var(--accent-yellow)] shrink-0" />
        <div className="flex-1 border-t border-dashed border-[var(--accent-yellow)]/40" />
        <span className="text-xs text-[var(--accent-yellow)] italic px-2 shrink-0">
          {msg.content}
        </span>
        <div className="flex-1 border-t border-dashed border-[var(--accent-yellow)]/40" />
      </div>
    );
  }

  if (msg.message_type === "phase_summary") {
    return (
      <div className="bg-[var(--bg-tertiary)]/50 rounded-lg p-3 border border-[var(--border)]">
        <div className="flex items-center gap-2 mb-2 text-xs font-medium text-[var(--accent-purple)]">
          <AlertCircle size={12} />
          Phase Summary
        </div>
        <div className="text-sm prose prose-invert prose-sm max-w-none">
          <Markdown remarkPlugins={[remarkGfm]}>{msg.content}</Markdown>
        </div>
      </div>
    );
  }

  if (msg.message_type === "decision") {
    return (
      <div className="bg-[var(--accent-green)]/8 rounded-lg p-3 border border-[var(--accent-green)]/20">
        <div className="flex items-center gap-2 mb-2 text-xs font-medium text-[var(--accent-green)]">
          <CheckCircle size={12} />
          Decision
        </div>
        <div className="text-sm prose prose-invert prose-sm max-w-none">
          <Markdown remarkPlugins={[remarkGfm]}>{msg.content}</Markdown>
        </div>
      </div>
    );
  }

  if (msg.message_type === "system_note") {
    return (
      <div className="text-xs text-[var(--text-secondary)] italic pl-3 py-1 border-l-2 border-[var(--text-secondary)]/20">
        {msg.content}
      </div>
    );
  }

  return (
    <div className="flex gap-3">
      <div className="shrink-0 mt-1">
        <div
          className="w-7 h-7 rounded-full flex items-center justify-center"
          style={{ background: color.bg, border: `1px solid ${color.border}33` }}
        >
          <Bot size={14} style={{ color: color.text }} />
        </div>
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-1">
          <span className="text-xs font-medium" style={{ color: color.text }}>
            {msg.role_name}
          </span>
          <span className="text-xs text-[var(--text-secondary)]">
            {formatTime(msg.timestamp)}
          </span>
          {msg.reply_to && (
            <span className="text-xs text-[var(--text-secondary)] italic">
              replying...
            </span>
          )}
        </div>
        <div
          className="rounded-lg rounded-tl-none px-3 py-2 text-sm"
          style={{
            background: color.bg,
            border: `1px solid ${color.border}22`,
          }}
        >
          <div className="prose prose-invert prose-sm max-w-none">
            <Markdown remarkPlugins={[remarkGfm]}>{msg.content}</Markdown>
          </div>
        </div>
      </div>
    </div>
  );
}

function ConfidenceBar({ value }: { value: number }) {
  const clamped = Math.max(0, Math.min(100, value));
  const color =
    clamped >= 70
      ? "var(--accent-green)"
      : clamped >= 40
        ? "var(--accent-yellow)"
        : "var(--accent-red)";
  return (
    <div className="flex items-center gap-2">
      <div className="w-24 h-1.5 bg-[var(--bg-tertiary)] rounded-full overflow-hidden">
        <div
          className="h-full rounded-full transition-all"
          style={{ width: `${clamped}%`, background: color }}
        />
      </div>
      <span className="text-xs font-mono" style={{ color }}>
        {clamped}%
      </span>
    </div>
  );
}

function MetaRow({
  icon,
  label,
  children,
}: {
  icon: React.ReactNode;
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <div className="flex items-center gap-1.5 text-[var(--text-secondary)] mb-1">
        {icon}
        <span className="text-xs">{label}</span>
      </div>
      <div className="pl-5">{children}</div>
    </div>
  );
}
