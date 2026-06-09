---
artifact: implementation-coverage
slug: {slug}
generated_by: shadow-implementer
slice: {slice-name}
last_updated: {date}
crate: crates/{slice-name}/
cargo_test_exit: 0
intent_blocks_total: 0
intent_blocks_covered: 0
query_anchors:
  - "这个 slice 的 intent 哪些已经有代码了？"
  - "还缺哪些 fn 或 test？"
---

# 实现覆盖报告 — {slug}

> 由 `shadow-implementer` 产出。把 `products/{slice}/intents/` 的每个
> acceptance/invariants 块映射到 `crates/{slice}/` 的具体实现与测试。

## Summary

| 指标 | 值 |
|---|---|
| Slice | {slice-name} |
| Crate | `crates/{slice-name}/` |
| acceptance + invariants block 总数 | 0 |
| 已覆盖（fn 或 test）| 0 |
| `cargo test -p {slice-name}` | PASS / FAIL（exit ）|

一句话结论：……

## Intent 覆盖表

| Intent / Safety | 层级 | Task ID | 实现位置 | Test | 状态 |
|---|---|---|---|---|---|
| `DocumentRoundTrips` | acceptance | T-AS-0002 | `document.rs::to_file_content` | `intent_properties.rs::document_round_trips_*` | ✅ |

状态：✅ 已覆盖 / ⚠️ 部分 / ❌ 缺失

## File Manifest 对账

| ADR/RFC 路径 | 预期责任 | 磁盘存在 | 备注 |
|---|---|---|---|
| `crates/artifact-system/` | guard + document + … | ✅ | |

## cargo test

```
（粘贴 cargo test -p <slice> 摘要：test result: ok. N passed; 0 failed）
```

## 待办

| 项 | 类型 | 跟进 |
|---|---|---|
| `UpstreamApprovalChecker` 实现 | 端口接线 | skill-runtime crate |
| storage `DocumentRow` | ADR-004 | 新 crate |

## 检查清单

- [ ] 每个 acceptance block 在表中有行
- [ ] 每个 ✅ 行有具体 `文件::符号` 与 test 名
- [ ] File Manifest 与 ADR Consequences 一致
- [ ] cargo test 已实跑并记录 exit code
- [ ] 待办项未冒充 ✅
