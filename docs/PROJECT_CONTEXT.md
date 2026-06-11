# Project Context

> **Status**: 骨架，由 `living-doc-author` skill 后续填充
> **Owner**: living-doc-author
> **Last-Updated**: 2026-06-08

本文件是 popsicle CLI（具体来说，`ProjectContextLayer`）会扫描并注入到每个 skill
LLM prompt 的"项目级背景"。它告诉 agent："你身处什么样的工程环境"。

> ⚠️ **不要手工填这份文件**。它由 `living-doc-author` skill 在首切片完成后扫
> `crates/` / `products/` / 工具链等信息自动刷新。bootstrap 期间这里只放骨架。

## Tech Stack

[TBD: needs archaeology] — by living-doc-author after first slice

## 主要工具链

[TBD] — Rust workspace（version pinned in `rust-toolchain.toml`，待 RFC 决定要不要加）

## 主要 module / pipeline

[TBD]

## 关键约束

[TBD]

---

popsicle CLI 通过 `popsicle context scan` 在每次 `popsicle init` 时自动生成
`.popsicle/project-context.md`（机器读）；本文件是 IDD 视角的对应物（人 + LLM 读）。
两者不冲突，但**同步刷新**——`living-doc-author` 守这个约束。
