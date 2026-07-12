# intent-coder 到生产路线图

> 本文件是**活文档**（现状 + 计划），随进展更新；不写历史叙事，过期内容就地修正。
> 方向性决策固化在 §2，后续如需更严格可抽成独立 ADR。
>
> Last-Updated: 2026-06-09

---

## 1. 现状快照

### 1.1 已经有什么

| 资产 | 状态 | 位置 |
|---|---|---|
| **intent-lang** | ✅ 已进仓库（0.1.0，Rust + Z3 可运行）| `intent-lang/`（crates: `intent-syntax` / `intent-core` / `intent-cli`）|
| 4 个已实现 skill | ✅ | `skills/{project-init,fact-extractor,product-debate,prd-writer}/` |
| 任务图范式（v0.3）| ✅ | `prd-writer` 产出「五件套」 |
| 端到端 demo | ✅（但 `.intent` 语法待对齐）| `example/saas-demo/` |
| popsicle 工作流引擎 | ✅ | `popsicle/`（**已降级为 intent-coder 私有引擎**，不再当通用平台维护——见 D4 / §5）|

### 1.2 intent-lang 真实形态（关键事实）

- **可运行实现**：lexer + parser + 类型检查 + VC 生成 + Z3，**非纯文档**。
- **验证入口**：`intent check --format json <file>` → 调系统 `z3` → exit 0（全过）/ 1（任一失败，含反例）。
- **真实语法**：`type` / `enum` / `function` / `intent`（`require` / `ensure` / `invariant`）/ `safety` / `theorem` / `axiom` / `goal` / `coverage`，Hoare 风格 + 后置态 `x'`，支持 `forall` / `exists` / `==>`。生命周期标注 `@tobe` / `@asis`。
- **明确不支持**：时序逻辑（LTL/CTL）、状态机、时间/性能 DSL（`within` / `eventually` / `never` 等）。POSITIONING 建议这类需求用 TLA+。
- **成熟度**：Alpha（核心链路通，测试偏少，`intent generate` / `fmt` / `intent-llm` 未实现）。

### 1.3 核心矛盾

仓库里**并存两套互不兼容的 `.intent` 语法**：

| | intent-lang（真实）| intent-coder 产出（saas-demo + prd-writer 模板）|
|---|---|---|
| 顶层构造 | `intent` / `safety` / `theorem` / `goal` | `acceptance` / `invariant` / `contract` |
| 行为描述 | `require` / `ensure` / `invariant` | `given` / `when` / `then` |
| 时间/性能约束 | **不支持** | 大量使用（`5秒内`、`P95<200ms`、`within()`）|
| 能否被 `intent check` 解析 | ✅ | ❌ **一行都过不了** |

**能力边界错位**：intent-lang 擅长「逻辑一致性 / 不变量」；而 saas-demo 的 `acceptance.intent` 塞满了 intent-lang 明确拒绝的「时间性能约束」。

---

## 2. 关键决策（固化）

| ID | 决策 | 理由 | 否决的替代 |
|---|---|---|---|
| **D1** | **路线甲**：intent-coder 适配 intent-lang 真实语法，**不**扩展 intent-lang 去吃 IDD 的 `given/when/then` | 尊重 intent-lang 的 POSITIONING；工作量集中在改模板/demo，可控 | 路线乙（扩 intent-lang 加时序/时间算子）：违背其定位、工作量大、污染语言 |
| **D2** | 时间 / 性能类约束**降级为测试断言**，写进 task 文件「可观察的成功标志」，**不进 Z3** | intent-lang 不做时序；这类约束本就该由 benchmark / e2e 测试守，不是 SMT | 硬塞进 `.intent`：制造「假形式化」，Z3 根本消费不了 |
| **D3** | `intent-spec-writer`（种子 → 正式 intent）**从外部契约位收回内置** | intent-lang 已进仓库，这一步直接决定能否喂给 Z3，是闭环必经环节，不该外包 | 保持外部：闭环断在仓库外，无法 dogfood |
| **D4** | **放弃通用平台定位**：popsicle 降级为 intent-coder 私有引擎，**不再做 RFC D2 拆分**（`ContentProvider` / `popsicle-doc-helpers` 全部取消）；废除「core 不准知道 PDR/ADR/task」边界铁律，允许 popsicle 直接长出 intent-coder 语义 | 通用平台养不起、无精力维护；只有一个 module 时泛化抽象是纯过度设计（YAGNI）；贵的是那条规则不是 crate | RFC D2 渐进拆分（旧 D4）：为不存在的第二个 module 交泛化税 |

