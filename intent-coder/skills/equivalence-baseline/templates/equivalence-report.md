---
artifact: equivalence-report
slug: {slug}
generated_by: equivalence-baseline
slice: {slice-name}
# feedback #18/#22：本切片的迁移模式，决定 golden 的性质与命名。
#   verbatim —— 逐字节/逐行平移，legacy 与 new 是同一份逻辑；golden 是
#               characterization（快照自证），等价平凡，**如实声明、别假装差分**。
#               此时「shadow / strangler-fig」措辞不适用（新代码直接是主路径）。
#   rewrite  —— 真正重写，需 legacy 录制 + new 回放的**差分**测试；golden 必须
#               跑 pinned legacy 取真实输出（未来 golden-capture skill，见 ROADMAP）。
migration_mode: verbatim   # verbatim | rewrite
last_updated: {date}
golden_total: 0
golden_pass: 0
golden_fail: 0
divergence_count: 0
equivalence_gate_pass: false
baseline_dir: docs/baseline/{date}/{slice-name}/
baseline_manifest: docs/baseline/{date}/{slice-name}/baseline.yaml
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
| 迁移模式 | verbatim / rewrite（见 frontmatter `migration_mode`；#18）|
| Legacy pin | 从 `git -C legacy/<name> rev-parse HEAD` 注入真实 pin（见 LEGACY_PIN.md；#21）|
| Golden 总数 | 0 |
| ✅ pass（diff 为空）| 0 |
| ❌ fail | 0 |
| ⚠️ divergence（已 ADR 登记）| 0 |
| **equivalence_gate_pass** | false |

门禁：`golden_pass >= 5` **或** 全部 fail 项已 divergence+ADR → pass。

## 迁移模式与 golden 性质（#18）

> 先声明本切片是 **verbatim** 还是 **rewrite**，别让 characterization 冒充差分。

- **verbatim（逐字节/逐行平移）**：legacy 与 new 是同一份逻辑，golden 是
  **characterization test**（快照自证），等价平凡。**如实写明「未做双进程 legacy↔new
  差分，因本切片为 verbatim 平移」**，不要假装跑了 diff。命名/措辞避免用
  「shadow / strangler-fig」（无影子并行，新代码即主路径）。
- **rewrite（真正重写）**：必须跑 **pinned legacy 录制 + new 回放**的差分。当前 pipeline
  未内置起 legacy submodule 录 fixture 的机制——见 ROADMAP 的 `golden-capture` 提案；
  在其落地前，rewrite 切片须在此显式记录如何取得 legacy 真实输出。

- [ ] 已声明 `migration_mode`（frontmatter）
- [ ] verbatim：已如实标注 golden=characterization；rewrite：已记录 legacy 录制方式

## Golden Inventory

- [ ] slice 已确认
- [ ] ≥5 条 golden 已列出（含输入/legacy/new/比较方式）
- [ ] 每条能追溯到 acceptance block 或 api-contracts 行
- [ ] 已知 divergence 已单独列出

## Baseline Manifest

本报告的 golden/pass/fail/divergence 数字必须来自
`docs/baseline/{date}/{slice-name}/baseline.yaml`，不要手写漂移。

- [ ] `baseline.yaml` 已创建
- [ ] Summary 计数与 `baseline.yaml` 一致
- [ ] Golden 清单状态与 `baseline.yaml` 一致

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
- [ ] pass/fail 数字与 `baseline.yaml` / Summary 一致
- [ ] divergence 未隐瞒
- [ ] equivalence_gate_pass 可复算
