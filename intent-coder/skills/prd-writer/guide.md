# prd-writer —— 编写指南（v0.2 任务图范式）

> 读者：跑 prd-writer skill 的 AI agent（也就是你）。开工前必读：本指南、
> `references/task-organization.md`、`references/quality-rubric.md`、
> `references/charter-compliance.md`，以及（如果存在）上游 `product-debate` 的
> 产出 `{slug}.prd-draft.md`、`docs/CHARTER.md`。

## 任务

把 `product-debate` 产出的 **prd-draft** 升级为 IDD 任务图五件套：

1. **PRD overview** —— `{slug}.prd.md`，本次 PRD 变更的清单 + PRODUCT.md 顶层增量
2. **N 份 task 文件** —— 每个用户意图一份独立 chunk，落到
   `products/<target_product>/tasks/{journey_stage}/T-XXXX-<slug>.md`
3. **tasks/README.md** —— task 目录索引（首次生成；后续由 living-doc-author 维护）
4. **acceptance.intent 种子** —— 单文件多 block，每个 block 与 task_id 双射
5. **PDR 骨架** —— 配套的 Product Decision Record，Status: Proposed

如果没有 prd-draft（用户绕过辩论直接来），先做任务图访谈，再产出五件套；在 PDR
中标注「未经多角色辩论」。

## 为什么是任务图而不是功能树（v0.2 核心范式转移）

旧 PRD 范式：

```
PRD
├── Functional Requirements
│   ├── Core Features (P0)
│   │   ├── Feature 1
│   │   └── Feature 2
│   └── Enhancement Features (P1)
└── Out of Scope
```

这是 30 年来的 PRD 范式，但在 AI 时代彻底崩溃。原因：

1. **AI 召回是 chunking 而不是阅读**。RAG 引擎把 PRD 切成小块给 LLM，「Feature
   1 描述」和「Feature 2 描述」会作为两个独立 chunk 召回——但用户问的不是
   「Feature 1 是什么」，而是「我想做 X 应该怎么办」。功能不是用户的问法。

2. **用户问 AI 的是 task 不是 feature**。「重置密码功能」是开发者视角；用户视
   角是「我手机端忘了密码，5 分钟内能不能重新登录」。这是一个完整 task，跨多
   个 feature（验证码服务 + 重置流程 + 登录态恢复）。

3. **功能列表会腐烂**。一个功能涉及多个 task；一个 task 涉及多个功能。功能-
   PRD 1:1 映射在产品早期还能维持，到 v3 就完全错位。

新范式（v0.2）：

```
products/<product>/
├── PRODUCT.md                # 顶层：概述 + Tasks Catalog + Intents Catalog
└── tasks/                    # 每个用户意图一份独立 chunk
    ├── README.md             # 索引 + 健康度统计
    ├── onboarding/           # 首次接触到首次成功
    ├── daily-ops/            # 日常使用
    ├── troubleshooting/      # 故障排查
    ├── admin/                # 管理类
    └── lifecycle/            # 终止/迁出/续费
```

**每个 task 就是一个独立 RAG chunk**——含 YAML frontmatter（audience / journey_stage /
prerequisites / limits 等可被 AI 精准过滤的元数据）、含 3-5 个用户原话 query 锚点、
含完整可独立回答的内容。

详细规则见 `references/task-organization.md`。

## 五件套的边界（严格遵守 IDD charter）

| Artifact | 内容 | 不准写什么 |
|---|---|---|
| **PRD overview** | 文件清单 / Catalog 索引 / PRODUCT.md 顶层增量 | 任何 task 正文 |
| **task 文件**（N 份）| 单个用户意图的完整可独立召回 chunk | 决策理由（进 PDR）、跨 task 内容（进 PRODUCT.md 顶层）、实现细节 |
| **tasks/README.md** | task 列表索引 + 健康度统计 | task 正文 |
| **acceptance.intent 种子** | 可形式化的硬约束（within / never / forall...）| 自然语言 / 商业策略 |
| **PDR 骨架** | 为什么 + Consequences（精确到文件级）| 「现在状态」描述（进 PRD overview 或 task）|

**判定准则**（决定一句话该放哪里）：

- 「这句话在 N 个月后会过期吗？」
  - 不会（永远成立）→ task / invariants.intent
  - 会，因为产品策略可能变 → task（活文档可改）
  - 会，且原因是「我们当时选了 X」→ PDR（决策档案只追加）

- 「这句话需要被 Z3 验证吗？」
  - 是 → acceptance.intent / invariants.intent / contracts.intent 种子
  - 否 → task 文件自然语言部分

- 「这句话回答的是 What 还是 Why？」
  - What → task 文件（「用户能 X」）
  - Why → PDR Decision Context（「我们选 X 不选 Y，因为 …」）

