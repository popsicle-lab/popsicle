# issue-author 使用指南

> **独立 skill**：不进入 `fix-regression` / `migration-slice-delivery` 等 pipeline yaml。在
> `popsicle issue create` **之前**由 Agent 执行；人类在 UI 创建 Issue 时可跳过。
>
> **本 guide 是 Issue 创建的唯一入口**：内含 pipeline 决策树、`migration-slice-delivery`
> 门禁、反模式与 retro spec 路径（原 `guides/pipeline-selection.md` 已删除）。

## 目标

1. 选对 `--pipeline`（见下方决策树）。
2. 建立 Issue ↔ Task **多对多**关联：
   - **linked**：关联 `products/<product>/tasks/` 中已有 task
   - **proposed**：语义上需要新 task，待 `living-doc-author` 晋升
3. 产出简体中文 `--title` / `--description`（遵循项目 `agent_context`）。

## Pipeline 决策树

在 `issue start` 之前走完本树，避免 spec 未完成就进入 `migration-slice-delivery` / `feature-delivery`。

```text
新工作项
├─ 全新 product 模块（无 legacy、无已定 spec）？
│  └─ yes → product-greenfield-spec
├─ 所属 slice 的 PRD / intent 尚未覆盖本能力（迁移域）？
│  └─ yes → migration-slice-spec（或 arch-decision，若仅需架构决策）
├─ 已有 product 增量能力 spec 未覆盖？
│  └─ yes → feature-spec
├─ 仅架构/技术选型（无实现）？
│  └─ yes → arch-decision
├─ 单点回归/缺陷修复？
│  └─ yes → fix-regression
├─ 代码已合并，只补 PDR/task/intent？
│  └─ yes → doc-retro-spec
└─ spec 已定（acceptance.intent 有对应 block 且 intent check 通过）？
   ├─ 迁移 slice + golden/cutover → migration-slice-delivery
   └─ 日常能力 → feature-delivery
```

### Issue 类型与默认 pipeline

| `--type` | 默认 pipeline | 典型用途 |
|---|---|---|
| `product` | `product-greenfield-spec` | 新产品/模块从零 spec |
| `technical` | `arch-decision` | 架构辩论 → RFC → ADR |
| `bug` | `fix-regression` | 最小修复环 |
| `idea` | `arch-decision` | 探索性技术想法 |

**不要**把 `--type technical` 的默认当成「功能实现」。在已有 slice 上增量交付时，应**显式**指定：

```bash
# 1) spec 未覆盖时（迁移 slice）
popsicle issue create --type technical --title "..." --product cli-ux \
  --pipeline migration-slice-spec --description "..."

# 1b) spec 未覆盖时（日常 feature）
popsicle issue create --type technical --title "..." --product cli-ux \
  --pipeline feature-spec --description "..."

# 2) spec 已覆盖、只写代码时
popsicle issue create --type technical --title "..." --product cli-ux \
  --pipeline feature-delivery --tasks T-CU-0002 \
  --description "实现 T-CU-0002 …（description 须含每个 --tasks id）"
```

## 流程

```text
扫描 tasks + intents + 用户意图
    → 应用上方决策树
    → 填 issue-create-report（可选，无 run_id 时写入 description 摘要）
    → popsicle issue create ... --tasks ... --proposed-task ...
    →（用户确认后）popsicle issue start
```

## CLI 示例

```bash
popsicle issue create --type technical \
  --title "Issue 与 Task 多对多关联" \
  --product cli-ux \
  --pipeline feature-delivery \
  --tasks T-CU-0010,T-CU-0011 \
  --description "实现 T-CU-0010、T-CU-0011 …（须写明每个 task_id）"
```

新旅程（spec 未覆盖）应 `--proposed-task` + `feature-spec` 或 `migration-slice-spec`，**不可** delivery pipeline + proposed：

```bash
popsicle issue create --type technical \
  --title "新能力 …" \
  --product cli-ux \
  --pipeline feature-spec \
  --proposed-task "新旅程标题|daily-ops" \
  --description "…"
```

`--epic-task` 已废弃；等价于单个 `--tasks`。

## migration-slice-delivery 硬门禁

### 文档门禁（进入 implement 前）

- [ ] 目标能力在 `products/<product>/intents/*.intent` 有 acceptance block
- [ ] `popsicle tool run intent-validate path=products` 通过（Z3 + 合并 goal 追溯：`realized_by` 非空且可解析）
- [ ] 相关 ADR File Manifest 列出将改动的路径
- [ ] 若跳过 spec 链，必须在 cutover ADR 的 Divergence 表登记（如 D-6xx）

### CLI 强制（`issue create` / `issue start`）

| 时机 | 规则 |
|---|---|
| `issue create` | `--pipeline migration-slice-delivery` 与 `--proposed-task` **不可同用** |
| `issue start` + `migration-slice-delivery` | 禁止含 `proposed` task link |
| 同上 | 至少一个 `linked` task（`--tasks` / `--epic-task`） |
| 同上 | `--description` 须包含每个 linked `task_id` |
| 同上 | linked task 的 `related_intents` 须在 `products/<product>/intents/` 可解析 |

违反时 CLI 返回 `migration-slice-delivery-gate:*` 错误与可执行 `next` 提示。旧 alias `slice-delivery` 同样适用。

## 反模式（PROJ-30 / PROJ-35 教训）

