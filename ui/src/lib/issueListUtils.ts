import type { IssueInfo } from "../hooks/useTauri";

export interface IssueListStats {
  total: number;
  inProgress: number;
  done: number;
  activeRuns: number;
}

export function getLinkedTaskIds(issue: IssueInfo): string[] {
  const fromLinks =
    issue.task_links
      ?.filter((l) => l.role === "linked" && l.task_id)
      .map((l) => l.task_id as string) ?? [];
  if (fromLinks.length > 0) return fromLinks;
  if (issue.epic_task_id?.trim()) return [issue.epic_task_id.trim()];
  return [];
}

export function computeIssueStats(issues: IssueInfo[]): IssueListStats {
  return {
    total: issues.length,
    inProgress: issues.filter((i) => i.status === "in_progress").length,
    done: issues.filter((i) => i.status === "done").length,
    activeRuns: issues.filter((i) => i.active_run_id).length,
  };
}

export function filterIssues(
  issues: IssueInfo[],
  opts: {
    search: string;
    taskId: string | null;
  }
): IssueInfo[] {
  const q = opts.search.trim().toLowerCase();
  return issues.filter((issue) => {
    if (opts.taskId) {
      const linked = getLinkedTaskIds(issue);
      if (!linked.includes(opts.taskId)) return false;
    }
    if (!q) return true;
    const hay = [
      issue.key,
      issue.title,
      issue.product_id,
      issue.pipeline ?? "",
      ...getLinkedTaskIds(issue),
    ]
      .join(" ")
      .toLowerCase();
    return hay.includes(q);
  });
}

export function collectTaskIdsFromIssues(issues: IssueInfo[]): string[] {
  const set = new Set<string>();
  for (const issue of issues) {
    for (const id of getLinkedTaskIds(issue)) set.add(id);
  }
  return [...set].sort();
}
