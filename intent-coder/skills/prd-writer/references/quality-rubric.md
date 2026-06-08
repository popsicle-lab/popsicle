# PRD 质量评分细则（v0.2 — 任务图范式）

总分 **100**，目标 ≥ 90。

v0.2 与 v0.1 的关键差异：新增「**AI 可消化度**」维度（20 分），重新分配其它维度
权重；评分目标从「单一 PRD 文件」扩展为「prd-overview + N 个 task 文件 + 三联体」
的整体一致性。

| 维度 | v0.1 | v0.2 |
|---|---|---|
| 完整性 | 25 | 20 |
| 清晰度 | 25 | 20 |
| 可测试性 | 25 | 15 |
| **AI 可消化度** | —— | **20**（新增）|
| IDD 适配度 | 25 | 25 |
| **总分** | 100 | 100 |

---

## 维度 1：完整性（20 分）

针对 **prd-overview** 主稿：

| 子项 | 分值 | 扣分情况 |
|------|------|---------|
| Core Intent（§1）一句话描述本次变更核心意图 | 2 | 缺 -2；超 3 句 -1 |
| Problem Statement（§2）现状 + 提议 + 影响 3 段齐全 | 3 | 每缺一段 -1 |
| Success Metrics（§3）≥ 2 个指标且有 Measurement 列 | 3 | 每缺一项 -1 |
| File Manifest（§4）含「新增/修改/删除 Tasks」三栏 | 3 | 缺一栏 -1 |
| User Intents Catalog（§5）≥ 3 行（每个 task 至少 1 行）| 3 | 每个 task 缺 query 行 -1 |
| Intent Mapping（§6）每条都有目标 intent 层 | 2 | 缺归类的每条 -1 |
| Out of Tasks（§7）显式列出 ≥ 2 项 | 2 | 缺 -2 |
| Risk Assessment（§8）≥ 3 项含 Affected Tasks 列 | 2 | 缺 Affected Tasks 列 -1 |

针对 **N 份 task 文件**（每份独立检查，按平均分计入本维度）：

| 子项 | 分值（合并到上面 20 分内）| 扣分情况 |
|------|------|---------|
| frontmatter 8 个必填字段齐全 | / | 每个 task 每缺 1 字段 -1 |
| 「本 task 可解答」≥ 3 个问句 | / | 每个 task 每少 1 个问句 -1 |
| 完成路径 ≥ 3 个步骤 | / | 每个 task 缺 -2 |
| Related Next Tasks ≥ 1 个（叶子例外）| / | 缺 -1 |

---

## 维度 2：清晰度（20 分）

| 子项 | 分值 | 扣分情况 |
|------|------|---------|
| 无歧义陈述 | 6 | 出现「应该」「可能」「适当」「合理」「良好」等模糊词，每处 -1 |
| 时态用「现在时」 | 5 | 出现「将会」「会」「计划于」等未来时每处 -1；出现「曾经」「之前」「originally」等历史叙事每处 -2 |
| 术语来自 Domain Glossary | 4 | 未在 glossary 中出现的术语每个 -1 |
| Task h1 标题是完整人话句子 | 3 | h1 形如 `# xxx 功能` / `# 实现 xxx` / `# xxx 模块` 每个 -1 |
| 章节结构遵循 template | 2 | 缺章节或顺序错乱 -1~-2 |

---

## 维度 3：可测试性（15 分）

| 子项 | 分值 | 扣分情况 |
|------|------|---------|
| 每个 task 的「可观察的成功标志」指向 acceptance.intent block | 5 | 每个 task 缺 -1 |
| Acceptance intent 落成 intent-lang 后置条件（`require` / `ensure`，必要时 `invariant`，后态用 primed `x'`）| 4 | 用自然语言模糊描述每个 -1；塞入 intent-lang 不支持的时间/性能算子（within/eventually 等）每处 -1（应降级到 task 成功标志）|
| Success Metrics 可量化 | 3 | 模糊指标每条 -1 |
| Intent Mapping 表与 acceptance.intent block 一一对应 | 3 | 每处 mismatch -1 |

---

## 维度 4：AI 可消化度（20 分）—— v0.2 新增

> 文章 strategy 1 / 2 / 4 在 quality-rubric 中的落地。这是任务图范式的核心评分点。

| 子项 | 分值 | 扣分情况 |
|------|------|---------|
| **YAML frontmatter 标准化** | 4 | task 文件无 frontmatter 每个 -2；frontmatter 字段命名不规范每处 -1 |
| **chunk 独立性**（无跨块指代）| 4 | task 内出现「如上所述」「参考前文」「上文提到」每处 -1；用裸 slug 引用其它 task（应用 task_id）每处 -1 |
| **Query 锚点覆盖** | 4 | 每个 task 缺 query 锚点 / 锚点 < 3 个 -1；锚点写产品术语不写用户原话每处 -0.5 |
| **Next-query 链** | 3 | 每个非叶子 task 缺 Related Next Tasks -1；叶子 task 未在 frontmatter 标 `is_leaf: true` -0.5 |
| **目录归类合规** | 3 | task 不在 5 个标准旅程目录之一 -2 / 项；ID 含分类前缀（`T-onboarding-001`）-1 / 项 |
| **元数据一致性** | 2 | frontmatter 的 journey_stage 与文件所在目录不一致 -1；title 与 h1 不一致 -1 |