| 反模式 | 后果 | 应用 |
|---|---|---|
| 直接 `--pipeline feature-delivery` / `migration-slice-delivery` 写 UI 功能 | 无 PRD/intent 前置，事后补 ADR | 先补 intent/task，或 `feature-spec` / `migration-slice-spec` |
| 把增量增强当成 `product` greenfield | 重复 debate/PRD，过重 | 见下方「已交付能力补 spec」 |
| 已交付能力 retro spec 却开 `migration-slice-spec` | 误跑 facts/debate 全链 | **`doc-retro-spec`** 或直接写 PDR + task + `acceptance.intent` |
| 未安装 intent-coder module 就指望 skill 模板 | `doc create` 只有空壳 | `popsicle init` + `admin sync-intent-coder` |
| 只 linked 邻近 task 却交付新能力 | 错开 delivery，spec 后置 | 新能力用 `--proposed-task` + `feature-spec` |
| **`--pipeline fix-regression` 滥用**（PROJ-49～51 教训） | spec/技能链绕过审批与 traceability | 见下方「fix-regression 硬门禁」 |

## fix-regression 硬门禁（PROJ-53，`issue create` CLI 强制）

`fix-regression` 仅用于**单点回归 / UI·CLI 缺陷**。`issue create` 在下列情况 **拒绝** `--pipeline fix-regression`（错误码 `fix-regression-gate:*`；旧 alias `bugfix` 同样适用）：

| 规则 | 触发 | 应改用 |
|---|---|---|
| `product-type` | `--type product` + `fix-regression` | `product-greenfield-spec` 或 `doc-retro-spec` |
| `intent-content` | title/description 触达 `products/*/intents`、`*.intent`、`realized_by` | `doc-retro-spec` 或 `feature-spec`；已定 spec → `feature-delivery` + `--tasks` |
| `skill-chain` | 触达 `intent-coder/skills/` 或技能名 + `intent-coder` | `--type technical` + `arch-decision` 或 `feature-spec` |
| `ui-capability` | 接入 visualizer / 多图等新 UI 能力（非「修复」措辞） | `feature-spec` 或 `feature-delivery` |

**允许 fix-regression 的例子**：关系图缩放/对比度、控件不可见、CLI 回归、Z3 误报修复（不改 intent 文件语义）。

实现 `fix-regression-gate` 本身的 Issue 可在 description 含 `fix-regression-gate` / `pipeline_gate` 时豁免（meta）。

## 已交付能力补 spec（retro）

代码已合并（如 PROJ-29/30/34），只需补 living docs 时：

1. **不要** `issue start` + `migration-slice-spec`（`facts` 面向 legacy 迁移，不是 retro）。
2. 优先 **`doc-retro-spec`** pipeline，或**直接**在 `products/<product>/` 写：PDR → task → `acceptance.intent` block。
3. 跑 `popsicle tool run intent-validate path=products/<product>/intents`。
4. 在相关 ADR 或 cutover 记 Divergence（若实现先于 spec）。
5. 可选：建 Issue 仅追踪，**不启动 pipeline**；或交付完成后 `issue close`。

## 模块安装（ADR-017）

| 场景 | 来源 |
|---|---|
| popsicle 单体仓库（根目录有 `intent-coder/`） | 从工作区根 **覆盖** 同步 |
| DMG / `cargo install` / 任意新项目 | 从 **二进制内嵌包** 解压到 `.popsicle/modules/intent-coder/` |

`popsicle module add` 在 self-host MVP 仍 **deferred**（ADR-011）；用 `popsicle init` 或
`admin sync-intent-coder`。`doctor --format json` 查看 `intent_coder_module` 与
`intent_coder_bundle`。

## Mermaid 画图（`mermaid-diagram` tool）

复杂工作项（跨 ≥3 个 linked task、或 pipeline 阶段多）时：

```bash
popsicle tool run mermaid-diagram action=guide
popsicle tool run mermaid-diagram action=scaffold type=flowchart title="本 Issue 与 task 关系"
```

将 scaffold 输出并入 `issue-create-report` 或 `--description` 末尾；节点用真实 `task_id`。
细节图留给 prd-writer / rfc-writer。

## 硬规则

| 做 | 不做 |
|---|---|
| 用 task 标题/意图语义选 linked | 机械绑定「Epic」父 task |
| proposed 写清 journey_stage | 在 implement 阶段偷偷改 `.intent` |
| spec 未覆盖时选 `feature-spec` / `migration-slice-spec` | 默认 `--type technical` 的 arch-decision |
| 读 `doctor` 的 `agent_context` 语言偏好 | 英文 title（除非用户要求） |
| `--description` 写明每个 `--tasks` id | 只 linked 邻近 task 却交付新能力 |
| 新旅程用 `--proposed-task` + spec pipeline | delivery pipeline + proposed |
| 单点缺陷用 `--type bug` + `fix-regression` | 用 `fix-regression` 改 intent 文件 / intent-coder 技能链 / 新 UI 能力 |

## 下游检查

- **shadow-implementer** 末：登记新发现的 task 关联（`discovered`）或 proposed。
- **living-doc-author**：task 文件落地后执行：
  ```bash
  popsicle issue link <KEY> --tasks T-XXXX --replace --drop-proposed
  ```
  将 `proposed` 晋升为 **linked**（不再依赖手改 SQLite / backfill 脚本）。
