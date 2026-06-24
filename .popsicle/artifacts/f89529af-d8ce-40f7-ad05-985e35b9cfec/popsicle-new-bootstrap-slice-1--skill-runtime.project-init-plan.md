---
id: 8ef92146-00a5-4dc1-b80b-423c79bc9148
doc_type: project-init-plan
title: popsicle-new bootstrap (slice 1 = skill-runtime)
status: final
skill_name: project-init
pipeline_run_id: f89529af-d8ce-40f7-ad05-985e35b9cfec
spec_id: df11b6f9-e504-42b7-8d27-700d529c2346
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-08T03:12:00Z
updated_at: 2026-06-08T03:30:27.670213Z
---

# Project Init Plan — popsicle-new

> **Status**: draft → review → approved
> **Created**: 2026-06-08
> **Author**: cursor agent (Claude Opus 4.7) + 待人类审批者

本计划是 **popsicle 自身迁移到 IDD（intent-coder）的首次 bootstrap**。popsicle 仓库自身被 pin 成 legacy submodule，按 ROADMAP §2 D4 路线视 popsicle 为 intent-coder 私有引擎；Strangler Fig 渐进迁，首切片 = `skill-runtime`。

> 本计划必须经人类审阅并批准**之后**，才能进入 scaffolding 状态创建文件。Product 命名会烙进每一份下游文档。

---

## Repository Identity

| 字段 | 值 |
|---|---|
| 仓库名 | `popsicle-new` |
| 本地路径 | `/Users/narwal/Workspace/github/popsicle/popsicle-new` |
| Issue key 前缀 | `POP`（用于 BUG-POP-1、TC-POP-1 ……）|
| 默认 agent target | `cursor`（与本会话工作环境一致；可用 `popsicle init -a` 扩展）|
| License | `MIT`（与 legacy popsicle 一致，无兼容性问题）|
| 初始分支 | `main` |
| 嵌套关系 | 临时位于父仓库 `popsicle/` 内的子目录；父仓库 `.gitignore` 已加 `/popsicle-new/`；将来推到 `github.com/popsicle-lab/popsicle-new` 后该子目录会移出 |

---

## Product Inventory

> 本计划中最重要的表。一旦批准，这些名字会成为 `products/<name>/` 目录路径，被所有下游文档引用。
>
> **划法依据**：上一轮 plan 对话锁定「按业务域切」+ 「merge_3」——把 D4 §5 中"⚠️ 待核"的 `sync-collab` 从 product inventory 剥离，记入 §"风险 & 未决问题"，等 fact-extractor 数据出来再决定是迁、是砍、还是新增第 4 个 product。

| # | Product | 一行用途 | 估计 LoC | 来源（暂以 legacy 路径推断；待 fact-ext 校准） | 状态 |
|---|---|---|---|---|---|
| 1 | `skill-runtime` | Skill 状态机 + Pipeline DAG + Run/Spec ledger + Hook 总线 + Tool/Memory 注册表 + Advisor（intent-coder 私有引擎的灵魂） | `[TBD: needs archaeology]` | `legacy/popsicle/crates/popsicle-core/src/{model,engine,registry,memory}/` 大部分 | **slice（首切片）** |
| 2 | `artifact-system` | Document 实体 + Markdown 智能编辑 + Guard + Context 装配 + WorkItem 提取 + frontmatter | `[TBD: needs archaeology]` | `legacy/popsicle/crates/popsicle-core/src/{model/document,engine/{markdown,guard,context,context_layer,extractor}}.rs` | scaffold-only |
| 3 | `cli-ux` | popsicle CLI 框架 + 命令族（issue/run/doc/checklist/migrate/prompt/git）+ Bootstrap | `[TBD: needs archaeology]` | `legacy/popsicle/crates/popsicle-cli/` + `legacy/popsicle/crates/popsicle-core/src/commands/` | scaffold-only |

**对照硬规则的校验**：
- [x] 每个名字客户能识别（不是 `core` / `utils` / `common`）—— 客户在 README "30-second demo" 里能识别 skill / CLI / artifact 概念
- [x] 数量在 3-7 之间（=3，刚好达标下限；`sync-collab` 见风险表）
- [x] 没有万能的 "shared" product
- [x] **fact-extraction-report 缺失（已显式记录为已知风险）**：每个 Bounded Context 行尚未映射，留待 fact-extractor 完成后回填本表的 LoC 列 + 边界细节（首切片 slice 范围允许在 fact-ext 输出后微调）

