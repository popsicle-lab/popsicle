# Domain Analysis Guide

领域分析是**项目级别的一次性活动**，在项目初始化时完成，不在每次 pipeline 流程中重复执行。分析结果作为项目基础上下文，供后续所有 pipeline run 引用。

## 何时执行

- **新项目初始化后**：`popsicle init` 之后，立即执行一次领域分析
- **项目方向重大变更时**：手动重新执行以更新领域模型
- **不在每次 pipeline run 中执行**：领域模型是稳定的项目基础，不随功能迭代变化

## 启动协议

领域分析支持两种模式，根据项目状态自动选择：

### 模式 1：已有代码库（自动分析）

当项目目录中已存在源代码时，自动分析代码结构：

1. 扫描项目目录结构、模块划分、包命名
2. 识别核心实体、服务、数据模型
3. 推断 Bounded Contexts 和模块边界
4. 分析依赖关系和模块间通信方式
5. 生成领域模型草稿，向用户确认

**分析重点**：
- 目录结构 → Bounded Contexts
- 数据模型/ORM/Schema → Entity Relationships
- 模块入口/聚合类 → Aggregate Roots
- 专业术语/命名约定 → Ubiquitous Language

### 模式 2：新项目（交互式创建）

当项目目录为空或无源代码时，通过对话引导用户：

1. **询问项目类型**："您打算做一个什么项目？" — 获取项目概述
2. **明确业务领域**：确定核心业务场景和用户角色
3. **划定边界**：协助用户划分 Bounded Contexts
4. **定义实体**：识别核心实体及其关系
5. **建立术语表**：统一领域术语

## Section Standards

### Bounded Contexts

Each bounded context must answer:
- What is its core responsibility?
- Where is the boundary with adjacent contexts?
- How does data cross boundaries (events, shared kernel, API)?

**Good:**
> **Order Context** manages the full order lifecycle from creation to fulfillment. It publishes `OrderPlaced` and `OrderShipped` events consumed by Payment and Shipping contexts. It never directly accesses payment or inventory data.

**Bad:**
> Order Context handles orders.

### Entity Relationships

Describe how core entities relate. Use "has-a", "belongs-to", "references". Clarify which relationships cross context boundaries.

### Aggregate Roots

Identify which entities serve as aggregate roots — entry points for consistency boundaries. Each aggregate should be independently transactable.

### Ubiquitous Language

Build a glossary of domain terms. Every term should have one unambiguous definition shared between dev and domain experts.

## 与后续流程的衔接

领域分析完成并 approved 后，产出的 `domain-model` 文档将：

- 作为 `product-debate` 的可选输入，帮助辩论参与者理解业务背景
- 作为整个项目的共享上下文，供所有 pipeline run 引用
- 存储在 `.popsicle/artifacts/` 目录，持久可用

## Common Mistakes

- **One big context**: putting everything into a single bounded context defeats the purpose
- **Entity vs Value Object confusion**: if it has no identity and is immutable, it's a value object
- **Missing domain events**: if one context needs to react to something in another, there should be an event
- **Premature technical decisions**: domain models should be technology-agnostic
