# Functional E2E — 业务驱动功能测试引擎

铁律：`先发现场景，再写 SPEC，最后生代码。人只审 SPEC，不审代码。`

## When To Use

- 新功能交付前的端到端功能验收
- 涉及认证、授权、隔离、配额等高风险边界的变更
- 跨多个服务的业务流程验证
- 实体生命周期状态转换测试
- 不同部署形态下的行为差异验证

不适用：
- 单接口参数化测试（输入/输出/错误码穷举）→ `api-test-gen`
- UI 界面功能测试 → `ui-test`
- Rust 单元测试 → `priority-test-gate`

## 六个测试视角

所有功能测试场景归入以下六个视角之一。设计 Spec 时，对每个涉及的领域逐一过六个视角，确保无遗漏。

| 视角 | 代号 | 聚焦点 | 典型问题 |
|------|------|--------|---------|
| 正向功能 | POS | 核心业务流程正确执行 | "注册后能正常使用所有功能吗？" |
| 边界条件 | BND | 高风险边界的防御能力 | "跨租户请求会泄漏数据吗？超限请求会 OOM 吗？" |
| 跨服务链路 | CHN | 数据在多个服务间流转的完整性 | "认证服务发的令牌在数据服务能正确鉴权吗？" |
| 安全不变量 | INV | 必须永远成立的安全规则 | "JWT 优先于 Header？跨租户返回统一 404？" |
| 状态生命周期 | LFC | 实体从创建到销毁的状态转换 | "设备从注册到上报到停用的每个转换都正确吗？" |
| 部署形态 | DPL | 不同部署模式下的行为差异 | "SaaS 专属 API 在私有部署下返回什么？" |

## References — 业务知识库

业务知识分两层存放：**共享 References** 存放在 `testsuite/specs/` 下供多 Skill 复用；**Skill 特有** 的格式规范保留在本 Skill 的 `references/` 目录下。

### Reference 文件清单

| 文件 | 位置 | 内容 | 共享/Skill特有 |
|------|------|------|---------------|
| `domains.yaml` | `testsuite/specs/domains.yaml` | 领域注册表 | 共享 |
| `domain-map.md` | `testsuite/specs/references/domain-map.md` | 业务领域地图 | 共享 |
| `service-catalog.md` | `testsuite/specs/references/service-catalog.md` | 服务目录与依赖关系 | 共享 |
| `entity-lifecycle.md` | `testsuite/specs/references/entity-lifecycle.md` | 关键实体状态机 | 共享 |
| `security-invariants.md` | `testsuite/specs/references/security-invariants.md` | 安全不变量清单 | 共享 |
| `deployment-parity.md` | `testsuite/specs/references/deployment-parity.md` | 部署形态差异矩阵 | 共享 |
| `spec-format-guide.md` | `references/spec-format-guide.md`（本 Skill 目录下） | Spec YAML 自然语言格式规范 | Skill 特有 |

### 如何构建平台特定的 References

首次使用本 skill 或业务发生重大变更时，按以下指引构建或更新 references。

#### 1. 领域地图（domain-map.md）

**信息来源**：PRD 文档、产品功能地图

**构建方法**：
1. 读取所有 PRD 文档，提取业务领域划分
2. 读取产品功能地图（如有），提取功能模块列表
3. 按 "领域 → 子领域 → 关键功能" 三级结构整理
4. 标注每个领域涉及的服务和核心实体

**包含信息**：领域 ID、名称、描述、涉及服务、核心实体、关键功能列表

#### 2. 服务目录（service-catalog.md）

**信息来源**：Proto 文件、ADR、服务启动代码

**构建方法**：
1. 扫描所有 `.proto` 文件，提取 service 和 rpc 声明
2. 读取 ADR 中的服务架构决策
3. 建立服务间依赖关系（调用关系、共享实体 ID）
4. 标注每个服务的监听地址和认证要求

**包含信息**：服务名称、职责、RPC 列表（含请求/响应结构摘要）、依赖服务、认证方式

#### 3. 实体生命周期（entity-lifecycle.md）

**信息来源**：Proto message 定义、ADR 状态描述、服务端业务逻辑

**构建方法**：
1. 识别 Proto 中的核心实体（凡是有 CRUD 操作的都是）
2. 分析每个实体的状态集合和合法转换
3. 标注触发每个转换的操作和必要前置条件
4. 标注每个状态下允许和禁止的操作

