# API Contracts — {project_name}

> `{slug}.fact-extraction-report.md` 的配套文件。记录项目的**公开表面**：调用方能依赖什么。按 bounded context 分组。

> **范围规则**：只有标记为 `pub`（Rust）/ `export`（TS/Go）/ 模块级非下划线（Python）的条目属于本文档。跨模块的内部 helper 进 `dependency-graph.md`；其它出范围。

---

## Bounded Context：{auth}

> **路径**：`src/auth/`
> **用途**（来自模块 doc-comment 或 README）：{"身份、会话、基于角色的访问控制。"}

### 公开函数

| 签名 | File:Line | 修改 | 备注 |
|---|---|---|---|
| `pub fn login(user: &str, pass: &str) -> Result<Token, Error>` | `src/auth/login.rs:42` | 全局 session 表 | 哈希存储 poisoned 时 panic |
| `pub fn logout(token: Token) -> Result<(), Error>` | `src/auth/login.rs:88` | 全局 session 表 | — |
| `pub fn check(token: &Token, role: Role) -> bool` | `src/auth/check.rs:12` | （只读）| — |

### 公开类型

| 类型 | 种类 | File:Line | 公开字段 / 变体 |
|---|---|---|---|
| `Token` | struct | `src/auth/types.rs:8` | `value: String`、`expires_at: SystemTime` |
| `Role` | enum | `src/auth/types.rs:34` | `Admin`、`Editor`、`Viewer`、`Guest` |
| `Error` | enum | `src/auth/types.rs:60` | `BadPassword`、`Locked`、`Expired`、`Internal(String)` |

### 公开 Trait

| Trait | File:Line | 实现者 |
|---|---|---|
| `SessionStore` | `src/auth/store.rs:5` | `MemoryStore`、`RedisStore` |

### HTTP/gRPC 端点（如有）

| 方法 | 路径 | Handler | 需要鉴权 |
|---|---|---|---|
| POST | `/api/v1/login` | `auth::http::login_handler`（`src/http/auth.rs:12`）| 否 |
| POST | `/api/v1/logout` | `auth::http::logout_handler`（`src/http/auth.rs:48`）| 是（Token）|

### 行为备注（只写已记录或测试编码的内容）

- **锁定策略**（编码在 `tests/auth/lockout.rs`）：15 分钟内连续 5 次失败 → 用户锁定 30 分钟。
- **Token TTL**（编码在 `src/auth/login.rs:55`）：24 小时。

> **不要**写推断出来的行为备注。只引用测试断言或代码字面表达。

---

## Bounded Context：{payment}

> **路径**：`src/payment/`
> **用途**：{"……"}

### 公开函数

| 签名 | File:Line | 修改 | 备注 |
|---|---|---|---|
| ... | ... | ... | ... |

（按 context 重复以上章节）

---

## 跨切面公开 API

> 不属于任何单一 bounded context 的 API（如日志、错误类型、通用 trait）。

| 条目 | File:Line | 备注 |
|---|---|---|
| `pub trait Telemetry` | `src/common/telemetry.rs:1` | 所有 context 共用 |

---

## 稳定性标记

> 任何被显式标注 `#[deprecated]`、`@deprecated`、`// EXPERIMENTAL` 等的条目。

| 条目 | 标记 | 原因 | File:Line |
|---|---|---|---|
| `pub fn legacy_login_v1` | `#[deprecated(since = "0.4.0")]` | 改用 `login` | `src/auth/legacy.rs:8` |

---

## Extraction Checklist

- [ ] `fact-extraction-report.md` 中每个 bounded context 在此都有独立章节
- [ ] 每个签名都含 file:line
- [ ] 每个「修改」单元格都填了（只读处写 `（只读）`；**不留空**）
- [ ] HTTP/gRPC 端点章节已填，或显式写 `(none — library only)`
- [ ] 行为备注章节只含测试断言或代码字面表达（无推断）
- [ ] 跨切面 API 章节已填，或写 `(none)`
- [ ] 稳定性标记章节已填，或写 `(no deprecated/experimental markers found)`
