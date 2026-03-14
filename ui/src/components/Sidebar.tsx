import { LayoutDashboard, Puzzle, GitBranch, MessageCircle, ClipboardList, Bug, FlaskConical, BookOpen, Brain } from "lucide-react";
import type { Page } from "../App";

interface Props {
  page: Page;
  setPage: (p: Page) => void;
}

const navItems = [
  { kind: "dashboard" as const, label: "Dashboard", icon: LayoutDashboard },
  { kind: "issues" as const, label: "Issues", icon: ClipboardList },
  { kind: "stories" as const, label: "User Stories", icon: BookOpen },
  { kind: "testcases" as const, label: "Test Cases", icon: FlaskConical },
  { kind: "bugs" as const, label: "Bugs", icon: Bug },
  { kind: "discussions" as const, label: "Discussions", icon: MessageCircle },
  { kind: "git" as const, label: "Git Tracking", icon: GitBranch },
  { kind: "memories" as const, label: "Memories", icon: Brain },
  { kind: "skills" as const, label: "Skills", icon: Puzzle },
];

export function Sidebar({ page, setPage }: Props) {
  return (
    <aside className="w-56 bg-[var(--bg-secondary)] border-r border-[var(--border)] flex flex-col">
      <div className="p-4 border-b border-[var(--border)]">
        <h1 className="text-lg font-bold tracking-tight flex items-center gap-2">
          <span className="text-2xl">🐕</span> Popsicle
        </h1>
        <p className="text-xs text-[var(--text-secondary)] mt-1">
          Spec-Driven Dev
        </p>
      </div>

      <nav className="flex-1 p-2 space-y-1">
        {navItems.map((item) => {
          const active = page.kind === item.kind;
          return (
            <button
              key={item.kind}
              onClick={() => setPage({ kind: item.kind })}
              className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors ${
                active
                  ? "bg-[var(--accent)]/15 text-[var(--accent)]"
                  : "text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]"
              }`}
            >
              <item.icon size={18} />
              {item.label}
            </button>
          );
        })}
      </nav>

      <div className="p-3 border-t border-[var(--border)] text-xs text-[var(--text-secondary)]">
        v0.1.0 — Read-only UI
      </div>
    </aside>
  );
}
