---
name: Pipeline-Adaptive Workflow
overview: 修复轻量 pipeline 因 skill 静态依赖而无法执行的核心矛盾。通过 pipeline 感知的 guard、自适应 prompt 上下文、advisor 预检查和 skill prompts 四层优化，让推荐的 pipeline 真正可用。
todos:
  - id: guard-pipeline-aware
    content: "Layer 1: 修改 guard.rs 使 upstream_approved 感知 pipeline，pipeline.rs 添加 all_skill_names()，doc.rs 传递 pipeline def"
    status: completed
  - id: prompt-adaptive
    content: "Layer 2: 修改 prompt.rs 区分 'missing' vs 'skipped by pipeline'，注入自适应引导消息"
    status: completed
  - id: advisor-enhance
    content: "Layer 3: advisor.rs 添加 guard 预检查和 pipeline 上下文 hints"
    status: completed
  - id: skill-prompts
    content: "Layer 4: 为所有 skill.yaml 添加关键状态的 prompts 内容"
    status: completed
  - id: test-verify
    content: 运行 cargo test 验证，补充 pipeline-aware guard 的新测试
    status: completed
isProject: false
---

# Pipeline-Adaptive Workflow 优化

## 核心问题

Pipeline Recommender 正确推荐了轻量 pipeline（如 `impl-test`），但 skill 的 `required: true` 依赖指向不在该 pipeline 中的上游 skill，导致 `upstream_approved` guard 必然失败。用户被迫回退到 `full-sdlc`。

具体依赖断裂表：

- **impl-test**: `implementation` 需要 `rfc`(required) + `adr`(required)，不在 pipeline 中
- **test-only**: `priority-test-spec`/`api-test-spec` 需要 `adr`(required)；`e2e-test-spec`/`ui-test` 需要 `prd`(required)
- **tech-sdlc**: `arch-debate`/`rfc` 需要 `prd`(required)，不在 pipeline 中

## 修复方案：四层优化

### Layer 1: Pipeline-Aware Guard（核心修复）

**原则**：如果 pipeline 设计者选择不包含某个 stage，那么该 stage 对应 skill 的产出物不应被 guard 要求。

**修改 `[guard.rs](crates/popsicle-core/src/engine/guard.rs)`**：

- `check_guard` 签名新增 `pipeline: Option<&PipelineDef>` 参数
- `check_upstream_approved` 在检查 required input 前，先判断 `input.from_skill` 是否在当前 pipeline 的 stages 中
- 不在 pipeline 中的 required input → 跳过检查（视为 pipeline context 下的 optional）

```rust
// guard.rs 核心逻辑变更
fn check_upstream_approved(
    doc: &Document,
    all_docs: &[DocumentRow],
    registry: &SkillRegistry,
    pipeline: Option<&PipelineDef>,  // 新增
) -> Result<GuardResult> {
    let skill = registry.get(&doc.skill_name)?;
    let pipeline_skills: Option<Vec<&str>> = pipeline.map(|p| p.all_skill_names());

    for input in &skill.inputs {
        if !input.required { continue; }
        // 新增：如果 from_skill 不在当前 pipeline 中，跳过
        if let Some(ref skills) = pipeline_skills {
            if !skills.contains(&input.from_skill.as_str()) { continue; }
        }
        // ... 原有检查逻辑 ...
    }
}
```

**修改 `[pipeline.rs](crates/popsicle-core/src/model/pipeline.rs)`**：

- 在 `PipelineDef` 上添加 `all_skill_names()` 方法，收集所有 stage 中的 skill 名称

**修改 `[doc.rs](crates/popsicle-cli/src/commands/doc.rs)`**：

- `transition_doc` 中调用 `check_guard` 时，加载 pipeline def 并传入
- 从 `doc.pipeline_run_id` 查找 pipeline run → 得到 `pipeline_name` → 加载 `PipelineDef`

### Layer 2: 自适应 Prompt 上下文

**原则**：当 pipeline 跳过了某个上游 skill 时，prompt 应指导 AI 直接从用户/代码库获取相关信息，而非显示"NOT YET CREATED"。

