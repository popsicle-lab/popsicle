# popsicle-new

> popsicle 自身向 IDD（Intent-Driven Development）的迁移目标仓库。
>
> 这里是新仓库的工作根。原 popsicle 通过 `legacy/popsicle/` submodule pin 在
> `c76d729`，由 [intent-coder](https://github.com/popsicle-lab/intent-coder) 技能包
> 驱动 Strangler Fig 渐进式迁移。

## 状态

- **架构定位**：按 [intent-coder ROADMAP](legacy/popsicle/intent-coder/ROADMAP.md) §2 D4 决策，
  popsicle 是 intent-coder 私有引擎（**不是**通用工作流平台）。RFC D2 拆分作废。
- **迁移粒度**：Strangler Fig，逐 slice 切流，legacy popsicle 持续可用。
- **首切片**：`products/skill-runtime/`（进行中）。
- **产品域**：3 个确认 + 1 个待裁决（详见 [`migration/progress.md`](migration/progress.md)）。

## 必读

1. [`docs/CHARTER.md`](docs/CHARTER.md) —— 文档体系的四条铁律。**任何贡献者**（人或
   AI agent）动 `docs/`、`products/*/PRODUCT.md`、`products/*/ARCHITECTURE.md`、决策
   文件之前都要先读。
2. [`CONTRIBUTING.md`](CONTRIBUTING.md) —— IDD 工作流落地规则。
3. [`AGENTS.md`](AGENTS.md) —— popsicle 自动生成的 agent 指南（cursor target）。

## 工程入口

由于本仓库通过 popsicle + intent-coder 驱动，主流程不是 `cargo run`，而是 popsicle
pipeline。常用命令：

```bash
popsicle pipeline status                          # 当前 migration-bootstrap 状态
popsicle pipeline next --run <run-id>             # 推荐下一步
popsicle doc check status --run <run-id>          # 查看所有文档 checklist
popsicle doc list --run <run-id>                  # 列出本次 run 的所有产物
```

## 已知约束

- popsicle-new 当前嵌套在父 popsicle 仓库内（父仓库 `.gitignore` 已排除）。
  推到 GitHub 后会真正分仓。
- legacy popsicle pin 在 `c76d729`，但仓库内 `intent-coder/` 与 `vender/intent-lang/`
  在该 SHA 时尚未 commit；popsicle-new 通过相对路径 `../intent-coder` 临时引用。
  详见 [`LEGACY_PIN.md`](LEGACY_PIN.md)。
