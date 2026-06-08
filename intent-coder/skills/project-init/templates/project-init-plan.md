# Project Init Plan — {repo_name}

> **Status**: draft → review → approved
> **Created**: {YYYY-MM-DD}
> **Author**: {agent + 人类确认者}

本计划必须经人类审阅并批准**之后**，才能创建任何文件。铺骨架是「不可逆-ish」的；product 命名会烙进每一份下游文档。

---

## Repository Identity

| 字段 | 值 |
|---|---|
| 仓库名 | `{new-intent-lang}` |
| 本地路径 | `{/Users/.../Workspace/github/new-intent-lang}` |
| Issue key 前缀 | `{INTENT}`（用于 BUG-INTENT-1、TC-INTENT-1 ……）|
| 默认 agent target | `{claude}`（claude / cursor / copilot / codex / opencode 之一）|
| License | `{MIT}`（必须与 legacy 源码 license 兼容）|
| 初始分支 | `main` |

---

## Product Inventory

> 本计划中最重要的表。一旦批准，这些名字会成为 `products/<name>/` 目录路径，被所有下游文档引用。

| # | Product | 一行用途 | 估计 LoC | 来源（fact-ext §）| 状态 |
|---|---|---|---|---|---|
| 1 | `{syntax}` | IntentLang parser & AST | {1,200} | Bounded Contexts row 1 | scaffold-only |
| 2 | `{verifier}` | Z3-backed 语义验证 | {2,400} | Bounded Contexts row 2 | **slice（首切片）** |
| 3 | `{cli}` | `intent` 命令行入口 | {600} | Bounded Contexts row 3 | scaffold-only |
| 4 | `{ui}` | Tauri 桌面可视化 | {1,800} | Bounded Contexts row 4 | scaffold-only |

**对照硬规则的校验**：
- [ ] 每个名字客户能识别（不是 `core` / `utils` / `common`）
- [ ] 数量在 3-7 之间
- [ ] 没有万能的 "shared" product
- [ ] 如果 fact-extraction-report 存在，每个 Bounded Context 行都映射到正好一个 product（或显式合并并附理由）

---

## Legacy Source

| 字段 | 值 |
|---|---|
| 仓库 URL | `{https://github.com/popsicle-lab/intent-lang.git}` |
| Submodule 路径 | `legacy/{intent-lang}` |
| Pinned commit SHA | `{abcdef1234...}` |
| Pin 理由 | "init 当日 main 的 HEAD；不会自动追踪" |
| License | `{MIT}` —— 与新仓库兼容 |
| 多仓库？| 否（单 submodule）|

> 若多仓库：每个 submodule 占一行。

---

## First Migration Slice

**选中**：`{verifier}`

**理由**（引用 fact-extraction-report 章节）：
1. {高 churn（最近 6 个月 47 次 commit），见 Risk Hotspots row 2}
2. {持有 Z3 binding —— 杠杆最高的形式化方法表面}
3. {3 个反向依赖（cli、ui、syntax）—— 可控，不至于孤立}

**考虑过的替代**：

- **`{syntax}`**：风险更低（只读 AST），但 invariant 少 → 作为学习切片 IDD ROI 低
- **`{cli}`**：LoC 最少，但它是聚合器——它的 spec 大多是转调用，不能充分锻炼文档体系
- **`{ui}`**：每行代码的 invariant 太少（按来源讨论 §六，可视化 product 的 ROI = low）

**「首切片」具体意味着什么**：
- 推进期间，只有 `products/verifier/` 被填进非骨架内容
- 其它 product 仍有完整目录树，但每个文件都停在 `[TBD: needs archaeology]`，等轮到它们
- verifier 的完整周期（PRFC → PDR → PRODUCT.md → contracts.intent → `crates/verifier/` 的首批代码）成为 playbook

---

## Module Dependencies

| 模块 | 来源 | 用途 | 状态 |
|---|---|---|---|
| `intent-coder` | `/Users/narwal/Workspace/github/intent-coder` | 本模块——迁移工具箱（含 .intent + Z3 能力）| 必装 |
| `{your-writers}`（可选）| `{用户自带 PRD/RFC/ADR/PDR/intent writer 模块}` | 内容生成 —— 由用户挑选 | 可选 |

> intent-coder 本身**不内置** PRD/RFC/ADR/PDR/intent writer。如果用户要 writer，得自己加进来。
> 所有路径在批准前都已验证存在。

---

## Scaffolding Manifest

