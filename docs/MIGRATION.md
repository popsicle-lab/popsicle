# Migration Guide（人类工程师）

> **Last-Updated**: 2026-06-23
> **Last-Decision-Ref**: ADR-026 / PDR-001 A04

本文件帮助工程师理解 **legacy popsicle → popsicle-new（main）** 的心智模型。逐路径对照见 [`migration/traceability.md`](../migration/traceability.md)；工程画像见 [`PROJECT_CONTEXT.md`](PROJECT_CONTEXT.md)。

## 架构对照（一句话）

| Legacy | New |
|---|---|
| 单体 `popsicle-core` + `popsicle-cli` | 按 slice 拆成 `skill-runtime` / `artifact-system` / `cli-ux` + `storage` |
| `popsicle.db` IndexDb | `.popsicle/state.db` SQLite |
| 14 页 legacy UI | `ui/` MVP+（Issues / Pipeline / Products / Settings） |
| `context scan` + `.popsicle/project-context.md` | `docs/PROJECT_CONTEXT.md` + Settings UI + weekly 巡检 |

## Slice 顺序

1. **skill-runtime** — 引擎：skill load、pipeline session、issue 实体
2. **artifact-system** — 文档生命周期、guard、context 装配
3. **cli-ux** — CLI 命令面、自托管 workspace、Tauri UI

每个 slice：**slice-spec**（PRD/RFC/ADR/intent）→ **slice-delivery**（implement → equivalence → cutover → living-docs）。

## 日常命令

```bash
make check          # fmt + clippy + test
popsicle issue list
popsicle pipeline next --run <run_id>
popsicle tool run intent-validate path=products
```

## Weekly 活文档

```bash
popsicle issue create --type technical --product cli-ux \
  --pipeline weekly-health-check \
  --title "Weekly 活文档健康巡检" --format json
popsicle issue start <KEY> --format json
# living-doc-author --target tasks-index,product-context
```

## 不要做的事

- 不要在活文档里写「曾经 / 之前 / 从 X 迁到 Y」（见 [`CHARTER.md`](CHARTER.md)）
- 不要跳过 spec 直接 `slice-delivery` 做新 UI 能力
- 不要把 legacy→new 对照表塞进 `PROJECT_CONTEXT.md`（保持 traceability 专表）
