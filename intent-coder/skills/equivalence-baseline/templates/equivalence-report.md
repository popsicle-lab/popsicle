---
artifact: equivalence-report
slug: {slug}
generated_by: equivalence-baseline
slice: {slice-name}
last_updated: {date}
golden_total: 0
golden_pass: 0
golden_fail: 0
divergence_count: 0
equivalence_gate_pass: false
baseline_dir: docs/baseline/{date}/{slice-name}/
query_anchors:
  - "legacy 和 new 行为一致吗？"
  - "golden 对账过了几条？"
---

# 等价性基线报告 — {slug}

> 由 `equivalence-baseline` 产出。legacy pin 见 `LEGACY_PIN.md`。

## Summary

| 指标 | 值 |
|---|---|
| Slice | {slice-name} |
| Legacy pin | `c76d729…`（见 LEGACY_PIN.md）|
| Golden 总数 | 0 |
| ✅ pass（diff 为空）| 0 |
| ❌ fail | 0 |
| ⚠️ divergence（已 ADR 登记）| 0 |
| **equivalence_gate_pass** | false |

门禁：`golden_pass >= 5` **或** 全部 fail 项已 divergence+ADR → pass。

## Golden 清单

| ID | 描述 | 脚本 | Legacy | New | 结果 | diff 摘要 |
|---|---|---|---|---|---|---|
| G-001 | doc roundtrip body | `golden-001.sh` | `legacy/...` | `crates/...` | PASS | （空）|

## 运行结果

```
（粘贴实跑命令与 exit code）
```

## Traceability（拟写入 migration/traceability.md）

| Legacy 路径 | 新位置 | 责任 Spec | 切流 ADR | 等价性 baseline | 状态 |
|---|---|---|---|---|---|
| `crates/popsicle-core/src/engine/guard.rs` | `crates/artifact-system/src/guard.rs` | slice-2-artifact-system | ADR-XXX-cutover（待）| `docs/baseline/{date}/{slice}/` | in-shadow |

## Divergence

| ID | 行为 | Legacy | New | 原因 | ADR |
|---|---|---|---|---|---|
| D-001 | body 解析 | `trim_start()` | 字节精确 | intent DocumentRoundTrips | 待 cutover ADR |

无则写「（无）」。

## 门禁判定

- [ ] ≥5 golden pass，或
- [ ] 全部 fail 已列入 Divergence 且 ADR Accepted
- [ ] baseline 目录已创建且 README 可复现
- [ ] traceability 草稿已写

## 检查清单

- [ ] 每条 golden 有实跑证据
- [ ] pass/fail 数字与 Summary 一致
- [ ] divergence 未隐瞒
- [ ] equivalence_gate_pass 可复算
