---
artifact: golden-capture-plan
slug: {slug}
generated_by: golden-capture
slice: {slice-name}
migration_mode: rewrite
legacy_pin: REPLACE_WITH_REAL_LEGACY_PIN
last_updated: {date}
query_anchors:
  - "怎么录 legacy 的真实输出做 golden？"
  - "rewrite 迁移的 legacy fixture 从哪来？"
---

# Golden 录制方案 — {slug}

> `golden-capture` 产出。**仅 rewrite 型迁移需要**（verbatim 平移的 golden 是
> characterization，见 `equivalence-baseline` 的「迁移模式与 golden 性质」段）。
> 本 skill 补上 pipeline 缺失的「起 legacy、录真实输出」环节（feedback #18）。

## Capture Environment

> 如何起 **pinned 的 legacy**（submodule，别用 new 代码）。

| 项 | 值 |
|---|---|
| Legacy submodule | `legacy/{name}` |
| Legacy pin | `git -C legacy/{name} rev-parse HEAD` → `<真实 sha>`（须 == LEGACY_PIN.md）|
| 构建命令 | `cargo build -p <legacy-crate>` / 起 legacy 服务 |
| 隔离方式 | 独立 target dir / 独立端口 / 只读 fixture 目录 |

```
（粘贴实跑：起 legacy 的命令 + exit code）
```

## Fixtures

> 每条 golden 一行。来源事实（fact_id）→ legacy 命令 → fixture 文件 → 真实 exit。

| golden_id | 来源 fact | legacy 命令 | fixture 路径 | exit | sha256 |
|---|---|---|---|---|---|
| G-ST-API-001 | F-ST-BEH-001 | `cargo test … roundtrip` | `fixtures/G-ST-API-001.out` | 0 | `<sha>` |

## Replay Contract

> new 侧如何回放同一输入、如何与 fixture 差分。equivalence-baseline 据此写 `golden-NNN.sh`。

- 输入：……
- new 回放命令：……
- 归一化规则（去时间戳/随机 id 等非确定性）：……
- 差分判定：`diff <(new 输出 | 归一化) fixtures/<id>.out` 为空 → pass

## 未能录制（如实登记，勿编造）

> legacy 起不来 / 无确定性输出 的条目在此说明原因，交 cutover ADR 登记为 divergence。

- （无）

## Capture Checklist

- [ ] `migration_mode: rewrite` 已确认（verbatim 不应走本 skill）
- [ ] `legacy_pin` 真实且 == submodule HEAD（非占位符）
- [ ] 每条 fixture 来自 facts.yaml 的 behavior 事实（golden_candidate）
- [ ] 每条 fixture 有真实 exit code 与 sha256（可复算）
- [ ] Replay Contract 写清 new 回放 + 归一化 + 差分判定
- [ ] 录不到的条目已如实登记，未编造 fixture