- **新增（任务图专属）：「这句话和具体某个用户意图绑定吗？」**
  - 是（绑某个 task）→ 该 task 文件
  - 否（跨 task）→ PRODUCT.md 顶层
  - 仍然不绑（跨 product）→ `docs/user-journeys/` 全局层

## 任务图起草纪律

起草任何 task 文件前，先回答 3 个问题：

### 问 1：这个 task 的用户原话标题是什么？

判别：能不能用一句完整的用户原话讲出来？

| 能讲 → 是 task | 讲不出 → 是 feature |
|---|---|
| 「我第一次跑 intent check 直到拿到 PASS」 | 「验证功能」 |
| 「手机端忘了密码后 5 分钟内重新登录」 | 「密码重置功能」 |
| 「主管临时给下属开通某项权限 1 小时后自动收回」 | 「权限系统」 |

讲不出 → **退回到 product-debate 重新做 task 识别**，不要在 prd-writer 里硬造。

### 问 2：这个 task 属于 5 个旅程阶段哪一个？

按 `references/task-organization.md` § 判定准则走优先级判断：

1. 用户在「首次成功」的旅程上？→ `onboarding/`
2. 在排查异常 / 失败？→ `troubleshooting/`
3. 涉及组织 / 配额 / 权限 / 审计？→ `admin/`
4. 涉及账号或数据的终止关系？→ `lifecycle/`
5. 其它 → `daily-ops/`

**5 个阶段是固定的**——出现命中不了任一阶段的 task 时，先重读判定准则；仍命中
不了再考虑「这真的是 task 吗」（可能是 epic）。引入第 6 个阶段需要 CADR。

### 问 3：这个 task 有多大？

- 完成路径 3-7 个 happy-path 步骤 → OK
- 步骤里大量 if-else → 拆分支到 `troubleshooting/` task
- 完整文件 > 250 行 → 它是 epic 不是 task，拆成多个

## 起草顺序

1. **先列 Tasks** —— 用 prd-draft 的「Tasks」段为蓝本
   - 分配 task_id（在 target_product 内连续递增）
   - 决定每个 task 的 journey_stage
   - 决定每个 task 的 slug 和 h1 标题
2. **写每个 task 文件** —— 用 `templates/task.md`
3. **写 prd-overview** —— 用 `templates/prd.md`
4. **写 PDR 骨架** —— 用 `templates/pdr-skeleton.md`，分配 PDR ID
5. **写 acceptance.intent 种子** —— 用 `templates/acceptance-intent-seed.intent`
6. **写 tasks/README.md** —— 用 `templates/tasks-readme.md`，健康度统计先留空

为什么这个顺序：先写 task 文件能定下所有内容的归属；prd-overview 是 task 的索引
和顶层封装；PDR 引用 task 文件路径需要 task 已经成型；intent 种子只是 task 中
形式化条目的析出；README 是其它都写完后自然生成的清单。

## PDR ID 与 Task ID 分配规则

### PDR ID

product 内递增，4 位 0 填充：

- 看 `products/<target_product>/decisions/pdr/` 现有最大号 +1
- `PDR-0001-{slug}.md`、`PDR-0042-{slug}.md`

### Task ID

product 内**全局**递增（不按旅程阶段分），4 位 0 填充：

- 看 `products/<target_product>/tasks/**/T-*.md` 现有最大号 +1
- 本次起草连续 N 个 ID
- `T-0001-{slug}.md`（不分目录都从 T-0001 开始计数；目录只决定 journey_stage）

为什么不按旅程分计数：
- 不按旅程分 → 同一 product 内 task_id 全局唯一 → 跨 task 引用稳定
- 按旅程分（`T-onboarding-0001`、`T-daily-0001`）→ task 改归类时 ID 就乱

### Journey ID（跨 product）

全局递增，4 位 0 填充：

- 看 `docs/user-journeys/J-*.md` 现有最大号 +1
- `J-0001-{slug}.md`

## 命名双轨制

详细规则见 `references/task-organization.md` § 2。要点：

| 元素 | 稳定性 | 用途 |
|---|---|---|
| `T-0001` | 永不变 | 所有引用都用它 |
| `first-time-verify` | 可改 | 文件名可读部分 |
| 「我第一次跑 intent check 直到拿到 PASS」 | 随产品演进 | h1 + frontmatter title |

slug 改名走 `git mv`，**不改 task_id**，其它引用文件不需要动。

## Supersedes 链：处理修订

如果本次修改的是既有 task：

1. 找到该 task 最近一次 Decision-Ref（看文件末尾）
2. 在新 PDR Metadata 加 `Supersedes: PDR-XXXX`
3. **不修改**旧 PDR 文件
4. task 文件可以修改（它是活文档），文件末尾 Decision-Ref 更新为新 PDR ID
5. PDR Consequences § Task File Updates › 修改 Tasks 表标注

如果本次**删除**既有 task：

