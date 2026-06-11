# Tech-Debt Inventory — popsicle@c76d729

> 配套：[`fact-extraction-report.md`](../../../.popsicle/artifacts/f89529af-d8ce-40f7-ad05-985e35b9cfec/popsicle-c76d729-fact-basis-slice-1--skill-runtime.fact-extraction-report.md)
>
> 基线：`legacy/popsicle/` submodule @ `c76d729db91c59009f0fa8f7c6f1e499eb0c7eb1`
> 范围：仅记**代码库自己标**的债务（TODO/FIXME/HACK/XXX、deprecated、dead_code、ignored 测试）+ **结构性候选删项**（D4 §5）。不记推断债务。

---

## 标记计数

| 标记 | 数量 | 来源命令 |
|---|---|---|
| `TODO` | 1 | `rg -i 'TODO:' crates/` |
| `FIXME` | 0 | `rg -i 'FIXME:' crates/` |
| `HACK` | 0 | `rg -i 'HACK:' crates/` |
| `XXX` | 0 | `rg -i 'XXX:' crates/` |
| `NOTE` | (not separately counted) | — |

> **意外低**：本仓库**几乎没有** TODO/FIXME 注释。这与 `unwrap()` 301 处的高数字形成对比——错误处理短路被采用，但作者没把短路点标 TODO。下游 ADR-writer 可能想要为这种模式记一份"为什么 unwrap 而不是 TODO"的决策。

---

## TODO / FIXME

| File:Line | 标记 | 注释 | 备注 |
|---|---|---|---|
| `crates/popsicle-core/src/engine/markdown.rs:37` | `"TODO"` | （字符串字面量，不是注释）| 用作模板占位符识别（`is_template_placeholder` 的正向 case）。**非真实债务**。 |

> 真实的代码级 TODO/FIXME 注释：**0 处**。

> `[reduced fidelity]` 此节：未跑 git blame 的 `--diff-filter=A -S "TODO: "` 反向考古；以本仓库当前的命中数（0）来说，反向考古无意义。

---

## Deprecated API 使用

| 调用方 | Deprecated API | 建议替代 | 自版本 |
|---|---|---|---|
| _（none —— `rg -t rust '#\[deprecated' crates/` 返回 0）_ | n/a | n/a | n/a |

> 本仓库无显式 deprecated API 使用，也无内部 deprecated 接口。v0.1.0 项目还未到需要兼容性管理。

---

## 死代码候选

> **只**记编译器 / linter 信号。

| 条目 | File:Line | 工具 | 置信度 |
|---|---|---|---|
| _(needs human input —— 本次未跑 `cargo build` / `cargo clippy`；如需精确死代码列表请补一次完整 build)_ | — | — | — |

---

## 禁用的测试

> `#[ignore]`、`#[cfg(ignore)]` 等。

| 测试 | File:Line | 跳过原因 |
|---|---|---|
| _(none —— `rg -t rust '#\[ignore' crates/` 返回 0)_ | n/a | n/a |

> 无被显式禁用的测试。

---

## 配置异味

| File:Line | 构造 | 置信度 |
|---|---|---|
| _(暂未做硬编码 URL/路径扫描——本次时间预算下未做)_ | — | — |

---

## Build 警告（快照）

> 未跑 `cargo build` —— 本次抽取以纯静态分析为主。
> `(build did not run)` —— 下游 living-doc-author 可在首次 CI 集成后补充。

---

## 按模块汇总

| 模块 | TODO | FIXME | HACK | Deprecated 调用 | 死代码（已知）|
|---|---|---|---|---|---|
| `popsicle-core` | 1（字面量）| 0 | 0 | 0 | (未跑 build) |
| `popsicle-cli` | 0 | 0 | 0 | 0 | (未跑 build) |
| `popsicle-sync` | 0 | 0 | 0 | 0 | (未跑 build) |
| **总计** | **1** | **0** | **0** | **0** | — |

---

## 结构性候选删项（来自 D4 §5「⚠️ 待核」表，与 commit/grep 事实交叉验证）

> 这一节**不是**代码自己标的 TODO，而是 **ROADMAP D4 §5** 已经识别的候选清理项。事实由本次抽取数据回填："是不是真的待裁"——下游 product-debate 决定。

