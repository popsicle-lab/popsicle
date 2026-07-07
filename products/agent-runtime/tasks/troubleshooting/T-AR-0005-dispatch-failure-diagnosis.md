---
task_id: T-AR-0005
slug: dispatch-failure-diagnosis
title: "派活失败时我知道是 Daemon 离线还是 doc check 没过"
journey_stage: troubleshooting
audience: ["end-user", "admin"]
task_type: 故障排查
decision_ref: PDR-001
last_updated: 2026-07-06
intent_kind: diagnose
involved_features: ["task-failure-reason", "runtime-heartbeat"]
prerequisites:
  - "dispatch.status in {failed, stuck_queued}"
related_intents:
  - "acceptance.intent#DispatchRejectedWhenRuntimeOffline"
related_next_tasks:
  - T-AR-0001
fact_cite:
  - "Multica docs: runtime_offline failure reason 类比"
---

# 派活失败时我知道是 Daemon 离线还是 doc check 没过

## 本 task 可解答

- 派活一直 queued 怎么办？
- 失败 reason `runtime_offline` 是什么意思？
- doc check 失败和 Daemon 崩溃怎么区分？

## 完成路径

1. **看失败 reason**（App 或 Server API）：
   - `runtime_offline` → 跑 T-AR-0001，`daemon status` + `daemon logs -f`
   - `doc_check_failed` → 读 `next` 字段，在桌面或 Daemon 重试填 artifact
   - `spec_lock` / `active-run` → 完成或取消已有 run 再派

2. **CLI 对照**（开发机）：

   ```bash
   popsicle daemon status --format json
   popsicle issue show <key> --format json
   ```

3. **重试**：修复根因后在 App 点 **重试派活**（retryable 来源自动 requeue `[假设]`）。

## 可观察的成功标志

用户能根据 reason 码在 1 步内选对修复 task（T-AR-0001 / doc 修复 / issue 关闭）。离线时 Server 拒绝新派活：`acceptance.intent#DispatchRejectedWhenRuntimeOffline`。

## Related Next Tasks

- **T-AR-0001** — Runtime 恢复

`Decision-Ref: PDR-001`
