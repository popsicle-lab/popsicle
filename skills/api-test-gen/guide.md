# API Integration Test — Spec 驱动单接口测试

铁律：

```
先生 SPEC，再生 CODE。人类只审 SPEC，不审测试代码。
```

## 核心理念

Spec 是人类可读的测试计划大纲。不读代码就知道测了什么。
Spec 由 Agent 生成，人类审阅后 Agent 再生成测试代码。

## 适用范围

- 单接口的输入输出正确性验证（gRPC / HTTP）
- 覆盖 M1-M9 标准检查矩阵的 case 生成

不适用：
- 跨接口 E2E 链路 → 使用 `story-e2e`
- UI 测试 → 使用 `ui-test`
- Rust 单元测试 → 使用 `priority-test-gate`
- 深度边界组合覆盖 → 使用 `boundary-tdd`

## 文件布局

```
testsuite/specs/
├── domains.yaml                    # 领域注册表（多 Skill 共用，只读）
├── references/                     # 共享业务知识（多 Skill 共用，只读）
│   ├── domain-map.md
│   ├── service-catalog.md
│   ├── entity-lifecycle.md
│   ├── security-invariants.md
│   └── deployment-parity.md
├── api/                            # 接口测试 spec（本 skill 产出 — Stage 1）
│   ├── auth/                       # ← 领域子目录，名称对应 domains.yaml 中的 domain key
│   │   ├── login.yaml
│   │   ├── refresh-token.yaml
│   │   └── create-user.yaml
│   ├── vera/
│   │   ├── report-properties.yaml
│   │   └── list-thing-models.yaml
│   └── arca/
│       └── request-upload-url.yaml
├── conftest.py                     # 共享 fixtures（不修改）
└── lib/                            # proto stubs、工具类（不修改）

tests/                              # 测试代码（项目根目录下，本 skill 产出 — Stage 2）
├── api/                            # 接口测试代码，与 testsuite/specs/api/ 镜像对应
│   ├── auth/                       # ← 领域子目录，与 specs/api/ 镜像
│   │   ├── __init__.py
│   │   ├── test_login.py           # ← 对应 specs/api/auth/login.yaml
│   │   ├── test_refresh_token.py
│   │   └── test_create_user.py
│   ├── vera/
│   │   ├── __init__.py
│   │   ├── test_report_properties.py
│   │   └── test_list_thing_models.py
│   └── arca/
│       ├── __init__.py
│       └── test_request_upload_url.py
└── conftest.py                     # 从 testsuite/conftest.py 导入共享 fixtures
```

**命名映射规则：**
- 领域子目录: 对应 `testsuite/specs/domains.yaml` 中的 domain key（kebab-case）
- Spec 文件: `testsuite/specs/api/{domain}/{method}.yaml`（kebab-case）
- 测试文件: `tests/api/{domain}/test_{method}.py`（kebab → snake_case，加 `test_` 前缀）
- 示例: `specs/api/auth/login.yaml` → `tests/api/auth/test_login.py`
- 每个领域子目录需要 `__init__.py`（pytest 包发现）

**隔离规则：**
- 只读: `testsuite/specs/domains.yaml`、`testsuite/specs/references/`
- 只写: `testsuite/specs/api/**/*.yaml`（Stage 1）和 `tests/api/**/*.py`（Stage 2）
- 不写: `testsuite/specs/functional/`、`testsuite/conftest.py`、`crates/`

## 接口定义来源

### 来源 1: Protobuf（gRPC）

从 `.proto` 文件提取接口定义：

1. 定位 proto 文件 — 在 `crates/*/proto/` 下搜索
2. 提取 `service` + `rpc` 声明 → `endpoint` section
3. 递归展开 Request message → `request_fields` section
4. 递归展开 Response message → `success_response` section
5. 阅读服务端 Rust 实现代码 → `business_rules` section（proto 无法自动推导）

