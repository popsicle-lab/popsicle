# Unsafe & Risk Report — {project_name}

> `{slug}.fact-extraction-report.md` 的配套文件。记录每一个**绕过语言安全网**或**带已知风险**的构造。被 safety-spec 与 invariant-spec 消费，用来判断哪些 invariant 最值得证。

> **范围**：本报告记录**存在什么**，不记录**该改什么**。建议属于下游 adr-writer / rfc-writer 的 artifact。

---

## 内存 / 类型安全旁路

### `unsafe` 块（Rust）

| File:Line | 上下文 | 周围注释 | 是否注明原因 |
|---|---|---|---|
| `src/auth/state.rs:22` | `static mut SESSIONS` | `// SAFETY: only accessed under MUTEX` | 是 —— mutex 论证 |
| `src/util/cast.rs:18` | `transmute::<u64, *const Header>` | （无注释）| （无 —— needs human input）|

### FFI / cgo / NAPI 调用

| File:Line | 函数 | 库 | 前置/后置条件是否记录 |
|---|---|---|---|
| `src/z3/binding.rs:55` | `Z3_mk_solver` | libz3 | 部分（处理了返回值，未处理错误路径）|

### 裸指针算术

| File:Line | 指针来源 | 操作 |
|---|---|---|
| `src/serde_fast/parse.rs:312` | `slice.as_ptr().add(i)` | 在长度受边界检查的 offset 内 |

---

## 失败模式热点

### `.unwrap()` / `.expect()` 调用点

| File:Line | 表达式 | 公开 API 可达？| 备注 |
|---|---|---|---|
| `src/payment/process.rs:115` | `gateway.charge(amount).unwrap()` | 是 | 网络错误时 panic |
| `src/parse/lexer.rs:88` | `iter.next().unwrap()` | 否（内部）| 循环不变量保证 |

> 「公开 API 可达」的先列在前；那是 panic 会变成用户可见崩溃的地方。

### `panic!` / `unreachable!` / `todo!`

| File:Line | 宏 | 消息 |
|---|---|---|
| `src/lifecycle.rs:142` | `panic!` | `"unhandled phase transition: {:?} → {:?}"` |
| `src/codegen/emit.rs:401` | `todo!` | `"emit array literal"` |

### 动态 eval / shell

| File:Line | 构造 | 输入来源 |
|---|---|---|
| `src/templates/render.rs:55` | `evaluate(user_template)` | 用户上传的模板 |
| `src/scripts/run.rs:18` | `Command::new("sh").arg("-c").arg(cmd)` | 配置文件 |

---

## 并发风险

### 共享可变状态

| 构造 | File:Line | 同步 | 备注 |
|---|---|---|---|
| `lazy_static! HASHES: Mutex<HashMap<…>>` | `src/auth/state.rs:14` | Mutex | 是否跨 await 持有？—— 需检查 |
| `static ATOMIC_COUNTER: AtomicU64` | `src/metrics.rs:8` | atomic | OK |

### 跨 `await` 持锁

| File:Line | 锁类型 | 跨 await 的表达式 |
|---|---|---|
| `src/cache/tiered.rs:88` | `tokio::sync::Mutex` | `db.fetch(key).await` |

---

## 密码学 / 密钥处理

| 构造 | File:Line | 算法 / 库 | 备注 |
|---|---|---|---|
| 密码哈希 | `src/auth/hash.rs:12` | `argon2id`（argon2 v0.5）| OK —— 现代 KDF |
| Token 签名 | `src/auth/token.rs:34` | `hmac-sha256` | 密钥从 `SECRET_KEY` 环境变量加载 |
| TLS | `src/http/server.rs:8` | `rustls 0.22` | OK |

### 源码中密钥扫描

| 匹配 | File:Line | 可能是误报？|
|---|---|---|
| `AKIA...` 正则命中 | `tests/fixtures/example.json` | 是 —— fixture 文件 |
| `password = "..."` | `src/dev/seed.rs:22` | 是 —— 开发 seed only |

---

## 网络 / IO 风险

| 端点 | File:Line | 校验 | 超时 |
|---|---|---|---|
| Stripe `/v1/charges` | `src/payment/gateway.rs:42` | 未检查 amount > 0 | 30s |
| GitHub `/repos/.../releases` | `src/release/check.rs:18` | 无 | 10s |

---

## 按模块的风险密度

> 聚合计数；帮你筛选首个迁移切片。

| 模块 | unsafe | unwrap | panic | dyn-eval | 总计 |
|---|---|---|---|---|---|
| `auth` | 2 | 5 | 1 | 0 | 8 |
| `payment` | 0 | 12 | 0 | 0 | 12 |
| `storage` | 0 | 3 | 0 | 0 | 3 |
| ... | ... | ... | ... | ... | ... |

---

## Extraction Checklist

- [ ] 每条都有 file:line 引用
- [ ] 每条都引用了周围注释（或写 `(no comment)`）
- [ ] `unsafe` 块的「是否注明原因」填了 `是 — <一行原因>`、`否`、或 `(无 — needs human input)`
- [ ] `.unwrap()` 表把「公开 API 可达」与「仅内部」分开
- [ ] 并发章节即使为空也要审过（写 `(none found)` 而不是省略）
- [ ] 密码学章节即使为空也要审过
- [ ] 风险密度表的合计与各分节计数匹配
- [ ] 没有句子含 "should be removed" / "should be replaced"（发表观点检查）