### 关键扣分细则

| 反模式（出现一次即扣）| 扣分 |
|---|---|
| Task h1 = `# xxx 功能` | -1 / 个（也算清晰度扣分；不重复扣 AI 可消化度）|
| Task 文件 > 250 行 | -3 / 个（硬上限）|
| Task 文件 > 150 行 | -1 / 个（软上限）|
| Task 完成路径含 ≥ 3 个 if-else 分支 | -2 / 个（提示拆 task）|
| 5 个旅程目录以外的自创目录 | -5 / 个（结构违规）|
| Tasks/README.md 缺失 | -3 |
| User Intents Catalog 缺整张表 | -5 |

---

## 维度 5：IDD 适配度（25 分）

| 子项 | 分值 | 扣分情况 |
|------|------|---------|
| **每项「数字 / LoC / 模块名 / 风险条目」cite fact-extraction-report** | 5 | 每处无 cite -1（无 fact-ext 时整项标 `[未经事实基验证]`，本项不扣分但 5 → 2.5）|
| **`Decision-Ref: PDR-{id}` 在 prd-overview / 每个 task 文件末尾** | 8 | **任一文件缺失 = 直接 0 分**（charter 第 3 条铁律） |
| **三联体一致性自检通过** | 5 | 每项失败 -1 |
| **PDR Consequences § Task File Updates 与实际产出文件一致** | 4 | 不一致 -1 / 处 |
| **无 charter 禁用短语**（「曾经」「originally」「we used to」「previously」「将会」） | 3 | **任一处出现 = 本项 0 分** |

---

## 总分与转换规则

| 总分区间 | 处理 |
|---------|------|
| 90-100 | `pass` → 进入 review |
| 75-89 | `refine` → 退回 drafting，列出 3-5 项最重要改进 |
| < 75 | `refine` → 退回 drafting，做 task 级改进对话（逐 task 问用户怎么改）|

**用户强制 pass**（< 90 但用户要继续）：
- 允许，但 prd-overview 首部加水印 `Status: Draft (quality bypass: <user-provided-reason>)`
- 在 PDR Metadata 加 `Quality-Bypass-Note: ...`
- 落地为 `PRODUCT.md` / `task.md` 时**保留**水印——PR 评审者必看到

---

## 评分输出格式（必须用此格式）

```
📊 prd-overview 质量评分 — {slug}

| 维度 | 得分 | 备注 |
|---|---|---|
| 完整性 | 18/20 | 缺 Out of Tasks 段；2 个 task 缺 Related Next Tasks |
| 清晰度 | 17/20 | 3 处「适当」类模糊词；1 处「将会」未来时 |
| 可测试性 | 13/15 | T-0002 的 acceptance intent 用自然语言写，未落成 `ensure` 后置条件 |
| AI 可消化度 | 16/20 | T-0002 缺 query 锚点；T-0003 标题「批量验证功能」是功能名 |
| IDD 适配度 | 22/25 | T-0010 文件末尾缺 Decision-Ref；2 处 Risk 未 cite |
| **总分** | **86 / 100** | **未达 90 阈值，建议修订** |

📝 建议改进（按 ROI 排序）:
1. 给 T-0010 文件末尾补 `Decision-Ref: PDR-0042`（+3 IDD 适配度，最关键）
2. 把 T-0003 h1 改成用户原话「我一次性校验一组 .intent」（+1 清晰度 +1 AI 可消化度）
3. 给 T-0002 补 3 个 query 锚点（+1.5 AI 可消化度）
4. 把 T-0002 落成 intent-lang 后置条件（`ensure state' == ...`）；时间约束（≤ N s）移到 task「可观察的成功标志」由 e2e 守护（+2 可测试性）
5. 在 prd-overview §7 补 Out of Tasks 段（+2 完整性）

预期修订后总分：**95.5 / 100** ✅
```

---

## Charter Compliance 0 分项（出现一次直接判 0 分对应维度）

| 违规 | 0 分维度 |
|---|---|
| 任一文件缺 `Decision-Ref: PDR-XXXX` | IDD 适配度 8 分子项 → 0 |
| 出现 charter 禁用短语（「曾经」「originally」「previously」「we used to」）| IDD 适配度 3 分子项 → 0 |
| Tasks 目录出现第 6 个非标准子目录 | AI 可消化度结构合规子项额外 -5 |

这些是 charter 硬约束的直接落地，**不可 bypass**——即使用户 quality bypass 也必须修。