> 闭环的第一刀切在 **invariants 层**，不是 acceptance 层——因为 `invariants.intent` 正好是 intent-lang 的主场。

---

## 3. 三层 intent ↔ intent-lang 构造映射

| IDD 三层（intent-coder）| 映射到 intent-lang | 能否进 Z3 |
|---|---|---|
| `invariants.intent` | `safety` / `invariant` | ✅ 完美契合 |
| `contracts.intent` | `intent`（`require` / `ensure` 模块操作契约）| ✅ 契合 |
| `acceptance.intent`（逻辑类，如「2FA 启用后不可关闭」「recovery code 一次性」）| `intent` / `theorem` | ✅ 可映射 |
| `acceptance.intent`（时间/性能类，如「P95<200ms」「90 秒内」）| —— | ❌ 不进 Z3，降级为测试断言（D2）|

**路线甲对 saas-demo 的直接后果**——`acceptance.intent` 会「瘦身 + 分流」。以 auth 为例：

| 原 acceptance block | 处理 |
|---|---|
| `T-0001 signup ≤ 90s` / `T-0010 P95<200ms` / `T-0011 OAuth<500ms` | 移出 `.intent` → task「可观察的成功标志」当测试断言 |
| `T-0002 recovery codes = 10 且只显示一次` | 保留 → intent-lang `intent`/`ensure` |
| `T-0021 recovery code 一次性` | 保留 → intent-lang `intent` |
| `T-0010 错误信息不区分用户/密码` | 保留 → intent-lang `intent` |

结果：`acceptance.intent` 从「一堆伪形式化时间约束」收敛成「少量逻辑契约」，**invariants.intent 成为 Z3 验证主力**。

---

## 4. 主线：打通「意图 → 机器判定」闭环

### Phase 0 — 地基对齐 + 第一个真实 PASS/FAIL ⭐ 最高优先级

| # | 任务 | 产出 |
|---|---|---|
| 0.1 | 把 intent-lang 封装成 popsicle tool | `tools/intent-validate/tool.yaml`（`command: intent check --format json {{path}}`）|
| 0.2 | 新建最小 `intent-consistency-check` skill，调用 0.1 的 tool、解析 JSON、**先观察不阻塞** | `skills/intent-consistency-check/` |
| 0.3 | 用 intent-lang 真实语法重写 `auth/invariants.intent`，跑出第一个真实 PASS | 改写后的 `.intent` |
| 0.4 | 改 `prd-writer` 种子模板，今后产出 intent-lang 合法语法（invariants/contracts 层）；时间约束写进 task 备注 | `skills/prd-writer/templates/acceptance-intent-seed.intent` |

**验收**：
- `intent check` 在 `auth/invariants.intent` 上输出真实 PASS；
- 故意注入矛盾（如同时要求 `no-self-revoke` 和一条允许自我吊销的规则）→ 输出 FAIL + 反例；
- `prd-writer` 不再产出无法解析的伪形式化文本。

**✅ Phase 0 已完成（2026-05-13）**

| # | 产出 | 实证 |
|---|---|---|
| 0.1 | `tools/intent-validate/{tool.yaml,guide.md}` | 封装 `intent check`；PATH 无 `intent` 时回退仓库内 `intent-lang/target/release/intent-cli`，并校验 z3。渲染命令实跑通过 |
| 0.2 | `skills/intent-consistency-check/`（skill.yaml + guide + report 模板）| observe 模式：枚举 `.intent` → 调 tool → 解析 JSON → 出报告，不阻塞；guard 已对照 popsicle 源码核对 |
| 0.3 | 重写 `auth/invariants.intent` | `EnableTwoFactor` / `ResetVia2FARecovery` **verified**，`EmailUniqueness` theorem skipped，`exit 0` |
| 0.4 | 重写 `acceptance-intent-seed.intent` + 同步 quality-rubric | 实测 2 个 intent verified、exit 0；rubric 去掉 `within()` 等伪算子 |
| 验证 | 注入矛盾（2FA 启用却 `totpEnabled=false`）| **FAILED + Z3 反例** `u.totpEnabledAt=0, u.totpEnabled=false` |

