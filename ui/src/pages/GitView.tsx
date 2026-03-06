import { useEffect, useState } from "react";
import {
  getGitStatus,
  getCommitLinks,
  listDocuments,
  type GitStatusInfo,
  type CommitLinkInfo,
  type DocInfo,
} from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import {
  GitBranch,
  GitCommit,
  FileText,
  AlertCircle,
  CheckCircle,
  Clock,
  XCircle,
  Link2,
} from "lucide-react";
import type { Page } from "../App";

interface Props {
  setPage: (p: Page) => void;
}

const reviewIcons: Record<string, React.ReactNode> = {
  pending: <Clock size={14} className="text-yellow-400" />,
  passed: <CheckCircle size={14} className="text-green-400" />,
  failed: <XCircle size={14} className="text-red-400" />,
  skipped: <AlertCircle size={14} className="text-gray-400" />,
};

export function GitView({ setPage }: Props) {
  const [status, setStatus] = useState<GitStatusInfo | null>(null);
  const [commits, setCommits] = useState<CommitLinkInfo[]>([]);
  const [docs, setDocs] = useState<DocInfo[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([getGitStatus(), getCommitLinks(), listDocuments()])
      .then(([s, c, d]) => {
        setStatus(s);
        setCommits(c);
        setDocs(d);
      })
      .catch((e) => setError(e?.toString()));
  }, []);

  if (error)
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );
  if (!status)
    return <div className="text-[var(--text-secondary)]">Loading...</div>;

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">Git Tracking</h2>

      {/* Git Status Cards */}
      <div className="grid grid-cols-4 gap-4">
        <StatusCard
          icon={<GitBranch size={18} />}
          label="Branch"
          value={status.branch}
          sub={`HEAD: ${status.head}`}
          color="var(--accent)"
        />
        <StatusCard
          icon={<GitCommit size={18} />}
          label="Tracked Commits"
          value={status.total_commits.toString()}
          sub={
            status.uncommitted_changes ? "Working tree: dirty" : "Clean"
          }
          color="var(--accent-purple)"
        />
        <StatusCard
          icon={<CheckCircle size={18} />}
          label="Passed"
          value={status.passed.toString()}
          sub={`${status.pending_review} pending`}
          color="var(--accent-green)"
        />
        <StatusCard
          icon={<XCircle size={18} />}
          label="Failed"
          value={status.failed.toString()}
          sub={status.failed > 0 ? "Needs attention" : "All good"}
          color={status.failed > 0 ? "var(--accent-red)" : "var(--accent-green)"}
        />
      </div>

      {/* Commit History with Document Links */}
      <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
        <div className="px-4 py-3 border-b border-[var(--border)] flex items-center gap-2">
          <GitCommit size={16} />
          <h3 className="font-medium">Commit History</h3>
          <span className="text-xs text-[var(--text-secondary)]">
            linked to pipeline &amp; documents
          </span>
        </div>

        {commits.length === 0 ? (
          <div className="p-6 text-center text-[var(--text-secondary)]">
            No tracked commits yet. Use{" "}
            <code className="bg-[var(--bg-tertiary)] px-1.5 py-0.5 rounded text-xs">
              popsicle git init
            </code>{" "}
            to install the post-commit hook.
          </div>
        ) : (
          <div className="divide-y divide-[var(--border)]">
            {commits.map((commit) => {
              const linkedDoc = commit.doc_id
                ? docs.find((d) => d.id === commit.doc_id)
                : null;

              return (
                <div key={commit.sha} className="px-4 py-3">
                  <div className="flex items-start gap-3">
                    {/* Review status icon */}
                    <div className="pt-0.5">
                      {reviewIcons[commit.review_status] || reviewIcons.pending}
                    </div>

                    {/* Commit info */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <code className="text-xs font-mono text-[var(--accent)]">
                          {commit.short_sha}
                        </code>
                        <span className="text-sm font-medium truncate">
                          {commit.message}
                        </span>
                      </div>

                      <div className="flex items-center gap-3 mt-1 text-xs text-[var(--text-secondary)]">
                        <span>{commit.author}</span>
                        {commit.timestamp && (
                          <span>
                            {new Date(commit.timestamp).toLocaleString()}
                          </span>
                        )}
                      </div>

                      {/* Links to stage/skill/document */}
                      <div className="flex items-center gap-2 mt-2 flex-wrap">
                        <StatusBadge status={commit.review_status} />

                        {commit.stage && (
                          <span className="inline-flex items-center gap-1 text-xs bg-blue-500/10 text-blue-300 px-2 py-0.5 rounded-full">
                            stage: {commit.stage}
                          </span>
                        )}

                        {commit.skill && (
                          <span className="inline-flex items-center gap-1 text-xs bg-purple-500/10 text-purple-300 px-2 py-0.5 rounded-full">
                            skill: {commit.skill}
                          </span>
                        )}

                        {linkedDoc && (
                          <button
                            onClick={() =>
                              setPage({
                                kind: "document",
                                docId: linkedDoc.id,
                              })
                            }
                            className="inline-flex items-center gap-1 text-xs bg-[var(--accent)]/10 text-[var(--accent)] px-2 py-0.5 rounded-full hover:bg-[var(--accent)]/20 transition-colors"
                          >
                            <Link2 size={10} />
                            <FileText size={10} />
                            {linkedDoc.title}
                          </button>
                        )}
                      </div>

                      {commit.review_summary && (
                        <div className="mt-2 text-xs bg-[var(--bg-primary)] rounded p-2 text-[var(--text-secondary)]">
                          {commit.review_summary}
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

function StatusCard({
  icon,
  label,
  value,
  sub,
  color,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  sub: string;
  color: string;
}) {
  return (
    <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
      <div className="flex items-center gap-2 mb-2">
        <span style={{ color }}>{icon}</span>
        <span className="text-xs text-[var(--text-secondary)]">{label}</span>
      </div>
      <div className="text-xl font-bold">{value}</div>
      <div className="text-xs text-[var(--text-secondary)] mt-0.5">{sub}</div>
    </div>
  );
}
