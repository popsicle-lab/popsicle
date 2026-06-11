---
task_id: T-0001
slug: first-pipeline-run
title: "我第一次给 intent-coder 加载 skill 包跑通 migration-bootstrap pipeline"
journey_stage: onboarding
audience: ["ai-coding-agent", "new-user"]
task_type: 操作指南
decision_ref: PDR-002
last_updated: 2026-06-08
last_verified: 2026-06-08   # living-doc-author 回填：related_intent 经 intent-check Z3 verified
intent_kind: orchestrate
involved_features: ["skill-loader", "pipeline-runner", "issue-as-run-entrypoint"]
prerequisites:
  - "popsicle-new 仓库已 git clone（含 submodule）"
  - "rustc / cargo 可用，PATH 含 popsicle CLI"
limits:
  - "本 task 不含 doc 操作（属 artifact-system，见 [T-AS-* 待 PDR]）"
related_intents:
  - "acceptance.intent#PipelineBootstrapsToFirstPause"
related_next_tasks:
  - T-0002
  - T-0003
fact_cite:
  - "PDR-001 §Phase 4 表 3 (UI-1 AI agent 加载 intent-coder skill 包，启动 migration-bootstrap pipeline)"
  - "fact-extraction-report § Bounded Contexts"
---

# 我第一次给 intent-coder 加载 skill 包跑通 migration-bootstrap pipeline

---

## 本 task 可解答

- "popsicle-new 第一次怎么开始用？"
- "intent-coder 的 skill 包要怎么装到 popsicle？"
- "migration-bootstrap pipeline 是什么？跑到哪里会停下来等我？"

---

## 前提与限制

**你需要先**：
- 在 `popsicle-new/` 目录下，仓库已 init 且 submodule `legacy/popsicle` 已 checkout 到 pinned SHA（见 `LEGACY_PIN.md`）
- popsicle CLI 可用：`popsicle --version` 返回非空

**本 task 受以下限制**：
- 仅覆盖 `init → facts → debate` 三 stage 前的 bootstrap；后续 stage 在 [T-0002](T-0002-... 待 daily-ops) 推进
- 本 task 不含 doc 创建（doc 创建归 artifact-system，跨 product，见后续 PDR）

---

## 完成路径

1. **加载 intent-coder skill 包**：在仓库根执行

   ```bash
   popsicle skill load --module ../intent-coder
   ```

   CLI 返回 `Loaded N skills from intent-coder/`，N ≥ 10。

2. **创建首个 PipelineRun 的承载 issue**：

   ```bash
   popsicle issue create \
     --namespace popsicle-migration \
     --title "把 popsicle 通过 IDD 迁移到 popsicle-new（首切片 skill-runtime）"
   ```

   CLI 返回 `Issue created: <uuid>`。

3. **启动 migration-bootstrap pipeline**：

   ```bash
   popsicle pipeline start migration-bootstrap --issue <uuid>
   ```

   CLI 返回 `Pipeline Run started: <run-id>`，pipeline 自动推进到 `init` stage 的 `requires_approval` 暂停点。

4. **查看当前 pipeline 状态**：

   ```bash
   popsicle pipeline status
   ```

   预期输出含一行 `init  in_progress  project-init` —— 表示已停在首个审批点等你确认 scaffolding。

5. **审批 init stage 并完成**（仅在 scaffolding 检查通过后）：

   ```bash
   popsicle pipeline stage complete init --confirm
   ```

   CLI 返回 `Stage 'init' → completed; Unblocked: facts`。

---

## 可观察的成功标志

`popsicle pipeline status` 输出中至少有一个 stage 标记 `completed`，且至少有一个 stage 标记 `ready` 或 `in_progress`。

形式化定义：见 [`acceptance.intent#T-0001-pipeline-bootstrap-to-first-pause`](../../intents/acceptance.intent)

---

## Related Next Tasks

- **[T-0002 - 我作为 AI agent 把 pipeline 推到下一个 stage（含审批暂停点）](../daily-ops/T-0002-advance-stage.md)** — 推进到 init 之后的每个 stage
- **[T-0003 - 我作为 AI agent 查询当前 pipeline / stage / doc 处于什么状态](../daily-ops/T-0003-inspect-state.md)** — 暂停点上看状态决定下一步
- **[T-0004 - 我的 pipeline 卡在 blocked / 失败的 stage，我要把它恢复](../troubleshooting/T-0004-recover-blocked.md)** — 任意 stage 失败时跳到这里

---

## Charter Compliance

- [x] frontmatter 8 个必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 是完整第一人称句子
- [x] 「本 task 可解答」3 个用户原话问句
- [x] 完成路径无 if-else 分支（失败路径跳到 T-0004）
- [x] Related Next Tasks 3 个

---

`Decision-Ref: PDR-002`