| 候选项 | D4 §5 标记 | 事实佐证 | 待裁结论建议给 product-debate |
|---|---|---|---|
| `popsicle-sync` crate（整体）| ⚠️ 候选删 | 不被 popsicle-core 引用（rg 0 行）；仅 `popsicle-cli/src/commands/sync.rs` 用；intent-coder 10 个 skill 无一依赖。LoC = 895，crate 大小最小。 | 砍：intent-coder dogfood 不需要多设备 sync |
| `popsicle sync` 子命令（cli）| ⚠️ 候选删 | 与上同根原因 | 与 popsicle-sync 一起处理 |
| `namespace` 实体（model）| ⚠️ 候选删 | 当前 popsicle-new 自身使用了 `popsicle admin namespace create popsicle-migration`——意味着该实体**正在**为 IDD 流程提供 spec 容器。砍它需要换一种方式表达"spec 容器"。 | 留，但**简化**：单 namespace per repo / 移除多租户暗示 |
| `issue` 实体（model + cli）| ⚠️ 候选删 | popsicle-new 当前使用 `popsicle issue create/start` 启动 pipeline run —— **被 IDD pipeline 启动流程依赖**。砍它需要换一种方式启动 PipelineRun。 | 留，但可重命名为 `task` 之类避免与 GitHub Issues 混淆 |
| `work_item` 实体（model + cli）| ⚠️ 候选删 | `engine/extractor.rs::extract_user_stories/test_cases/bugs` 三个公开 fn 全依赖它（每个抽取都返回 `Vec<WorkItem>`）；CLI `popsicle item` / `popsicle extract` 子命令依赖。在 intent-coder v0.3 任务图范式下，"task chunk"已替代 work_item 的部分价值。 | 待 fact-extractor + product-debate 决：保留三类抽取 fn / 抽象为 task chunk |
| `doc` 命令族（cli）| ⚠️ 部分候选裁 | `doc create/show/list/check/extract/summarize` 6 个子命令——`doc create/check` 是 IDD 主流程刚才用过的；`extract` 喂 work_item；`summarize` 用于检索。intent-coder 不依赖 `doc extract/summarize`。 | 留 create/check/show/list；考察 extract/summarize 是否独立子命令 |
| `prompt` 命令（cli）| ⚠️ 候选删 | popsicle-new 刚刚用过 `popsicle prompt project-init --state surveying`——是 IDD 主流程必需（每个 skill 状态的 LLM prompt）。砍它会让 agent 无法显式取 prompt。 | 留 |
| `migrate` 命令（cli）| ⚠️ 候选裁剪 | DB schema migration —— intent-coder 不直接调用。但 popsicle 自身演进时需要。 | 留作 admin 子命令 |
| `checklist` 命令（cli）| ⚠️ 候选裁剪 | 与 `doc check` 重复（`popsicle doc check status/check/uncheck`）；存在原因（兼容老用法？）需要 fact-ext 历史考察。 | 砍：`doc check` 已覆盖 |

### 三类抽取功能（model/work_item.rs + engine/extractor.rs）

| 功能 | File:Line | 状态 |
|---|---|---|
| `extract_user_stories` | `engine/extractor.rs:10` | 公开 |
| `extract_test_cases` | `engine/extractor.rs:69` | 公开 |
| `extract_bugs` | `engine/extractor.rs:125` | 公开 |

> 这三类是 D4 §5 work_item 候选删的**真正**实现表面。删 work_item 等于删这三 fn 的返回类型；最简方案是把 `Vec<WorkItem>` 改成 `Vec<TaskChunk>`（intent-coder v0.3 范式）。决定权交 product-debate。

---

## Commit 热点（一年）—— 高频改动 ≈ 高风险

> top 30 高 churn 文件已合并到 `fact-extraction-report.md` 的 Risk Hotspots 表。这里复述 top 10：

| commits/yr | 文件 |
|---|---|
| 26 | `crates/popsicle-core/src/agent/mod.rs` |
| 26 | `crates/popsicle-cli/src/ui/commands.rs`（Tauri UI） |
| 22 | `crates/popsicle-core/src/storage/index.rs` |
| 18 | `crates/popsicle-cli/src/commands/pipeline.rs` |
| 17 | `crates/popsicle-core/src/dto.rs` |
| 17 | `crates/popsicle-cli/src/commands/doc.rs` |
| 16 | `crates/popsicle-cli/src/commands/mod.rs` |
| 13 | `crates/popsicle-cli/src/ui/mod.rs` |
| 12 | `crates/popsicle-cli/src/commands/issue.rs` |
| 12 | `crates/popsicle-cli/src/commands/init.rs` |

> `agent/mod.rs` 高 churn 反映 agent target 接入（claude / cursor / codex / copilot / opencode）的反复迭代。
> `ui/commands.rs` 高 churn 反映 Tauri UI 子项目活跃度——但 UI 是 optional feature。

---

## Extraction Checklist

- [x] 每条 TODO/FIXME 都有 file:line（命中 1，已记 markdown.rs:37）
- [x] git history 不可用时（本次未跑 blame 反查）已标 `[reduced fidelity]`
- [x] Deprecated API 章节已填（写 `(none found)`）
- [x] 死代码章节只来自编译器输出（本次未跑 build，已写 `(needs human input)`）
- [x] 禁用测试章节已审阅（写 `(none)`）
- [x] Build 警告快照已声明（`(build did not run)`）
- [x] 按模块汇总的合计与各分节匹配
- [x] 没有句子含 "should be done" / "ought to fix" / "is bad"（发表观点检查）
- [x] D4 §5 候选删项与 grep 事实交叉验证（额外章节）
