import { ClipboardList, Package } from "lucide-react";
import type { Page } from "../App";
import type { ProjectInfo } from "../hooks/useTauri";
import { ProjectSwitcher } from "./ProjectSwitcher";

interface Props {
  page: Page;
  setPage: (p: Page) => void;
  project: ProjectInfo;
  onSwitchProject: (path: string) => Promise<void>;
  onBrowseOther: () => void;
}

const navItems: {
  kind: Page["kind"];
  label: string;
  icon: typeof ClipboardList;
}[] = [
  { kind: "issues", label: "Issues", icon: ClipboardList },
  { kind: "products", label: "Products", icon: Package },
];

export function Sidebar({
  page,
  setPage,
  project,
  onSwitchProject,
  onBrowseOther,
}: Props) {
  const isActive = (kind: Page["kind"]) => {
    if (kind === "issues") {
      return (
        page.kind === "issues" ||
        page.kind === "issue" ||
        page.kind === "pipeline" ||
        page.kind === "document"
      );
    }
    if (kind === "products") {
      return (
        page.kind === "products" ||
        page.kind === "task" ||
        page.kind === "intent"
      );
    }
    return page.kind === kind;
  };

  return (
    <aside className="sidebar flex w-64 shrink-0 flex-col border-r border-[var(--border)] bg-[var(--bg-secondary)]/80">
      <div className="border-b border-[var(--border)] p-4">
        <h1 className="mb-3 flex items-center gap-2 text-lg font-bold tracking-tight">
          <span className="text-2xl">🐕</span> Popsicle
        </h1>
        <ProjectSwitcher
          current={project}
          onSwitch={onSwitchProject}
          onBrowseOther={onBrowseOther}
        />
      </div>

      <nav className="flex-1 space-y-1 p-3">
        {navItems.map((item) => {
          const active = isActive(item.kind);
          return (
            <button
              key={item.kind}
              type="button"
              onClick={() => {
                if (item.kind === "products") {
                  setPage({ kind: "products", tab: "tasks" });
                } else {
                  setPage({ kind: "issues" });
                }
              }}
              className={`flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-sm transition-colors ${
                active
                  ? "bg-[var(--accent)]/15 font-medium text-[var(--accent)]"
                  : "text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)]/50 hover:text-[var(--text-primary)]"
              }`}
            >
              <item.icon size={18} />
              {item.label}
            </button>
          );
        })}
      </nav>

      <div className="border-t border-[var(--border)] p-3 text-[11px] text-[var(--text-secondary)]">
        Spec-driven development
      </div>
    </aside>
  );
}
