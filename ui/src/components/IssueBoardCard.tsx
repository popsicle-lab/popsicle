import { GitBranch } from "lucide-react";
import type { IssueInfo } from "../hooks/useTauri";
import { IssueTypeBadge } from "./IssueTypeBadge";
import { StatusBadge } from "./StatusBadge";

interface Props {
  issue: IssueInfo;
  onClick: () => void;
  showProduct?: boolean;
}

export function IssueBoardCard({ issue, onClick, showProduct }: Props) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`issue-mosaic-card issue-mosaic-card-${issue.issue_type} card-interactive`}
    >
      <div className="issue-mosaic-card-top">
        <span className="issue-mosaic-card-key">{issue.key}</span>
        <IssueTypeBadge type={issue.issue_type} />
      </div>
      <p className="issue-mosaic-card-title">{issue.title}</p>
      <div className="issue-mosaic-card-foot">
        <StatusBadge status={issue.status} />
        {issue.pipeline && (
          <span className="issue-mosaic-card-pipeline">{issue.pipeline}</span>
        )}
        {showProduct && (
          <span className="issue-mosaic-card-product">{issue.product_id}</span>
        )}
        {issue.active_run_id && (
          <span className="issue-mosaic-card-run" title="Active run">
            <GitBranch size={12} />
          </span>
        )}
      </div>
    </button>
  );
}
