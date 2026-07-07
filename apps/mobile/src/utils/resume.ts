const RESUME_REASON_ZH: Record<string, string> = {
  runtime_offline: "Runtime 离线，请先在本机启动 Daemon",
  run_completed: "Run 已完成，无需恢复",
  resume_already_queued: "恢复任务已在队列中，请等待 Daemon 认领",
};

export function resumeReasonLabel(reason: string): string {
  return RESUME_REASON_ZH[reason] ?? reason;
}