1. 新 PDR 标注 `Supersedes: PDR-XXXX` 和「废止 task T-YYYY」
2. PDR Consequences § Task File Updates › 删除 Tasks 表登记
3. **不真的删除文件**——把 task 文件 frontmatter 加 `status: deprecated`，并在
   文件头部加 `> ⚠️ Deprecated by PDR-{new-id}; Last-Valid-Until: YYYY-MM-DD`
   横幅。这样 git log 仍能查到，但 tasks-index 不再列出

## 与 arch-debate / rfc-writer 的边界

本 skill **不产出** ADR 或 contracts.intent 内容。在 prd-overview § Intent
Mapping 表中标了 `contracts.intent` 的条目：

1. 在对应 task 文件的「完成路径」部分只写自然语言描述，加备注「[ADR 候选：技术
   方案待 arch-debate 确认]」
2. 在 PDR Decision Context 标注「本决策待 ADR-XXXX 落地后才完整」
3. 在 acceptance.intent 种子里**不产出** contracts 内容
4. 在 PDR Consequences › Intent Updates 把 contracts 行标 `**等 ADR-XXXX**`
5. 建议用户：审批本 PDR 之前，先跑 arch-debate / rfc-writer 完成 ADR

## 与 living-doc-author 的边界

`tasks/README.md` 的**首次生成**由 prd-writer 负责（review 状态产出）。但后续
维护——尤其是健康度统计（Task 数 / 平均行数 / 最后更新最久的 task / 未引用的
task）——是 **living-doc-author 的天职**。

prd-writer 在 README 健康度统计表里只填表头和已知数据，未知项留 `——`，标注
「由 living-doc-author 在重跑时刷新」。

## Charter 自检清单

每次起草后，对 prd-overview 和每个 task 文件跑一遍：

- [ ] 无「曾经」「originally」「we used to」类历史叙事
- [ ] 无「将会」「计划于」类未来叙事
- [ ] prd-overview 每个章节末尾有 `Decision-Ref: PDR-{id}`
- [ ] 每个 task 文件末尾有 `Decision-Ref: PDR-{id}`
- [ ] Intent Mapping 表里每个 `acceptance.intent` 条目，种子文件里都有对应 block
- [ ] 每个 acceptance block 名形如 `T-XXXX-<short-desc>`，与 task_id 双射
- [ ] PDR Consequences § Task File Updates 列出的所有文件，本次实际产出
- [ ] 每个 task 文件 ≤ 250 行
- [ ] 每个 task h1 不形如 `# xxx 功能` / `# 实现 xxx` / `# xxx 模块`

任一项不满足 → 退回 drafting。

## 异常处理

**用户没跑 product-debate 直接来**：
- 警告：「跳过辩论 + 跳过 task 识别会让本 skill 替 PM 做判定，质量风险高」
- 用户坚持 → 启动**任务图访谈**（见 `prompts.ingesting`）

**fact-extraction-report 不存在**：
- task 中所有「数字 / 模块名 / 风险条目」改写为「[未经事实基验证]」标记
- 质量评分扣 IDD 适配度的相应分数

**PRD 评分一直 < 90**：
- 第一次低分：列改进项退回 drafting
- 第二次低分：进入 **task 级改进对话**（逐 task 问用户怎么改）
- 第三次低分：建议用户接受 `Status: Draft (quality bypass)` 落地

**target_product 的 tasks/ 目录不存在**：
- 这说明 project-init 还没用 v0.2 的 Scaffolding Manifest 跑过
- 警告用户先跑 `popsicle skill start project-init`（或手工 `mkdir -p` 5 个旅程目录）
- 不阻塞本 skill 流程，但 PDR Consequences § Tasks Index Updates 会跑空

**辩论中识别出 ≥ 8 个 task**：
- 提示用户：本次 PDR 涵盖范围可能太大
- 建议拆成多个 PDR（每个 PDR 3-5 个 task）

## 参考资源

### Reference 文件

- **`references/task-organization.md`** —— 5 个旅程阶段定义 + 命名双轨制 + 反模式
- **`references/quality-rubric.md`** —— 5 维度评分细则（v0.2 加 AI 可消化度）
- **`references/charter-compliance.md`** —— charter 四条铁律在 PRD 起草中的实操

### 上游 artifact

- **`{slug}.prd-draft.md`** —— 来自 product-debate（必须是 task-centric 形态）
- **`{slug}.product-debate.md`** —— 来自 product-debate（用于 PDR Decision Context）
- **`{slug}.fact-extraction-report.md`** —— 数字 / 模块名引用源
- **`{slug}.project-init-plan.md`** —— target product 锁定

### 下游 hand-off

- **arch-debate / rfc-writer** —— 处理 contracts.intent 条目
- **项目自带 intent spec writer** —— 把 acceptance.intent 种子收紧
- **living-doc-author** —— 维护 tasks/README.md 健康度统计
- **intent-consistency-check** —— Z3 闸（intent 合并后跑）
