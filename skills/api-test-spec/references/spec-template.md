# 接口测试 Spec 模板

本文件定义接口测试 spec 的 YAML 格式规范。
spec 文件存放在 `testsuite/specs/api/` 目录下。

---

## 完整模板

```yaml
# ═══════════════════════════════════════════════════════════════
# 接口测试 Spec — <接口名称>
# 来源: <proto 文件路径 或 HTTP 路由定义路径>
# ═══════════════════════════════════════════════════════════════

name: <spec 唯一标识，kebab-case>
version: 1
source: <proto 文件路径或 HTTP 路由文件路径>

# ─── 接口基本信息 ───

endpoint:
  protocol: grpc          # grpc | http
  service: <service 名>   # gRPC: package.ServiceName
  method: <method 名>     # gRPC: RpcName | HTTP: GET/POST/PUT/DELETE
  path: <路径>            # HTTP only, 如 /api/v1/devices/{device_id}

description: <一句话描述接口功能>

# ─── 认证与权限 ───

auth:
  required: true
  method: Bearer Token     # Bearer Token | API Key | mTLS
  min_role: editor         # admin | org_admin | editor | viewer | device | service

# ─── 请求参数 ───
# 按来源分 section：path_params（仅 HTTP）、request_fields（gRPC message 字段）

# HTTP 路径参数（仅 protocol: http 时使用）
path_params:
  - name: device_id
    type: string
    format: uuid
    required: true

# 请求字段（gRPC message fields 或 HTTP request body fields）
request_fields:
  - name: model_id
    type: string
    required: true
    description: 物模型 ID
    is_resource_id: true      # 标记为资源标识符，M8 自动生成 not_found case

  - name: device_id
    type: string
    required: true
    description: 设备 ID
    is_resource_id: true

  - name: properties
    type: array
    required: true
    min_items: 1
    max_items: 100
    description: 属性值列表
    items:
      - name: property_id
        type: string
        required: true
      - name: value
        type: any
        required: true

  - name: client_hlc
    type: uint64
    required: false
    proto_optional: true      # proto3 optional 修饰符，可区分未传和零值
    description: 客户端 HLC 时间戳

# ─── 成功响应 ───

success_response:
  status: OK                  # gRPC: OK | HTTP: 200/201/204
  fields:
    - name: version
      type: uint64
      constraint: ">= 1"
    - name: server_hlc
      type: uint64
      constraint: "> 0"

# ─── 已知错误码 ───

error_codes:
  INVALID_ARGUMENT: 参数校验失败（缺失必填字段、类型错误、格式错误）
  UNAUTHENTICATED: 未认证（无 token / 过期 / 格式错误）
  PERMISSION_DENIED: 权限不足
  NOT_FOUND: 资源不存在
  RESOURCE_EXHAUSTED: 写入速率超限

# ─── 业务规则 ───
# 此接口特有的行为逻辑，M1-M9 无法从字段声明推导出来的部分

business_rules:
  - property_id 不在物模型中 → 静默忽略，不报错，不持久化
  - value 类型与 schema 不匹配 → 静默降级为 null 持久化
  - client_hlc 为 0 或超出合理范围 → 服务端忽略，使用自身时钟

# ═══════════════════════════════════════════════════════════════
# 以下是本接口计划生成的全部测试用例
# 审阅者：请逐项确认是否符合预期，可直接批注增删改
# ═══════════════════════════════════════════════════════════════

checks:
  M1_happy_path:
    title: 正常请求
    cases:
      - id: M1-01
        input: <角色 + 合法参数描述>
        expect: <状态码 + 关键字段断言>

  M2_required_missing:
    title: 必填参数缺失
    cases:
      - id: M2-01
        input: <移除某 required 字段>
        expect: <INVALID_ARGUMENT + 错误信息包含字段名>

  M3_type_mismatch:
    title: 参数类型错误
    cases:
      - id: M3-01
        input: <某字段传入错误类型>
        expect: <INVALID_ARGUMENT>

  M4_format_violation:
    title: 参数格式错误
    cases:
      - id: M4-01
        input: <某字段传入格式错误的值>
        expect: <INVALID_ARGUMENT 或 NOT_FOUND>

  M5_boundary_values:
    title: 边界值
    cases:
      - id: M5-01
        input: <极端取值描述>
        expect: <预期行为>

  M6_unauthenticated:
    title: 认证缺失
    cases:
      - id: M6-01
        input: 不带 metadata（无 token）
        expect: UNAUTHENTICATED
      - id: M6-02
        input: 过期 token
        expect: UNAUTHENTICATED
      - id: M6-03
        input: 格式错误的 token（乱码字符串）
        expect: UNAUTHENTICATED

  M7_unauthorized:
    title: 权限不足
    cases:
      - id: M7-01
        input: <低于 min_role 的角色调用>
        expect: PERMISSION_DENIED

  M8_not_found:
    title: 资源不存在
    cases:
      - id: M8-01
        input: <某资源 ID 合法但不存在>
        expect: NOT_FOUND

  M9_error_contract:
    title: 错误响应格式
    cases:
      - id: M9-01
        input: 触发任意 INVALID_ARGUMENT 错误
        expect: status.details 非空且包含人类可读描述
      - id: M9-02
        input: 触发任意错误
        expect: status.details 不含堆栈、SQL、内部路径等敏感信息
```

---

## 字段类型与 proto 类型映射

| spec type | proto3 类型 | 说明 |
|-----------|-------------|------|
| string | string | |
| int32 | int32/sint32 | |
| int64 | int64/sint64 | |
| uint32 | uint32 | |
| uint64 | uint64 | |
| float | float | |
| double | double | |
| bool | bool | |
| bytes | bytes | |
| enum | enum | spec 中列出允许值 |
| message | message | 嵌套对象，展开为 items |
| array | repeated | 包含 items 子字段 |
| any | oneof / google.protobuf.Value | 值类型不固定 |
| map | map<K,V> | 键值对 |

## gRPC Status Code 与 HTTP 状态码映射

| gRPC Status | HTTP | 含义 |
|-------------|------|------|
| OK | 200 | 成功 |
| INVALID_ARGUMENT | 400 | 参数校验失败 |
| UNAUTHENTICATED | 401 | 未认证 |
| PERMISSION_DENIED | 403 | 权限不足 |
| NOT_FOUND | 404 | 资源不存在 |
| ALREADY_EXISTS | 409 | 资源已存在 |
| RESOURCE_EXHAUSTED | 429 | 限流/配额耗尽 |
| INTERNAL | 500 | 服务端内部错误 |

## 从 Proto 文件提取 Spec 的规则

1. **endpoint**: 从 `service` 和 `rpc` 声明提取
2. **request_fields**: 从 rpc 的 `Request` message 递归展开字段
   - `required`: proto3 默认所有字段可选；标注 `required: true` 需根据业务语义判断
   - `is_resource_id`: 字段名含 `_id` 后缀且为 string 类型 → 标记为资源标识
   - `proto_optional`: proto3 `optional` 修饰符 → 标记为 true
3. **success_response**: 从 rpc 的 `Response` message 提取
4. **auth**: 根据 conftest.py 中的 fixture 和服务端拦截器逻辑确定
5. **business_rules**: 需阅读服务端实现代码提取，无法从 proto 自动推导
