---
doc_type: shadow-implementer
id: doc-56
pipeline_run_id: 0000001a-0000-401a-8001-1a00000000001a
status: active
title: PROJ-26 devops tooling migration coverage
version: 1
---

# PROJ-26 devops tooling migration coverage

> **Issue**: PROJ-26 · migrate legacy devops tooling
> **Stage**: implement (slice-delivery)
> **Legacy source**: `legacy/popsicle` @ c76d729
> **Date**: 2026-06-11

## Inventory(legacy → new 映射)

| Legacy 资产 | 新位置 | 适配 |
|---|---|---|
| `Makefile`(check/fmt/clippy/test/build/build-ui/install-hooks)| `Makefile` | 删 build-ui(无 UI);新增 `golden`(全链基线)、`intent`(Z3 校验)、`fmt-fix`、`install` |
| `scripts/install.sh`(UI 构建、completions、PATH、卸载)| `scripts/install.sh` | 删 UI 特性与 `--no-ui`;删 completions(deferred 命令);装 `crates/cli-ux`;增工作区 provenance 提示 |
| `hooks/pre-commit`(fmt+clippy+test)| `hooks/pre-commit` | 原样保留三件套 |
| `.github/workflows/ci.yml` | 同路径 | 删 node/npm/webkit 依赖;保留 fmt/clippy/test;cargo test 已含 golden_001-011、后端测试与隔离 e2e smoke |
| `.github/workflows/release.yml` | 同路径 | 删 UI 工具链与 Linux webkit 依赖;4 平台矩阵保留(darwin x2 / linux / windows);rusqlite bundled 仅需 cc |

不迁移:`ui/` 相关一切(新架构无 UI)、completions 安装(命令 deferred,
裁决后再补)。

## 前置整备:让三件套真正可绿

迁移 CI 的前提是 `-Dwarnings` 全干净,本次顺手清掉历史欠账:

- `cargo fmt --all` 应用(2 处历史漂移)
- clippy 修复 3 处:`bool_assert_comparison`(skill-runtime 测试)、
  `ptr_arg` + `collapsible_str_replace`(cli-ux self_host)、
  `cmp_owned`(smoke 测试)

## Verification

- `make check`(fmt + clippy + test)全绿;`make install-hooks` 落位
- `scripts/install.sh --prefix /tmp/...` 真实安装 → 运行 `popsicle help` →
  `--uninstall` 清理,全闭环
- workflows YAML 解析 + 结构断言通过
- 全链 golden 27/27(22 回归 + devops 4 + usability 计数随新 issue 更新)
- `tool run intent-validate path=products` exit 0(见 equivalence)

## Problems Hit & Optimizations (记录并优化)

| # | 问题 | 处置 |
|---|---|---|
| P1 | 代码基线非 fmt/clippy 干净,直接迁移 CI 会红 | 先清欠账再迁移;`make check` 与 pre-commit hook 防再积累 |
| P2 | legacy install.sh 依赖 `popsicle completions` 子命令,新 CLI 已 deferred | completions 安装整段移除,在脚本头注明依据(AGENTS.md deferred 表)|
| P3 | legacy CI 的 webkit/node 矩阵与新仓库无关,照搬会拖慢且必失败 | 按新工作区实情裁剪;差异逐条记录在 ADR-014 与工作流文件头注释 |

- [x] Makefile 迁移 + golden/intent 新目标
- [x] install.sh 适配迁移并真实安装/卸载验证
- [x] pre-commit hook 迁移并安装
- [x] CI/Release workflows 迁移适配
- [x] fmt/clippy 历史欠账清零
