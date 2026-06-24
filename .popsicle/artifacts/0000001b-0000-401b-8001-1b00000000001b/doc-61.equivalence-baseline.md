---
doc_type: equivalence-baseline
id: doc-61
pipeline_run_id: 0000001b-0000-401b-8001-1b00000000001b
status: active
title: PROJ-27 UI equivalence baseline
version: 1
---

# 等价性基线报告 — proj-27-ui

## Summary

| 指标 | 值 |
|---|---|
| Slice | cli-ux-ui |
| Golden 总数 | 4 |
| ✅ pass | 4 |
| ❌ fail | 0 |
| ⚠️ divergence | 2（ADR-015）|
| **equivalence_gate_pass** | true |

## Golden Inventory

- [x] slice 已确认
- [x] golden 已列出并实跑
- [x] divergence 已 ADR 登记

## Baseline Manifest

- [x] `docs/baseline/2026-06-11/cli-ux-ui/baseline.yaml` 已创建且一致

## Golden 清单

| ID | 描述 | 脚本 | 结果 |
|---|---|---|---|
| G-001 | help 含 ui | golden-001-help-ui.sh | PASS |
| G-002 | UI 构建 | golden-002-build-ui.sh | PASS |
| G-003 | readers 测试 | golden-003-readers-test.sh | PASS |
| G-004 | task 扫描 | golden-004-task-scan.sh | PASS |

## 运行结果

```
bash docs/baseline/2026-06-11/cli-ux-ui/run-all.sh → All passed
make check → exit 0
intent-validate path=products → exit 0
```

## Traceability

见 `migration/traceability.md` 新增 slice-4-ui 三行（本 run 已写入）。

## Divergence

| ID | 行为 | ADR |
|---|---|---|
| D-501 | SelfHost 非 legacy DB | ADR-015 |
| D-502 | MVP+ 页面裁剪 | ADR-015 |

## 门禁判定

- [x] golden pass + divergence ADR 补偿
- [x] baseline 可复现
- [x] traceability 已更新

## 检查清单

- [x] 每条 golden 有实跑证据
- [x] equivalence_gate_pass 可复算
