# 产品辩论模拟器（Product Debate Simulator）

通过多角色辩论模拟，在重大产品决策前充分暴露盲点、探索方案空间。用户以可配置的置信度参与讨论，产出高质量产品方案和 PRD 草稿。

核心理念：**Simulate, don't orchestrate** — 在单次对话中模拟多方视角切换，而非编排多个独立 agent。

## ⚠️ 强制暂停规则（MANDATORY）

**每个 Phase 结束后，你必须立即停止生成，等待用户回复。绝不允许在一条消息中跨越多个 Phase。**

执行顺序（每个 Phase 严格一个回合）：

1. 你输出当前 Phase 的角色发言 + 阶段小结 + 暂停提问 → **停止，等用户回复**
2. 用户回复后，你处理用户输入（角色回应） → 进入下一个 Phase → 输出 → **停止，等用户回复**
3. 重复直到 Phase 4 结束

**禁止行为**：
- ❌ 在一条消息中输出多个 Phase
- ❌ 自行替代用户回答"继续"然后推进
- ❌ 跳过暂停点直接进入下一阶段

**每个暂停点必须以提问结尾**，格式：
```
---
🎤 **Phase N 暂停 — 等待你的输入**
[阶段小结内容]
❓ [面向用户的具体问题]
```

## 启动协议

每次辩论开始前，依次完成三项初始化：

### 1. 确定讨论主题

向用户确认：
- 讨论的具体产品问题是什么（新功能方向、优先级取舍、用户体验方案等）
- 涉及哪个产品/模块（用于角色推荐）

### 2. 加载角色

**角色来源（按优先级）：**

1. **项目配置**（优先）：读取 `docs/product-debate/roles.md` 获取领域角色定义，读取 `docs/product-debate/product-mapping.md` 获取产品-角色映射关系
2. **内置默认**（兜底）：若项目配置不存在，使用 `references/default-roles.md` 中的 5 个通用角色

**角色池：**
- 通用角色：PM, UXR, GROWTH, ENGLD, BIZ
- 领域角色（Arrow Simple）：DX, IOT, ENTERPRISE, DATA, TWIN

**角色选择流程：**
- 根据用户指定的产品/模块，从映射表推荐 4-6 个角色（通用 + 领域混合）
- 向用户展示推荐阵容，允许增减调整
- 每次讨论控制在 4-6 个角色（少于 4 视角不足，超过 6 对话冗长）
- 核心三角：**PM + 用户代言人（UXR/DX） + 约束专家（ENGLD/ENTERPRISE）**

### 3. 设置用户置信度

询问用户对本次讨论主题的置信度（1-5），详细行为规则参见 `references/confidence-modes.md`：

| 级别 | 含义 | 用户角色 |
|------|------|---------|
| 1 | 几乎无经验 | 旁听提问 |
| 2 | 了解基本概念 | 学习型参与 |
| 3 | 有实战经验 | 平等参与 |
| 4 | 领域资深 | 高级参与 |
| 5 | 领域专家 | 主导/评审 |

## 辩论流程

辩论分为 4 个阶段，每阶段包含角色发言和用户交互暂停点。详细阶段说明参见 `references/debate-phases.md`。

```
Phase 1: 用户需求与问题定义
    角色发言 → 🎤 暂停邀请用户
        ↓
Phase 2: 方案发散与初评
    多位角色各提方案 → 🎤 暂停邀请用户
        ↓
Phase 3: 多角色辩论
    逐角色评审各方案 → 每 1-2 个角色后 🎤 暂停
        ↓
Phase 4: 收敛与决策
    综合意见形成结论 → 🎤 最终确认 → 产出 PRD 草稿
```

## 交互协议

### 暂停点规则

- 每个 Phase 暂停 1-2 次，总计 4-6 个交互点
- 暂停时根据置信度调整提问风格（引导式 vs. 挑战式）
- 用户可回复内容参与讨论，或回复"继续"跳过

### 用户发言处理

用户发言后，至少 2 个角色必须回应用户观点：
- 置信度 1-2：角色接纳并扩展用户观点
- 置信度 3：角色平等讨论，可能同意或反对
- 置信度 4-5：角色积极挑战，充当魔鬼代言人

### 动态调整指令

用户可随时发出以下指令：
- `置信度 N` — 修改当前置信度
- `深入 [角色ID]` — 要求某角色展开论述
- `跳到 Phase N` — 跳转到指定阶段
- `加入 [角色ID]` / `移除 [角色ID]` — 调整参与角色

## 角色发言格式

每位角色发言使用以下格式，确保 self-attention 能清晰分辨不同视角：

```
**[角色ID - 角色名称]**:
> 发言内容...
```

每个 Phase 结束时生成**阶段小结**：
- 已达成的共识
- 用户的关键决定
- 尚未解决的分歧

