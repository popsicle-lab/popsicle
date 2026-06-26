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
import { BootstrapConfirmModal } from "./components/BootstrapConfirmModal";
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
import { WorkflowsView } from "./pages/WorkflowsView";
import {
  LocaleProvider,
  normalizeLocale,
  type Locale,
} from "./i18n/LocaleContext";
import { getProjectConfig } from "./hooks/useTauri";

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
  | { kind: "settings" }
  | {
      kind: "workflows";
      tab?: "pipelines" | "skills";
      pipeline?: string;
      skill?: string;
      contextRunId?: string;
      contextIssueKey?: string;
      highlightStage?: string;
    };

export default function App() {
  const {
    project,
    openProjectDir,
    closeProject,
    syncProject,
    bootstrapPromptPath,
    finishBootstrapPrompt,
  } = useProjectSession();
  const [page, setPage] = useState<Page>({ kind: "issues" });
  const [projectSwitchKey, setProjectSwitchKey] = useState(0);
  const [workspaceTick, setWorkspaceTick] = useState(0);
  const [bootstrapped, setBootstrapped] = useState(false);
  const [workspace, setWorkspace] = useState<WorkspaceInfo | null>(null);
  const [locale, setLocale] = useState<Locale>("zh-CN");
  const [sidebarCollapsed, setSidebarCollapsed] = useState(() => {
    try {
      return localStorage.getItem("popsicle.sidebarCollapsed") === "1";
    } catch {
      return false;
    }
  });

  const onWorkspaceDataChange = useCallback(() => {
    setWorkspaceTick((k) => k + 1);
  }, []);
  useRefresh(onWorkspaceDataChange);

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
          const opened = await openProjectDir(startup);
          if (!opened) {
            // user declined bootstrap on startup
          }
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
    getProjectConfig()
      .then((cfg) => setLocale(normalizeLocale(cfg.language)))
      .catch(() => {});
  }, [project, workspaceTick]);

  const handleSwitchProject = useCallback(
    async (path: string) => {
      const opened = await openProjectDir(path);
      if (!opened) return;
      setPage({ kind: "issues" });
      setProjectSwitchKey((k) => k + 1);
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

  const bootstrapModal = bootstrapPromptPath ? (
    <BootstrapConfirmModal
      path={bootstrapPromptPath}
      locale={locale}
      onConfirm={() => finishBootstrapPrompt(true)}
      onCancel={() => finishBootstrapPrompt(false)}
    />
  ) : null;

  if (!bootstrapped) {
    return (
      <>
        {bootstrapModal}
        <div className="flex h-screen flex-col items-center justify-center gap-3 text-[var(--text-muted)]">
          <div className="spinner" aria-hidden />
          <span className="text-[13px]">Loading workspace…</span>
        </div>
      </>
    );
  }

  if (!project) {
    return (
      <>
        {bootstrapModal}
        <ProjectPicker
          onSelect={async (path) => {
            const opened = await openProjectDir(path);
            if (opened) setPage({ kind: "issues" });
          }}
        />
      </>
    );
  }

  return (
    <LocaleProvider locale={locale} onLocaleChange={setLocale}>
    {bootstrapModal}
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
                key={projectSwitchKey}
                setPage={setPage}
                initialSelectedKey={page.selectedKey}
              />
            )}
            {page.kind === "issue" && (
              <IssueDetailView
                key={`${page.issueKey}-${projectSwitchKey}`}
                issueKey={page.issueKey}
                setPage={setPage}
              />
            )}
            {page.kind === "pipeline" && (
              <PipelineView
                key={`${page.runId}-${projectSwitchKey}`}
                runId={page.runId}
                setPage={setPage}
              />
            )}
            {page.kind === "document" && (
              <DocumentView
                key={`${page.docId}-${projectSwitchKey}`}
                docId={page.docId}
                setPage={setPage}
              />
            )}
            {page.kind === "products" && (
              <ProductExplorerView
                key={`${page.product ?? ""}-${projectSwitchKey}`}
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
                key={`${page.taskId}-${projectSwitchKey}`}
                product={page.product}
                taskId={page.taskId}
                returnTo={page.returnTo}
                setPage={setPage}
              />
            )}
            {page.kind === "intent" && (
              <IntentDetailPage
                key={`${page.file}-${page.block ?? ""}-${projectSwitchKey}`}
                product={page.product}
                file={page.file}
                block={page.block}
                returnTo={page.returnTo}
                setPage={setPage}
              />
            )}
            {page.kind === "settings" && (
              <SettingsView
                key={projectSwitchKey}
                setPage={setPage}
                onSaved={() => setWorkspaceTick((k) => k + 1)}
              />
            )}
            {page.kind === "workflows" && (
              <WorkflowsView
                key={`${page.tab ?? "pipelines"}-${page.contextRunId ?? ""}-${projectSwitchKey}`}
                setPage={setPage}
                tab={page.tab}
                pipeline={page.pipeline}
                skill={page.skill}
                contextRunId={page.contextRunId}
                contextIssueKey={page.contextIssueKey}
                highlightStage={page.highlightStage}
              />
            )}
          </main>
        </div>
      </div>
    </div>
    </LocaleProvider>
  );
}
