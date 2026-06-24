---
id: f9c75a54-e07c-4d3d-abf4-cebb1d5605c8
doc_type: arch-debate-record
title: workspace 布局决策：crate 切分与 products/* 归位（ADR-Workspace-Layout 候选）
status: final
skill_name: arch-debate
pipeline_run_id: 9efd7ec4-921a-4457-85ac-02ee082c8bec
spec_id: df11b6f9-e504-42b7-8d27-700d529c2346
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-08T09:04:15.404341Z
updated_at: 2026-06-08T09:10:58.941780Z
---

---
artifact: arch-debate-record
slug: workspace-layout
topic: "member crate 物理归位：根级扁平 crates/<slice>/（沿用 legacy）vs products/<product>/crates/*"
participants: [ARCH, SEC, PERF, OPS, DATA, DEV]
confidence: 4
date: 2026-06-08
query_anchors:
  - "这个架构问题当时为什么这么定？"
  - "哪些方案被否了，理由是什么？"
  - "这个决策要走 ADR 还是 CADR？"
---

# 架构辩论纪要 — workspace-layout

> 由 `arch-debate` skill 生成。本纪要是技术决策的**审计轨迹**，供 rfc-writer /
> adr-writer 追溯论据，也供后人理解「当时为什么这么选」。

## Topic

skill-runtime 进入 in-shadow 实现期前，必须确定 member crate 的**物理落点**与
workspace `members` glob 形态。来源：`products/skill-runtime/ARCHITECTURE.md` §
Open Decisions 第 59 行 `[TBD] ADR-Workspace-Layout`。
**关键澄清（用户）**：`skill-runtime / artifact-system / cli-ux` 三个 slice 是
**同一个产品 popsicle** 的切片（迁移/归属单元），**不是三个独立产品**——因此沿用
legacy 的「根级扁平 `crates/`」目录约定，而非按产品嵌套。此为 **ADR 候选**（不触 charter 铁律，非 CADR）。

## Participants

| 角色 | 立场速写 |
|---|---|
| ARCH | 主持。要 crate 边界与**迁移切片**（migration/progress.md 的逐切片 shadow/cutover）对齐，再加切片时零重排。 |
| SEC | 关注 crate 间「偷引用」——依赖图应让切片边界在编译期可见。 |
| PERF | 单 workspace 增量编译；crate 切分别太细，避免链接膨胀；布局对编译时长基本中性。 |
| OPS | 代言简单性与 legacy 一致性：`members` glob 越短越好，沿用 legacy `crates/` 降低迁移认知成本。 |
| DATA | 关注 intents（`products/<slice>/`）与实现 crate（`crates/<slice>/`）按 slice 名平行对齐，便于溯源。 |
| DEV | 实现专家：一个 slice = 一个 crate，shadow/cutover 时按 crate 整体切换，边界干净。 |

用户置信度：4/5

## Phase 1 — 技术问题 + 质量属性（NFR）

- 要解决的问题：root `Cargo.toml` 现为 `members = []`（占位），其注释倾向
  `products/<product>/crates/*`，而 `ARCHITECTURE.md` § File Manifest 写根级
  `crates/<name>/`——**二者自相矛盾**，实现前必须收敛成单一事实。
- 硬约束：
  - 单一 cargo workspace，`resolver = "2"`（root `Cargo.toml` 已固定，不重开多 workspace）。
  - 不破坏 `legacy/` 子模块 pin（`legacy/popsicle` 自带独立 workspace，互不侵入）。
  - **单产品约束（用户澄清）**：三个 slice 同属 **一个产品 popsicle**（一套 CLI / 一个发布物），
    `products/{skill-runtime,artifact-system,cli-ux}/` 是 **IDD 文档/spec 分组**，**不是**产品边界；
    沿用 legacy「根级扁平 `crates/`」目录约定。
  - 每个 slice 在 `migration/progress.md` 里**独立**走 not-started→in-shadow→cutover——
    crate 边界应与 slice 对齐，以获得干净的 shadow/cutover 隔离单元。
- 质量属性优先级（高→低）：
  1. **迁移局部性**——每个 slice 是独立的 shadow/cutover 单元，crate = slice 给最干净的切换边界。
  2. **legacy 约定一致**——沿用根级 `crates/`，降低迁移期认知成本。
  3. **可演进性**——再加 slice（crate）时零目录重排。
  4. **工具简单**——`members` glob 一行可表达。
- 事实基引用：
  - F-legacy：`legacy/popsicle/Cargo.toml` = 单 workspace、扁平
    `crates/{popsicle-core,popsicle-cli,popsicle-sync}`（sync 已按 PDR-001 砍掉）。
  - F-new：`products/{skill-runtime,artifact-system,cli-ux}/` 各含
    `tasks/ decisions/ intents/ proposals/` 文档目录、**暂无代码**；root `Cargo.toml`
    `members = []`。
  - F-product：三 slice 同属 popsicle 一个产品 / 一套 CLI 二进制（用户澄清）。
  - 注：以上为 `cargo` 元数据与目录直读，非重跑 fact-extraction-report。

## Phase 2 — 方案发散

- **方案 A**（提案者 ARCH，**前提被推翻**）：**每产品自带 crates**——`products/<product>/crates/<name>/`，
  `members = ["products/*/crates/*"]`。**前提是多产品**；用户澄清三 slice 同属一个产品后，此前提不成立。
- **方案 B**（提案者 OPS）：**根级扁平 crates/，按技术层**——`crates/popsicle-core` / `crates/popsicle-cli`
  （legacy 原样），`members = ["crates/*"]`。最贴 legacy；但技术层 crate **横跨多个 slice**，
  与 migration 的逐 slice shadow/cutover 边界**不对齐**。
- **方案 C**（提案者 DEV / ARCH）：**根级扁平 crates/，每 slice 一 crate**——
  `crates/skill-runtime` / `crates/artifact-system` / `crates/cli-ux`，`members = ["crates/*"]`。
  沿用 legacy 扁平约定，且 crate = slice = 独立可影子/切流的迁移单元；`products/<slice>/` 平行承载 IDD 文档。

## Phase 3 — 多角色评审

| 方案 | SEC | PERF | OPS | DATA | DEV |
|---|---|---|---|---|---|
| A | 边界清晰但前提（多产品）不成立 ✗ | 中性 | 嵌套深、与 legacy 不一致 ✗ | 文档/代码同产品树，但前提不成立 ✗ | 前提不成立，否决 ✗ |
| B | 技术层 crate 跨 slice，切片边界编译期不可见 ✗ | 中性 | glob 最短、最贴 legacy ✓✓ | intents 按 slice、crate 按技术层，溯源错位 ✗ | shadow/cutover 要跨 crate 协调，边界不干净 ✗ |
| C | crate=slice，切片边界即依赖边界，偷引用可见 ✓ | 单 workspace 增量编译，中性 ✓ | glob `crates/*` 一行、沿用 legacy ✓ | `crates/<slice>` 与 `products/<slice>` 同名平行，溯源直观 ✓ | 一 slice 一 crate，按 crate 整体 shadow/cutover，最干净 ✓✓ |

## Phase 4 — 收敛与决策

- ARCH 综合：用户澄清「三 slice 同属一个产品 popsicle」后，A 的多产品前提被推翻，出局。
  剩 B（技术层 crate）vs C（每 slice crate）：首要 NFR 是**迁移局部性**——`migration/progress.md`
  让每个 slice 独立走 in-shadow→cutover，crate=slice（C）使每次切换是一个 crate 的整体替换，
  边界最干净；B 的技术层 crate 横跨多 slice，shadow/cutover 要跨 crate 协调，边界模糊。
  两者都沿用 legacy 扁平 `crates/`、`members=["crates/*"]`，工具简单性持平。
- 角色投票：C = ARCH / DEV / DATA / SEC（4）；B = OPS（1，纯 legacy 一致性）；A = 0（前提出局）。
- **用户最终决策**：采纳**方案 C**——根级扁平 `crates/<slice>/`，每 slice 一 crate，`members = ["crates/*"]`。
  用户置信度 4/5。（用户澄清「几个 slice 是一个产品，参考老 popsicle 目录结构」，
  据此从最初倾向的 A **改判** C。）
- intent 层归位（IDD 纪律 §2）：本决策为**目录/构建布局**约定，不对外暴露跨模块 API，
  **不进 `contracts.intent`**；迁移局部性/编译期边界为质量属性目标（D2），写进 RFC「质量属性目标」，
  由 `cargo` 结构与 CI 守护，不塞 `.intent`。决策标 **ADR 候选**（非 CADR）。

## Decision

popsicle 的 member crate 一律落在**根级** `crates/<slice>/`（如 `crates/skill-runtime`），
每个迁移 slice 对应一个 crate，workspace `members = ["crates/*"]`，沿用 legacy 扁平约定。
`products/<slice>/` 仅承载该 slice 的 IDD 文档（intents/tasks/decisions/proposals），
与 `crates/<slice>/` 按 slice 名平行对齐。跨 crate 依赖经 `workspace.dependencies` 显式声明。
`ARCHITECTURE.md` § File Manifest 的 `crates/<name>/` 明确为 `crates/<slice>/`，root
`Cargo.toml` 注释里的 `products/<product>/crates/*` 倾向改为 `crates/*`。

## 关键分歧

- **方案 A 的前提**：ARCH 最初按「每 product 一个名词组」倾向 A（products/<product>/crates/*）
  → 用户澄清三 slice 同属一个产品 popsicle，前提推翻，A 出局。
- **B（技术层 crate）vs C（每 slice crate）**：OPS 主张 B（最贴 legacy core/cli）vs DEV/ARCH 主张 C
  （crate=slice=迁移单元）→ 收敛到 C；C 同样沿用扁平 `crates/`，但让 crate 边界对齐 migration 的
  逐 slice shadow/cutover，切换更干净。

## 用户决策点

- [x] 用户决策是否覆盖了多数角色意见？**覆盖了 ARCH 的初始倾向**——ARCH 起初倾向 A，
  用户澄清「单产品 + 参考 legacy 目录」后改判 C；该澄清同时获 DEV/DATA/SEC 认同，C 成为多数（4/6）。
  覆盖理由充分（A 的多产品前提客观不成立），**低风险**。
- 冷静期建议：C 沿用 legacy 既有约定、加 slice 零重排，决策稳健，**无需冷静期**。
- 次级待决（移交 rfc / adr 细化）：`crates/skill-runtime` 首版是 **lib** 还是同时含 bin、
  cli-ux crate 是否为唯一 bin——本场不强定，留 RFC 细化、由 adr 批准闸 ratify。

## 下游接驳建议

- rfc-writer：把本纪要打磨成正式 RFC + `ARCHITECTURE.md` 增量（① § File Manifest 路径明确为
  根级 `crates/<slice>/`；② root `Cargo.toml` `members = ["crates/*"]` 并改占位注释；
  ③「质量属性目标」记迁移局部性/编译期边界）+ ADR 骨架 `ADR-003-workspace-layout`（Proposed）。
- 本决策**无**触及 charter「四条铁律 / Layer Map」的条目 → 走 **ADR**，非 CADR。
- 无 contracts 种子产出（纯布局约定，不暴露跨模块 API）。

## Setup Checklist

- [x] 技术议题已用一句话表达（来自 ARCHITECTURE.md § Open Decisions 第 59 行 [TBD] ADR-Workspace-Layout）
- [x] 边界已绑定（产品 = popsicle，决策对全 `crates/<slice>` 通用）
- [x] 事实基状态已记录（直读 cargo 元数据/目录：F-legacy 扁平 crates/、F-new products/* 无代码、F-product 单产品）
- [x] 技术角色阵容确定（6 人：ARCH + SEC/PERF/OPS/DATA 约束代言 + DEV 实现专家）
- [x] 用户置信度已设置（4/5）
- [x] **已展示 setup 摘要并取得 `start` 确认**（用户经 ask_user 两轮选定方案 C）

## Phase Coverage

- [x] Phase 1：技术问题 + NFR 优先级 + 硬约束（含单产品澄清）已明确
- [x] Phase 2：3 个差异化架构方案（A 每产品 crates / B 技术层 crate / C 每 slice crate），各含边界/数据流
- [x] Phase 3：全部角色（含 SEC / PERF / OPS / DATA / DEV）已评审
- [x] Phase 4：收敛到方案 C + 用户决策（改判）+ 声明已标 intent 层（不进 contracts）/ ADR 候选
- [x] 至少 4 个用户交互暂停点（setup 确认 + 两轮核心抉择 ask_user + 本阶段批准闸 + 下游 rfc 接驳）

## Output Checklist

- [x] Phase 1-4 小结齐全
- [x] 关键分歧与各方立场已记录
- [x] 用户决策点已显式记录（含覆盖情况）
- [x] 每个数字/模块名引用可追溯到事实基（F-legacy / F-new / F-product，cargo 元数据直读 + 用户澄清）
- [x] Topic 与另两份 artifact 一致（rfc / adr 均以 ADR-003 workspace-layout 为题）
