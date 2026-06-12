# mermaid-diagram 使用指南

Popsicle **tool**（不是 Cursor 独立 skill）。在 PRD / task / RFC / ADR / Issue 中需要
Mermaid 图时，Agent **先调 tool**，再把输出粘贴进 Markdown；UI 用 `MarkdownWithMermaid`
渲染。

## 调用方式

```bash
# 1. 读完整技能（类型路由、语法红线、各 skill 要求）
popsicle tool run mermaid-diagram action=guide

# 2. 生成可粘贴的 Markdown 片段（占位节点，写入前改成真实 task_id / 模块名）
popsicle tool run mermaid-diagram action=scaffold type=task-flow title="首次验证 happy path"
popsicle tool run mermaid-diagram action=scaffold type=architecture title="cli-ux 模块边界"
popsicle tool run mermaid-diagram action=scaffold type=task-relations title="本次 PRD task 链"
popsicle tool run mermaid-diagram action=scaffold type=sequence title="Verify 主路径"
popsicle tool run mermaid-diagram action=scaffold format=json type=flowchart title="示例"

# 3. 校验已写入的 Markdown（写完 task/RFC/ADR 后）
popsicle tool run mermaid-diagram action=validate path=products/cli-ux/tasks/onboarding/T-CU-0001-first-init-next-step.md
popsicle tool run mermaid-diagram action=validate path=products/cli-ux/decisions/adr format=json
```

| 参数 | 必填 | 默认 | 说明 |
|------|------|------|------|
| `action` | 是 | — | `guide` / `scaffold` / `validate` |
| `type` | scaffold 时 | `flowchart` | `flowchart` `task-flow` `task-relations` `sequence` `state` `er` `architecture` |
| `path` | validate 时 | — | `.md` 文件或目录（相对仓库根） |
| `format` | 否 | `text` | `text` 或 `json` |
| `title` | 否 | `未命名` | scaffold 的 Diagram 标题 |

## 退出码

| exit | 含义 |
|------|------|
| `0` | 成功（validate：全部 mermaid 块通过） |
| `1` | validate 发现语法/保守规则问题，或参数错误 |
| `127` | tool 目录或 `ui/node_modules/mermaid` 缺失 |

validate 在未安装 `ui` 依赖时仍会做保守检查（emoji、`\n`、HTML）；安装后额外跑
`mermaid.parse`（与 UI 同族渲染器）。

## Agent 工作流（推荐）

1. `action=guide` 或本文件 — 确认当前 skill 是否必须画图（见下表）
2. `action=scaffold type=…` — 拿模板片段
3. 用上游事实（task_id、crate 名、接口）**替换占位节点**，写入 artifact
4. `action=validate path=…` — 提交前校验
5. `popsicle doc check` — 文档占位符闸（与 tool 互补）

## 按场景选图类型

| 要问什么 | scaffold `type` | Mermaid 关键字 |
|----------|-----------------|----------------|
| task 步骤 / pipeline | `task-flow` / `flowchart` | `flowchart TD` |
| PRD task 关系链 | `task-relations` | `flowchart LR` |
| 模块边界 | `architecture` | `flowchart TD` + `subgraph` |
| API / CLI 时序 | `sequence` | `sequenceDiagram` |
| 状态机 | `state` | `stateDiagram-v2` |
| 表结构 | `er` | `erDiagram` |

**慎用**：`mindmap`、`sankey`、`C4`、`gitGraph` — 改用 `flowchart`。不要写 `theme` /
`classDef` / `style`（UI 自有主题）。

## 语法保守清单

1. 节点 ID：`camelCase`，不用 `end` / `graph` / `subgraph` 作 ID
2. 含空格标签：双引号 `A["User Input"]`
3. 换行：`<br/>`，禁止字面量 `\n`
4. 无 emoji、无 HTML 标签
5. 围栏从列 0 开始，语言 `mermaid`
6. 上文一行：`Diagram: <名称> (<类型>)`

## 各 Skill 何时调用本 tool

| Skill | 时机 |
|-------|------|
| **prd-writer** | 写 task 前 `scaffold type=task-flow`；prd-overview 用 `task-relations`；完成后 `validate path=products/.../tasks/` |
| **rfc-writer** / **arch-debate** | 起草 RFC § Proposed Design 前 `scaffold type=architecture` 或 `sequence`；**至少 1 张图** |
| **adr-writer** | 固化前 `validate` ADR/RFC；不新画图，不一致则退回 rfc-writer |
| **issue-author** | 复杂 issue `scaffold type=flowchart` 写入 description |
| **fact-extractor** | `dependency-graph.md` 手写后 `validate path=…` |

## 质量检查清单

- [ ] 已跑 `scaffold` 或等价手写，且 `validate` exit 0
- [ ] 图中 task_id / 模块名与正文、Consequences 一致
- [ ] 一图一主题；task 图只画 happy path
- [ ] 超过 15 节点 → 拆图

## 参考

- [Mermaid 官方文档](https://mermaid.js.org/intro/)
- [社区 mermaid-diagrams skill](https://github.com/ShushukovaN/dd_----------/blob/main/.cursor/skills/mermaid-diagrams/SKILL.md) — 保守语法、ASCII 侧车
- 本仓库：`ui/src/components/MarkdownWithMermaid.tsx`
- 兼容说明：`intent-coder/guides/mermaid-diagrams.md`（指向本 tool 的短链接）
