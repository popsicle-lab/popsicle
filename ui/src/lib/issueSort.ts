import type { IssueInfo } from "../hooks/useTauri";

export type IssueSortKey =
  | "key_desc"
  | "key_asc"
  | "title_asc"
  | "title_desc"
  | "type"
  | "status";

export const ISSUE_SORT_OPTIONS: { value: IssueSortKey; label: string }[] = [
  { value: "key_desc", label: "Key ↓" },
  { value: "key_asc", label: "Key ↑" },
  { value: "title_asc", label: "Title A–Z" },
  { value: "title_desc", label: "Title Z–A" },
  { value: "type", label: "Type" },
  { value: "status", label: "Status" },
];

const STATUS_ORDER: Record<string, number> = {
  in_progress: 0,
  ready: 1,
  backlog: 2,
  done: 3,
};

const TYPE_ORDER: Record<string, number> = {
  bug: 0,
  technical: 1,
  product: 2,
  idea: 3,
};

function issueNum(key: string): number {
  const m = key.match(/(\d+)$/);
  return m ? Number.parseInt(m[1], 10) : 0;
}

export function sortIssues(list: IssueInfo[], sort: IssueSortKey): IssueInfo[] {
  const copy = [...list];
  switch (sort) {
    case "key_desc":
      return copy.sort((a, b) => issueNum(b.key) - issueNum(a.key));
    case "key_asc":
      return copy.sort((a, b) => issueNum(a.key) - issueNum(b.key));
    case "title_asc":
      return copy.sort((a, b) => a.title.localeCompare(b.title));
    case "title_desc":
      return copy.sort((a, b) => b.title.localeCompare(a.title));
    case "type":
      return copy.sort(
        (a, b) =>
          (TYPE_ORDER[a.issue_type] ?? 9) - (TYPE_ORDER[b.issue_type] ?? 9) ||
          issueNum(b.key) - issueNum(a.key)
      );
    case "status":
      return copy.sort(
        (a, b) =>
          (STATUS_ORDER[a.status] ?? 9) - (STATUS_ORDER[b.status] ?? 9) ||
          issueNum(b.key) - issueNum(a.key)
      );
    default:
      return copy;
  }
}
