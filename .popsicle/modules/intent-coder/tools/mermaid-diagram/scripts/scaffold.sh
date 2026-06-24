#!/bin/sh
# Emit a Markdown fragment with Diagram title + mermaid fence.
set -e
TYPE="${1:-flowchart}"
TITLE="${2:-未命名}"
FORMAT="${3:-text}"

emit() {
  if [ "$FORMAT" = "json" ]; then
    printf '%s' "$1" | node -e "
      const fs = require('fs');
      const body = fs.readFileSync(0, 'utf8');
      console.log(JSON.stringify({ ok: true, type: process.argv[1], title: process.argv[2], markdown: body }));
    " "$TYPE" "$TITLE"
  else
    printf '%s\n' "$1"
  fi
}

case "$TYPE" in
  flowchart|task-flow)
    emit "## 流程示意

Diagram: ${TITLE} (flowchart)

\`\`\`mermaid
flowchart TD
  s1[\"步骤 1\"] --> s2[\"步骤 2\"]
  s2 --> ok[\"成功标志\"]
\`\`\`"
    ;;
  task-relations)
    emit "## 旅程与 Task 关系

Diagram: ${TITLE} (flowchart)

\`\`\`mermaid
flowchart LR
  T0001[\"T-0001 onboarding\"] --> T0002[\"T-0002 troubleshooting\"]
  T0001 --> T0010[\"T-0010 daily-ops\"]
\`\`\`"
    ;;
  sequence)
    emit "### 主路径时序

Diagram: ${TITLE} (sequenceDiagram)

\`\`\`mermaid
sequenceDiagram
  participant Caller
  participant ServiceA
  participant ServiceB
  Caller->>ServiceA: 请求
  ServiceA->>ServiceB: 内部调用
  ServiceB-->>ServiceA: 响应
  ServiceA-->>Caller: 结果
\`\`\`"
    ;;
  state)
    emit "### 状态迁移

Diagram: ${TITLE} (stateDiagram-v2)

\`\`\`mermaid
stateDiagram-v2
  [*] --> draft
  draft --> in_progress
  in_progress --> done
  in_progress --> blocked
  blocked --> in_progress
  done --> [*]
\`\`\`"
    ;;
  er)
    emit "### 数据模型

Diagram: ${TITLE} (erDiagram)

\`\`\`mermaid
erDiagram
  ENTITY_A ||--o{ ENTITY_B : relates
  ENTITY_A {
    string id PK
    string name
  }
  ENTITY_B {
    string id PK
    string entity_a_id FK
  }
\`\`\`"
    ;;
  architecture)
    emit "### 架构 / 模块图

Diagram: ${TITLE} (flowchart)

\`\`\`mermaid
flowchart TD
  subgraph client [\"客户端\"]
    uiMod[\"UI 模块\"]
  end
  subgraph core [\"核心\"]
    modA[\"模块 A\"]
    modB[\"模块 B\"]
  end
  uiMod --> modA
  modA --> modB
\`\`\`"
    ;;
  *)
    echo "error: unknown type=$TYPE" >&2
    echo "hint: flowchart|task-flow|task-relations|sequence|state|er|architecture" >&2
    exit 1
    ;;
esac
