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
├─ 【先问一句】spec 是否已具备？（acceptance.intent 有对应 block 且 intent check 通过）  ← P1 显式分叉
│   否 → 走「spec 未具备」分支（下面前半段），先补 spec，别直接 delivery
│   是 → 跳到最底「spec 已定」分支选交付快车道
│
├─ 全新 product 模块（无 legacy、无已定 spec）？
│  └─ yes → product-greenfield-spec
├─ 所属 slice 的 PRD / intent 尚未覆盖本能力（迁移域）？
│  └─ yes → migration-slice-spec（或 arch-decision，若仅需架构决策）
├─ 已有 product 增量能力 spec 未覆盖？
│  ├─ 小增量、无跨模块架构 ADR？ → feature-spec
│  └─ 大增量：需 PDR+ADR+task+intent，但不是新 product？ → feature-arch-spec
├─ 仅架构/技术选型（无实现）？
│  └─ yes → arch-decision
├─ 单点回归/缺陷修复？
│  └─ yes → fix-regression
├─ 代码已合并，只补 PDR/task/intent？
│  └─ yes → doc-retro-spec
└─ spec 已定（acceptance.intent 有对应 block 且 intent check 通过）？
   ├─ 迁移 slice：以**保留 legacy 行为**为主、bootstrap 已跑过 spec 链、无重设计？
   │      → migration-preserve（快车道：facts→@asis→intent-check→implement→equivalence→cutover→living-docs）
   ├─ 迁移 slice：有重设计（新 ADR）+ golden/cutover → migration-slice-delivery
   └─ 日常能力 → feature-delivery
```

> **P1 教训（为什么加「spec 是否已具备」这句）**：bootstrap 已把 store 切片的 spec 链跑完后，
> 本该走 `migration-preserve` 快车道，却因决策树无此显式分叉误选了 `migration-slice-delivery`
> （又重走一遍重设计前半段）。`migration-preserve` 与 `migration-slice-delivery` **共享同一条交付尾**
> （`implement → equivalence → cutover → living-docs`，见下「共享交付尾」），区别只在前半段：
> preserve 是 facts-first 的行为保留，delivery 含重设计。**没有重设计需求就选 preserve。**

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

## 共享交付尾（P1）

`migration-preserve` 与 `migration-slice-delivery` 的后四阶段**一字不差相同**：

```text
implement → equivalence → cutover → living-docs      ← 共享交付尾（含 cutover/equivalence 机验 gate）
```

差异只在**前半段**：`migration-slice-delivery` 前面是重设计链（含新 ADR），
`migration-preserve` 前面是 facts-first 的 `facts → @asis intent → intent-check`。选 pipeline 时只需判断
**前半段要不要重设计**；交付尾（及其 gate、审批轴）两者一致，无需重新学。

> 注：pipeline yaml 目前无 include 机制，两条 bundled pipeline 的交付尾各自内联但保持同步；
> 维护时改一处须同步另一处（golden-capture/verifier 等新 gate 也应同时挂到两者的 cutover/equivalence）。
> 真正的「共享 stage 组」引擎机制留待 ROADMAP（避免为 2 个内置 pipeline 过度设计）。

## 交付中发现 spec 缺口：回流到 spec（P2）

pipeline 是单向阶段链，但迁移/探索中常在 **delivery 才发现 spec 偏差**（例：implement/equivalence 阶段
比对 legacy 发现 `@asis` 实然行为与当初 `@tobe` spec 不符，如 store 切片的 ADR-0003）。**不要在 implement
里偷偷改 `.intent` 打补丁**——那会让「spec 完成」名不副实。正确回流：

1. **停下**，在当前 run 的 coverage / equivalence-report 里如实记录「发现的 spec 缺口」。
2. 选其一回流：
   - 缺口是**新能力/新旅程** → 另开 `feature-spec` / `migration-slice-spec` Issue（`--proposed-task`），补 spec 后再回来交付。
   - 缺口是**有意分叉**（保留 legacy 怪癖 vs 改进）→ 在 cutover ADR 的 Divergence 表登记 + Accepted ADR，intent 用 `@asis`/`@tobe` 显式化（见 intent-consistency-check 的「@asis↔@tobe 分叉」）。
   - 缺口是**spec 本身写错** → `doc-retro-spec` 或直接修 `.intent` + 记 PDR，并重跑 `intent-validate`。
3. 回流产生的 spec 变更须让下游门禁重算（equivalence/cutover gate 会拦不一致）。

> 引擎层的「阶段回退边 / spec↔impl 双向绑定」是更大的状态机改动，留待 ROADMAP；当前用「另开 spec issue + ADR 登记」
> 这一可操作回流闭环，配合机验 gate 保证偏差不被静默吞掉。

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

## Agent 观测

Issue / pipeline 工作中 Agent 应先 `popsicle tool run telemetry action=guide`，再按 guide 上报 `gen_ai.chat` 与可选 score；编排事件已自动 span，勿重复。

## 下游检查

- **shadow-implementer** 末：登记新发现的 task 关联（`discovered`）或 proposed。
- **living-doc-author**：task 文件落地后执行：
  ```bash
  popsicle issue link <KEY> --tasks T-XXXX --replace --drop-proposed
  ```
  将 `proposed` 晋升为 **linked**（不再依赖手改 SQLite / backfill 脚本）。
