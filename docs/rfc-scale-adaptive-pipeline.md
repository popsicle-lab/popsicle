## Summary

本 RFC 实现了规模自适应的 Pipeline 推荐机制：将现有的 3 条 Pipeline 扩展为 5 条，覆盖从"仅测试"到"完整 SDLC"的完整复杂度谱；引入 `PipelineRecommender` 引擎，基于任务描述的关键词匹配自动推荐最合适的 Pipeline；并将推荐器与 Issue 系统集成，`issue start` 时自动根据 issue 的标题和描述选择 Pipeline。

## Motivation

### 问题 1：Pipeline 复杂度谱存在 gap

原有 3 条 Pipeline 覆盖了 3 个级别：

| Pipeline | 适用场景 |
|----------|---------|
| `full-sdlc` | 新功能开发 |
| `tech-sdlc` | 技术重构 |
| `test-only` | 补充测试 |

但两个常见场景缺少对应 Pipeline：

- **设计已完成，直接实现+测试**：设计文档（RFC/ADR）已有，但必须走 `tech-sdlc` 重新做架构辩论，或走 `test-only` 跳过实现
- **仅做前期规划，不涉及实现**：如可行性评估、架构探索，必须走 `full-sdlc` 然后中途停止

### 问题 2：Pipeline 选择依赖人工判断

用户需要自行判断任务该走哪条 Pipeline。一个小 bug 走 `full-sdlc` 是浪费，大功能走 `test-only` 则覆盖不足。缺少自动推荐机制。

### 问题 3：Issue 与 Pipeline 的映射过于粗糙

`IssueType::default_pipeline()` 是一对一硬编码映射，同一 `Technical` 类型下"重构认证模块"和"加个配置字段"需要完全不同深度的 Pipeline，但都被映射到 `tech-sdlc`。`Idea` 类型甚至没有映射（返回 `None`，直接报错）。

## Proposal

### 1. 五条 Pipeline 覆盖完整复杂度谱

新增 2 条 Pipeline，形成从轻到重的连续谱：

```
test-only → impl-test → tech-sdlc → full-sdlc
                                       ↑
                              design-only (正交)
```

| Pipeline | scale | 覆盖阶段 | 适用场景 |
|----------|-------|---------|---------|
| `test-only` | minimal | test-spec → test-codegen → quality | 为已有代码补充测试 |
| `impl-test` | light | impl → test-codegen → quality | 设计已明确，直接编码测试 |
| `tech-sdlc` | standard | arch-debate → rfc+adr → test-spec → impl → test-codegen → quality | 技术重构，需架构讨论 |
| `full-sdlc` | full | product-debate → prd → arch-debate → ... → quality | 新功能，完整产品+技术流程 |
| `design-only` | planning | product-debate → prd → arch-debate → rfc+adr | 前期规划，不涉及实现 |

每条 Pipeline 都是更完整 Pipeline 的严格子集（或正交补充），保证推荐逻辑的简洁性。

### 2. PipelineDef 扩展：声明式推荐元数据

在 `PipelineDef` 上新增两个可选字段：

```rust
pub struct PipelineDef {
    pub name: String,
    pub description: String,
    pub stages: Vec<StageDef>,
    #[serde(default)]
    pub keywords: Vec<String>,    // 用于匹配任务描述
    #[serde(default)]
    pub scale: Option<String>,    // "minimal" | "light" | "standard" | "full" | "planning"
}
```

每条 Pipeline 在 YAML 中声明自己的 `keywords` 和 `scale`：

```yaml
name: full-sdlc
scale: full
keywords: [feature, user story, product, 功能, 需求, 新功能, cross-module, 跨模块]
```

这保持了 Popsicle 的声明式哲学——Pipeline 定义"什么任务适合我"，引擎决定"哪个最匹配"。

### 3. PipelineRecommender 引擎

`PipelineRecommender::recommend(task: &str, pipelines: &[PipelineDef]) -> Recommendation`

匹配策略：

1. **分词 + 子串匹配**：对任务描述小写化后分词，同时对完整文本做子串匹配（兼容中文无空格分词）
2. **评分排序**：每条 Pipeline 按 keyword 命中数评分，相同得分时轻量级 Pipeline 优先
3. **Fallback**：无匹配时默认推荐 `tech-sdlc`（中等复杂度）

