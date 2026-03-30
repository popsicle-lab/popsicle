import { useCallback, useEffect, useState } from "react";
import { useProjectDir, useRefresh, getInitialDir } from "./hooks/useTauri";
import { Sidebar } from "./components/Sidebar";
import { Dashboard } from "./pages/Dashboard";
import { PipelineView } from "./pages/PipelineView";
import { DocumentView } from "./pages/DocumentView";
import { SkillsView } from "./pages/SkillsView";
import { GitView } from "./pages/GitView";
import { DiscussionDetailView } from "./pages/DiscussionDetailView";
import { IssuesView } from "./pages/IssuesView";
import { IssueDetailView } from "./pages/IssueDetailView";
import { BugDetailView } from "./pages/BugDetailView";
import { TestCaseDetailView } from "./pages/TestCaseDetailView";
import { StoryDetailView } from "./pages/StoryDetailView";
import { MemoriesView } from "./pages/MemoriesView";
import { SearchView } from "./pages/SearchView";
import { TopicsView } from "./pages/TopicsView";
import { TopicDetailView } from "./pages/TopicDetailView";
import { NamespacesView } from "./pages/NamespacesView";
import { NamespaceDetailView } from "./pages/NamespaceDetailView";
import { NamespacePicker } from "./components/NamespacePicker";

export type Page =
  | { kind: "dashboard" }
  | { kind: "pipeline"; runId: string }
  | { kind: "document"; docId: string }
  | { kind: "skills" }
  | { kind: "git" }
  | { kind: "issues" }
  | { kind: "issue"; issueKey: string; tab?: string }
  | { kind: "discussion"; discussionId: string; fromIssue?: string }
  | { kind: "bug"; bugKey: string; fromIssue?: string }
  | { kind: "testcase"; testCaseKey: string; fromIssue?: string }
  | { kind: "story"; storyKey: string; fromIssue?: string }
  | { kind: "memories" }
  | { kind: "search" }
  | { kind: "topics" }
  | { kind: "topic"; topicName: string }
  | { kind: "namespaces" }
  | { kind: "namespace"; namespaceId: string };

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
    return <NamespacePicker onSelect={setProjectDir} initialPath={initialDir ?? undefined} />;
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
        {page.kind === "issues" && (
          <IssuesView key={refreshKey} setPage={setPage} />
        )}
        {page.kind === "issue" && (
          <IssueDetailView
            key={`${page.issueKey}-${refreshKey}`}
            issueKey={page.issueKey}
            setPage={setPage}
            initialTab={page.tab as any}
          />
        )}
        {page.kind === "discussion" && (
          <DiscussionDetailView
            key={`${page.discussionId}-${refreshKey}`}
            discussionId={page.discussionId}
            setPage={setPage}
            fromIssue={page.fromIssue}
          />
        )}
        {page.kind === "bug" && (
          <BugDetailView
            key={`${page.bugKey}-${refreshKey}`}
            bugKey={page.bugKey}
            setPage={setPage}
            fromIssue={page.fromIssue}
          />
        )}
        {page.kind === "testcase" && (
          <TestCaseDetailView
            key={`${page.testCaseKey}-${refreshKey}`}
            testCaseKey={page.testCaseKey}
            setPage={setPage}
            fromIssue={page.fromIssue}
          />
        )}
        {page.kind === "story" && (
          <StoryDetailView
            key={`${page.storyKey}-${refreshKey}`}
            storyKey={page.storyKey}
            setPage={setPage}
            fromIssue={page.fromIssue}
          />
        )}
        {page.kind === "memories" && (
          <MemoriesView key={refreshKey} />
        )}
        {page.kind === "search" && (
          <SearchView key={refreshKey} setPage={setPage} />
        )}
        {page.kind === "topics" && (
          <TopicsView key={refreshKey} setPage={setPage} />
        )}
        {page.kind === "topic" && (
          <TopicDetailView
            key={`${page.topicName}-${refreshKey}`}
            topicName={page.topicName}
            setPage={setPage}
          />
        )}
        {page.kind === "namespaces" && (
          <NamespacesView key={refreshKey} setPage={setPage} />
        )}
        {page.kind === "namespace" && (
          <NamespaceDetailView
            key={`${page.namespaceId}-${refreshKey}`}
            namespaceId={page.namespaceId}
            setPage={setPage}
          />
        )}
      </main>
    </div>
  );
}