**修改 `[prompt.rs](crates/popsicle-cli/src/commands/prompt.rs)`**：

- `build_input_context` 加载 pipeline def（从 run_id → pipeline_name）
- 区分两种 missing 场景：


| 场景                       | 当前行为              | 优化后                                                                                                  |
| ------------------------ | ----------------- | ---------------------------------------------------------------------------------------------------- |
| skill 在 pipeline 中但文档未创建 | "NOT YET CREATED" | 保持不变                                                                                                 |
| skill 不在 pipeline 中（被跳过） | "NOT YET CREATED" | "This pipeline skips {skill}. Describe relevant {artifact} context directly in your implementation." |


### Layer 3: Advisor Guard 预检查 + Pipeline 上下文提示

**修改 `[advisor.rs](crates/popsicle-core/src/engine/advisor.rs)`**：

- `next_steps` 方法已经持有 `pipeline_def`，传递给 guard 做预检查
- 对有 guard 的 transition，预执行 `check_guard`；若失败且原因是缺少上游文档，标记为 `guard_warning` 而非推荐为可执行
- 在 `NextStep` 中添加 `hints: Vec<String>` 字段，注入 pipeline 上下文提示：
  - "This pipeline skips PRD/RFC — gather requirements from the user directly"
  - 若 project-context.md 不存在 → "Run `popsicle context scan` to improve AI context"

### Layer 4: Skill Prompts 填充

**修改所有 `skills/*/skill.yaml`**：当前所有 skill 的 `prompts` 字段为空，导致 `pipeline next` 无法给出有意义的 AI 指导。

为每个 skill 的关键状态添加 prompt，举例：

```yaml
# implementation/skill.yaml
prompts:
  planning: |
    Review the design documents (RFC, ADR) and create an implementation plan.
    Break down into components, define implementation order, and identify key interfaces.
    Use: popsicle prompt implementation --state planning --run {run_id}
  coding: |
    Implement components per the plan. Follow project coding conventions.
    After each significant commit: popsicle git link --doc {doc_id} --stage implementation
```

```yaml
# arch-debate/skill.yaml
prompts:
  setup: |
    Load the PRD context and prepare an architecture debate.
    Identify 4-6 expert roles relevant to the technical domain.
    Use: popsicle prompt arch-debate --state setup --run {run_id}
```

需要添加 prompts 的 skill（按优先级）：

1. `implementation` — planning, coding
2. `arch-debate` — setup, debating
3. `rfc` — draft
4. `prd` — gathering
5. `priority-test-spec` — discovery
6. 其余 skill — 至少初始状态

## 修改文件清单


| 文件                                           | 变更类型  | 说明                                                       |
| -------------------------------------------- | ----- | -------------------------------------------------------- |
| `crates/popsicle-core/src/engine/guard.rs`   | 核心修改  | `check_guard` + `check_upstream_approved` 增加 pipeline 感知 |
| `crates/popsicle-core/src/model/pipeline.rs` | 新增方法  | `PipelineDef::all_skill_names()`                         |
| `crates/popsicle-core/src/engine/mod.rs`     | 调整导出  | 更新 `check_guard` 的 re-export 签名                          |
| `crates/popsicle-cli/src/commands/doc.rs`    | 调用调整  | `transition_doc` 传递 pipeline def                         |
| `crates/popsicle-cli/src/commands/prompt.rs` | 上下文优化 | `build_input_context` 区分 "missing" vs "skipped"          |
| `crates/popsicle-core/src/engine/advisor.rs` | 增强    | Guard 预检查 + hints + `NextStep` 新字段                       |
| `skills/*/skill.yaml` (16 个)                 | 内容补充  | 添加 `prompts` 到关键状态                                       |


## 测试验证

修改完成后运行 `cargo test` 确保所有现有测试通过，特别是 `guard.rs` 和 `advisor.rs` 中的测试。补充新测试：

- guard 在有 pipeline 参数时跳过非 pipeline skill 的检查
- guard 在无 pipeline 参数时保持原有行为
- prompt 对 pipeline-skipped 输入生成不同的占位文本

