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
import { TaskGraphView } from "./pages/TaskGraphView";
import { IntentGraphView } from "./pages/IntentGraphView";

export type Page =
  | { kind: "issues" }
  | { kind: "issue"; issueKey: string }
  | { kind: "pipeline"; runId: string }
  | { kind: "document"; docId: string }
  | { kind: "tasks" }
  | { kind: "intents" };

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
          {page.kind === "tasks" && <TaskGraphView key={refreshKey} />}
          {page.kind === "intents" && <IntentGraphView key={refreshKey} />}
        </main>
      </div>
    </div>
  );
}
