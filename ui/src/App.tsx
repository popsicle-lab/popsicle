import { useCallback, useEffect, useMemo, useState } from "react";
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

export type Page =
  | { kind: "issues" }
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
  | { kind: "task"; taskId: string; product: string }
  | { kind: "intent"; product: string; file: string; block?: string };

function pageTitle(page: Page): string {
  switch (page.kind) {
    case "issues":
      return "Issues";
    case "issue":
      return page.issueKey;
    case "pipeline":
      return "Pipeline";
    case "document":
      return "Document";
    case "products":
      return page.product ? `Products · ${page.product}` : "Products";
    case "task":
      return page.taskId;
    case "intent":
      return page.block ?? page.file;
    default:
      return "Popsicle";
  }
}

export default function App() {
  const { project, openProjectDir, closeProject, syncProject } =
    useProjectSession();
  const [page, setPage] = useState<Page>({ kind: "issues" });
  const [refreshKey, setRefreshKey] = useState(0);
  const [bootstrapped, setBootstrapped] = useState(false);
  const [workspace, setWorkspace] = useState<WorkspaceInfo | null>(null);

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

  const title = useMemo(() => pageTitle(page), [page]);

  if (!bootstrapped) {
    return <div className="flex h-screen items-center justify-center text-sm text-[var(--text-secondary)]">Loading…</div>;
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
          onSwitchProject={handleSwitchProject}
          onBrowseOther={handleBrowseOther}
        />
        <div className="flex min-w-0 flex-1 flex-col">
          <AppHeader workspace={workspace} pageTitle={title} />
          <main className="main-content flex-1 overflow-auto p-6">
            {page.kind === "issues" && (
              <IssuesView key={refreshKey} setPage={setPage} />
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
                setPage={setPage}
              />
            )}
            {page.kind === "intent" && (
              <IntentDetailPage
                key={`${page.file}-${page.block ?? ""}-${refreshKey}`}
                product={page.product}
                file={page.file}
                block={page.block}
                setPage={setPage}
              />
            )}
          </main>
        </div>
      </div>
    </div>
  );
}
