# Glossary

> **Status**: 骨架，无内容
> **Owner**: 任何贡献者（轻量、低门槛）
> **Last-Updated**: 2026-06-08

本词表收录 popsicle-new 仓库内会被多次使用、且**容易被误解**的术语。"显而易见"
的术语（如 "PR"、"CLI"）不进。

## 词条

| 术语 | 定义 | 出处 |
|---|---|---|
| **IDD** | Intent-Driven Development（意图驱动开发）。区别于 SDD（spec 用完即弃），IDD 的 intent 是长期资产，活到产品下线 | `docs/CHARTER.md` |
| **D4** | intent-coder ROADMAP §2 决策 4：放弃通用平台定位，popsicle 降级为 intent-coder 私有引擎 | `legacy/popsicle/intent-coder/ROADMAP.md` |
| **Strangler Fig** | 渐进式迁移模式：新旧并存、逐切片替换、每切片由自动化验证等价性 | `docs/AI辅助编程对工程自动化的要求.md` §已有项目分析 |
| **slice / 切片** | 一个完整迁移单元的 product 范围，对应一次完整的 `migration-bootstrap` pipeline run | `migration/progress.md` |
| **Z3 闸** | `intent check` 调用 Z3 SMT 求解器对 `.intent` 做形式化一致性验证；observe 模式 + CI 硬闸两层 | `intent-coder/skills/intent-consistency-check/` |
| **playbook（首切片）** | 首切片完成后形成的端到端样板，后续 slice 抄它做剪裁 | `intent-coder/skills/project-init/guide.md` |
| **全局项目注册表** | `~/.popsicle/global.json` 记录多台机器/多个仓库的 popsicle 工作区路径、默认项与 `last_opened_at`（最近打开）；CLI `popsicle project`、UI 切换器与 `--project` 共用 | `crates/cli-ux/src/global_config.rs` · ADR-016 |
| _[TBD: 其它术语]_ | _按需追加，不要预先填充_ | n/a |

## 红线

- 一个术语在仓库里只出现一次时**不**进 glossary
- 普通编程术语（"crate"、"trait"、"async"）不进 glossary
- 商业术语放 `products/<name>/PRODUCT.md` 的词表，不放这里
