---
name: popsicle-mermaid-diagrams
description: Popsicle tool mermaid-diagram——在 PRD、task、RFC、ADR、Issue 中生成与校验 Mermaid 图。通过 popsicle tool run 调用，不是 Cursor 内置画图。撰写 prd-writer、rfc-writer、adr-writer、issue-author 产出或用户要流程图/架构图/序列图时使用。
---

> **权威技能**：`intent-coder/tools/mermaid-diagram/guide.md`
> **调用**：`popsicle tool run mermaid-diagram action=…`

## 三步

```bash
# 1. 读技能（类型路由、各 pipeline skill 要求、语法红线）
popsicle tool run mermaid-diagram action=guide

# 2. 拿可粘贴模板（再改成真实 task_id / 模块名）
popsicle tool run mermaid-diagram action=scaffold type=task-flow title="…"
popsicle tool run mermaid-diagram action=scaffold type=architecture title="…"
popsicle tool run mermaid-diagram action=scaffold type=task-relations title="…"

# 3. 写完 Markdown 后校验
popsicle tool run mermaid-diagram action=validate path=<file-or-dir>
```

## scaffold `type`

| type | 用于 |
|------|------|
| `task-flow` / `flowchart` | task 完成路径 |
| `task-relations` | PRD task 链 |
| `architecture` | RFC/ADR 模块边界 |
| `sequence` | API/CLI 时序 |
| `state` | 状态机 |
| `er` | 数据模型 |

## 各 skill 最低要求

- **prd-writer**：task ≥4 步必须有图；prd-overview 建议 task-relations
- **rfc-writer**：RFC § Proposed Design 至少 1 图
- **adr-writer**：`validate` 核对，不新画
- **issue-author**：复杂 issue 可 scaffold 进 description

## 语法红线

双引号标签、`<br/>` 换行、无 emoji、无 `\n`、无 theme/classDef。详见 `action=guide`。