**包含信息**：实体名称、状态列表、状态转换表（from → trigger → to → 断言条件）

#### 4. 安全不变量（security-invariants.md）

**信息来源**（按推导优先级）：
1. 威胁建模（STRIDE 六类分析）
2. OWASP API Security Top 10 行业标准
3. ADR 架构决策中的安全约束
4. 合规要求（等保 / GDPR）
5. 多租户行业最佳实践（AWS / Azure / GCP）
6. 敏感数据流分析（跟踪密码、令牌、Key 的全链路）
7. Bug 修复记录中的安全缺陷
8. 代码审计发现的违规模式

**构建方法**：
1. **STRIDE 威胁建模**：对每个服务和数据流，逐一分析 Spoofing / Tampering / Repudiation / Information Disclosure / DoS / Elevation of Privilege 六类威胁
2. **OWASP 对照**：逐条过 OWASP API Security Top 10，标注"已覆盖 / 部分覆盖 / 缺失"
3. **ADR 提取**：从认证、授权、隔离相关 ADR 中提取"必须永远成立"的约束
4. **合规映射**：从等保 / GDPR 条款中提取强制性技术要求
5. **数据流追踪**：跟踪每种敏感数据（密码、JWT、API Key）从产生到销毁的路径，在每个边界点检查泄漏风险
6. **Bug 回溯**：从安全 Bug 修复中提炼规则，确保同类问题不再发生
7. 每个不变量包含：规则描述、反模式、守护代码位置（如有）、Quick-Check 命令（如有）
8. 编号规则：INV-XX，全局唯一递增

**包含信息**：编号、规则、反模式、来源（STRIDE/OWASP/ADR/合规/数据流/Bug）、守护位置、Quick-Check 命令

**维护规则**：每次测试发现新的安全缺陷，修复后必须同步新增 INV 条目。

#### 5. 部署形态差异（deployment-parity.md）

**信息来源**：PRD 部署形态描述、代码中的 feature flag / 条件编译

**构建方法**：
1. 读取 PRD 中的部署形态功能对比表
2. 扫描代码中的条件编译标记
3. 列出每个差异点：功能名称、各形态下的预期行为
4. 标注哪些接口在哪些形态下不可用

**包含信息**：差异矩阵表格（行=功能/接口，列=部署形态，单元格=预期行为）

### References 更新时机

| 触发事件 | 更新哪些 references |
|---------|-------------------|
| 新功能 PRD 发布 | domain-map |
| 新 ADR 被 Accepted | service-catalog、security-invariants |
| Proto 新增 service/rpc | service-catalog、entity-lifecycle |
| 安全 Bug 发现或修复 | security-invariants（走"安全 Bug 反馈闭环"流程） |
| 代码审计 / 渗透测试发现 | security-invariants（走"安全 Bug 反馈闭环"流程） |
| 新增部署形态或 feature flag | deployment-parity |

## File Layout

```
testsuite/specs/
├── domains.yaml                    # 领域注册表（多 Skill 共用，只读）
├── references/                     # 共享业务知识（多 Skill 共用）
│   ├── domain-map.md
│   ├── service-catalog.md
│   ├── entity-lifecycle.md
│   ├── security-invariants.md
│   └── deployment-parity.md
└── functional/                     # 功能测试 Spec（Stage 2 产出）
    ├── auth/                       # ← 领域子目录，名称对应 domains.yaml 中的 domain key
    │   └── login.yaml
    ├── device-data/
    │   └── lifecycle.yaml
    └── tenant/
        └── isolation.yaml

tests/functional/                   # 功能测试代码（Stage 3 产出）
├── auth/                           # ← 领域子目录，与 specs/functional/ 镜像
│   ├── __init__.py
│   └── test_login.py              # ← 对应 specs/functional/auth/login.yaml
├── device-data/
│   ├── __init__.py
│   └── test_lifecycle.py
└── tenant/
    ├── __init__.py
    └── test_isolation.py
```

命名映射：
- 领域子目录: 对应 `testsuite/specs/domains.yaml` 中的 domain key（kebab-case）
- Spec: `testsuite/specs/functional/{domain}/{name}.yaml`（kebab-case）
- Code: `tests/functional/{domain}/test_{name}.py`（kebab → snake_case，加 `test_` 前缀）
- 示例: `specs/functional/auth/login.yaml` → `tests/functional/auth/test_login.py`
- 每个领域子目录需要 `__init__.py`（pytest 包发现）