**关键 dogfood 发现（已写进各 guide，后续 Phase 必须遵守）**：

1. **后态须 primed**：`safety`/`invariant` 子句写 unprimed 只验「旧态」→ 假通过。约束操作**结果**必须用 `x'`（见 `vcgen.rs`：`ensure` 进 assumes，safety goal 直接取用户写的表达式）。
2. **一文件 = 一验证作用域**：vcgen 把每条 `safety` **无条件注入**文件内所有 intent，且靠**参数名**绑定。不相关操作放各自文件，否则自由变量会误判 FAIL。→ 约定：`acceptance.intent` 只放操作 intent，保持型不变量进 `invariants.intent`。
3. **struct-typed `forall` theorem 当前 skipped**（未实现）：email/session 唯一性这类「双实体关系」暂只能**声明**不能**验证**，记入 intent-lang 待补清单。
4. **工具链坑**：tool 需 `popsicle tool install ./tools/intent-validate` 装到 `.popsicle/tools/`；`tool run --format json` 会**双层包裹**（intent JSON 在内层 `stdout`）；FAIL 时 `exit 1` 会让 `tool run` 报错，observe 模式须当数据处理。

### Phase 1 — 「种子 → 正式 intent」内置化

| # | 任务 |
|---|---|
| 1.1 ✅ | 新建 `intent-spec-writer` skill：把 prd-writer 的种子收紧成 intent-lang 合法语法（D3）|
| 1.2 ✅ | 完善 `intent-consistency-check`：反例展示、「观察 → 门禁」退出判据（连续 N 迭代零偏差才升级为闸门）|
| 1.3 ✅ | 把 saas-demo 全部 9 份 `.intent`（auth / billing / admin-console 三件套）迁到 intent-lang 真实语法 |

**✅ Phase 1 已完成（2026-05-13）**：

- **1.1 `intent-spec-writer`**（skill.yaml + guide + 2 模板）：承接 prd-writer 种子，做
  五件事——分层归位（acceptance/invariants/contracts）、剥离 D2 约束（登记到 task）、
  四规则审查、与现有 `.intent` 去重查冲突、交付前 `intent check` 自验。`formal-acceptance.intent`
  模板实跑 **verified、exit 0**。定位刻意保持薄：只做语法收紧与冲突检查，不发明语义。
- **1.2 `intent-consistency-check` 收紧**：明确 **observe = skill 行为 / gate = CI 行为**
  （gate 不是 skill 状态，而是 CI 跑 `intent-validate` tool 靠 exit code 拦合并）。
  新增量化退出判据 `Gate Readiness`：`consecutive_clean_runs >= 3` 且本次 pass → `gate_ready`，
  附 CI 开闸 YAML 片段；强化反例展示（要求写「哪个字段=什么值违反哪条约束」+ 常见根因）。
  guard 已加 `Gate Readiness` section 校验。

**✅ Phase 1.3 已完成（2026-05-13）**：saas-demo 9 份 `.intent` 全部真实语法、`intent check` 全绿——**17 intent verified，4 theorem skipped，0 failed**。分层落地：

- `acceptance.intent` = **操作后置规约**（require/ensure，trivial verified）：auth 9 + billing 3 + admin 1。
- `invariants.intent` = **Z3 真验证的安全不变量**（safety + primed + 完整 ensure）：auth「2FA 保持」、billing「降级清零 seats」、admin「跨账户查询必审计」各 1 条 verified。
- `contracts.intent` = **契约位**（`goal` 块声明意图 + 注释登记 `[Awaiting ADR]`，0 VC）。
- 聚合类（`single-active` / `no-double-charge` 的 `count`）与双实体关系（`prorate-symmetric` / 唯一性）→ struct-forall theorem，当前 **skipped**（intent-lang 待补）。
- 时间 / 性能 / 运行时事实（p95、retry 间隔、卡号不入库、明文密码不入日志）→ 全部剥离到 task「可观察的成功标志」（D2）。

**补充 dogfood 发现（已写进各文件头 + guide）**：

