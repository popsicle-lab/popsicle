import { BookOpen, ClipboardList, Layers, Package, Settings } from "lucide-react";
import type { Page } from "../App";
import type { ProjectInfo } from "../hooks/useTauri";
import { useLocale } from "../i18n/LocaleContext";
import { ProjectSwitcher } from "./ProjectSwitcher";

interface Props {
  page: Page;
  setPage: (p: Page) => void;
  project: ProjectInfo;
  collapsed: boolean;
  onSwitchProject: (path: string) => Promise<void>;
  onBrowseOther: () => void;
}

export function Sidebar({
  page,
  setPage,
  project,
  collapsed,
  onSwitchProject,
  onBrowseOther,
}: Props) {
  const { m } = useLocale();
  const navItems: {
    kind: Page["kind"];
    label: string;
    icon: typeof ClipboardList;
  }[] = [
    { kind: "issues", label: m.nav.issues, icon: ClipboardList },
    { kind: "products", label: m.nav.products, icon: Package },
    { kind: "workflows", label: m.nav.workflows, icon: BookOpen },
    { kind: "settings", label: m.nav.settings, icon: Settings },
  ];

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
    if (kind === "workflows") {
      return page.kind === "workflows";
    }
    if (kind === "settings") {
      return page.kind === "settings";
    }
    return page.kind === kind;
  };

  return (
    <aside
      className={`sidebar sidebar-transition flex shrink-0 flex-col border-r border-[var(--border)] ${collapsed ? "w-[3.75rem]" : "w-[15.5rem]"}`}
    >
      <div
        className={`border-b border-[var(--border)] ${collapsed ? "px-2 py-3" : "px-4 py-4"}`}
      >
        <div
          className={`mb-3 flex items-center ${collapsed ? "justify-center" : "gap-2.5"}`}
        >
          <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-[var(--radius-sm)] bg-[var(--accent-muted)] text-[var(--accent)]">
            <Layers size={16} strokeWidth={2.25} />
          </div>
          {!collapsed && (
            <div className="min-w-0">
              <h1 className="text-sm font-semibold tracking-tight">Popsicle</h1>
              <p className="text-[11px] text-[var(--text-muted)]">{m.nav.tagline}</p>
            </div>
          )}
        </div>
        {!collapsed ? (
          <ProjectSwitcher
            current={project}
            onSwitch={onSwitchProject}
            onBrowseOther={onBrowseOther}
          />
        ) : (
          <button
            type="button"
            title={project.name}
            onClick={onBrowseOther}
            className="mx-auto flex h-9 w-9 items-center justify-center rounded-[var(--radius-sm)] border border-[var(--border)] text-[var(--accent)] transition-colors hover:bg-[var(--bg-hover)]"
          >
            <Package size={15} />
          </button>
        )}
      </div>

      <nav className={`flex-1 space-y-0.5 ${collapsed ? "p-1.5" : "p-2"}`}>
        {navItems.map((item) => {
          const active = isActive(item.kind);
          return (
            <button
              key={item.kind}
              type="button"
              title={item.label}
              onClick={() => {
                if (item.kind === "products") {
                  setPage({ kind: "products", tab: "tasks" });
                } else if (item.kind === "settings") {
                  setPage({ kind: "settings" });
                } else if (item.kind === "workflows") {
                  setPage({ kind: "workflows", tab: "pipelines" });
                } else {
                  setPage({ kind: "issues" });
                }
              }}
              className={`flex w-full items-center rounded-[var(--radius-sm)] text-[13px] transition-all duration-150 ${
                collapsed
                  ? "justify-center px-0 py-2.5"
                  : "gap-2.5 px-3 py-2"
              } ${
                active
                  ? "bg-[var(--accent-muted)] font-medium text-[#93c5fd]"
                  : "text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]"
              }`}
            >
              <item.icon size={16} strokeWidth={active ? 2.25 : 2} />
              {!collapsed && item.label}
            </button>
          );
        })}
      </nav>

      {!collapsed && (
        <div className="border-t border-[var(--border)] px-4 py-3 text-[11px] text-[var(--text-muted)]">
          {m.nav.footer}
        </div>
      )}
    </aside>
  );
}
