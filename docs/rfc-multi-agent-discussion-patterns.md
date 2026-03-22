# RFC: 多Agent多角色讨论架构模式对比

> 日期: 2026-03-22

## 背景

在LLM多角色讨论场景中，存在三种主流架构模式。本文基于前沿论文梳理各模式的核心机制、优劣势，并给出选型建议。

---

## 三种架构模式

### 1. 多Agent多Session — 各Agent独立进程

每个Agent运行在独立Session中，拥有独立上下文、记忆和模型参数，通过消息传递协作。

**代表论文:**
- **MoA** (Wang et al., 2024) — 分层流水线架构，Layer N独立生成 → Layer N+1聚合综合，开源模型超越GPT-4o
- **iAgents** (Liu et al., NeurIPS 2024) — 信息不对称下Agent协作，每个Agent私有记忆+InfoNav推理，支持30+轮自主通信
- **DyLAN** (Liu et al., COLM 2024) — 动态组队+自适应通信拓扑，按Agent重要性评分选取最优子集

**最佳实现:** [AG2 (原AutoGen)](https://github.com/ag2ai/ag2) — 支持Group Chat/Swarm/Sequential/Nested多种编排，生态最成熟

### 2. 单Session顺序多角色 — 依次扮演

单一LLM在同一Session中依次切换角色，每次只激活一个角色发言，前序角色输出作为后续角色的输入。

**代表方法:** Solo Performance Prompting (SPP), Self-Debate, Self-Collaboration

### 3. 单Session并发多角色 — 同上下文模拟

单一LLM在同一次生成中同时模拟多个角色的讨论。

**代表方法:** Multi-Perspective Prompting, Internal Debate

---

## 横向对比

| 维度 | 多Agent多Session | 单Session顺序 | 单Session并发 |
|------|:---:|:---:|:---:|
| **角色分化** | 强 | 中 | 弱 |
| **观点多样性** | 高（可用不同模型） | 中 | 低（同源偏差） |
| **信息隔离** | 天然隔离 | 部分隔离 | 无隔离 |
| **成本** | 高（多次调用） | 中 | 低（单次调用） |
| **延迟** | 高 | 中 | 低 |
| **上下文利用** | 各自独立 | 共享顺序使用 | 共享且交叉污染 |
| **实现复杂度** | 高 | 低 | 最低 |
| **适用场景** | 复杂决策/真实辩论 | 头脑风暴/多视角分析 | 快速多角度检查 |

---

## 核心洞察：单Session并发角色分化弱的根因

三重机制叠加导致角色不可避免地趋同：

**1. 自回归锚定** — token从左到右生成，后面的角色被前面的角色输出锚定，不存在真正并发。

**2. 自洽性压力** — Loss函数优化模型生成连贯一致的文本。"A说X、B反驳X"天然违背训练目标，模型倾向让角色趋同而非真正对立。

**3. 注意力交叉污染** — Self-attention让所有token互相可见，角色间无信息屏障。多Agent系统中各Agent上下文物理隔离，仅通过显式消息交互。

> Du et al. (2023) 实验验证：独立LLM实例辩论的观点多样性显著优于单实例自辩论，后者2-3轮后多样性快速坍缩。

**一句话:** 同一组权重在同一上下文中无法真正"与自己disagree"。

---

## 选型建议

```
需要真正的观点对立和深度辩论？ → 多Agent多Session
需要多视角但预算有限？         → 单Session顺序
只需要快速sanity check？       → 单Session并发
```

对于 Popsicle 项目中的多角色讨论场景，建议采用**多Agent多Session**作为核心架构，必要时用**单Session顺序**做轻量级补充。

---

## 参考文献

- [Mixture-of-Agents](https://arxiv.org/abs/2406.04692) (Wang et al., 2024)
- [iAgents](https://arxiv.org/abs/2406.14928) (Liu et al., NeurIPS 2024)
- [DyLAN](https://arxiv.org/abs/2310.02170) (Liu et al., COLM 2024)
- [AG2/AutoGen](https://arxiv.org/abs/2308.08155) (Wu et al., 2023)
- [Multiagent Debate](https://arxiv.org/abs/2305.14325) (Du et al., 2023)
- [Multi-Agent Survey](https://arxiv.org/abs/2402.01680) (Guo et al., 2024)