5. **无 frame 假设**：intent-lang **不**默认「未提及字段不变」。要声明操作不改某字段，必须显式 `ensure x' == x`，否则该 primed 字段自由 → 若被 invariant/safety 约束则必 FAIL。
6. **纯 require+ensure = trivial verified**：只有 `invariant`/`safety` 子句产生 goals，`ensure` 只是假设。所以「操作规约」本身不被证伪，真正的一致性验证来自 invariants 的 safety。
7. **不支持聚合**：无 `count` / `where`；集合基数约束只能转写成双实体关系 theorem，且当前 skipped。

### Phase 2 — 文档保活 + 编排

| # | 任务 |
|---|---|
| 2.1 ✅ | 新建 `living-doc-author` skill（多处模板已挂钩 `--target tasks-index` 等，防 doc-code drift）|
| 2.2 ✅ | 写 `migration-bootstrap` pipeline YAML，串联 4 + 新增 skill 成 DAG，结束手动逐个 `skill start` |

### Phase 4 — slice 交付链（v0.4，2026-06-09）

| # | 任务 | 产出 |
|---|---|---|
| 4.1 ✅ | `shadow-implementer` / `equivalence-baseline` / `cutover-author` 三 skill | `skills/{shadow-implementer,equivalence-baseline,cutover-author}/` |
| 4.2 ✅ | `slice-spec` + `slice-delivery` pipeline | 与 migration-bootstrap 分离，避免 14 stage |
| 4.3 ✅ | `living-doc-author` 扩展 delivery target | implementation-status / architecture-manifest / product-header |

**✅ Phase 2 已完成（2026-05-13）**：

- **2.1 `living-doc-author`**（skill.yaml + guide + sync-report 模板）：保活/对账 skill，
  扫四类 doc-code drift（过期 / 断链 / 孤儿 / 未验证），按 `--target`
  （tasks-index / task-backrefs / last-verified / product-context / all）刷新活文档
  元数据、`tasks/README` 健康度、反向引用、`last_verified` 回填。红线：**只对账不创作正文**，
  正文 drift 转「待人工处置」走 prd-writer + PDR。
- **2.2 `migration-bootstrap` pipeline**（`pipelines/migration-bootstrap.pipeline.yaml`）：
  按 popsicle `PipelineDef` 真实 schema 串起全部 7 个自带 skill 成 DAG——
  `init → facts → debate → prd → intent-spec → intent-check → living-docs`，
  4 个审批点（init/debate/prd/living-docs）。已校验：字段合法、依赖指向存在的 stage/skill、
  拓扑无环。**关键变化**：`intent-spec-writer` Phase 1 起内置，不再是 bootstrap.md 里
  的外部契约位【】；只剩架构 ADR / `contracts.intent` 最终形式留给 Phase 3 外部 writer。

### Phase 3 — 技术侧 writer（已决定：内置）

| # | 任务 |
|---|---|
| 3.1 ✅ | `arch-debate` / `rfc-writer` / `adr-writer`：决定 `contracts.intent` 的最终形式 |

**✅ Phase 3 已完成（2026-05-13，决策：全内置）**：技术侧三件套与产品侧完全对称落地——

- **`arch-debate`**（skill.yaml + guide + tech-roles + 3 模板）：product-debate 的技术侧
  对称体。技术角色 ARCH（主持）/ SEC（代言攻击者）/ PERF / OPS / DATA / DEV，复用
  setup→debating→concluding 四 Phase + 强制暂停机制。消费 PRD § Intent Mapping 标
  `contracts` / [ADR 候选] 的行，产 rfc-draft + tech-decision-matrix + 辩论纪要。
- **`rfc-writer`**（skill.yaml + guide + 3 模板）：prd-writer 的对称体。产 RFC（含
  ARCHITECTURE.md 增量 File Manifest）+ contracts.intent 种子（`[Awaiting ADR]`，
  goal 块，实跑 `intent check` 0 VC exit 0）+ ADR 骨架（Proposed）。质量门 ≥ 90。
- **`adr-writer`**（skill.yaml + guide + 2 模板）：技术决策审批闸。固化 ADR
  Proposed→Accepted（不可变），解锁 contracts 种子的 `[Awaiting ADR]`，列收紧工单交
  intent-spec-writer。刻意保持薄（不发明内容），对称 intent-spec-writer 的「固化+解锁」定位。
