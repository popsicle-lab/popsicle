---
task_id: T-0006
slug: skill-version-bump
title: "我作为 skill 包维护者发布一个新 skill 版本并把现有 pipeline 升上去"
journey_stage: lifecycle
audience: ["human-maintainer", "skill-author"]
task_type: 操作指南
decision_ref: PDR-002
last_updated: 2026-06-08
last_verified: 2026-06-08   # living-doc-author 回填：related_intent 经 intent-check Z3 verified
intent_kind: upgrade
involved_features: ["skill-loader", "skill-version-resolution", "pipeline-revise"]
prerequisites:
  - "在 intent-coder（或其他 skill 包）目录里把 `skill.yaml` 的 `version:` 字段升过 semver"
  - "已 completed 的 pipeline run 至少 1 个（要避免影响）"
limits:
  - "已 completed 的 run 不会被升级影响（INV: completed run 状态冻结）"
  - "未 completed 的 run 升级到新 skill 时需要重启该 stage（非透明）"
related_intents:
  - "acceptance.intent#UpgradeDoesNotAffectCompletedRuns"
related_next_tasks:
  - T-0001
  - T-0005
fact_cite:
  - "fact-extraction-report § api-contracts.md §popsicle-cli §skill load / pipeline revise"
  - "PDR-001 §Phase 4 §Action Items (skill 包升级的 IDD 流程)"
---

# 我作为 skill 包维护者发布一个新 skill 版本并把现有 pipeline 升上去

---

## 本 task 可解答

- "skill 怎么升级到新版本？老的 pipeline 会被自动用新版吗？"
- "已经跑完的 run 用了旧版 skill —— 升级后那些 run 的 artifact 还可信吗？"
- "in-progress 的 run 跨越 skill 升级会发生什么？"

---

## 前提与限制

**你需要先**：
- 在 skill 包仓库（如 `intent-coder/`）改了 `skill.yaml` 的 `version`（遵循 semver）
- 已跑 skill 包自身的 lint / schema-validate（避免装一个坏 skill）

**本 task 受以下限制**：
- 已 `completed` 的 run 不会自动用新版 skill（INV: completed run frozen）
- 未 `completed` 的 run 跨升级需要在 stage 边界手工重启该 stage

---

## 完成路径

1. **在 popsicle-new 仓库重新加载 skill 包**：

   ```bash
   popsicle skill load --module ../intent-coder --force
   ```

   `--force` 触发版本升级（不带则维持原版本）。CLI 返回 `Updated <n> skill(s): <name>@<old> → <new>`。

2. **看升级影响**：

   ```bash
   popsicle skill show <skill-name>
   popsicle pipeline list --filter "uses:<skill-name>,status:in_progress"
   ```

   找出哪些活跑 run 受影响。

3. **对每个受影响 in-progress run 做选择**：
   - 该 run 当前所在 stage 已 completed → 后续 stage 自动用新版（无需操作）
   - 该 run 当前所在 stage 是 in_progress → 选择 `pipeline revise` 创建一份基于该 run 的修订 run，从当前 stage 重启

   ```bash
   popsicle pipeline revise --from-run <run-id> --reason "skill <name> bumped to <new>"
   ```

4. **完成的 run 不动**：跑 `popsicle pipeline verify --run <old-run-id>` 确认旧 run 仍 PASS（用旧版 skill 的口径）。完成的 run 不可被升级污染（charter #2）。

5. **记录升级到 ADR**（可选）：当 skill 升级是 breaking change 时，新 ADR 标 `Supersedes: ADR-XXXX-<old>`。

---

## 可观察的成功标志

`popsicle pipeline verify` 对所有 completed run 仍 PASS；新启的 stage 用新版 skill；ADR 升级链可追溯。

形式化定义：见 [`acceptance.intent#T-0006-upgrade-does-not-affect-completed-runs`](../../intents/acceptance.intent)

---

## Related Next Tasks

- **[T-0001 - 第一次给 intent-coder 加载 skill 包跑通 pipeline](../onboarding/T-0001-first-pipeline-run.md)** — 升级后跑一遍验证
- **[T-0005 - 审查 pipeline 产物链做合规复盘](../admin/T-0005-audit-trail.md)** — 升级前后做对照

---

## Charter Compliance

- [x] frontmatter 8 个必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 第一人称
- [x] 完成路径无大量 if-else（step 3 的 2 个分支是 stage 状态差，非用户选择）

---

`Decision-Ref: PDR-002`
