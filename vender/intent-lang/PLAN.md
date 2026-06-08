# intent-lang 实现计划

## 项目概述

**定位**：一门通用的 intent-based 逻辑表达式语言，支持形式化验证（formally provable）。

**核心理念**：用户声明"意图是什么"（What），而非"如何实现"（How），系统自动验证意图的逻辑正确性。

**技术路线**：
- 方向 C — 结构化声明语法 + LLM 辅助生成
- L2 验证 — SMT 自动验证（Z3 solver）
- 实现语言 — Rust
- 领域插件架构 — 核心语言不变，通过插件适配不同领域
- 未来支持 WASM 编译，面向 Web Playground

**适用领域**：
- 软件开发（PRD → 意图 → 验证、API 契约、业务规则）
- 智能家居（语音指令安全验证、设备联动规则）
- 金融（交易合规、复式记账、反洗钱）
- 医疗（用药安全、剂量约束）
- 权限控制（RBAC/ABAC 策略验证）
- 更多领域通过插件扩展

**文档索引**：见 [README.md](README.md) 中的文档导航。

> **语法规范**已移至 [docs/lang/SPEC.md](docs/lang/SPEC.md)，此处不再重复。

---

## 项目结构

```
intent-lang/
├── Cargo.toml
├── README.md
├── crates/
│   ├── intent-syntax/           # M1: 语法定义 + Parser
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ast.rs           # AST 节点定义
│   │       ├── lexer.rs         # 词法分析
│   │       ├── parser.rs        # 语法分析（递归下降或 pest）
│   │       └── grammar.pest     # PEG 语法文件（如果用 pest）
│   │
│   ├── intent-core/             # M2: 语义分析 + VCGen
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── typeck.rs        # 类型检查
│   │       ├── vcgen.rs         # 验证条件生成
│   │       └── smt.rs           # SMT-LIB2 编码 + Z3 调用
│   │
│   ├── intent-cli/              # M3: 命令行工具
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   │
│   └── intent-llm/             # M4: LLM 翻译层
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           └── translate.rs     # 自然语言 → intent-lang
│
├── examples/                    # 示例 .intent 文件
│   ├── transfer.intent
│   ├── sorting.intent
│   └── auth.intent
│
└── tests/                       # 集成测试
    ├── parse_tests.rs
    ├── verify_tests.rs
    └── fixtures/
```

---

## 里程碑详细规划

### M1: 语法设计 + Parser

**目标**：实现完整的 lexer + parser，能将 `.intent` 文件解析为 AST。

**任务清单**：
1. 初始化 Rust workspace（Cargo workspace + crates 结构）
2. 定义 AST 数据结构（`ast.rs`）
   - `Program`, `TypeDef`, `EnumDef`, `IntentDef`, `TheoremDef`, `FunctionDef`
   - `Expr` 枚举（算术/布尔/比较/量词/蕴含/字段访问/prime）
   - `Type` 枚举（基础类型/自定义类型/泛型）
3. 实现 Lexer 或 PEG 语法
   - 推荐 `logos` (lexer) + 手写递归下降 parser（控制力最强）
   - 或使用 `pest` PEG parser（开发快，但错误信息需要额外处理）
4. 实现 Parser
   - 支持所有语法要素（type/enum/intent/theorem/function）
   - Pratt parsing 处理表达式优先级
   - 量词使用逗号分隔（`forall x: T, P(x)`）
   - 支持 `after(x)` 作为 `x'` 的别名，解析时脱糖为 primed 形式
5. 实现 pretty-printer（AST → 源码，用于调试和 LLM 输出格式化）
6. 单元测试 + 集成测试（解析示例文件）

**验收标准**：能正确解析上述语法草案中的所有示例代码。

---

### M2: 类型检查 + VCGen + SMT 验证

**目标**：对 AST 进行语义分析，生成验证条件，调用 Z3 完成验证。

**任务清单**：
1. 类型检查器（`typeck.rs`）
   - 环境/作用域管理（type/function/intent 符号表）
   - 表达式类型推导与检查
   - require/ensure/invariant 中表达式必须为 Bool 类型
   - Primed 变量的类型与原变量一致
   - 泛型实例化
2. 验证条件生成（`vcgen.rs`）
   - Intent 验证：`require ∧ invariant → ensure ∧ invariant'`
   - Theorem 验证：直接编码定理体为 SMT 查询
   - 处理量词（forall/exists）
3. SMT 编码（`smt.rs`）
   - VC → SMT-LIB2 格式字符串
   - 类型映射：Int→Int, Bool→Bool, String→String, struct→datatype
   - 调用 Z3 CLI（`z3 -smt2 -in`）或使用 `z3` crate bindings
   - 解析 Z3 输出：sat/unsat/unknown + 反例模型
