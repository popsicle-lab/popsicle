import type { IssueGroupBy, IssueInfo } from "../hooks/useTauri";
import { getLinkedTaskIds } from "./issueListUtils";

export interface TaskMeta {
  task_id: string;
  title: string;
  journey_stage: string;
}

export interface IssueGroupLabels {
  unlinkedEpic: string;
  noPipeline: string;
  pipelinePrefix: string;
}

export type IssueGroup = {
  key: string;
  label: string;
  subtitle?: string;
  epicTaskId?: string | null;
  journeyStage?: string;
  issues: IssueInfo[];
  doneCount: number;
};

const UNLINKED_KEY = "__unlinked__";

export function groupIssues(
  list: IssueInfo[],
  groupBy: IssueGroupBy,
  taskMeta: Map<string, TaskMeta>,
  labels: IssueGroupLabels
): IssueGroup[] {
  if (groupBy === "none") {
    return [toGroup("", "", list, taskMeta)];
  }

  const map = new Map<string, IssueInfo[]>();
  for (const issue of list) {
    let key: string;
    if (groupBy === "product") {
      key = issue.product_id || "(unknown)";
    } else if (groupBy === "pipeline") {
      key = issue.pipeline ?? labels.noPipeline;
    } else {
      const linked = getLinkedTaskIds(issue);
      const keys = linked.length > 0 ? linked : [UNLINKED_KEY];
      for (const k of keys) {
        const bucket = map.get(k);
        if (bucket) bucket.push(issue);
        else map.set(k, [issue]);
      }
      continue;
    }
    const bucket = map.get(key);
    if (bucket) bucket.push(issue);
    else map.set(key, [issue]);
  }

  const entries = [...map.entries()].sort(([a], [b]) => {
    if (groupBy === "epic") {
      if (a === UNLINKED_KEY) return 1;
      if (b === UNLINKED_KEY) return -1;
    }
    return a.localeCompare(b);
  });

  return entries.map(([key, issues]) => {
    if (groupBy === "product") {
      return toGroup(key, key, issues, taskMeta);
    }
    if (groupBy === "pipeline") {
      const label =
        key === labels.noPipeline ? labels.noPipeline : `${labels.pipelinePrefix} ${key}`;
      return toGroup(key, label, issues, taskMeta);
    }
    if (key === UNLINKED_KEY) {
      return toGroup(key, labels.unlinkedEpic, issues, taskMeta);
    }
    const meta = taskMeta.get(key);
    return toGroup(
      key,
      meta?.title ?? key,
      issues,
      taskMeta,
      key,
      meta?.journey_stage
    );
  });
}

function toGroup(
  key: string,
  label: string,
  issues: IssueInfo[],
  _taskMeta: Map<string, TaskMeta>,
  epicTaskId?: string | null,
  journeyStage?: string
): IssueGroup {
  const doneCount = issues.filter((i) => i.status === "done").length;
  return {
    key,
    label,
    subtitle: epicTaskId ? epicTaskId : undefined,
    epicTaskId: epicTaskId ?? null,
    journeyStage,
    issues,
    doneCount,
  };
}

export function buildTaskMetaMap(
  nodes: { task_id: string; title: string; journey_stage: string }[]
): Map<string, TaskMeta> {
  const map = new Map<string, TaskMeta>();
  for (const n of nodes) {
    map.set(n.task_id, n);
  }
  return map;
}
