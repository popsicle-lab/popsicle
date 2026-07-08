# Tasks — telemetry

> **Product**: telemetry | **PDR**: PDR-001/002 | **Last-Updated**: 2026-07-08
> **Status**: 4/4 task 已实现（PROJ-67–75 MVP）

| 旅程阶段 | 任务数 | 已实施 | 健康度 |
|---|---|---|---|
| `admin/` | 1 | 1 | 🟢 implemented |
| `daily-ops/` | 3 | 3 | 🟢 implemented |
| `onboarding/` | 0 | 0 | ⚪ 待补充 |
| `troubleshooting/` | 0 | 0 | ⚪ 待补充 |
| `lifecycle/` | 0 | 0 | ⚪ 待补充 |

## Task 清单

| task_id | journey | title | 关联 |
|---------|---------|-------|------|
| T-TEL-0001 | admin | 我配置 OTLP 端点并确认 trace 已上云 | PDR-001 |
| T-TEL-0002 | daily-ops | Agent 通过 tool run telemetry 上报 gen_ai span | PDR-001 · `action=guide` |
| T-TEL-0003 | daily-ops | 编排命令自动 emit popsicle.run 与 stage span | PDR-001 |
| T-TEL-0004 | daily-ops | 我对单个 pipeline run 生成 telemetry 聚合报告 | PDR-002 |

## 旅程入口

- [Admin](admin/) — OTLP 配置与上云验证（1）
- [Daily-Ops](daily-ops/) — Agent 上报、编排 auto-span、run 报告（3）
- [Onboarding](onboarding/) — （待补充）
- [Troubleshooting](troubleshooting/) — （待补充）
- [Lifecycle](lifecycle/) — （待补充）

## 健康度统计

> 由 living-doc-author 在重跑时刷新（doc-sync-weekly · PROJ-96 · 2026-07-08）。

| 旅程阶段 | Task 数 | 平均行数 | 上次更新最久的 task | 未引用的 task |
|---|---|---|---|---|
| admin | 1 | 45 | T-TEL-0001（12 天前）| 无 |
| daily-ops | 3 | 51 | T-TEL-0003（12 天前）| 无 |
| onboarding | 0 | — | — | — |
| troubleshooting | 0 | — | — | — |
| lifecycle | 0 | — | — | — |
