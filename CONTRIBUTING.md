# Contributing to popsicle-new

> 人和 AI agent 都读这一份。落地 IDD 的"刚性"在于：所有人按相同规则改文档、改决策、
> 改代码——没有快速通道，没有"我有特殊情况"。

---

## 1. 文档体系的硬约束

**必读**：[`docs/CHARTER.md`](docs/CHARTER.md)（四条铁律）。下面是它在协作场景的展开。

| 在 PR 中做什么 | 必须 / 禁止 |
|---|---|
| 改 `products/*/PRODUCT.md` 或 `products/*/ARCHITECTURE.md`（除错别字/链接/措辞外） | **必须**在 PR 描述里引用一个 Decision ID（ADR/PDR/CADR） |
| 改 `products/*/intents/*.intent` | **必须**附 `intent check` 通过的输出（Z3 验证）|
| 写"曾经..." / "originally..." / "we used to..." 这类历史叙述 | **禁止**写进活文档；改成 `[ADR-XXXX]` 引用即可 |
| 改 `docs/CHARTER.md` | **必须**通过 CADR（Charter Amendment Decision Record）流程，提案位于 `docs/decisions/cadr/`（暂不存在，首次需要时建）|
| 加新 `products/<name>/` | **必须**先跑 `project-init` skill 重新评估 product 命名 + 4 件套铺设 |

---

## 2. 决策文件的硬约束

- **PDR / ADR 在 `Status: Accepted` 之后永不修改**。错的决策通过写一份**新**决策（带
  `Supersedes: PDR-XXXX`）来纠正。
- 每份 PDR / ADR 必须有 `Intent Impact` 章节，指明它修改的是哪一层 intent
  （global invariants / product invariants / acceptance / contracts）。
- 一份决策一旦 Accepted，它的 `Consequences` 章节**必须**列出所有被它强制更新的活文档段落；
  PR 必须在同一次提交里把那些段落同步更新。

---

## 3. IDD pipeline 与 PR 的关系

本仓库迁移分 **两条 pipeline**（同一 issue 可链式跑，避免 14 stage 过重）：

**仓库级（Day-1 一次）** — `migration-bootstrap`（10 stage）：

```
project-init → fact-extractor → product-debate → prd-writer
              → arch-debate → rfc-writer → adr-writer
              → intent-spec-writer → intent-consistency-check → living-doc-author
```

**每 slice** — spec 已有可跳过 `migration-slice-spec`，迁移交付必跑 `migration-slice-delivery`（4 stage）：

```
shadow-implementer → equivalence-baseline → cutover-author → living-doc-author
  (--target implementation-status,architecture-manifest,product-header)
```

| 谁创建 PR | 何时 |
|---|---|
| project-init 完成（init stage approved） | 整个仓库 bootstrap 后第一次 commit |
| 每个 skill 产出新文档 | 那个 skill 的 stage approved 后，一次 PR |
| ADR Accepted | 触发后续 contracts.intent 收紧的 PR（包含被更新的活文档段落）|
| 任何代码改动 | 必须命中一个开放的 PDR/ADR，或在 PR 描述说明"无 spec 改动"（受 reviewer 严格审视）|

---

## 4. Strangler Fig 切流前的硬门禁

任何 slice 在 popsicle-new 切流（用 popsicle-new 输出代替 legacy popsicle）前**必须**满足：

1. 全部 `.intent` 文件 `intent check` Z3 PASS（`intent-consistency-check` skill 的 `gate_ready` 判据：连续 N=3 次零失败）
2. 至少 5 条 golden CLI 命令对账脚本通过（legacy 与 new 同输入同输出，diff 为空）
3. ADR Accepted（切流本身的决策需要单独 ADR）

切流后 legacy 该 product 范围进入 `Sunset` 状态，由 `migration/progress.md` 追踪。

---

## 5. AI agent 的特别注意事项

- 不要默默改 `intent-coder/` —— 那是 legacy 资产，迁移期任何改动必须记入 `LEGACY_PIN.md` 的"已知 patch"清单
- popsicle skill 是状态机，**不要**用 `pipeline stage complete --confirm` 跳过中间状态——除非状态机本身设计为可跳过
- 任何 `[TBD: needs archaeology]` 占位符**禁止**由 AI agent 编造内容填充——那是 fact-extractor / PRD writer 的活
- 商业策略 / 定价 / 客户分层等内容**禁止**进 `ARCHITECTURE.md`；实现细节 / 技术选型理由**禁止**进 `PRODUCT.md`

---

## 6. 起步

```bash
popsicle pipeline status                          # 看主迁移 pipeline 当前状态
popsicle pipeline next --run <run-id>             # 让 popsicle 告诉你下一步
popsicle prompt <skill-name> --run <run-id>       # 取该 skill 当前状态的 AI prompt
```

不知道做什么 → 看 `migration/progress.md` 的 in-progress 列。
