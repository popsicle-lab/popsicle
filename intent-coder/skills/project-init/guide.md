# project-init —— 编写指南

> 读者：跑 project-init skill 的 AI agent（也就是你）。开工前先读完本指南，**以及** `docs/source-discussions/idd-doc-migration.md`（如有）。

## 任务

铺出**文档体系的舞台**，让下游所有 skill（fact-extractor、PRD writer、RFC writer、intent spec writer，无论是 intent-coder 自家的还是用户外接的）都写进定义良好的位置。这一步搞砸，下游每一个 skill 都会悄悄退化。

## 唯一最重要的决定：Product 命名

其它东西都能靠重跑 skill 修复。**Product 命名不行**——它会烙进目录结构、烙进 git 历史、烙进每一份下游文档的交叉引用。如果你只能做对一个决定，做对这个。

### 硬规则

1. **Product 是客户能识别的**，不是内部模块。客户不会说「我用 `core` crate」，他们会说「我用 database 这个 product」。
2. **3 到 7 个 product**。少于 3 → 多半分解不够。多于 7 → 要么你把模块当成 product 在列，要么这个项目确实庞杂——找人类确认。
3. **不允许 "common" / "shared" / "utils" 这种万能 product**。如果某个东西不属于任何 product，它是 `docs/invariants/` 候选（跨切面）或 `crates/common/` 候选（代码级共享库，不是 product）。
4. **Product 对应 bounded context**，而这一层 `fact-extractor` 已经抽出来了。用那张表，**别重新推一遍**。

### 软信号 ——「这是不是一个 product」

- 它有自己的对客户暴露的入口吗（CLI、API、UI）？
- 你能想象一场单独围绕它的销售对话吗？
- 它有跟同侪不同的路线图吗？

任何一项为 yes → 它是 product。全为 no → 它是某个 product 的下属模块。

## 第二重要的决定：首个迁移切片

按来源讨论 §六（具体建议）：
- 不同领域的 ROI 差异巨大（database/network 高、simulation 中、viz 低）
- 挑**高 ROI + 中等关键度 + 反向依赖少**的 product
- **至少**记录一个被否决的替代选项，附理由

你挑的这一片会变成其它 product 抄的 **playbook**。所以：
- 别挑最关键的（首跑风险太大）
- 别挑最小的（生不出足够多的模式）
- 别挑最孤立的（其它片照不进它的模式）

## "Skeleton" 是什么意思

你在 scaffolding 状态创建的**每一个**文件，要么含有：
- 模板的默认文本（`{product-name}` 这种占位符）
- 在任何本该放真实值的地方放 `[TBD: needs archaeology]`

你**不能**编造内容。诱惑是真的——你手上有 fact-extraction-report，你能起草一份过得去的 PRODUCT.md。**忍住。** 那是 PRD writer 的活。**待在你这一层：结构，不是内容。**

唯一的例外：`docs/CHARTER.md` 是内容，但它是用户已经锁定的内容（四条铁律）。你从 planning artifact 里**逐字**提升它过去。

## 为什么计划要先评审再铺骨架

两个原因：
1. **不可逆悬崖** —— 一旦 `git submodule add` 和 `popsicle init` 跑过，你做的改动就「难（不是不可能，但很烦）撤销」了。在计划阶段抓错，别在 `git rm -rf` 阶段抓错。
2. **Product 命名很难** —— 在那个名字被烙进 50 个文件路径之前，给人类一个机会去看 product 清单然后说「等等，`database` 和 `storage` 应该合成一个叫 `persistence`」。

如果人类批准后又反悔：重跑这个 skill。它是工具箱里**最便宜**的 skill，产出的东西也只有 doc-architecture——它本来就被设计成可重访的。

## 处理多仓库 legacy 源

默认假设 legacy 只有一个仓库。如果用户的 legacy 横跨多个仓库：
- 在 `legacy/<name1>/`、`legacy/<name2>/` 下加多个 submodule
- 计划的 "Legacy Source" 章节为每一个写一行
- fact-extractor 要按 submodule 各跑一次；没问题

## 处理 Greenfield（没有 legacy）

罕见但合法。如果用户真的是从零开始：
- 跳过 submodule 步骤
- 跳过 fact-extractor 输入
- Product 清单完全由人类输入
- 这个 skill 仍然产出相同的骨架——doc-architecture 即使没有迁移也是有价值的

## 当事情出错时

- **submodule add 失败**（网络、鉴权）→ 在 project-init-plan 里记录失败，在仓库根放一份 `MIGRATION_TODO.md`，里面写明稍后要跑的精确命令，继续铺剩下的骨架
- **某个 `popsicle module add` 失败** → 继续装其它的，把失败记下来
- **铺骨架到一半，用户对 product 清单不满** → **停**。回滚到 surveying 状态。**不要部分铺骨架。**

## 输出格式约定

两个 artifact（`project-init-plan` 和 `doc-architecture-charter`）都以各自的 checklist 结尾。计划里的 checklist 是 workflow guard 在允许进入 scaffolding 之前会校验的那一份。