- **职责不重叠的关键设计**：arch-debate 辩论 → rfc-writer 起草骨架 → adr-writer 固化+解锁
  → intent-spec-writer 收紧 → intent-consistency-check 验证。固化（不可变审批）与起草
  （可反复改）分离，且 adr-writer 兼做 contracts 解锁触发器。
- **pipeline 扩成 10 stage DAG**：在 prd 与 intent-spec 间插入 arch-debate→rfc→adr
  技术侧支线（无 contracts 候选时整段可 skip，popsicle 视 skipped 为依赖已满足）。
  已校验：字段合法、依赖指向存在的 stage/skill、拓扑无环、7 个审批点。
- **D2 一致贯彻**：性能/时延/容量不进 contracts goal（写 RFC § Quality Attributes，
  压测守护）；契约逻辑前后置待 ADR Accepted 后才收紧进 acceptance/invariants。
- bootstrap.md 流程图的两个外部虚线框（架构/RFC/ADR + intent spec writer）已改为内置实心；
  README skill 表 6→10、依赖顺序重排、pipeline 链路与使用步骤更新。

---

## 5. 支线：popsicle 降级为私有引擎 + 通用遗产瘦身

**决策（D4）**：放弃「通用工作流平台」定位——养不起、无精力。popsicle 不再是
intent-coder 所依附的通用 module 宿主，而是 **intent-coder 的私有引擎**。
`popsicle/docs/rfc-workflow-only-core.md`（RFC D2）随之**作废**：它的全部动机是多
module 复用，单 module 下是过度设计。

**作废的**（不再做）：

- ❌ `ContentProvider` trait 间接层（RFC 步骤 1）。
- ❌ 拆 `popsicle-doc-helpers` crate（RFC 步骤 2–6）。
- ❌ 「popsicle-core 不准知道 L0–L6 / PDR / ADR / task / intent」这条边界铁律 ——
  **正式废除**。从此允许 popsicle 直接长出 intent-coder 专用的 command / 表 / UI 视图。

**保留的**（能跑就别动）：skill 状态机、pipeline DAG、run / spec ledger、memory、
git 集成——泛化成本低且已在用，拆它才是过早优化。`markdown` / `extractor` / `guard`
留在原地，并可大胆加 intent-coder 专用解析（如 frontmatter + task-graph 提取器），
不再纠结「够不够通用」。

**新主线（替代旧 RFC 落地）：通用遗产瘦身**。盘点 popsicle 为「通用平台」野心写的
模块，区分 intent-coder 真用 vs 纯遗产，列可删清单：

| 模块 | intent-coder 是否真用 | 处置（待逐项核实）|
|---|---|---|
| skill / pipeline / run / spec | ✅ 核心 | 保留 |
| memory | ✅（dogfood 用）| 保留 |
| git 集成 / commit links | ✅ | 保留 |
| `markdown` / `extractor` / `guard` / `context` | ✅（可视化 + guard 用）| 保留，可加专用解析 |
| `namespace` | ⚠️ 待核 | 若仅平台多租户用途 → 候选删 |
| `issue` | ⚠️ 待核 | 与 spec / run 是否重叠 → 候选合并 / 删 |
| `work_item`（story / bug / test extractor）| ⚠️ 待核 | 与 task 图是否重叠 → 候选删 / 改造 |
| `popsicle-sync`（CRDT / http）| ⚠️ 待核 | 多端协作是平台特性 → 候选删 |
| `doc` / `checklist` / `migrate` / `prompt` CLI 命令族 | ⚠️ 待核 | 按 intent-coder 实际用量逐个裁 |

> 原则：**先核实再删**（每项确认无 intent-coder 引用后才动）。这是新的「简化」靶子，
> 比拆 doc-helpers 更能真正减重，但优先级仍低于闭环——闲时核实、攒够一批再砍。

---

## 6. 明确不做（范围边界）

- ❌ **无约束的代码/测试 codegen**：shadow-implementer 只按 ADR File Manifest + intent
  范围编排 in-shadow 实现与 property test，不发明 scope 外 API，不替代人工架构判断。
- ❌ **扩展 intent-lang 支持时序/时间逻辑**：见 D1/D2，那是 TLA+ 的领域。
- ❌ **重写 / 大改 popsicle 既有可跑能力**：见 D4 / §5，降级 ≠ 重写；能跑的引擎能力保留，只裁通用遗产，且先核实再删。
- ❌ **维护 popsicle 的通用平台定位 / RFC D2 拆分**：见 D4，已放弃。

