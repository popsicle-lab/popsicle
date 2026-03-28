import {
  LayoutDashboard,
  Puzzle,
  GitBranch,
  ClipboardList,
  Brain,
  Search,
  Tags,
} from "lucide-react";
import type { Page } from "../App";

interface Props {
  page: Page;
  setPage: (p: Page) => void;
}

const navItems: {
  kind: Page["kind"];
  label: string;
  icon: typeof LayoutDashboard;
  group?: string;
}[] = [
  { kind: "dashboard", label: "Dashboard", icon: LayoutDashboard },
  { kind: "issues", label: "Issues", icon: ClipboardList },
  { kind: "topics", label: "Topics", icon: Tags },
  { kind: "search", label: "Doc Search", icon: Search },
  { kind: "git", label: "Git Tracking", icon: GitBranch },
  { kind: "memories", label: "Memories", icon: Brain },
  { kind: "skills", label: "Skills", icon: Puzzle },
];

const issueChildKinds = new Set<Page["kind"]>([
  "issue",
  "story",
  "bug",
  "testcase",
  "discussion",
]);

export function Sidebar({ page, setPage }: Props) {
  const isActive = (kind: Page["kind"]) => {
    if (kind === "issues") {
      return page.kind === "issues" || issueChildKinds.has(page.kind);
    }
    if (kind === "topics") {
      return page.kind === "topics" || page.kind === "topic";
    }
    return page.kind === kind;
  };

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
          const active = isActive(item.kind);
          return (
            <button
              key={item.kind}
              onClick={() => setPage({ kind: item.kind } as Page)}
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