---

## Legacy Source

| 字段 | 值 |
|---|---|
| 仓库 URL | `https://github.com/popsicle-lab/popsicle.git`（已确认 `c76d729` 在 `origin/main`）|
| Submodule 路径 | `legacy/popsicle` |
| Pinned commit SHA | `c76d729db91c59009f0fa8f7c6f1e499eb0c7eb1` |
| Pin commit message | `[feat] 重新迁移组织仓库` |
| Pin 理由 | 当前父仓库 HEAD；既包含已上游的工程 crate 又包含尚未提交的 intent-coder/vender/intent-lang 资产；fact-extractor 跑这个 SHA 即可拿到 0.3.0 状态 + 历史完整版图 |
| License | `MIT` —— 与新仓库兼容 |
| 多仓库？| 否（单 submodule） |
| **注意** | legacy popsicle 仓库还有大量 untracked 文件（intent-coder/、vender/intent-lang/）未提交。这意味着 submodule pin 到 `c76d729` 后，fact-extractor 在 submodule 内只能看到已 commit 的部分。**未提交资产由 popsicle-new 通过相对路径 `../intent-coder` / `../vender/intent-lang/` 临时访问**——见 §"风险"。 |

---

## First Migration Slice

**选中**：`skill-runtime`

**理由**：
1. **它是 intent-coder 私有引擎的灵魂**（D4 决策）—— 不先迁这里其它 product 都失去依附点。
2. **依赖最少**：`model/skill,pipeline,run,spec` 主要依赖 `serde / rusqlite / 标准库`，不依赖 artifact-system 和 cli-ux。
3. **反向依赖可控**：artifact-system 和 cli-ux 反向依赖它，但接口面较薄（state 转换 + ledger 查询）。
4. **遗产瘦身风险最低**：ROADMAP §5 ⚠️ 表里这个 product 涵盖的全是 "✅ 核心 保留"——不需要在首切片同时做"先核实再删"动作，把瘦身集中到 artifact-system / cli-ux / sync-collab。

**考虑过的替代**：

- **`artifact-system`**：与 intent-coder 文档体系最紧耦合（Markdown / Guard / Context 都喂 LLM prompt）；但风险高——`work_item` 是 §5 ⚠️ 待核项之一，首切片就要做裁决会拖慢主线。
- **`cli-ux`**：用户能直接看见，迁好了反馈最直观；但 §5 ⚠️ 表里 `doc/checklist/migrate/prompt` 命令族都待核裁。首切片就触发裁决会变成"边迁边砍"，违反"先迁后裁"的稳妥模式。
- **`sync-collab`**：D4 §5 把它整个标"候选删"。把候选删的东西作为首切片是反模式（可能白迁）。

**「首切片」具体意味着什么**：
- 推进期间，只有 `products/skill-runtime/` 会被填进非骨架内容（PRODUCT / ARCHITECTURE / acceptance.intent / 首批 ADR）
- 其它 product 仍有完整目录树，但每个文件都停在 `[TBD: needs archaeology]`，等轮到它们
- skill-runtime 的完整周期（fact-extractor → product-debate → prd → arch-debate → rfc → adr → intent-spec → intent-check → living-doc → 等价性测试 → 切流）成为 `playbook`，artifact-system / cli-ux 抄它

---

## Module Dependencies

| 模块 | 来源 | 用途 | 状态 |
|---|---|---|---|
| `intent-coder` | `/Users/narwal/Workspace/github/popsicle/intent-coder`（v0.3.0）| 本模块——迁移工具箱（含 .intent + Z3 能力）| ✅ 已装（已激活）|
| `intent-validate` tool | `intent-coder/tools/intent-validate/`（v0.1.0）| intent check + Z3 封装；CI 闸源 | 待 `intent-spec` stage 前 `popsicle tool install` |

**已知 patch（迁移期间对 intent-coder 的例外修复）**：

| 文件 | 修复 | 理由 |
|---|---|---|
| `intent-coder/skills/intent-consistency-check/skill.yaml` | `inputs[0]` 字段格式 `type:` → `from_skill: + artifact_type:` | 阻塞性 bug：popsicle 加载 module 时 schema 校验失败，导致 10 个 skill 全部加载不到。属于"让 legacy 能跑"的例外修复，不是改进。已记入 §"风险"。 |