**Proto 特殊处理：**
- proto3 默认所有字段可选，`required: true` 需根据业务语义判断（阅读服务端校验逻辑）
- `optional` 修饰符标记为 `proto_optional: true`（可区分未传和零值）
- M3（类型错误）在 proto 协议层已保证，标注 `proto_layer: true` 即可
- enum 未知值在 proto3 默认保留不报错，需确认服务端是否额外校验

### 来源 2: HTTP

从路由定义或 OpenAPI spec 提取：

1. 提取 HTTP method + path → `endpoint` section
2. 提取 path params → `path_params` section
3. 提取 request body schema → `request_fields` section
4. 提取 response schema → `success_response` section

## Workflow — 两阶段

### Stage 1: 生成 Spec（阻塞等待审阅）

**输入：** 用户指定接口（RPC 名称 / HTTP 路由 / proto 文件路径）

**执行步骤：**

1. **定位接口定义** — 找到对应的 proto 文件或 HTTP 路由定义
2. **提取接口元信息** — 按 `references/spec-template.md` 填充上半部分
   - endpoint、auth、request_fields、success_response、error_codes
3. **提取业务规则** — 阅读服务端实现代码，提取该接口特有的行为逻辑
4. **生成 checks** — 按 `references/coverage-matrix.md` 的 M1-M9 逐项推导
   - 每条 case 写明 `id`、`input`（自然语言）、`expect`（自然语言）
   - input/expect 面向非技术人员可读
5. **输出 spec 文件** — 写入 `testsuite/specs/api/<domain>/<method>.yaml`
6. **呈现 spec 给用户** — 显示完整 checks section，请求审阅

**等待用户回复：**
- "approved" / "通过" → 进入 Stage 2
- "加 XX case" / "删掉 M5-03" / 其他修改意见 → 修改 spec 后重新呈现
- "reject" / "拒绝" → 终止，不生成代码

### Stage 2: 生成代码（需 Stage 1 approved）

**执行步骤：**

1. **确定目标文件** — 保持领域子目录，将 spec 文件名 kebab-case 转为 snake_case，加 `test_` 前缀
   - 例: `testsuite/specs/api/auth/login.yaml` → `tests/api/auth/test_login.py`
   - 若目标领域子目录不存在，创建目录和 `__init__.py`
2. **读取 conftest.py** — 确认可用的 fixtures（grpc_channel、role_token_factory 等）
3. **逐条生成测试** — 将 checks 中每条 case 翻译为 pytest 测试函数
   - Case ID 嵌入 `@pytest.mark.parametrize` 的 `ids` 参数
   - 测试函数名格式: `test_{M类别}_{简短描述}`
4. **文件头标注来源** —

```python
"""
Generated from: testsuite/specs/api/{domain}/{method}.yaml
Endpoint: {service}/{rpc}
"""
```

5. **Self-Consistency Check** — 逐条对照 spec 和代码
   - 验证 checks 中每条 case 都有对应的测试函数
   - 验证没有 spec 中未声明的多余测试
   - 输出一致性报告

## Test Code Conventions

- Case ID 嵌入 parametrize ids: `@pytest.mark.parametrize(..., ids=["M1-01", "M1-02"])`
- Profile 过滤: `@pytest.mark.saas_only` / `@pytest.mark.standalone_only`
- 共享 fixtures 来自 `testsuite/conftest.py`，不重复定义
- gRPC 错误断言使用 `pytest.raises(grpc.RpcError)` + `e.code()` 检查

## Output Contract

1. **Stage 1 输出** — `testsuite/specs/api/<domain>/<method>.yaml` + checks 摘要展示
2. **Stage 2 输出** — `tests/api/<domain>/test_<method>.py` + 一致性报告
3. **隔离声明** — 声明未修改 `testsuite/specs/api/` 和 `tests/api/` 以外的文件

## Additional Resources

### Reference Files

- **`references/coverage-matrix.md`** — M1-M9 覆盖矩阵详细定义与推导规则
- **`references/spec-template.md`** — spec YAML 完整模板、字段说明、proto 映射规则
