import { useCallback, useEffect, useState } from "react";
import { useProjectDir, useRefresh, getInitialDir } from "./hooks/useTauri";
import { Sidebar } from "./components/Sidebar";
import { Dashboard } from "./pages/Dashboard";
import { PipelineView } from "./pages/PipelineView";
import { DocumentView } from "./pages/DocumentView";
import { SkillsView } from "./pages/SkillsView";
import { GitView } from "./pages/GitView";
import { DiscussionsView } from "./pages/DiscussionsView";
import { DiscussionDetailView } from "./pages/DiscussionDetailView";
import { IssuesView } from "./pages/IssuesView";
import { IssueDetailView } from "./pages/IssueDetailView";
import { ProjectPicker } from "./components/ProjectPicker";

export type Page =
  | { kind: "dashboard" }
  | { kind: "pipeline"; runId: string }
  | { kind: "document"; docId: string }
  | { kind: "skills" }
  | { kind: "git" }
  | { kind: "discussions" }
  | { kind: "discussion"; discussionId: string }
  | { kind: "issues" }
  | { kind: "issue"; issueKey: string };

export default function App() {
  const { dir, setProjectDir } = useProjectDir();
  const [page, setPage] = useState<Page>({ kind: "dashboard" });
  const [refreshKey, setRefreshKey] = useState(0);
  const [initialDir, setInitialDir] = useState<string | null>(null);
  const [autoOpenAttempted, setAutoOpenAttempted] = useState(false);

  const refresh = useCallback(() => setRefreshKey((k) => k + 1), []);
  useRefresh(refresh);

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const projectPath = params.get("project");
    if (projectPath) {
      setProjectDir(projectPath).catch(console.error);
      setAutoOpenAttempted(true);
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

  if (!autoOpenAttempted) {
    return null;
  }

  if (!dir) {
    return <ProjectPicker onSelect={setProjectDir} initialPath={initialDir ?? undefined} />;
  }

  return (
    <div className="flex h-screen">
      <Sidebar page={page} setPage={setPage} />
      <main className="flex-1 overflow-auto p-6">
        {page.kind === "dashboard" && (
          <Dashboard key={refreshKey} setPage={setPage} />
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
        {page.kind === "skills" && <SkillsView key={refreshKey} />}
        {page.kind === "git" && (
          <GitView key={refreshKey} setPage={setPage} />
        )}
        {page.kind === "discussions" && (
          <DiscussionsView key={refreshKey} setPage={setPage} />
        )}
        {page.kind === "discussion" && (
          <DiscussionDetailView
            key={`${page.discussionId}-${refreshKey}`}
            discussionId={page.discussionId}
          />
        )}
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
      </main>
    </div>
  );
}
