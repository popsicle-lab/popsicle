# Skill 统一生命周期（feedback S2）

每个 skill 的 `workflow.states` 名字各异（`scanning→drafting→review→completed` /
`inventory→running→reporting→review→completed` / `scoping→implementing→verifying→completed` …），
学习成本高、难以横向比较「一个 skill 走到哪了」。本文件定义**统一生命周期语义**：五个规范阶段，
各 skill 的 bespoke state 只是它的**别名**，都映射到同一骨架。

> **兼容性**：这是**语义层**统一，不强制重命名既有 skill 的 state（避免破坏 catalog / 既有 run）。
> 运行时状态机本就是规范的（`skill-runtime` 用 `StateMachine::canonical()`）；per-skill 的 states
> 是文档/引导层。新 skill 建议直接采用规范名。

## 五个规范阶段

| 规范阶段 | 语义 | 典型别名 |
|---|---|---|
| **scope** | 划定输入、读上游 artifact/字段（见 `inputs.consumes`）、确定要产出什么 | scoping / scanning / inventory / setup / ingesting |
| **produce** | 干活：抽取 / 起草 / 实现 / 录制 / 重算，产出 artifact 主体 | drafting / implementing / running / capturing / tightening |
| **gate** | 机器可校验的自检（可执行谓词，非仅「有这些标题」）：命令退出码、数字重算、引用解析 | verifying / checking / scoring |
| **review** | 人验或独立验收（`requires_approval`），与 gate 正交（见 ROADMAP §10） | review / reporting / concluding / finalizing |
| **done** | 终态 | completed（`final: true`）|

## gate 阶段与机验轴的关系

`gate` 规范阶段对应两处真实机制：
1. **skill 内自检**：`workflow` guard 词表——从 `has_sections` / `checklist_complete`（文档层）
   升级到可执行谓词的方向（`command_exit_zero` / `manifest_recomputes` / `ref_resolvable` / `assert`）。
2. **pipeline 引擎 gate**：stage 的 `gate:` 声明，`stage complete` 时引擎实跑、`auto` 不可绕（ROADMAP §10）。

二者是同一「机验」理念在 skill 层与 pipeline 层的体现；`review` 才是人验轴。

## 新 skill 作者约定

新写 skill 时，`workflow.states` 尽量用 `scope → produce → (gate) → review → completed` 骨架，
guard 挂在 `produce → review` 转移上，`requires_approval` 挂在 `review → completed`。本仓库
S5 新增的 `golden-capture` / `traceability-gen` / `verifier` / `drift-detector` 已按此骨架编写，可作参考。

## 现有 skill 的映射（速查）

| skill | 现有 states | 映射 |
|---|---|---|
| fact-extractor | scanning→drafting→review→completed | scope→produce→review→done |
| prd-writer | ingesting→drafting→scoring→review→completed | scope→produce→gate→review→done |
| intent-consistency-check | checking→reporting→review→completed | produce→gate→review→done |
| equivalence-baseline | inventory→running→reporting→review→completed | scope→produce→gate→review→done |
| shadow-implementer | scoping→implementing→verifying→review→completed | scope→produce→gate→review→done |
| verifier (S5) | scope→verify→review→completed | scope→gate→review→done |