---

## 7. 风险与未决

| 风险 / 未决 | 缓解 |
|---|---|
| intent-lang 是 Alpha，表达力可能撑不起真实 invariant（如 billing `prorate-symmetric`「增减对称」）| Phase 0 用 saas-demo 真实约束做压力测试；表达不了的暴露出来 = intent-lang 的待补清单 |
| `intent check` 依赖系统 PATH 上有 `z3` | tool/skill 启动时检查 z3 可用性，缺失给明确报错 |
| 路线甲会让 acceptance.intent 大幅瘦身，部分「可验证性」转移到测试侧而测试侧暂无 skill | 接受现状；测试生成在边界外，靠 task「可观察的成功标志」+ 人工/CI 守 |
| `intent-spec-writer` 内置化增加 intent-coder 重量 | 保持薄：只做「种子语法收紧 + 冲突检查」，不做语义发明 |
| 通用遗产瘦身与闭环并行可能分散精力 | D4 已锁定：popsicle 降级为私有引擎，瘦身先核实再删、闭环优先；不做 RFC D2 拆分 |
| 废除边界铁律后 popsicle 与 intent-coder 语义耦合加深，未来若真要第二个 module 需补做抽象 | 接受：YAGNI——只有一个 module 时耦合是正确取舍；真出现第二个 module 再当一次性重构处理 |

---

## 8. 一句话状态

> Phase 0-3 spec 链闭环 + **v0.4 delivery 链**：intent-lang 封装为 `intent-validate` tool；**13 个自带 skill**
> （project-init / fact-extractor / product-debate / prd-writer / **arch-debate / rfc-writer /
> adr-writer** / intent-spec-writer / intent-consistency-check / living-doc-author /
> **shadow-implementer / equivalence-baseline / cutover-author**）spec 10 + delivery 3 全内置；
> `migration-bootstrap`（仓库级 10 stage）+ `slice-spec` + `slice-delivery` 三 pipeline；
> saas-demo 9 份 `.intent`
> 真实语法全绿（17 verified / 4 skipped / 0 failed），注入矛盾跑出 FAIL + Z3 反例。整条
> 「需求 → PRD 任务图 → 架构辩论 → RFC/ADR → intent 收紧 → 机器验证 → CI 闸门 → 活文档保活」
> 链全程走通。**路径调整（2026-06-04，D4）**：放弃通用平台定位，popsicle 降级为
> intent-coder 私有引擎，RFC D2 拆分作废，新支线改为「通用遗产瘦身」（§5）。
> **剩余边界外项**：无约束 codegen、popsicle 通用遗产瘦身（D4/§5）、intent-lang 待补能力。

---

## 9. 迁移等价强化（提案，来自使用反馈 #11–#17）

真实迁移（arrow-simple → astra-faber）跑通后暴露的方法论级缺口。**已落地**为脚手架/模板默认；
**提案中**的项待专用 skill 后再固化。

**已落地（模板 / 技能 / 脚手架默认）：**
- **#11 结构化事实基**：`fact-extractor` 产 `facts.yaml`（`fact_id` + `kind` + `legacy@sha:path#L` 溯源 +
  `evidence`，含 `behavior` 事实/收割 legacy 测试为 golden 候选）；5 份 markdown 降级为其渲染。
- **#12 迁移第三轴**：task frontmatter 增 `migrates_from` + `equivalence{golden_id,status}`，
  task ↔ legacy ↔ golden ↔ intent 覆盖矩阵可机器派生。
- **#13 方法论**：`@asis 优先 → 逐条决定保留/改进 → @tobe`，intent 当等价 oracle，golden 骨架从
  `facts.yaml` 派生，漂移检测（换 commit 重跑 diff facts）。写进 intent-spec-writer / fact-extractor guide。
- **#15 RFC 持久归宿**：`rfc-writer` 落 `products/<p>/proposals/<lifecycle>/RFC-NNNN` + 双向回链 +
  `legacy_pin`/`source_artifact`。
