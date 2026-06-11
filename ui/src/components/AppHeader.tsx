import type { WorkspaceInfo } from "../hooks/useTauri";

interface Props {
  workspace: WorkspaceInfo | null;
  pageTitle: string;
}

export function AppHeader({ workspace, pageTitle }: Props) {
  return (
    <header className="app-header flex h-12 shrink-0 items-center justify-between border-b border-[var(--border)] bg-[var(--bg-secondary)]/50 px-6 backdrop-blur-sm">
      <div className="min-w-0">
        <h2 className="truncate text-sm font-semibold">{pageTitle}</h2>
        {workspace && (
          <p className="truncate text-[11px] text-[var(--text-secondary)]">
            {workspace.storage_backend}
          </p>
        )}
      </div>
      {workspace && !workspace.binary_match && (
        <span className="rounded-full bg-[var(--accent-yellow)]/15 px-2.5 py-1 text-[11px] font-medium text-[var(--accent-yellow)]">
          Binary mismatch
        </span>
      )}
    </header>
  );
}
