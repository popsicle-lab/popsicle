# 单接口测试覆盖矩阵（M1-M9）

每个接口 spec 的 `checks` section 必须逐项覆盖 M1-M9。
生成 spec 时按本矩阵逐项推导 case；审阅时按本矩阵逐项核查完整性。

---

## M1. 正常请求（Happy Path）

验证合法参数 + 合法身份下接口能返回预期结果。

**推导规则：**
- 从 `success_response` 提取预期状态码和响应字段
- 从 `request_body` / `request_fields` 构造最小合法输入
- 从 `auth.min_role` 确定使用哪个角色的 token
- 若有 `constraint`（如 `>= 1`），写入 expect 断言

**必须覆盖：**
- 最小合法输入（每个 required 字段刚好满足）
- 若有数组字段，补一个多元素 case 验证批量处理

---

## M2. 必填参数缺失（Required Field Missing）

验证每个 required 字段缺失时返回 400 且错误信息准确。

**推导规则：**
- 遍历 `request_body` / `path_params` 中 `required: true` 的字段
- 每个字段生成一条 case：移除该字段 → 400
- 嵌套对象中的 required 字段也需覆盖（如 `properties[].property_id`）

**expect 格式：**
- gRPC: `INVALID_ARGUMENT`，details 包含缺失字段名
- HTTP: `400`，响应体包含缺失字段名

---

## M3. 参数类型错误（Type Mismatch）

验证字段传入错误类型时返回 400。

**推导规则：**
- 遍历 `request_body` 中有明确 `type` 的字段
- 每个字段生成一条 case：传入不兼容类型
  - string 字段 → 传 int/bool
  - int/uint 字段 → 传 string
  - array 字段 → 传 string/object
  - object/message 字段 → 传 string/array

**注意：** protobuf 在序列化层即拒绝类型错误（decode 失败），
此时 gRPC status 为 `INVALID_ARGUMENT` 或连接层直接报错。
对 proto 来源的接口，M3 可标注为 `proto_layer: true` 表示由协议层保证。

---

## M4. 参数格式错误（Format Violation）

验证字段值格式不合法时返回错误。

**推导规则：**
- 检查字段是否有 `format` 约束（uuid、email、date 等）
- 检查字段是否有 `enum` 值范围
- 每种格式约束生成一条 case：传入格式错误的值
  - uuid 字段 → 传 `"not-a-uuid"`
  - enum 字段 → 传未定义的枚举值（proto 中为未知数字值）

**注意：** proto3 对未知 enum 值默认保留（不报错），
需根据服务端实际行为判断 expect。

---

## M5. 边界值（Boundary Values）

验证字段在极端取值下的行为。

**推导规则：**
- 字符串：空串 `""`、单字符、若有 max_length 则测 max 和 max+1
- 数字：`0`、负数（若为 unsigned 则 0 即边界）、最大值、溢出值
- 数组：空数组 `[]`、单元素、若有 max_items 则测 max 和 max+1；若有 min_items 则测 min-1
- optional 字段：不传 vs 显式传零值（proto3 零值 = 默认值）

**gRPC 特殊考虑：**
- proto3 不区分"未设置"和"零值"（除 optional 修饰符）
- `optional` 字段才能区分"未传"和"传了零值"
- repeated 字段空数组 = 默认值（未传）

---

## M6. 认证缺失（Unauthenticated）

验证无合法身份时返回 401 / UNAUTHENTICATED。

**推导规则：**
- 从 `auth` section 确认需要认证
- 生成以下固定 case：
  1. 不带 metadata/header（无 token）→ UNAUTHENTICATED
  2. 带过期 token → UNAUTHENTICATED
  3. 带格式错误的 token（乱码）→ UNAUTHENTICATED

**gRPC 映射：** `grpc.StatusCode.UNAUTHENTICATED` (code 16)

---

## M7. 权限不足（Unauthorized / Permission Denied）

验证低权限角色调用时返回 403 / PERMISSION_DENIED。

**推导规则：**
- 从 `auth.min_role` 确定最低所需角色
- 选择低于 min_role 的角色生成 case
  - 角色层级（高→低）：admin > org_admin > editor > viewer > device
  - 例：min_role=editor → viewer 调用 → PERMISSION_DENIED

**gRPC 映射：** `grpc.StatusCode.PERMISSION_DENIED` (code 7)

---

## M8. 资源不存在（Not Found）

验证引用不存在的资源时返回 404 / NOT_FOUND。

**推导规则：**
- 检查 `path_params` 和 `request_body` 中作为资源标识的字段（通常含 `_id` 后缀）
- 每个资源标识字段生成一条 case：传入格式合法但不存在的值 → NOT_FOUND
- 若接口引用多个资源（如 `model_id` + `device_id`），分别测试各自不存在的情况

**gRPC 映射：** `grpc.StatusCode.NOT_FOUND` (code 5)

---

## M9. 错误响应格式（Error Response Contract）

验证所有错误响应符合统一格式、不泄露敏感信息。

**推导规则：**
- 复用 M2-M8 中任意一个触发错误的 case
- 额外断言：
  1. gRPC: status code 正确 + details 非空 + details 不含堆栈/SQL/内部路径
  2. HTTP: 响应体含 `code` + `message` + 不含敏感信息
- 针对 gRPC 场景，验证 `status.details` 中的错误描述是人类可读的

---

## 生成 checks 的标准格式

每条 case 使用以下结构：

```yaml
checks:
  M1_happy_path:
    title: 正常请求
    cases:
      - id: M1-01
        input: <自然语言描述输入条件>
        expect: <自然语言描述预期结果>
```

**id 命名规则：** `{M类别}-{两位序号}`，如 `M1-01`、`M5-03`

**input 写法要求：**
- 明确指出使用哪个角色/token
- 明确指出关键参数的具体值或特征
- 非技术人员能读懂

**expect 写法要求：**
- 明确指出预期状态码 / gRPC status
- 明确指出需要断言的响应字段和值
- 若有业务规则相关的断言，引用 `business_rules` 中的条目
