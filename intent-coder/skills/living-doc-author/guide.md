# living-doc-author 使用指南

活文档**保活 / 对账**。它不创作内容（那是 prd-writer 的活），只做一件事：让
`tasks/README.md`、task 文件的元数据 / 反向引用、`PRODUCT.md` 的「现在状态」章节
永远代表**现在**——发现 doc-code drift 就刷新，刷不了的转人工。

这是 IDD「AI 反馈闭环」的**保活端**：上游（task / intent / PDR / 代码）一直在变，
活文档若不对账就会腐烂，AI 召回到的就是过期答案。

## 何时跑

- 首个迁移切片完成、已有 PRD/PDR/intent 之后；
- 任何时候上游变了（合并了新 task、intent 验证结果更新、PDR 落地）；
- 多处模板挂钩：`popsicle skill start living-doc-author --target <target>`。

**不是** CI 每次 PR 都跑的硬闸——它是周期性 / 触发式的保活工具。

## --target 模式

| target | 刷新对象 |
|---|---|
| `tasks-index` | `products/{p}/tasks/README.md` 索引表 + 健康度统计 |
| `task-backrefs` | 各 task 文件「反向引用」节 |
| `last-verified` | task frontmatter `last_verified`（用 intent-consistency-report 回填）|
| `product-context` | `docs/PROJECT_CONTEXT.md` 的「现在状态」类章节 |
| `implementation-status` | `tasks/README.md`「已实施」列（用 implementation-coverage 回填）|
| `architecture-manifest` | `ARCHITECTURE.md` § File Manifest（合并 ADR Consequences）|
| `product-header` | `PRODUCT.md` 双行头（PDR-001 模板）|
| `all`（缺省）| 以上全部 |

`slice-delivery` 末尾建议：
`living-doc-author --target implementation-status,architecture-manifest,product-header`

## 四类 doc-code drift 信号

1. **过期 staleness** — `last_updated` 距今 > 60 天告警 / > 90 天进归档评审；
   PRODUCT.md 比它引用的 task/intent 还旧。
2. **断链 broken-ref** — `related_intents` / `decision_ref` / `related_next_tasks` /
   README 链接指向不存在的目标。
3. **孤儿 orphan** — acceptance block ↔ task 的 task_id 双射断裂；task 无任何反向引用。
4. **未验证 unverified** — task `last_verified: ~` 但报告里对应 intent 已 `verified`。

## 红线：只对账，不创作

living-doc-author **只动元数据 / 索引 / 反向引用 / last_verified**。任何 task 正文、
PRODUCT 实质内容、intent 逻辑的改动都越界——发现了就记到 sync-report 的「待人工处置」，
让它走 **prd-writer + 新 PDR**（charter 铁律：改 task 必须有 PDR）。

`last_verified` 回填只认 `verified`；failed/unknown/skipped 保持 `~` 并在报告点名，
避免把「没验过」伪装成「验过了」。

## 产物

`{slug}.living-doc-sync-report.md`：drift 信号四类明细 + 刷新动作 + 健康度快照 +
待人工处置。被刷新的活文档（README、task 元数据）作为副作用直接落到现有文件。