4. 错误报告
   - 类型错误：定位到源码行列
   - 验证失败：展示反例（如 `sender.balance = 5, amount = 10`）
   - **隐式安全规则展示**：验证报告中列出所有被合并的 `safety` 规则，展示完整验证条件
     - 帮助 LLM 理解失败原因，支持自动修正闭环
     - 帮助用户审计验证过程，避免 safety 规则的黑盒效果
5. 测试
   - 正确意图应 pass
   - 错误意图应 fail 并给出反例

**验收标准**：能验证 transfer 示例的正确性 + 检测出有 bug 的变体。

**关键设计决策**：
- Z3 集成方式：推荐先用 CLI 模式（生成 .smt2 文件 → 调用 z3），成熟后切换到 bindings
- 反例展示：将 Z3 model 映射回源码变量名

---

### M3: CLI 工具

**目标**：提供用户友好的命令行界面。

**任务清单**：
1. 使用 `clap` 实现 CLI 参数解析
2. 子命令：
   - `intent check <file.intent>` — 解析 + 类型检查 + 验证
   - `intent parse <file.intent>` — 仅解析，输出 AST（调试用）
   - `intent fmt <file.intent>` — 格式化
3. 彩色输出（`colored` crate）
   - ✅ `Verified: TransferSafe` (绿色)
   - ❌ `Failed: TransferSafe — counterexample: sender.balance = 5, amount = 10` (红色)
4. 错误报告格式化（类似 rustc 风格的诊断信息）
5. Watch 模式（`intent check --watch`）可选

**验收标准**：`intent check examples/basics/transfer.intent` 输出清晰的验证结果。

---

### M4: LLM 翻译层

**目标**：支持自然语言描述 → intent-lang 代码的自动生成。

**任务清单**：
1. 设计 system prompt + few-shot examples
   - 包含语法规范 + 多个领域的示例
2. **构建 few-shot 示例库**
   - intent-lang 是新语言，LLM 训练数据中不存在，必须依赖 few-shot 学习
   - 按领域组织：金融、权限、智能家居、排序/数据结构等
   - 每个示例包含：自然语言描述 → intent 代码 → 验证结果
   - 示例应覆盖常见模式：简单 require/ensure、primed 变量（含 `after()` 别名）、量词、safety 规则组合
   - 目标：50+ 高质量示例，作为 LLM prompt 的上下文窗口素材
3. 实现 LLM API 调用（OpenAI/Anthropic/本地模型）
   - 使用 `reqwest` 或 `async-openai`
3. 生成后自动验证 + **反例驱动的修正闭环**
   - LLM 输出 → parser → typeck → verify
   - 如果失败，将**结构化反例**（`variable=value` 格式）反馈给 LLM 重试（最多 N 次）
   - 反例格式对 LLM 高度可解读（区别于 Lean 4 的 proof state 或 TLA+ 的长 trace）
   - 同时反馈被合并的 safety 规则，帮助 LLM 理解隐式约束
4. CLI 集成
   - `intent generate "确保转账金额不超过余额"` → 输出 intent 代码
   - 交互式模式：生成 → 展示 → 用户确认/修改 → 验证
5. 测试：用预设的自然语言描述测试生成质量

**验收标准**：能将简单的自然语言描述转为可通过验证的 intent 代码。

---

### M5: Web Playground（未来）

**目标**：浏览器内在线编辑器 + 实时验证。

- 将 intent-syntax + intent-core 编译为 WASM
- 前端使用 Monaco Editor + WASM 调用
- Z3 也有 WASM 版本（z3-wasm）
- 实时解析/类型检查/验证反馈

---

## 技术依赖

| 用途 | Crate | 备注 |
|------|-------|------|
| Lexer | `logos` | 高性能词法分析 |
| Parser | 手写递归下降 | 最佳错误信息控制 |
| SMT Solver | `z3` 或 CLI 调用 | 形式化验证核心 |
| CLI | `clap` | 参数解析 |
| 错误报告 | `ariadne` 或 `miette` | rustc 风格诊断 |
| 彩色输出 | `colored` | 终端美化 |
| LLM API | `reqwest` + `serde_json` | HTTP 调用 |
| 序列化 | `serde`, `serde_json` | AST 序列化/调试 |
| 测试 | `insta` | 快照测试（AST/SMT 输出） |

---

## 风险与应对

| 风险 | 影响 | 应对 |
|------|------|------|
| VCGen 逻辑错误导致 unsound | 高 | 参考 Dafny/Why3 论文；大量测试用例覆盖 |
| Z3 对复杂查询 timeout | 中 | 设置超时；提示用户简化意图；分拆验证条件 |
| 泛型 + 量词组合复杂度爆炸 | 中 | MVP 阶段限制泛型深度和量词嵌套 |
| LLM 生成代码质量不稳定 | 低 | 生成后强制验证；多轮重试；用户可手动修改 |

---

## 开发顺序建议

M1 → M2 → M3 → M4 → M5

M1-M3 为核心路径，形成可用的 CLI 工具。M4 为增值功能。M5 为展示层。