- **#16 仓库级决策位**：`project-init` 铺 `docs/decisions/{adr,pdr,cadr}/`，`ADR-G-`/`PDR-G-`/`CADR-` 编号。
- **#17 真相源**：Charter 明确 Accepted 后 decisions/ 为准、proposals 原地加状态标记不 move；迁移期 RFC 收敛。

**已落地（本轮补全 #13 pipeline / #14 tool）：**
- **`migration-preserve` 快车道 pipeline（#13）**：`intent-coder/pipelines/migration-preserve.pipeline.yaml`，
  7 stage：facts → intent-spec(@asis) → intent-check → implement → equivalence → cutover → living-docs，
  **复用现有 skill**、刻意跳过 debate→prd→arch→rfc→adr 重设计链。（未来若加 `@asis`-capture /
  golden-skeleton 专用 skill 可进一步特化，但当前 DAG 已可跑：所有跨阶段 skill 输入依赖满足。）
- **#14 整目录一个 program（opt-in）**：新增 `intent-validate merge=true`，把每个产品
  `intents/*.intent` 合并成单 program 跑一次 `intent check`，跨文件 `realized_by` 同作用域解析、
  **无 W0010**。⚠️ 这是**更严格**的整程序语义（合并后 `safety` 无条件应用到所有 intent，未约束后态的
  操作 intent 会因自由变量 FAIL），是「有意识检查跨文件交互」的诊断模式，**非盲目消噪**。默认 per-file
  行为不变；W0010 本身无害（合并 goal 追溯闸已权威校验 realized_by）。popsicle 侧实现，无需上游改动。

## 10. 迁移门禁可信化（已落地，来自使用反馈 #18–#22 / H1–H6 / P3–P5）

**核心**：给引擎加「机验 gate 轴」，`stage complete` 时**先实跑 gate 再推进**，任何
`approval_mode`（含 `auto`）都不能绕过；gate（机器验）与 approval（人验）**正交两轴**。

**已落地（引擎 / bundled pipeline / 模板）：**
- **可执行 gate 引擎（#18/#19/P3）**：pipeline stage 声明白名单结构化 gate，引擎在
  `stage complete` 内、approval 之前**实跑**（`crates/cli-ux/src/gate.rs`）：
  - `command_exit_zero` —— 在工作区根跑命令读退出码（如 `cargo test`）；
  - `assert` —— manifest 字段满足 `op value`（数字/字符串，支持 dotted path + glob 取最新）；
  - `manifest_recomputes` —— 汇总字段 == 从明细列表按 `where` 重算的计数（**抓手工编造的 golden 数字**）；
  - `ref_resolvable` —— `realized_by` 目标追溯可解析（复用 `intent_goal_trace`）。
  失败即 `gate:<stage>:<name> — <可复算证据>`，fail-closed。
- **gate/approval 正交（P4/H6）**：gate 永远执行、`auto` 不可绕；`requires_approval` +
  `approval_mode`(manual/auto/delegate-dangerous) 仍是独立的**人验**轴。`pipeline next` 输出 gate 提示。
- **gate-only stage（P5）**：`skill` 为空且有 `gate` 的 stage —— `pipeline next` 提示「run gate」，
  `stage complete` 只评估 gate、不产 artifact。
- **bundled pipeline 挂 gate**：`migration-slice-delivery` / `migration-preserve` 的 `cutover` 挂
  `cargo test` + `summary.golden_pass>=5` + `manifest_recomputes` + `equivalence_gate_pass==true` +
  `legacy_pin!=占位符` + `ref_resolvable`；`equivalence` 挂 `ref_resolvable`。
- **#21 真实 legacy_pin**：`baseline.yaml` 模板改用占位符 `REPLACE_WITH_REAL_LEGACY_PIN`（附注入指令），
  cutover gate 校验 `legacy_pin != 占位符` —— 未填真实 pin 即拦切流。
- **#18/#22 verbatim vs rewrite**：`equivalence-report` / `task` 加 `migration_mode: verbatim|rewrite`；
  verbatim 显式声明 golden=characterization（快照自证，别假装差分），rewrite 才需 legacy 录制+回放差分；
  `shadow`/`strangler-fig` 措辞加 verbatim 例外说明（`shadow-implementer` guide）。
- **#20 @asis 暴露**：`intent-consistency-check` 指导用 `intent-validate include_asis=true`
  让 `@asis` 进 Z3，并产「@asis ↔ @tobe 分叉」报告段（有意分叉须对应 divergence+ADR）。

