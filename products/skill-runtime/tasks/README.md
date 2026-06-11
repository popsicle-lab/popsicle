# skill-runtime — Tasks 索引

> **Status**: prd-writer 首次填（PDR-002）— 健康度统计由 living-doc-author 在重跑时刷新
> **Last-Generated**: 2026-06-09（living-doc-author · implementation-status）
> **Source PDR**: [`../decisions/pdr/PDR-002-skill-runtime-tasks-mvp.md`](../decisions/pdr/PDR-002-skill-runtime-tasks-mvp.md)

5 个固定旅程阶段。**缺一不可，也不允许第 6 个**（intent-coder/skills/prd-writer/references/task-organization.md）。

`skill-runtime` 的用户可见行为按 task chunk 组织 — 每个 chunk 含 YAML frontmatter（audience / journey_stage / prerequisites / limits / query_anchors），可被 AI 单独召回回答用户问题。

---

## 索引

### onboarding/ — 首次接触到首次成功

| Task | 用户问句锚点（节选）| Audience | 已实施 | Last-Updated |
|---|---|---|---|---|
| [T-0001 我第一次给 intent-coder 加载 skill 包跑通 migration-bootstrap pipeline](onboarding/T-0001-first-pipeline-run.md) | "popsicle-new 第一次怎么开始用？" / "intent-coder 的 skill 包怎么装到 popsicle？" | ai-coding-agent / new-user | ✅ `pipeline_session` + `loader` | 2026-06-08 |

### daily-ops/ — 日常使用

| Task | 用户问句锚点 | Audience | 已实施 | Last-Updated |
|---|---|---|---|---|
| [T-0002 我作为 AI agent 把 pipeline 推到下一个 stage（含审批暂停点）](daily-ops/T-0002-advance-stage.md) | "怎么把一个 stage 标记为 completed？" / "stage complete 一定要带 --confirm 吗？" | ai-coding-agent / human-maintainer | ✅ `state_machine` + `pipeline_session` | 2026-06-08 |
| [T-0003 我作为 AI agent 查询当前 pipeline / stage / doc 处于什么状态](daily-ops/T-0003-inspect-state.md) | "怎么知道一个 pipeline run 到哪个 stage 了？" / "popsicle 里 skill 有哪些？" | ai-coding-agent / human-maintainer | ✅ `inspect` + `registry`（lib）| 2026-06-08 |

### troubleshooting/ — 故障排查

| Task | 用户问句锚点 | Audience | 已实施 | Last-Updated |
|---|---|---|---|---|
| [T-0004 我的 pipeline 卡在 blocked / 失败的 stage，我要把它恢复](troubleshooting/T-0004-recover-blocked.md) | "stage 卡在 blocked 是什么意思？" / "popsicle pipeline unlock 在干嘛？" | ai-coding-agent / human-maintainer | ✅ `runs::recover_blocked_pipeline` | 2026-06-08 |

### admin/ — 管理类（合规审查 / 配额 / 审计）

| Task | 用户问句锚点 | Audience | 已实施 | Last-Updated |
|---|---|---|---|---|
| [T-0005 我作为人类维护者审查一个 pipeline run 的完整产物链以做合规复盘](admin/T-0005-audit-trail.md) | "怎么看一个 run 的全部产出？" / "pipeline verify 检查什么？" | human-maintainer / compliance-reviewer | ⏳ CLI 审计链（cli-ux）| 2026-06-08 |

### lifecycle/ — 终止 / 迁出 / 升级

| Task | 用户问句锚点 | Audience | 已实施 | Last-Updated |
|---|---|---|---|---|
| [T-0006 我作为 skill 包维护者发布一个新 skill 版本并把现有 pipeline 升上去](lifecycle/T-0006-skill-version-bump.md) | "skill 怎么升级到新版本？" / "已经跑完的 run 用了旧版 skill — 升级后还可信吗？" | human-maintainer / skill-author | ✅ `runs::apply_skill_upgrade` | 2026-06-08 |

---

## 健康度统计

> 由 living-doc-author 在重跑时刷新。本统计是「文档腐烂预警」的核心信号。**2026-06-08 living-doc-author 刷新**：行数/日期取自 task 文件实测；未引用列基于反向引用扫描。

| 旅程阶段 | Task 数 | 平均行数 | 上次更新最久的 task | 未引用的 task |
|---|---|---|---|---|
| onboarding | 1 | 124 | T-0001（0 天前）| 无 |
| daily-ops | 2 | 117 | T-0002 / T-0003（0 天前）| 无 |
| troubleshooting | 1 | 112 | T-0004（0 天前）| 无 |
| admin | 1 | 116 | T-0005（0 天前）| 无 |
| lifecycle | 1 | 109 | T-0006（0 天前）| 无 |

> **last_verified 回填（intent-check Z3 verified）**：T-0001 / T-0004 / T-0006 已回填 `last_verified: 2026-06-08`。
> T-0002 暂不回填——其第二条 related_intent `#stage-transitions-forward-only` 为 orphan（未形式化），待人工解决后再认证。
> T-0003 / T-0005 无 verified intent 支撑（约束按 intent-spec D2 降级为 task 级），不回填。

⚠️ **未引用的 task** 是 AI 反馈闭环的输入：超 90 天无反向引用的 task 进入「归档评审」流程，由 PM 决定是否真的有用户在用。

---

## 命名约定

task 文件：`<journey-stage>/T-<NNNN>-<verb-noun-phrase>.md`（kebab-case，task_id 永不变；slug 可改）。

frontmatter 8 个必填字段：`task_id` / `slug` / `title` / `journey_stage` / `audience` / `task_type` / `decision_ref` / `last_updated`。

---

## 跨 Product 旅程

涉及本 product 的跨 product 旅程：

- 暂未注册（首切片仅 skill-runtime 内部；artifact-system 与 cli-ux 落地后会产生跨 product 旅程）

---

## 维护规则

1. **新增 task** 由 `prd-writer` skill 通过 PDR 引入
2. **修改 task** 必须有新 PDR（charter 铁律 #3）
3. **删除 task** 必须有标注 `Supersedes` 的 PDR 显式废止
4. **重新分类 task**（在 5 个目录之间移动）算修改，需要 PDR
5. **重命名 slug**（不改 task_id）算小改，不需要新 PDR，但要更新 Last-Updated
