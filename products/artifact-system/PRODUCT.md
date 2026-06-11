# Product: artifact-system

> **Layer**: L2（用户可见行为）
> **Audience**: PM、销售、客户成功、AI Copilot
> **Status**: cutover-done（ADR-006 Accepted 2026-06-09；CLI 仍 legacy）
> **Last-Updated**: 2026-06-09
> **Last-Decision-Ref**: ADR-006（artifact-system cutover）

## 一行用途

Document 实体 + Markdown 智能编辑 + Guard + Context 装配 + WorkItem 提取 + frontmatter —— popsicle 文档体系的引擎。

## 用户视角的入口

- 文档制品生命周期：`Document` frontmatter + body 往返、revision 链、active/final 状态。
- 文档校验：`has_sections` / `checklist_complete` 纯文档 guard，以及 `upstream_approved` 注入端口。
- Prompt 上下文：`ContextLayer` 按 relevance/priority/id 稳定装配，高相关内容靠近 base prompt。
- 结构化抽取：从 Markdown 中抽取 story / testcase / bug chunk，未知输入返回空结果而非 panic。
- 迁移实体：legacy `work_item` 以 `task_chunk` 保持 kind + fields blob。

## Tasks Catalog

> 5 个旅程阶段的入口（v0.2 任务图）。具体 task 文件由 prd-writer 写到 `tasks/<stage>/` 下。

- [Onboarding](tasks/onboarding/) — 首次接触 → 首次成功（[TBD]）
- [Daily-Ops](tasks/daily-ops/) — 日常使用（[TBD]）
- [Troubleshooting](tasks/troubleshooting/) — 故障排查（[TBD]）
- [Admin](tasks/admin/) — 管理类（[TBD]）
- [Lifecycle](tasks/lifecycle/) — 终止 / 迁出 / 续费（[TBD]）

详见 [`tasks/README.md`](tasks/README.md)。

## Intents Catalog

- [`intents/acceptance.intent`](intents/acceptance.intent) — 验收契约（5 intent，Z3 verified 2026-06-09）
- [`intents/invariants.intent`](intents/invariants.intent) — product 自然律（1 invariant `EvaluateGuard` + safety `UnknownGuardIsInvalid`，Z3 verified 2026-06-09）
- [`intents/contracts.intent`](intents/contracts.intent) — 模块 API 契约（3 goal，[ADR-004 Accepted]，0 VC）

## Committed Roadmap

- ADR-006：lib-level artifact-system 已切为 popsicle-new 主路径。
- cli-ux slice：补齐 `doc` / `prompt` / `extract` 命令级 golden 与 binary wiring。
- storage/cli-ux：SQLite `IndexDb` 全量持久化切流另行 ADR。

## Open Questions

- CLI byte parity 与 legacy 命令输出差异归 `cli-ux` slice 处理。
- 是否保留 legacy YAML 完整 wire format，还是继续采用 intent-driven deterministic frontmatter，由后续 CLI ADR 判定。

---

> 本文件是首切片骨架；下游 skill 执行顺序：
> `fact-extractor` → `product-debate` → `prd-writer` 三步后会填充本表所有 `[TBD]`。
> 修订本文件遵循 [`docs/CHARTER.md`](../../docs/CHARTER.md) 第 3 条铁律：必须引用 Decision ID。
