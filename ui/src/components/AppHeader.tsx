import { ChevronLeft, ChevronRight, PanelLeft } from "lucide-react";
import type { Page } from "../App";
import type { WorkspaceInfo } from "../hooks/useTauri";
import { useLocale } from "../i18n/LocaleContext";
import { pageBack, pageCrumbs } from "../lib/navigation";

interface Props {
  workspace: WorkspaceInfo | null;
  page: Page;
  sidebarCollapsed: boolean;
  onToggleSidebar: () => void;
  onNavigate: (page: Page) => void;
}

export function AppHeader({
  workspace,
  page,
  sidebarCollapsed,
  onToggleSidebar,
  onNavigate,
}: Props) {
  const { m } = useLocale();
  const crumbs = pageCrumbs(page, m.crumbs);
  const back = pageBack(page);

  return (
    <header className="app-header flex h-11 shrink-0 items-center gap-3 border-b border-[var(--border)] px-4">
      <button
        type="button"
        onClick={onToggleSidebar}
        className="btn btn-ghost shrink-0 p-2"
        title={
          sidebarCollapsed ? m.header.expandSidebar : m.header.collapseSidebar
        }
        aria-label={
          sidebarCollapsed ? m.header.expandSidebar : m.header.collapseSidebar
        }
      >
        <PanelLeft size={16} />
      </button>

      {back && (
        <button
          type="button"
          onClick={() => onNavigate(back)}
          className="btn btn-ghost shrink-0 gap-1 px-2"
        >
          <ChevronLeft size={16} />
          <span className="hidden sm:inline">{m.header.back}</span>
        </button>
      )}

      <nav
        className="flex min-w-0 flex-1 items-center gap-1 text-[13px]"
        aria-label="Breadcrumb"
      >
        {crumbs.map((crumb, i) => {
          const isLast = i === crumbs.length - 1;
          return (
            <span key={`${crumb.label}-${i}`} className="flex min-w-0 items-center gap-1">
              {i > 0 && (
                <ChevronRight
                  size={12}
                  className="shrink-0 text-[var(--text-muted)]"
                />
              )}
              {crumb.page && !isLast ? (
                <button
                  type="button"
                  onClick={() => onNavigate(crumb.page!)}
                  className="truncate text-[var(--text-secondary)] transition-colors hover:text-[var(--text-primary)]"
                >
                  {crumb.label}
                </button>
              ) : (
                <span
                  className={`truncate ${isLast ? "font-semibold text-[var(--text-primary)]" : "text-[var(--text-secondary)]"}`}
                >
                  {crumb.label}
                </span>
              )}
            </span>
          );
        })}
      </nav>

      <div className="flex shrink-0 items-center gap-2">
        {workspace && !workspace.binary_match && (
          <span className="badge badge-warning">{m.header.binaryMismatch}</span>
        )}
        {workspace && (
          <span className="hidden max-w-[12rem] truncate text-[11px] text-[var(--text-muted)] md:inline">
            {workspace.storage_backend}
          </span>
        )}
      </div>
    </header>
  );
}