返回结构包含推荐结果、理由和备选方案：

```rust
pub struct Recommendation {
    pub pipeline_name: String,
    pub scale: String,
    pub reason: String,
    pub alternatives: Vec<Alternative>,
    pub cli_command: String,
}
```

### 4. CLI 命令：`popsicle pipeline recommend`

```bash
$ popsicle pipeline recommend "添加用户认证功能"
=== Pipeline Recommendation ===

  Task:      添加用户认证功能
  Pipeline:  full-sdlc (scale: full)
  Reason:    Matched keywords [功能] → scale 'full'

  Start with:
  $ popsicle pipeline run full-sdlc --title "<title>"

  Alternatives:
    - design-only (scale: planning) — Design and planning only
```

支持 `--format json` 输出供 Agent 消费。

### 5. Issue 集成：智能 Pipeline 选择

`popsicle issue start` 的 Pipeline 选择从硬编码映射升级为两层策略：

```
resolve_pipeline_for_issue(issue, pipelines):
  1. 用 issue.title + issue.description 调用 PipelineRecommender
  2. 如果有 keyword 命中 → 使用推荐结果（PipelineSource::Recommender）
  3. 如果无匹配 → fallback 到 IssueType::default_pipeline()（PipelineSource::IssueTypeDefault）
```

`IssueType::default_pipeline()` 更新：

| IssueType | 原映射 | 新映射 |
|-----------|-------|-------|
| Product | full-sdlc | full-sdlc（不变） |
| Technical | tech-sdlc | tech-sdlc（不变） |
| Bug | test-only | test-only（不变） |
| Idea | None（报错） | **design-only** |

实际效果：

- "实现缓存模块"（Technical）→ 推荐器命中 "实现" → `impl-test`（而非硬编码的 `tech-sdlc`）
- "重构数据库连接池"（Technical）→ 推荐器命中 "重构" → `tech-sdlc`
- "探索微服务拆分"（Idea）→ 推荐器命中 "探索" → `design-only`
- "修复登录 bug"（Bug）→ 无匹配 → fallback `test-only`

## Rationale and Alternatives

### 为什么用关键词匹配而非 LLM 分类？

关键词匹配是近期的低成本实现，有以下优势：

- **确定性**：相同输入始终产出相同推荐，无随机性
- **零延迟**：纯字符串操作，不需要 API 调用
- **可调试**：`reason` 字段明确显示匹配了哪些关键词
- **用户可扩展**：通过修改 pipeline YAML 中的 `keywords` 即可调整推荐行为

长期方向（设计哲学第 12 节）包括基于历史数据的学习模型，但当前阶段关键词匹配已足够。

### 为什么不完全移除 `default_pipeline()`？

保留 `default_pipeline()` 作为 fallback 有两个原因：

1. **稳健性**：即使所有 keyword 都不匹配（如极短的 issue 标题 "fix it"），仍有合理默认值
2. **向后兼容**：已有代码可能直接调用 `default_pipeline()`

### 为什么 `design-only` 是独立 Pipeline 而非 `full-sdlc` 的前缀？

虽然 `design-only` 的 stage 集合是 `full-sdlc` 的严格子集，但将其作为独立 Pipeline 有明确的语义边界——用户知道这条 Pipeline 到 RFC/ADR 就结束了，不会有"中途停止 full-sdlc"的模糊感。

## Open Questions

1. **推荐是否可以覆盖到 Pipeline 内部？** 对于 `full-sdlc`，是否建议跳过某些阶段（如已有架构的项目跳过 `arch-debate`）？
2. **是否需要支持用户在 `config.toml` 中自定义推荐规则？** 当前通过修改 pipeline YAML 的 `keywords` 即可实现一定程度的自定义。
3. **错误推荐的成本如何控制？** 推荐过轻（大功能走 `test-only`）比推荐过重（小修复走 `full-sdlc`）危害更大。是否需要"推荐偏重"的保守策略？
4. **长期方向**：是否根据历史 Pipeline run 数据（token 消耗、回退次数）学习推荐模型，自动调整阈值？