## 10b. 迁移方法论补全（已落地，来自 S1–S5 / P1–P2 / #20 upstream）

**已落地（skill / schema / 引导 / CLI）：**
- **S5 五个新 skill**（`intent-coder/skills/`）：
  - `port` —— verbatim 平移的实现 skill（对偶 shadow-implementer 的 rewrite），产 `port-coverage`，
    登记 legacy 段↔new 段，`cargo build` guard，诚实声明 golden=characterization（不套 shadow 叙事）。
  - `golden-capture` —— rewrite 切片起 pinned legacy 录真实输出为可回放 fixture，产机器可复算
    `golden-capture-manifest`（每条带 `recorded_exit` + `stdout_sha256`）。
  - `traceability-gen` —— 从 task `migrates_from`/`equivalence` + `baseline.yaml` 机器派生覆盖矩阵，
    自动检出「缺 golden / 未对齐 / 缺 intent / 孤儿 golden」四类缺口。
  - `verifier` —— 独立验收（H6：执行/验收分离），复算门禁并出 verdict，破「运动员=裁判」。
  - `drift-detector` —— 换 legacy pin 重跑 fact-extractor，按 `fact_id`+`source` diff，反查受影响下游。
- **S1 字段级 inputs**：`inputs` 加 `consumes:`（如 `facts[kind=behavior].golden_candidate`），
  声明消费的具体字段/ID，机器可校验衔接（当前契约级；Rust 侧忽略未知字段）。
- **S2 统一生命周期**：`intent-coder/skills/LIFECYCLE.md` 定义 `scope→produce→gate→review→done`，
  bespoke state 映射到规范骨架（不破坏性重命名既有 skill）。
- **S4 RFC-inline-ADR**：`adr-writer` guide 增「冻结/等价保留」类决策直接写自包含 ADR（`rfc_inline: true`），
  免 RFC↔ADR 双文档复述（#17）。
- **P1 决策树 spec 就绪分叉**：`issue-author` guide 决策树顶部加「spec 是否已具备」显式分叉，
  spec 已具备 + 行为保留 → `migration-preserve`（不再误选 delivery）；文档化「共享交付尾」。
- **P2 delivery→spec 回流**：`issue-author` guide 增「交付中发现 spec 缺口」回流闭环
  （另开 spec issue / ADR 登记 / doc-retro-spec，禁止 implement 里偷改 `.intent`）。
- **#20 CLI 贯通**：`popsicle tool run intent-validate include_asis=true`（接受 `true/1/yes` 或原始
  `--include-asis`）现真正把 `@asis` 送进 Z3——此前 CLI 的 goal-trace 包装层会吞掉该参数，已修复
  （`run_intent_validate_opts` 增 `include_asis` 形参贯通到 merged 与 per-file 两路）。

**仍然另行立项（属上游 / 大重构，本次不做）：**
- 让 `@asis` **默认**进 Z3、或**自动**生成 @asis↔@tobe 分叉报告 → 属**上游 intent-lang** 语义
  （popsicle 侧已把 `include_asis` 贯通到位；默认策略与自动分叉需改 vendored intent-lang）。
- **S3 破坏性拆分 / S4 破坏性合并**：把 `implement` stage 真正拆成 port/implement 两 stage、
  删 `product-debate`+`arch-debate` 合并为 `design-debate --scope` —— 会打断引用旧 skill 的 bundled
  pipeline 与在途 run，需迁移脚本；当前以**加性 skill + 文档统一心智**过渡。
- **P1 引擎级共享 stage 组**（pipeline yaml `include`）、**P2 引擎级阶段回退边 / spec↔impl 双向绑定**
  —— 状态机层改动，2 个内置 pipeline 尚不值当；当前内联同步 + 可操作回流闭环。
- gate 谓词扩展（正则 `matches`、跨 manifest 交叉校验、网络类断言）——白名单四类已覆盖迁移主诉求。

> 安全说明：`command_exit_zero` 会在 `stage complete` 时于工作区根执行 pipeline yaml 里的命令字符串。
> bundled pipeline 可信；**自定义 pipeline 的 gate 命令同样会被执行**，审阅第三方 pipeline 时须留意。