> 下一 state 即将创建的每个目录、每个文件的穷举清单。**清单之外的东西不允许创建。清单上的东西不允许跳过。**

### 仓库级

```
.gitattributes                              # 追加：*.intent linguist-language=Scala
.gitmodules                                 # 由 `git submodule add` 自动管理
CONTRIBUTING.md                             # IDD 工作流规则（人 + AI agent 共读）
```

### docs/（跨 product 层）

```
docs/
├── CHARTER.md                              # 从 doc-architecture-charter artifact 落地
├── invariants/
│   └── README.md                           # 空索引，[TBD: needs archaeology]
├── user-journeys/                          # v0.2 任务图：跨 product 旅程的全局层
│   ├── README.md                           # 空索引；首切片完成后再考虑填
│   └── .gitkeep
├── baseline/
│   └── .gitkeep
├── glossary.md                             # 骨架，空 term 表
└── PROJECT_CONTEXT.md                      # 骨架；后由 living-doc-author 填
```

### products/（每个 product 一份 4 件套，含 v0.2 任务图目录）

每个 inventory 中的 product：

```
products/<name>/
├── PRODUCT.md                              # 顶层索引（Tasks Catalog + Intents Catalog），[TBD] 占位
├── ARCHITECTURE.md                         # 技术活文档，[TBD] 占位
├── tasks/                                  # v0.2 任务图：5 个固定旅程阶段目录
│   ├── README.md                           # 索引骨架；由 prd-writer 首次填、living-doc-author 维护
│   ├── onboarding/.gitkeep                 # 首次接触到首次成功
│   ├── daily-ops/.gitkeep                  # 日常使用
│   ├── troubleshooting/.gitkeep            # 故障排查
│   ├── admin/.gitkeep                      # 管理类（配额 / 权限 / 审计）
│   └── lifecycle/.gitkeep                  # 终止 / 迁出 / 续费
├── intents/
│   ├── invariants.intent                   # 空 —— 只含头部注释
│   ├── contracts.intent                    # 空
│   └── acceptance.intent                   # 空（v0.2：单文件，所有 acceptance block 共住）
├── decisions/
│   ├── adr/.gitkeep
│   └── pdr/.gitkeep
└── proposals/
    ├── exploring/.gitkeep
    ├── proposed/.gitkeep
    ├── accepted/.gitkeep
    └── rejected/.gitkeep
```

> ⚠️ **5 个旅程阶段目录是固定的**——`onboarding` / `daily-ops` / `troubleshooting` /
> `admin` / `lifecycle`，缺一不可，也不允许第 6 个。详细规则见 prd-writer 的
> `references/task-organization.md`。

### migration/（一次性 bootstrap 辅助）

```
migration/
├── slices/.gitkeep
├── traceability.md                         # legacy 路径 → 新路径 + 责任 spec
└── progress.md                             # kanban：每个 product 的 not-started / in-progress / done
```

### legacy/（submodule —— 由 `git submodule add` 创建）

```
legacy/<legacy-name>/                       # 固定到 {commit-sha}
```

---

## 风险 & 未决问题

| 风险 / 问题 | 缓解 / 负责人 |
|---|---|
| Product 命名在干系人评审后可能需要修订 | 重跑 project-init；便宜 |
| Submodule URL 可能在 CI 里不可达 | 在 CONTRIBUTING.md 中说明；必要时降级为 vendoring |
| 某些 bounded context 合并成一个 product（或拆分）| 在本计划里解决，**不要**等铺骨架之后 |

---

## Plan Checklist

- [ ] Repository Identity 表全填满（无占位符）
- [ ] Product Inventory 有 3-7 条，每条客户能识别
- [ ] 每个 product 条目有状态（slice / scaffold-only）和一行用途
- [ ] 正好一个 product 标 **slice**
- [ ] Legacy Source 章节有 URL + pinned SHA + license 检查
- [ ] First Migration Slice 章节列了 ≥1 个替代并说明否决理由
- [ ] Module Dependencies —— 每条路径都已验证存在
- [ ] Scaffolding Manifest 已穷举（下一 state 要创建的每个文件都列了）
- [ ] **v0.2 任务图：每个 product 的 Scaffolding Manifest 含 5 个旅程阶段目录
      （onboarding / daily-ops / troubleshooting / admin / lifecycle）**
- [ ] **v0.2 任务图：`docs/user-journeys/` 全局层已铺**
- [ ] 风险 & 未决问题 —— 每个已知顾虑都已记录
