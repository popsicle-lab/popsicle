# Pipeline 选择指南

> **读者**：创建 Issue 的 agent / 人类。在 `issue start` 之前读完本页，避免 spec 未完成就进入 `slice-delivery`。

## 决策树

```text
新工作项
├─ 全新 product 模块（无 legacy、无已定 spec）？
│  └─ yes → greenfield-product-spec
├─ 所属 slice 的 PRD / intent 尚未覆盖本能力？
│  └─ yes → slice-spec（或 tech-decision，若仅需架构决策）
├─ 仅架构/技术选型（无实现）？
│  └─ yes → tech-decision
├─ 单点回归/缺陷修复？
│  └─ yes → bugfix
└─ spec 已定（acceptance.intent 有对应 block 且 intent check 通过）？
   └─ yes → slice-delivery
```

## Issue 类型与默认 pipeline

| `--type` | 默认 pipeline | 典型用途 |
|---|---|---|
| `product` | `greenfield-product-spec` | 新产品/模块从零 spec |
| `technical` | `tech-decision` | 架构辩论 → RFC → ADR |
| `bug` | `bugfix` | 最小修复环 |
| `idea` | `tech-decision` | 探索性技术想法 |

**不要**把 `--type technical` 的默认当成「功能实现」。在已有 slice 上增量交付时，应**显式**指定：

```bash
# 1) spec 未覆盖时
popsicle issue create --type technical --title "..." --spec slice-4-ui \
  --pipeline slice-spec --description "..."

# 2) spec 已覆盖、只写代码时
popsicle issue create --type technical --title "..." --spec slice-4-ui \
  --pipeline slice-delivery --description "..."
```

## slice-delivery 硬门禁（进入 implement 前）

- [ ] 目标能力在 `products/<spec>/intents/*.intent` 有 acceptance block
- [ ] `popsicle tool run intent-validate path=products` 通过（或 observe 零失败）
- [ ] 相关 ADR File Manifest 列出将改动的路径
- [ ] 若跳过 spec 链，必须在 cutover ADR 的 Divergence 表登记（如 D-6xx）

## 反模式（PROJ-30 教训）

| 反模式 | 后果 | 应用 |
|---|---|---|
| 直接 `--pipeline slice-delivery` 写 UI 功能 | 无 PRD/intent 前置，事后补 ADR | 先 slice-spec 或更新 intent |
| 把增量增强当成 `product` greenfield | 重复 debate/PRD，过重 | 用 slice-spec 短链 |
| 未安装 intent-coder module 就指望 skill 模板 | `doc create` 只有空壳 | `popsicle init` + `admin sync-intent-coder` |

## 模块安装（ADR-017）

| 场景 | 来源 |
|---|---|
| popsicle 单体仓库（根目录有 `intent-coder/`） | 从工作区根 **覆盖** 同步 |
| DMG / `cargo install` / 任意新项目 | 从 **二进制内嵌包** 解压到 `.popsicle/modules/intent-coder/` |

`popsicle module add` 在 self-host MVP 仍 **deferred**（ADR-011）；用 `popsicle init` 或 `admin sync-intent-coder`。
`doctor --format json` 查看 `intent_coder_module` 与 `intent_coder_bundle`。