隔离规则：
```
只读: docs/, crates/*/proto/, testsuite/conftest.py, testsuite/lib/,
      testsuite/specs/domains.yaml, testsuite/specs/references/
只写: testsuite/specs/functional/**/*.yaml, tests/functional/**/*.py,
      testsuite/specs/references/security-invariants.md（安全 Bug 反馈闭环时可写）
不写: tests/api/, tests/ui/, crates/*/src/
```

## Workflow — 三阶段

### Stage 0: References 检查（每次触发必执行）

1. 检查 `testsuite/specs/references/` 目录下的共享业务知识库是否完整
   - 缺失任何平台特定文件 → 按上述指引构建
   - 文件存在但内容明显过时（如 Proto 新增了服务但 service-catalog 未更新）→ 先更新
2. 检查 `testsuite/specs/domains.yaml` 中领域注册表是否与 proto/PRD 一致
3. 读取 `testsuite/specs/references/security-invariants.md`
   - 如有 Quick-Check 命令 → 执行，确认全部通过
   - 若任何 Quick-Check 失败 → 优先修复回归，再进入 Stage 1

### Stage 1: 场景发现（Discover）

**输入**：用户指定的功能范围（领域名称 / 模块名称 / 变更描述 / PRD 引用）

**执行步骤**：

1. **加载 References** — 读取领域地图、服务目录、实体生命周期、安全不变量、部署差异
2. **定位变更范围** — 根据用户输入，确定涉及的领域、服务、实体
3. **六视角场景推导** — 对涉及的每个领域，从六个测试视角分别推导候选场景：
   - **POS**：该领域的核心业务流程是什么？正常路径是否覆盖？
   - **BND**：有哪些高风险输入组合？维度交叉后有多少测试点？
   - **CHN**：数据跨了哪些服务？每个交接点的数据是否正确传递？
   - **INV**：涉及了哪些 INV-XX 条目？是否需要新增？
   - **LFC**：涉及的实体有哪些状态转换？异常转换是否被拒绝？
   - **DPL**：该功能在不同部署下行为是否不同？
4. **输出发现报告** — 展示给用户：
   - 涉及的领域和服务
   - 每个视角的候选场景列表（含简短描述）
   - 各场景的建议优先级
   - 预估的总测试点数

**等待用户确认**：确认范围和优先级后进入 Stage 2

### Stage 2: Spec 设计（Design）— 阻塞等审阅

**执行步骤**：

1. 为确认范围内的每个领域生成一个 Spec YAML 文件
2. 每个场景使用**自然语言**描述（格式详见 `references/spec-format-guide.md`）
3. 各视角的特殊处理：
   - **BND**：用自然语言描述测试矩阵的维度和排除逻辑，标注展开后的预估测试点数
   - **INV**：引用 INV-XX 编号，写明护城河断言（精确到可直接翻译为 assert）
   - **LFC**：以"状态 → 触发 → 目标状态"的自然语言链条描述
   - **DPL**：以对偶矩阵描述各形态下的预期行为差异
4. 将 Spec 写入 `testsuite/specs/functional/{domain}/{name}.yaml`（若领域子目录不存在则创建）
5. 呈现完整 Spec 给用户审阅

**等待用户回复**：
- "通过" / "approved" → 进入 Stage 3
- 具体修改意见 → 修改 Spec 后重新呈现
- "拒绝" / "reject" → 终止，不生成代码

### Stage 3: 代码生成（Generate）— 需 Stage 2 approved

**执行步骤**：

1. **读取测试基建** — 读取 conftest.py 和 lib/ 目录，确认可用的 fixtures 和工具函数
2. **逐场景生成测试** — 将 Spec 中每个场景翻译为 pytest 测试代码：
   - POS / CHN / LFC → 多步骤顺序测试函数，步骤间通过变量传递上下文
   - BND → 参数化测试（`@pytest.mark.parametrize`），按矩阵维度展开
   - INV → 精确断言测试，Case ID 对应 INV-XX 编号
   - DPL → 双模式对比测试，用 profile marker 过滤执行
3. **Self-Consistency Check** — 逐条对照 Spec 和代码：
   - 每个场景都有对应测试函数或测试类
   - 每个验证要点都有对应 assert
   - 边界矩阵展开数与预估一致
   - 无 Spec 未声明的多余测试