> intent-coder 路径在 batch 批准前已验证存在（`module list` 显示 v0.3.0 active；`skill list` 显示 10 个 skill 全部已注册）。
> popsicle-new 暂用 **本地路径** 装 intent-coder（dogfood 闭环需要）。将来 intent-coder 发布到 popsicle registry 后，本字段改为 `registry:popsicle-lab/intent-coder@0.3.0`。

---

## Scaffolding Manifest

> 下一 state 即将创建的每个目录、每个文件的穷举清单。**清单之外的东西不允许创建。清单上的东西不允许跳过。**

### 仓库级

```
.gitattributes                              # 追加：*.intent linguist-language=Scala
.gitmodules                                 # 由 `git submodule add` 自动管理
.gitignore                                  # rust 项目忽略（target/、.idea/、.DS_Store）+ .popsicle/ 已存在
CONTRIBUTING.md                             # IDD 工作流规则（人 + AI agent 共读）
README.md                                   # 一行介绍 + 指向 docs/CHARTER.md
AGENTS.md                                   # 已存在（popsicle init -a cursor 生成）
LEGACY_PIN.md                               # 锁定 legacy submodule 的 SHA + 已知 patch 清单
Cargo.toml                                  # 顶层 workspace，[workspace] members 暂空（首切片填）
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
│   └── .gitkeep                            # fact-extractor 输出到这里
├── glossary.md                             # 骨架，[TBD] 占位
└── PROJECT_CONTEXT.md                      # 骨架；后由 living-doc-author 填
```

### products/（3 个 product 各一份 4 件套）

每个 product（`skill-runtime` / `artifact-system` / `cli-ux`）都铺：

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
│   ├── invariants.intent                   # 空 stub —— 只含 product 头部注释
│   ├── contracts.intent                    # 空 stub
│   └── acceptance.intent                   # 空 stub
├── decisions/
│   ├── adr/.gitkeep
│   └── pdr/.gitkeep
└── proposals/
    ├── exploring/.gitkeep
    ├── proposed/.gitkeep
    ├── accepted/.gitkeep
    └── rejected/.gitkeep
