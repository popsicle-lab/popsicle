---
doc_type: cutover-author
id: doc-58
pipeline_run_id: 0000001a-0000-401a-8001-1a00000000001a
status: active
title: PROJ-26 devops cutover ADR
version: 1
---

# PROJ-26 devops cutover ADR

> **Promoted to**: `products/cli-ux/decisions/adr/ADR-014-devops-tooling-migration.md`
> **Stage**: cutover (slice-delivery)
> **Date**: 2026-06-11

本文档是 ADR-014 的工作副本;正式决策见 promoted 路径。核心:Makefile /
install.sh / pre-commit / CI / Release 五件套按新架构事实迁移,UI 与
completions 面裁剪并记录,fmt/clippy 欠账清零保证门禁即刻可绿。

## Cutover Gate Checklist

- [x] intent gate:intent-validate exit 0
- [x] equivalence gate:27/27 golden pass
- [x] make check(fmt/clippy/test -Dwarnings)全绿
- [x] install.sh 安装→运行→卸载 dogfood 闭环
- [x] ADR-014 已 promoted 至 products/cli-ux/decisions/adr/

## Waiver Checklist

- [x] 无豁免