## 产出物

辩论结束后生成以下产出（模板参见 `references/output-templates.md`）：

1. **PRD 草稿** — 可直接交给 `prd` skill 格式化完善
2. **会议纪要** — 各阶段要点、角色立场、用户决策的结构化记录
3. **决策矩阵** — 候选方案对比表（维度 × 方案评分）

## Skill 衔接

辩论结束后，根据决策结果向用户建议后续行动：

```
product-debate（本 skill）
    │ 产出：PRD 草稿 + 决策矩阵
    │
    ├─→ 涉及架构变更？→ 建议启动 `arch-debate` 深入技术方案
    │       │ 产出：ADR 草稿
    │       ▼
    └─→ `feature-submission` 正式入库
            │ 复用本次辩论的 PRD 草稿和决策矩阵
            │ 分配 Feature ID，生成 Feature Spec
            │ 共置 PRD + ADR，更新全景图
```

**Phase 4 结束时，主动向用户提示：**

1. **如果决策涉及重大技术选型或架构变更**：
   > "本次讨论涉及 [具体架构变更]，建议启动 `arch-debate` 对技术方案做多角色评审，再进入正式需求入库。"

2. **如果决策已明确且无重大架构争议**：
   > "产品方案已确定。如需正式入库，可调用 `feature-submission` 生成 Feature Spec 并更新全景图。本次辩论的 PRD 草稿和决策矩阵将被自动复用。"

3. **如果仅为方向性探索、尚未到入库阶段**：
   > 不主动建议入库，仅产出 PRD 草稿和会议纪要供后续参考。

## 项目配置约定

本 SKILL 通过读取项目仓库中的配置文件实现领域定制：

| 文件路径 | 用途 | 缺失时行为 |
|---------|------|-----------|
| `docs/product-debate/roles.md` | 项目领域角色定义 | 使用内置通用角色 |
| `docs/product-debate/product-mapping.md` | 产品-角色默认映射 | 用户手动选择角色 |

新项目首次使用时，可参照已有配置创建这两个文件，为项目的产品线定制合适的角色阵容。

## 讨论持久化协议

辩论过程通过 `popsicle discussion` 命令持久化，确保多角色讨论不丢失。

### 辩论开始时

```bash
# 创建讨论会话
popsicle discussion create --skill product-debate --topic "<讨论主题>" --run <run-id> --confidence <1-5>

# 注册参与角色
popsicle discussion role <discussion-id> --role-id pm --name "产品经理" --perspective "用户价值与优先级" --source builtin
popsicle discussion role <discussion-id> --role-id uxr --name "用户体验研究员" --perspective "用户行为与体验" --source builtin
# ... 其他参与角色
```

### 每次角色发言后

```bash
popsicle discussion message <discussion-id> \
  --role <role-id> --role-name "<显示名>" \
  --phase "Phase 1: 用户需求与问题定义" \
  --type role-statement \
  --content "<发言内容>"
```

### 用户输入后

```bash
popsicle discussion message <discussion-id> \
  --role user --role-name "User" \
  --phase "Phase 1: 用户需求与问题定义" \
  --type user-input \
  --content "<用户输入>"
```

### 暂停点

```bash
popsicle discussion message <discussion-id> \
  --role system --role-name "System" \
  --phase "Phase 1: 用户需求与问题定义" \
  --type pause-point \
  --content "请分享您对这个问题的看法？"
```

### 阶段小结

```bash
popsicle discussion message <discussion-id> \
  --role system --role-name "System" \
  --phase "Phase 1: 用户需求与问题定义" \
  --type phase-summary \
  --content "共识: ...\n分歧: ..."
```

### 辩论结束时

```bash
# 结束讨论并自动导出 Markdown
popsicle discussion conclude <discussion-id>
```

这将自动在 `.popsicle/artifacts/<run-id>/` 目录下生成 `<slug>.product-debate.discussion.md` 文件，完整记录讨论过程。

**最低要求**：至少在每个 Phase 的暂停点和阶段小结时调用 `discussion message`，确保关键决策节点被记录。

## 参考资源

### Reference 文件

- **`references/default-roles.md`** — 5 个内置通用角色定义（PM, UXR, GROWTH, ENGLD, BIZ）
- **`references/confidence-modes.md`** — 5 级置信度的详细行为规则和交互差异
- **`references/debate-phases.md`** — 4 个辩论阶段的详细说明、参与者、暂停点设计
- **`references/output-templates.md`** — PRD 草稿、会议纪要、决策矩阵的产出模板

### 项目配置文件

- **`docs/product-debate/roles.md`** — 5 个项目领域角色定义（DX, IOT, ENTERPRISE, DATA, TWIN）
- **`docs/product-debate/product-mapping.md`** — 6 个产品线 + 跨产品话题的角色映射表