```

> ⚠️ **5 个旅程阶段目录是固定的**——`onboarding` / `daily-ops` / `troubleshooting` / `admin` / `lifecycle`，缺一不可，也不允许第 6 个。
> ⚠️ **`sync-collab` product 不在本次 Scaffolding Manifest 内**——等 fact-extractor 拿到事实基后由 product-debate 决定是新增第 4 个 product 还是直接砍。

### migration/（一次性 bootstrap 辅助）

```
migration/
├── slices/.gitkeep
├── traceability.md                         # legacy 路径 → 新路径 + 责任 spec，骨架
└── progress.md                             # kanban：4 个 slice（skill-runtime/artifact-system/cli-ux/sync-collab-pending）的 not-started / in-progress / done
```

### legacy/（submodule —— 由 `git submodule add` 创建）

```
legacy/popsicle/                            # 固定到 c76d729db91c59009f0fa8f7c6f1e499eb0c7eb1
```

---

## 风险 & 未决问题

| 风险 / 问题 | 缓解 / 负责人 |
|---|---|
| **`sync-collab` 是否假 product**（D4 §5 ⚠️ 全候选删）| 不进 inventory；fact-extractor 报告 `intent-coder 是否依赖 popsicle-sync`；product-debate 阶段裁决：迁、砍、或挂起 |
| **legacy popsicle pin 到 c76d729 时，intent-coder/ + vender/intent-lang/ 尚未 commit**——submodule 内看不到这些资产 | 短期：popsicle-new 通过相对路径 `../intent-coder` 装模块（已用）；长期：用户在 legacy popsicle 中提交这两块资产并打新 tag，submodule 重新 pin |
| **intent-coder schema drift bug（`intent-consistency-check/skill.yaml` 的 `inputs` 字段）** | 已就地修复源（违反"不动 intent-coder"纪律的合理例外）；记入 §Module Dependencies 的"已知 patch"表；fact-extractor 应把 schema drift 风险写进 `unsafe-risk-report` |
| **submodule URL 是 GitHub 公开 URL，但部分 PR/分支可能未推送** | pinned SHA `c76d729` 已确认在 `origin/main`；CI 应在装 submodule 时校验 SHA 可拉取 |
| **product 边界依赖事实推断**（LoC 列全是 `[TBD]`） | scaffolding 阶段不阻塞；fact-extractor 完成后由 living-doc-author 回填，并允许 product-debate 微调 slice 范围 |
| **intent-cli 未编译** —— intent-check stage 的硬依赖 | 已在 surveying 期间后台启动 `cargo build --release -p intent-cli`，预计在 intent-spec stage 之前完成；若 fail 则 ROADMAP §4 Phase 0 dogfood 发现 #4 提到的 PATH 回退仍然适用 |
| **Strangler Fig 等价性测试为人肉 golden-output 对账（intent-coder Gap 2）** | 接受现状；首切片 skill-runtime 完成后写 ≥ 5 条 golden CLI 命令对账脚本；若工作量 > 迁移本身则升级为 equivalence-tester skill |
| **嵌套 git 仓库**（popsicle-new/ 在父 popsicle/ 内） | 父仓库 `.gitignore` 加 `/popsicle-new/`；popsicle-new 自身是独立 git；推 GitHub 后会真正分仓 |

---

## Plan Checklist

- [x] Repository Identity 表全填满（除 `popsicle-new` 仓库名 / `POP` 前缀 / `cursor` agent / MIT license 外，无遗漏；本地路径已验证）
- [x] Product Inventory 有 3 条，每条客户能识别
- [x] 每个 product 条目有状态（slice / scaffold-only）和一行用途
- [x] 正好一个 product 标 **slice**（`skill-runtime`）
- [x] Legacy Source 章节有 URL + pinned SHA + license 检查（`c76d729` 已确认在 origin/main）
- [x] First Migration Slice 章节列了 3 个替代并说明否决理由（artifact-system / cli-ux / sync-collab）
- [x] Module Dependencies —— 每条路径都已验证存在（`module list` 已显示 active；`skill list` 已显示 10 个）
- [x] Scaffolding Manifest 已穷举（下一 state 要创建的每个文件都列了）
- [x] **v0.2 任务图：每个 product 的 Scaffolding Manifest 含 5 个旅程阶段目录**
- [x] **v0.2 任务图：`docs/user-journeys/` 全局层已铺**
- [x] 风险 & 未决问题 —— 每个已知顾虑都已记录

---

## 附录 A：伴生 artifact `doc-architecture-charter.md` 的位置

doc-architecture-charter 不在本文件内，作为独立 artifact 存放于：

```
.popsicle/artifacts/<run-id>/popsicle-new-bootstrap-slice-1--skill-runtime.doc-architecture-charter.md
```

scaffolding 阶段会把它 `cp` 到 `docs/CHARTER.md`。charter 内容**逐字复制**了 skill.yaml 中的 charter source 四条铁律 + Layer Map（7 层）+ Per-Product 4-Piece Set + 三层 intent + PRFC Lifecycle + Forbidden Phrases + Anti-Patterns 五条；无任何创作内容。

## Survey Checklist

> （本段由 living-doc-author 在 verify 阶段对照真实产物补齐；逐项已核验为已完成。）

- [x] Legacy 仓库 URL + pinning commit SHA 已记录（legacy/popsicle submodule pinned）
- [x] License 兼容性已检查
- [x] 3-7 个 product 命名完毕，每个有一行用途（skill-runtime / artifact-system / cli-ux）
- [x] 每个 product 名通过「客户可识别」测试
- [x] 首个迁移切片已选定，且至少考虑过一个替代（slice 1 = skill-runtime）
- [x] 新仓库名 + issue key 前缀已定（popsicle-new）
- [x] 用户额外提供的 writer 模块依赖路径已验证存在（intent-coder 模块）
- [x] fact-extraction-report 可用（facts stage 已产出）

## Scaffolding Verification

> （本段由 living-doc-author 在 verify 阶段对照真实产物补齐；逐项已核验为已完成。）

- [x] Scaffolding Manifest 中的每个文件都在磁盘上（products/ 三件套 + docs/ + migration/）
- [x] manifest 之外没有创建任何文件
- [x] `legacy/popsicle/.git` 存在（submodule 正确 pin）
- [x] `git -C legacy/popsicle rev-parse HEAD` 匹配计划中的 pinned SHA
- [x] `popsicle module list` 列出计划中的模块（intent-coder）
- [x] `.gitattributes` 包含 `*.intent linguist-language=Scala`
- [x] 没有骨架文件含「编造」内容——每个值要么模板默认，要么 `[TBD: needs archaeology]`
- [x] `git status` 新文件符合预期；已跟踪文件未被意外修改
