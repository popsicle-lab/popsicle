# Popsicle

> **Intent-Driven Development（IDD）** 的 spec-and-verify 运行时——把产品任务图、形式化 intent、
> pipeline 阶段文档与 Rust 实现绑在一起，让人和 AI agent 按同一套规则协作。

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](Cargo.toml)
[![Repository](https://img.shields.io/badge/github-popsicle--lab%2Fpopsicle-181717)](https://github.com/popsicle-lab/popsicle)

---

## 这是什么

Popsicle 是 [intent-coder](intent-coder/README.md) 的**私有引擎**（[D4 决策](legacy/popsicle/intent-coder/ROADMAP.md)）：
它不是通用工作流平台，而是为 IDD 迁移与交付定制的 **Issue → Pipeline → Document → Intent 验证** 闭环。

典型用法：

1. 在 `products/<product>/` 下维护 **任务图**（`tasks/`）、**形式化契约**（`intents/*.intent`）与 **活文档**（`PRODUCT.md` / `ARCHITECTURE.md`）。
2. 用 `popsicle issue start` 启动 pipeline run，按阶段产出 artifact 文档。
3. 用 `intent-validate`（Z3）校验 intent 与实现是否一致。
4. 可选：用 **Tauri 桌面 UI** 浏览 Issue、Pipeline、文档与产品内容。

本仓库同时是 **popsicle 自身的 dogfood 场**：用 intent-coder 技能包，以 Strangler Fig 方式把 legacy 单体逐步迁到新的 crate 布局。

---

## 分支说明

本仓库是开源项目 [`popsicle-lab/popsicle`](https://github.com/popsicle-lab/popsicle)。**两条主线分支用途不同**：

| 分支 | 定位 | 适合谁 |
|---|---|---|
| **`main`**（默认）| IDD 迁移后的**新架构**：`crates/{skill-runtime,artifact-system,cli-ux,storage}` + 自托管 CLI MVP + 可选 Tauri UI | 新用户、贡献者、IDD 工作流 |
| **`backup-v0.5`** | 迁移前的 **legacy 全量 popsicle**（单体 `popsicle-core`、完整命令面、旧存储模型） | 需要对照 legacy 行为、跑 golden 对账、查阅历史实现 |

```bash
# 新架构（推荐）
git checkout main
make check && scripts/install.sh

# Legacy 对照
git checkout backup-v0.5
# 按该分支自带 README / Makefile 构建
```

`main` 通过 `legacy/popsicle/` **git submodule**（跟踪 `backup-v0.5` 分支，见 [`LEGACY_PIN.md`](LEGACY_PIN.md)）保留 legacy 全量源码，供 fact-extractor 与 equivalence baseline 对账。

---

## 与 Legacy Popsicle（`backup-v0.5`）的设计差异

`main` 不是旧版的小修小补，而是按 IDD 迁移目标**重划边界**。若你熟悉 legacy 单体（`popsicle-core` + SQLite `.popsicle/popsicle.db`），下面几条是行为与心智模型上最重要的差别。

### 审批工作流与文档 Guard 分离

| | Legacy（`backup-v0.5`） | 新架构（`main`） |
|---|---|---|
| **阶段能否推进** | 强依赖 **文档 Guard**：`has_sections` / `checklist_complete` / `upstream_approved` 等必须在**当前 artifact** 上即时通过，状态机与文档内容绑在一起 | **Pipeline 审批**（`requires_approval` + `stage complete [--confirm]`）与 **文档质量检查**（`doc check`）分开；审批模式可在 `.popsicle/project.yaml` 配置（人工 / 全自动 / 危险阶段代批） |
| **谁拦你** | Guard 失败 → stage 无法 complete，改文档是唯一出路 | 先按工作流推进阶段；文档占位符、空 checkbox 由 `doc check` 在提交前拦截，不与审批闸混为一谈 |

Legacy 里「能不能过这一关」≈「这篇 Markdown 现在够不够格」。新架构里「能不能过这一关」≈「工作流策略是否允许推进」；文档是否达标是另一条检查线。

### 文档固化在仓库，而不只躺在 `.popsicle/artifacts/`

| | Legacy | 新架构 |
|---|---|---|
| **长期真相源** | 大量内容在 `.popsicle/artifacts/<run_id>/` 与 DB 行中，与 Git 仓库弱绑定 | **`products/<product>/`** 下的活文档（`PRODUCT.md`、`ARCHITECTURE.md`、`tasks/`、`intents/`、`decisions/`）是产品边界内的**持久真相** |
| **Run artifact** | 往往是主交付物 | **阶段工作草稿**（debate、PRD 草案等），经 living-docs / 人工整理后**晋升**进 `products/` |
| **灵活性** | 改 spec 常意味着改 DB/artifact 路径 | 同一 Git 仓库可挂多个 product 目录；每个 product 自洽维护自己的任务图与 intent，不共用一套 checklist 模板 |

`.popsicle/` 仍保存 Issue、Run、阶段 session 与**过程性** artifact；但「产品是什么、任务怎么串、intent 约束什么」以 **`products/` + Git** 为准，而不是以某次 run 的临时文件为准。

### 验证节奏：先实现工作流，再对账

Legacy 倾向在**每次 stage complete 时**用 Guard 把文档扣死。新架构采用 **implement → reconcile** 两段式：

1. **交付环**：按 pipeline 产出 stage 文档、完成实现（`shadow-implementer` 等），`doc check` 保证 artifact 无占位符。
2. **对账环**：`equivalence-baseline` / `intent-validate` / `living-doc-author` 把实现、intent 与 `products/` 活文档**再次对齐**——差异在切片结束时收敛，而不是在每个 Guard 上卡死。

这与 Strangler Fig 迁移一致：先让新运行时跑通闭环，再用 golden + Z3 + living-docs 证明与 legacy 语义等价或有意偏离（ADR 记录）。

### 路线：同一仓库、多种产品模型（规划中）

当前每个 product 已按 IDD **4 件套**组织（`PRODUCT` / `ARCHITECTURE` / `intents` / `decisions` + `tasks/`）。**规划中**进一步支持：

- **一仓多 product**，且各 product 可选用**不同的产品框架**（不限于同一套 PRD/任务模板）——例如用户旅程地图、能力地图、纯技术 RFC 链等。
- Product 元数据（框架类型、目录约定）将挂在 product 级配置，由 Products 浏览器与 pipeline 路由识别；**不**强迫所有 product 共用 legacy 那套「debate → PRD → arch-debate → RFC」单一路径。

若你正在从 legacy 迁移，建议：用 `backup-v0.5` + golden 对账理解旧行为，在 `main` 上把**活文档写进 `products/`**、把**过程文档留在 artifact**、把**审批与 Guard 分开想**。

---

## 架构一览

```mermaid
flowchart TB
  subgraph docs [IDD 文档层 products/]
    Tasks["tasks/**/*.md"]
    Intents["intents/*.intent"]
    LiveDocs["PRODUCT.md / ARCHITECTURE.md"]
    Decisions["decisions/{pdr,adr}/"]
  end

  subgraph runtime [Rust 运行时 crates/]
    SR["skill-runtime<br/>Pipeline · Skill · Run"]
    AS["artifact-system<br/>Document · Guard · Context"]
    ST["storage<br/>SQLite / TSV"]
    CU["cli-ux<br/>popsicle CLI + Tauri UI"]
  end

  subgraph orchestration [编排]
    IC["intent-coder 技能包"]
    PopsicleCLI["popsicle issue / pipeline / doc"]
  end

  docs --> PopsicleCLI
  IC --> PopsicleCLI
  PopsicleCLI --> CU
  CU --> SR
  CU --> AS
  CU --> ST
  Intents -->|"Z3 intent-validate"| PopsicleCLI
```

### Crate 与职责

| Crate | 职责 | 状态 |
|---|---|---|
| [`skill-runtime`](crates/skill-runtime/) | Skill 状态机、Pipeline DAG、Run/Spec、Hook、Tool/Memory 注册表 | cutover-done |
| [`artifact-system`](crates/artifact-system/) | Document 实体、Markdown guard、Context 装配、chunk 抽取 | cutover-done |
| [`storage`](crates/storage/) | 工作区持久化（SQLite `.popsicle/self-host/state.db`，兼容 legacy TSV） | 使用中 |
| [`cli-ux`](crates/cli-ux/) | `popsicle` 二进制、自托管 domain、Tauri IPC（feature `ui`） | cutover-done · dogfood-usable |

### Product 域（`products/`）

每个 product 是 IDD 文档边界（4 件套：`PRODUCT.md` / `ARCHITECTURE.md` / `intents/` / `decisions/`），**不等于**一个 Rust crate。

| Product | 说明 | 迁移状态 |
|---|---|---|
| [`skill-runtime`](products/skill-runtime/) | Pipeline/Skill 引擎灵魂 | lib 已切流 |
| [`artifact-system`](products/artifact-system/) | 文档与 guard 引擎 | 已切流 |
| [`cli-ux`](products/cli-ux/) | CLI + 桌面 UI 用户面 | 已切流（ADR-008~015）|

迁移看板：[`migration/progress.md`](migration/progress.md)

---

## 功能特性（`main` 分支）

### CLI（自托管 MVP）

| 命令族 | 能力 |
|---|---|
| `popsicle init` / `doctor` | 初始化 `.popsicle/` 工作区；校验二进制与工作区来源 |
| `popsicle issue` | create / list / show / start / **close** |
| `popsicle pipeline` | status / next / stage complete（`requires_approval` 阶段按 `project.yaml` 审批模式处理）|
| `popsicle doc` | create / list / show / **check**（拒绝占位符与空 checkbox）|
| `popsicle tool run intent-validate` | 对 `products/` 跑 Z3 intent 校验 |
| `popsicle admin` | migrate（TSV→SQLite）/ reinit |

- 全命令支持 `--format json`；错误同样 JSON 化并带可执行 `next` 字段。
- 完整命令面与 agent 工作流：[`AGENTS.md`](AGENTS.md)
- **Deferred**（调用返回结构化错误，不在 help 宣传）：`module` / `skill` / `spec` / `namespace` / `prompt` / `git` / `memory` / `context` / `registry` / `completions`
- **Removed**：`checklist` / `item` / `sync`

### 桌面 UI（`popsicle ui`，ADR-015）

可选 Cargo feature `ui`（Tauri 2）。IPC **直连** `LocalWorkspace`，不 subprocess CLI、不读 legacy `.popsicle/popsicle.db`。

| 页面 | 能力 |
|---|---|
| **Issues** | Issue 列表与详情、启动 run、文档列表、**工作流引导**（spec→product 推荐 task/intent）|
| **Pipeline** | 当前 run 阶段 DAG、stage complete |
| **Documents** | Stage artifact Markdown 预览 |
| **Products** | **Tasks**（全文 Markdown + frontmatter）· **Intents**（`.intent` 块级正文）· **Graph**（Task 关系图 + Intent Mermaid）|

UI 当前为**只读浏览**；编辑 task/intent 仍通过仓库与 pipeline 完成。

---

## 仓库结构

```
popsicle/
├── crates/                 # Rust 运行时（按 slice 拆分）
│   ├── skill-runtime/
│   ├── artifact-system/
│   ├── storage/
│   └── cli-ux/             # popsicle 二进制 + ui/ Tauri 桥
├── products/               # IDD 产品文档（任务图 + intent + 决策）
├── docs/                   # 仓库级 charter、baseline、invariants
├── intent-coder/           # IDD 编排技能包（10+ skills、pipeline 模板）
├── vender/intent-lang/     # intent DSL 源码镜像（AI/文档用；运行时用独立安装的 intent v0.1.1+）
├── ui/                     # Tauri 前端（Vite + React 19）
├── migration/              # Strangler Fig 进度与 traceability
├── legacy/popsicle/        # legacy 源码 submodule（对账用）
├── .popsicle/              # 本仓库自托管工作区（init 后生成）
├── AGENTS.md               # AI agent 命令指南（自动生成目标）
├── CONTRIBUTING.md         # 贡献与 IDD 硬约束
└── Makefile                # check / golden / intent / build-ui
```

文档体系铁律：[`docs/CHARTER.md`](docs/CHARTER.md)（活文档无版本号、决策只追加、编辑须引用 Decision ID）。

---

## 快速开始

### 前置要求

- Rust stable（`cargo` / `rustc`）
- 构建 UI 时另需 Node.js 18+

### macOS 安装（DMG，推荐）

1. 从 [GitHub Releases](https://github.com/popsicle-lab/popsicle/releases) 下载 `Popsicle_*_aarch64.dmg` 或 `x86_64.dmg`。
2. 挂载 DMG，将 **Popsicle.app** 拖入 **Applications**，从 Applications **首次打开** Popsicle（会静默把 CLI 安装到 `~/.local/bin` 并创建 `~/.popsicle/`）。
3. 重启终端（或 `source ~/.zshrc`），运行 `popsicle doctor --format json`。
4. *(可选)* 若暂不打开 App，可双击 DMG 内的 **Install CLI.command** 手动安装 CLI。

未签名 DMG 首次打开需在系统设置中放行。详见 [`packaging/macos/README.md`](packaging/macos/README.md)。

### 从源码安装 CLI（开发者）

```bash
git clone https://github.com/popsicle-lab/popsicle.git
cd popsicle
git checkout main

make check                    # fmt + clippy + test（CI 同等）
scripts/install.sh            # 安装 popsicle 到 ~/.cargo/bin

popsicle init
popsicle doctor --format json # current_workspace_binary_match 应为 true
```

本地打 DMG：`make build-dmg`（仅 macOS）。

### 跑一次 IDD 工作流

```bash
popsicle issue list --format json

popsicle issue create --type technical --title "My change" --product cli-ux \
  --description "What and why" --format json

popsicle issue start PROJ-NN --format json    # 返回 run_id

popsicle pipeline next --run <run_id> --format json
popsicle doc create fact-extractor --title "Facts" --run <run_id>
popsicle doc check <doc_id>
popsicle pipeline stage complete <stage> --run <run_id>

popsicle tool run intent-validate path=products
```

Bundled pipeline 模板：`greenfield-product-spec` · `slice-spec` · `slice-delivery` · `tech-decision` · `bugfix` · `migration-bootstrap`（详见 `AGENTS.md`）。

### 多项目（全局 CLI）

注册表位于 `~/.popsicle/global.json`（可用 `POPSICLE_HOME` 覆盖目录）。

```bash
cd ~/project-a && popsicle init
cd ~/project-b && popsicle init

popsicle project add ~/project-a --name a
popsicle project add ~/project-b --name b
popsicle project use a              # 设置默认项目
popsicle issue list                 # 操作 project-a

popsicle issue list --project ~/project-b   # 单次覆盖
export POPSICLE_PROJECT=~/project-b         # 或环境变量
popsicle project list
popsicle project current
```

工作区解析优先级：`--project` → `POPSICLE_PROJECT` → 默认项目 → 当前目录向上扫描。

### 桌面 UI

```bash
make build-ui                 # npm build + cargo build --features ui
./target/debug/popsicle ui [--project <workspace-path>]

# 开发热更新
make ui-dev                   # 另开终端运行 popsicle ui
```

---

## DevOps

| 命令 | 用途 |
|---|---|
| `make check` | fmt + clippy + test（`-Dwarnings`，CI 主路径）|
| `make golden` | golden-baseline 全链（legacy vs new 对账）|
| `make intent` | Z3 intent 校验 |
| `make build-ui` / `make ui-dev` | Tauri UI 构建 / 开发 |
| `make build-dmg` | macOS DMG（App + CLI + 安装脚本）|
| `make install-hooks` | pre-commit（fmt/clippy/test）|

发布：推送 `v*` tag → [`.github/workflows/release.yml`](.github/workflows/release.yml) 构建 CLI 包（4 目标）+ macOS DMG（aarch64 / x86_64）。

---

## Roadmap（UI / 工作流 / 大型项目适配）

按**优先级**推进；每项完成后在表中勾选。设计原则：Epic 语义在 **task 图**，Issue 管**一次交付 run**；重迁移链与轻日常链并存。

| 优先级 | 主题 | 目标 | 状态 |
|:---:|---|---|:---:|
| P1 | **工作流画像 `workflow_profile`** | `project.yaml` 增加 `daily-dev` / `migration` / `pm-spec-only` / `opc-full`；Issue 创建向导与 Settings 随画像切换默认 pipeline / 审批模式 | ☑ |
| P2 | **Product 健康仪表盘** | Products 页展示 living-doc 四类信号（过期、断链、孤儿、未验证）+ 旅程阶段覆盖 | ☑ |
| P3 | **Issue 分组 / Epic 视图** | 按 `product` / `pipeline` 聚合列表；列表行展示 `epic_task_id` | ☑ |
| P4 | **Retro doc 闭环** | 增量合并后的标准 checklist；无 pipeline Issue 显示 UI 横幅 + `intent-coder/guides/retro-doc-checklist.md` | ☑ |
| P5 | **文档内嵌图表** | task Markdown 中 ` ```mermaid ` 块渲染（`MarkdownWithMermaid` 组件） | ☑ |
| P6 | **Issue ↔ Epic 关联** | 可选 `epic_task_id` 字段（SQLite/TSV）；创建 Issue 时可绑定 task | ☑ |

### 两条用户旅程（不变）

| 场景 | 路径 | 文档义务 |
|---|---|---|
| 大型迁移 / 新 slice | `slice-spec` → `slice-delivery` + intent gate | 五件套 + 全 task 图 |
| 日常 bug / 小增强 | `bugfix` 或已有 spec 的 `slice-delivery` | 最小 PDR + 单 task + intent block |
| 已合并增量（retro） | **不跑 pipeline**；直接写 `products/` + `living-doc-author` | 见 P4 checklist |

详见 [`intent-coder/skills/issue-author/guide.md`](intent-coder/skills/issue-author/guide.md)。

---

## 相关项目

| 项目 | 关系 |
|---|---|
| [intent-coder](intent-coder/README.md) | IDD 编排技能包：fact-extract → debate → PRD → RFC/ADR → intent → living-doc |
| [intent-lang](https://github.com/popsicle-lab/intent-lang) | `.intent` DSL 与 Z3 验证（独立安装 / DMG 捆绑；`vender/` 仅为文档镜像）|
| Legacy popsicle（`backup-v0.5` / submodule） | 迁移前单体实现，equivalence 对账基准 |

---

## 贡献

1. 读 [`CONTRIBUTING.md`](CONTRIBUTING.md) 与 [`docs/CHARTER.md`](docs/CHARTER.md)
2. AI agent 额外读 [`AGENTS.md`](AGENTS.md)（须先 `issue start` 再写代码）
3. 改 `products/*/intents/*.intent` 须附 `intent-validate` 通过证据
4. 切流硬门禁：Z3 PASS · ≥5 golden · 切流 ADR Accepted（见 `migration/progress.md`）

---

## 许可证

- 本仓库 workspace crate：**MIT**（见各 crate `Cargo.toml`）
- `legacy/popsicle/` submodule：**Apache-2.0**

---

## 进一步阅读

| 文档 | 内容 |
|---|---|
| [`migration/progress.md`](migration/progress.md) | 切片迁移看板 |
| [`products/cli-ux/PRODUCT.md`](products/cli-ux/PRODUCT.md) | CLI/UI 用户面规格 |
| [`products/cli-ux/decisions/adr/ADR-015-tauri-ui-self-host-bridge.md`](products/cli-ux/decisions/adr/ADR-015-tauri-ui-self-host-bridge.md) | 桌面 UI 架构决策 |
| [`docs/glossary.md`](docs/glossary.md) | IDD / slice / Z3 闸等术语 |
| [`LEGACY_PIN.md`](LEGACY_PIN.md) | Legacy submodule pin 与已知限制 |