4. **安全不变量同步** — 如有新增 INV 条目，同步更新 `testsuite/specs/references/security-invariants.md`
5. **输出一致性报告** — 列出 Spec 场景 → 测试函数的映射表

## Test Code Conventions

- 文件头标注 Spec 来源路径和领域名称
- 类命名: `Test_{视角代号}_{场景短名}` — 如 `Test_CHN_CrossTenantIsolation`
- 函数命名: `test_{视角小写}_{描述}` — 如 `test_bnd_cross_tenant_device_write`
- 标记组合: `pytest.mark.{priority}` + `pytest.mark.{profile}` + `pytest.mark.{视角小写}`
- 边界矩阵的 Case ID 嵌入 `@pytest.mark.parametrize` 的 `ids` 参数，支持 grep 追溯
- 多步骤测试中每步都有 assert，任一步失败立即中止
- 使用 conftest.py 现有 fixtures（如 tenant_factory、role_token_factory），不重复定义
- 步骤间通过变量传递上下文，变量名与 Spec 中的"捕获"字段对应
- gRPC 错误断言使用 `pytest.raises(grpc.RpcError)` + `e.code()` 检查

## Priority Strategy

**P0（必须覆盖）**：
- 核心业务正向流程（用户旅程的 happy path）
- 安全不变量守护（INV-XX 回归）
- 信任边界防御（跨租户隔离、权限矩阵核心路径）

**P1（应当覆盖）**：
- 完整实体生命周期状态转换
- 跨服务数据一致性
- 资源边界（配额、限流、大请求体）

**P2（可选覆盖）**：
- 部署形态对偶验证
- 非核心状态转换分支
- 数据边界（时间戳乱序、格式校验、压缩炸弹）

## Output Contract

| 阶段 | 产出 |
|------|------|
| Stage 0 | References 完整性检查报告 + Quick-Check 结果（PASS/FAIL） |
| Stage 1 | 场景发现报告（六视角覆盖、候选场景列表、优先级建议、预估测试点数） |
| Stage 2 | `testsuite/specs/functional/{domain}/*.yaml` + 场景摘要展示 |
| Stage 3 | `tests/functional/{domain}/test_*.py` + Spec↔Code 一致性报告 + 安全不变量更新记录 |
| 安全 Bug 反馈 | 更新后的 `testsuite/specs/references/security-invariants.md` + 联动影响提示 |

本 Skill 只生成 Spec 和测试代码，不执行测试。执行由 CI 或人工完成。

## 安全 Bug 反馈闭环

测试执行（CI 或人工）发现安全类 Bug 后，按以下流程更新安全不变量，形成"发现 → 修复 → 固化规则 → 守护"的闭环。

### 触发条件

以下任一情况发生时启动本流程：
- 测试执行发现安全相关的失败（跨租户泄漏、权限绕过、信息泄露等）
- `bug-tracker` 登记了安全类型的 Bug
- 代码审计或渗透测试发现安全缺陷
- 安全 Bug 修复完成后

### 操作流程

用户向 Agent 提供以下信息即可触发更新：

**输入格式**（自然语言即可）：
- Bug 编号或描述（如 "BUG-2026-0015: Viewer 角色可以调用 DeleteThingModel"）
- 根因分析（如 "DeleteThingModel handler 未调用 RbacChecker"）
- 修复方案（如 "在 handler 入口添加 RbacChecker::check(role, ManageSchema)"）

**Agent 执行步骤**：

1. **查重** — 读取 `testsuite/specs/references/security-invariants.md`，检查是否已有 INV 条目覆盖此类问题
   - 已有且规则完整 → 仅标注"已知违规"到对应 INV 条目，不新增
   - 已有但规则不够严格 → 更新已有 INV 条目的规则描述
   - 无覆盖 → 新增 INV 条目
2. **提炼规则** — 从具体 Bug 中抽象出通用规则（不是记录 Bug 本身，而是提炼"什么必须永远成立"）
3. **写入 INV 条目** — 按标准格式追加到 `testsuite/specs/references/security-invariants.md`：
   - 规则、反模式、来源（Bug 编号）
   - 守护代码位置（修复代码在哪）
   - Quick-Check 命令（如何快速验证规则是否成立）
4. **呈现变更** — 展示新增/更新的 INV 条目，等待用户确认
5. **联动更新** — 如果新 INV 条目影响已有 Spec，提示用户是否需要重新触发 Stage 1-3 生成补充测试
