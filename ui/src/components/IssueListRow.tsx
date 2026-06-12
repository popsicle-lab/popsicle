import { CircleDot, GitBranch } from "lucide-react";
import type { IssueInfo } from "../hooks/useTauri";
import { IssueTypeBadge, issueTypeAccentClass } from "./IssueTypeBadge";
import { StatusBadge } from "./StatusBadge";
import { getLinkedTaskIds } from "../lib/issueListUtils";

interface Props {
  issue: IssueInfo;
  active: boolean;
  compact: boolean;
  activeRunLabel: string;
  onClick: () => void;
  taskTitle?: (taskId: string) => string | undefined;
}

export function IssueListRow({
  issue,
  active,
  compact,
  activeRunLabel,
  onClick,
  taskTitle,
}: Props) {
  const typeAccent = issueTypeAccentClass(issue.issue_type);
  const linked = getLinkedTaskIds(issue);
  const showTasks = linked.slice(0, compact ? 2 : 3);
  const extraTasks = linked.length - showTasks.length;

  return (
    <button
      type="button"
      onClick={onClick}
      className={`issue-row ${typeAccent} ${active ? "issue-row-active" : ""} ${compact ? "issue-row-compact" : ""}`}
    >
      <span className="issue-row-accent" aria-hidden />
      <span className="issue-row-key">{issue.key}</span>
      <span className="issue-row-title" title={issue.title}>
        {issue.title}
      </span>
      <span className="issue-row-tasks">
        {linked.length === 0 ? (
          <span className="issue-row-task-empty">—</span>
        ) : (
          <>
            {showTasks.map((tid) => (
              <span key={tid} className="issue-task-chip" title={taskTitle?.(tid)}>
                {tid}
              </span>
            ))}
            {extraTasks > 0 && (
              <span className="issue-task-chip issue-task-chip-more">+{extraTasks}</span>
            )}
          </>
        )}
      </span>
      <span className="issue-row-status">
        <StatusBadge status={issue.status} />
      </span>
      <span className="issue-row-pipeline">
        {issue.pipeline ? (
          <span className="issue-pipeline-pill">{issue.pipeline}</span>
        ) : (
          <span className="issue-row-muted">—</span>
        )}
      </span>
      <span className="issue-row-signals">
        {issue.active_run_id && (
          <span className="issue-run-indicator" title={activeRunLabel}>
            <CircleDot size={12} />
            <GitBranch size={11} />
          </span>
        )}
        <IssueTypeBadge type={issue.issue_type} />
      </span>
    </button>
  );
}
