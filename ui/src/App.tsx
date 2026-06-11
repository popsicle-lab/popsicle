import { useCallback, useEffect, useState } from "react";
import {
  getInitialDir,
  getWorkspaceInfo,
  useProjectDir,
  useRefresh,
} from "./hooks/useTauri";
import { Sidebar } from "./components/Sidebar";
import { ProjectPicker } from "./components/ProjectPicker";
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

export default function App() {
  const { dir, setProjectDir } = useProjectDir();
  const [page, setPage] = useState<Page>({ kind: "issues" });
  const [refreshKey, setRefreshKey] = useState(0);
  const [initialDir, setInitialDir] = useState<string | null>(null);
  const [autoOpenAttempted, setAutoOpenAttempted] = useState(false);
  const [binaryWarning, setBinaryWarning] = useState<string | null>(null);

  const refresh = useCallback(() => setRefreshKey((k) => k + 1), []);
  useRefresh(refresh);

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const projectPath = params.get("project");
    if (projectPath) {
      setProjectDir(projectPath)
        .catch(console.error)
        .finally(() => setAutoOpenAttempted(true));
      return;
    }

    getInitialDir().then((cwd) => {
      setInitialDir(cwd || null);
      if (cwd) {
        setProjectDir(cwd)
          .catch(() => {})
          .finally(() => setAutoOpenAttempted(true));
      } else {
        setAutoOpenAttempted(true);
      }
    });
  }, [setProjectDir]);

  useEffect(() => {
    if (!dir) return;
    getWorkspaceInfo()
      .then((info) => {
        if (!info.binary_match) {
          setBinaryWarning(
            `Binary mismatch: rebuild with cargo build -p cli-ux --features ui`
          );
        } else {
          setBinaryWarning(null);
        }
      })
      .catch(() => {});
  }, [dir, refreshKey]);

  if (!autoOpenAttempted) return null;

  if (!dir) {
    return (
      <ProjectPicker onSelect={setProjectDir} initialPath={initialDir ?? undefined} />
    );
  }

  return (
    <div className="flex h-screen flex-col">
      {binaryWarning && (
        <div className="bg-yellow-500/15 text-yellow-200 text-xs px-4 py-2 border-b border-yellow-500/30">
          {binaryWarning}
        </div>
      )}
      <div className="flex flex-1 min-h-0">
        <Sidebar page={page} setPage={setPage} />
        <main className="flex-1 overflow-auto p-6">
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
  );
}
