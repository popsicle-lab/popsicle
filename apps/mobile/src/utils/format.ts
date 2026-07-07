const RUN_STATUS_ZH: Record<string, string> = {
  completed: "已完成",
  in_progress: "进行中",
  failed: "失败",
  pending: "等待中",
  cancelled: "已取消",
};

const STAGE_STATUS_ZH: Record<string, string> = {
  completed: "已完成",
  in_progress: "进行中",
  pending: "等待",
  skipped: "跳过",
};

export function runStatusLabel(status: string): string {
  return RUN_STATUS_ZH[status] ?? status;
}

export function stageStatusLabel(status: string): string {
  return STAGE_STATUS_ZH[status] ?? status;
}

export function formatRelativeTime(epoch: number): string {
  const epochMs = normalizeEpochMs(epoch);
  const diff = Date.now() - epochMs;
  const sec = Math.floor(diff / 1000);
  if (sec < 60) return "刚刚";
  const min = Math.floor(sec / 60);
  if (min < 60) return `${min} 分钟前`;
  const hr = Math.floor(min / 60);
  if (hr < 24) return `${hr} 小时前`;
  const day = Math.floor(hr / 24);
  if (day < 7) return `${day} 天前`;
  return new Date(epochMs).toLocaleDateString("zh-CN", {
    month: "short",
    day: "numeric",
  });
}

export function truncateMiddle(value: string, head = 8, tail = 6): string {
  if (value.length <= head + tail + 1) return value;
  return `${value.slice(0, head)}…${value.slice(-tail)}`;
}

export function shortWorkspace(path: string): string {
  const parts = path.split("/").filter(Boolean);
  if (parts.length <= 2) return path;
  return `…/${parts.slice(-2).join("/")}`;
}

/** Server `updated_at` is unix seconds; run log `ts` is milliseconds. */
export function normalizeEpochMs(value: number): number {
  if (!Number.isFinite(value) || value <= 0) return Date.now();
  return value < 1_000_000_000_000 ? value * 1000 : value;
}

export function isValidIssueKey(key?: string | null): key is string {
  if (!key) return false;
  const trimmed = key.trim();
  return trimmed.length > 0 && trimmed !== "UNKNOWN";
}

export function displayRunTitle(run: {
  issue_key?: string | null;
  run_id: string;
}): string {
  if (isValidIssueKey(run.issue_key)) return run.issue_key;
  return truncateMiddle(run.run_id, 10, 8);
}

export function displayRunSubtitle(run: {
  issue_key?: string | null;
  run_id: string;
  pipeline: string;
}): string {
  const issuePart = isValidIssueKey(run.issue_key)
    ? run.issue_key
    : `Run ${truncateMiddle(run.run_id, 8, 6)}`;
  return `${issuePart} · ${run.pipeline}`;
}
