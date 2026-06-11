import { useCallback, useEffect, useState } from "react";
import {
  getActiveProject,
  getWorkspaceInfo,
  resolveStartupProject,
  useProjectSession,
  useRefresh,
  type WorkspaceInfo,
} from "./hooks/useTauri";
import { Sidebar } from "./components/Sidebar";
import { ProjectPicker } from "./components/ProjectPicker";
import { AppHeader } from "./components/AppHeader";
import { IssuesView } from "./pages/IssuesView";
import { IssueDetailView } from "./pages/IssueDetailView";
import { PipelineView } from "./pages/PipelineView";
import { DocumentView } from "./pages/DocumentView";
import { ProductExplorerView } from "./pages/ProductExplorerView";
import { TaskDetailPage } from "./pages/TaskDetailPage";
import { IntentDetailPage } from "./pages/IntentDetailPage";
import { SettingsView } from "./pages/SettingsView";

export type Page =
  | { kind: "issues"; selectedKey?: string }
  | { kind: "issue"; issueKey: string }
  | { kind: "pipeline"; runId: string }
  | { kind: "document"; docId: string }
  | {
      kind: "products";
      product?: string;
      tab?: "tasks" | "intents" | "graph";
      taskId?: string;
      intentFile?: string;
      intentBlock?: string;
    }
  | { kind: "task"; taskId: string; product: string; returnTo?: Page }
  | {
      kind: "intent";
      product: string;
      file: string;
      block?: string;
      returnTo?: Page;
    }
  | { kind: "settings" };

export default function App() {
  const { project, openProjectDir, closeProject, syncProject } =
    useProjectSession();
  const [page, setPage] = useState<Page>({ kind: "issues" });
  const [refreshKey, setRefreshKey] = useState(0);
  const [bootstrapped, setBootstrapped] = useState(false);
  const [workspace, setWorkspace] = useState<WorkspaceInfo | null>(null);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(() => {
    try {
      return localStorage.getItem("popsicle.sidebarCollapsed") === "1";
    } catch {
      return false;
    }
  });

  const refresh = useCallback(() => setRefreshKey((k) => k + 1), []);
  useRefresh(refresh);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      const params = new URLSearchParams(window.location.search);
      const queryProject = params.get("project") ?? undefined;

      const active = await getActiveProject();
      if (active && !cancelled) {
        await syncProject(active);
        setBootstrapped(true);
        return;
      }

      const startup = await resolveStartupProject(queryProject);
      if (startup && !cancelled) {
        try {
          await openProjectDir(startup);
        } catch {
          // fall through to picker
        }
      }
      if (!cancelled) setBootstrapped(true);
    })();
    return () => {
      cancelled = true;
    };
  }, [openProjectDir, syncProject]);

  useEffect(() => {
    if (!project) {
      setWorkspace(null);
      return;
    }
    getWorkspaceInfo()
      .then(setWorkspace)
      .catch(() => setWorkspace(null));
  }, [project, refreshKey]);

  const handleSwitchProject = useCallback(
    async (path: string) => {
      await openProjectDir(path);
      setPage({ kind: "issues" });
      setRefreshKey((k) => k + 1);
    },
    [openProjectDir]
  );

  const handleBrowseOther = useCallback(() => {
    closeProject();
    setPage({ kind: "issues" });
  }, [closeProject]);

  const toggleSidebar = useCallback(() => {
    setSidebarCollapsed((v) => {
      const next = !v;
      try {
        localStorage.setItem("popsicle.sidebarCollapsed", next ? "1" : "0");
      } catch {
        /* ignore */
      }
      return next;
    });
  }, []);

  if (!bootstrapped) {
    return (
      <div className="flex h-screen flex-col items-center justify-center gap-3 text-[var(--text-muted)]">
        <div className="spinner" aria-hidden />
        <span className="text-[13px]">Loading workspace…</span>
      </div>
    );
  }

  if (!project) {
    return (
      <ProjectPicker
        onSelect={async (path) => {
          await openProjectDir(path);
          setPage({ kind: "issues" });
        }}
      />
    );
  }

  return (
    <div className="flex h-screen flex-col overflow-hidden">
      <div className="flex min-h-0 flex-1">
        <Sidebar
          page={page}
          setPage={setPage}
          project={project}
          collapsed={sidebarCollapsed}
          onSwitchProject={handleSwitchProject}
          onBrowseOther={handleBrowseOther}
        />
        <div className="flex min-w-0 flex-1 flex-col">
          <AppHeader
            workspace={workspace}
            page={page}
            sidebarCollapsed={sidebarCollapsed}
            onToggleSidebar={toggleSidebar}
            onNavigate={setPage}
          />
          <main className="main-content page-enter flex-1 overflow-hidden px-4 py-3">
            {page.kind === "issues" && (
              <IssuesView
                key={refreshKey}
                setPage={setPage}
                initialSelectedKey={page.selectedKey}
              />
            )}
            {page.kind === "issue" && (
              <IssueDetailView
                key={`${page.issueKey}-${refreshKey}`}
                issueKey={page.issueKey}
                setPage={setPage}
              />
            )}
            {page.kind === "pipeline" && (
              <PipelineView
                key={`${page.runId}-${refreshKey}`}
                runId={page.runId}
                setPage={setPage}
              />
            )}
            {page.kind === "document" && (
              <DocumentView
                key={`${page.docId}-${refreshKey}`}
                docId={page.docId}
                setPage={setPage}
              />
            )}
            {page.kind === "products" && (
              <ProductExplorerView
                key={`${page.product ?? ""}-${refreshKey}`}
                setPage={setPage}
                product={page.product}
                tab={page.tab}
                taskId={page.taskId}
                intentFile={page.intentFile}
                intentBlock={page.intentBlock}
              />
            )}
            {page.kind === "task" && (
              <TaskDetailPage
                key={`${page.taskId}-${refreshKey}`}
                product={page.product}
                taskId={page.taskId}
                returnTo={page.returnTo}
                setPage={setPage}
              />
            )}
            {page.kind === "intent" && (
              <IntentDetailPage
                key={`${page.file}-${page.block ?? ""}-${refreshKey}`}
                product={page.product}
                file={page.file}
                block={page.block}
                returnTo={page.returnTo}
                setPage={setPage}
              />
            )}
            {page.kind === "settings" && (
              <SettingsView
                key={refreshKey}
                onSaved={() => setRefreshKey((k) => k + 1)}
              />
            )}
          </main>
        </div>
      </div>
    </div>
  );
}
