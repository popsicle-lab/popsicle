# Charter 合规清单

本文件解释 doc-architecture charter 四条铁律在 prd-writer 起草过程中的实操检查。
Charter 原文见仓库的 `docs/CHARTER.md`（由 `project-init` 落地）。

---

## 铁律 1：活文档没有版本号

> 活文档（PRODUCT.md / ARCHITECTURE.md）只有 `Last-Updated` 和 `Last-Decision-Ref`。
> 它们永远代表「现在」。过期内容就地修正；不写历史叙事。

**对 PRD 主稿的影响**：

- ✅ 用现在时写「用户提交 `.intent` 后 5 秒内看到 PASS/FAIL」
- ❌ 不写「上线后用户将看到 5 秒响应」
- ❌ 不写「初版我们用 polling，后改为 SSE」（这是 PDR 的内容）

PRD 主稿落地为 PRODUCT.md 章节时，文件头部会有：

```yaml
---
Last-Updated: 2026-05-13
Last-Decision-Ref: PDR-0042
---
```

**检查方式**：grep PRD 主稿，找以下短语，每处出现都是扣分点：

| 禁用短语 | 替换 |
|---|---|
| 「将会」「将能」「将提供」 | 现在时（「能」「提供」）|
| 「上线后」「发布后」| 删掉这个修饰，直接陈述功能 |
| 「曾经」「原本」「初版」「we used to」「originally」| 删掉整句——这是 PDR 内容 |
| 「计划于 Q3 …」 | 改为 Roadmap 章节的 [PDR-XXXX] 引用 |

---

## 铁律 2：决策档案只追加

> ADR/PDR 文件在 `Status: Accepted` 之后**永不修改**。错的决策通过写一份**新**
> 决策（标注 Supersedes）来纠正。

**对 PDR 骨架的影响**：

- 本 skill 起草的 PDR Status 永远是 `Proposed` —— 用户审批通过后才能改 `Accepted`
- 起草时**不要**预填 `Accepted`，这是审批权限不是起草权限
- 如果用户要修订既有 PDR：**不要修改旧 PDR 文件**，而是新建一份 PDR 标注 `Supersedes: PDR-XXXX`

**检查方式**：
- PDR Metadata 段查找 `Status: Proposed`，确认不是 Accepted/Rejected
- 如果用户提供了「Supersedes: PDR-XXXX」，确认旧 PDR 存在且 Status: Accepted

---

## 铁律 3：每次活文档编辑必须引用 Decision-Ref

> 改 PRODUCT.md 或 ARCHITECTURE.md 的 PR 由 CI 强制引用 Decision ID。

**对 PRD 主稿的影响**：

- PRD 主稿**每个**修改章节末尾必须有 `Decision-Ref: PDR-XXXX`
- 新增章节用 `Decision-Ref: PDR-XXXX (new)`
- 修订章节用 `Decision-Ref: PDR-XXXX (supersedes PDR-YYYY)`

**例**：

```markdown
### Replication 策略

Multi-Paxos 跨 3 个区域，单分区主节点故障切换 < 30 秒。

`Decision-Ref: PDR-0042`
```

**检查方式**：评分维度 4「IDD 适配度」直接打分，无 Decision-Ref = 0 分。

---

## 铁律 4：一次变更可能波及多份活文档

> 触发它的 ADR/PDR 的 `Consequences` 章节**必须**列出所有被它强制更新的活文档段落；
> PR 必须在一次提交里全部更新。

**对 PDR 骨架的影响**：

`templates/pdr-skeleton.md` 的 Consequences 章节模板已经强制结构化输出**所有**
被强制更新的活文档段落：

```markdown
## Consequences

### Living Doc Updates (required by this PDR)

- [ ] `products/<name>/PRODUCT.md` § Functional Requirements › Replication
- [ ] `products/<name>/ARCHITECTURE.md` § Replication Topology
- [ ] `docs/glossary.md` 新增术语：Multi-Paxos Region

### Intent Updates (required by this PDR)

- [ ] `products/<name>/intents/acceptance.intent` 新增 block: `replication_failover_under_30s`
- [ ] `products/<name>/intents/invariants.intent` 新增 block: `single_active_primary_per_region`
```

**检查方式**：
- 起草 PDR 后，确认 Consequences 列出的所有活文档段落，**当前 PRD 主稿都已实
  际覆盖**
- 如果列出的段落 PRD 没覆盖 → 退回 drafting 补 PRD，**不要**从 Consequences 删
  漏掉的项（这会让 charter 第 4 条变成空文）

---

## 综合自检清单（每次起草后跑）

按顺序检查：

1. **铁律 1（无历史/未来叙事）**
   - [ ] PRD 主稿 grep 「曾经/originally/previously/we used to/将会/计划于」=
         无结果
2. **铁律 2（PDR 不可变）**
   - [ ] PDR Status = Proposed
   - [ ] 如有 Supersedes 字段，旧 PDR 文件确实存在且 Status: Accepted
3. **铁律 3（Decision-Ref 引用）**
   - [ ] PRD 主稿每个章节末尾有 `Decision-Ref: PDR-XXXX`
   - [ ] 引用的 PDR ID 与本次 PDR 骨架的 ID 一致
4. **铁律 4（Consequences 列举）**
   - [ ] PDR Consequences 列出**所有**被强制更新的活文档段落
   - [ ] PRD 主稿已覆盖 Consequences 列出的所有段落
   - [ ] acceptance.intent 种子已覆盖 Consequences 的 Intent Updates 部分

任一项不满足 → 退回 drafting。

---

## charter 不强制的部分（但本 skill 仍然检查）

以下是 charter 没硬性规定但本 skill 的质量评分会考虑的：

- **Domain Glossary 一致性**：PRD 中术语来自 `docs/glossary.md` 或 fact-extraction-report
- **事实基引用密度**：每处数字 / 模块名 / 风险条目都 cite fact-ext
- **Intent 层归类的完整性**：Phase 4 的 Intent Mapping 表完整对应到种子文件

这些不算 charter 违规，但少了会扣分；可以在用户 bypass 后保留水印放行。
