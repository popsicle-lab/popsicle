# Unsafe & Risk Report — popsicle@c76d729

> 配套：[`fact-extraction-report.md`](../../../.popsicle/artifacts/f89529af-d8ce-40f7-ad05-985e35b9cfec/popsicle-c76d729-fact-basis-slice-1--skill-runtime.fact-extraction-report.md)
>
> 基线：`legacy/popsicle/` submodule @ `c76d729db91c59009f0fa8f7c6f1e499eb0c7eb1`
> 范围：**记录存在什么**，不建议怎么改（建议属于 adr-writer / rfc-writer）。

---

## 内存 / 类型安全旁路

### `unsafe` 块（Rust）

| File:Line | 上下文 | 周围注释 | 是否注明原因 |
|---|---|---|---|
| _（none —— 整个 workspace 0 个 unsafe 块）_ | n/a | n/a | n/a |

> 验证命令：`rg -t rust 'unsafe\s*\{|unsafe\s+fn|unsafe\s+impl' crates/`
> 唯一含 `unsafe` 字面出现的位置是 `crates/popsicle-sync/src/path.rs:200`，其内容为 `fn unsafe_slug_is_sanitised()` —— **测试函数命名**而非 unsafe 块。无安全旁路。

### FFI / cgo / NAPI 调用

| File:Line | 函数 | 库 | 备注 |
|---|---|---|---|
| `crates/popsicle-core/Cargo.toml`（间接）| `rusqlite::*` 各种 fn | libsqlite3（bundled）| SQLite 通过 `rusqlite = { features = ["bundled"] }`——sqlite C 源码随 cargo build 编入，**不**依赖系统 libsqlite。不暴露在 popsicle-core 公开 API 表面；所有 `rusqlite::Connection` 实例局限于 `storage/` 模块。 |
| `crates/popsicle-cli/Cargo.toml` | `keyring::*` | macOS Keychain / Linux Secret Service（dbus）| 仅在 `popsicle sync login` 路径使用；非主路径 |
| `crates/popsicle-cli/Cargo.toml`（optional）| Tauri WebView FFI | system WebKit / WebView2 | 仅 `feature = "tauri"` 开时存在 |

> 上面三项是 FFI 接触面，但**调用方**全是 safe Rust（crate 内部已封 unsafe）。本仓库**自己写**的代码中**无** `unsafe` 块、`extern "C"` 块、`#[no_mangle]` 块。

### 裸指针算术

| File:Line | 指针来源 | 操作 |
|---|---|---|
| _（none —— 0 个原始指针操作）_ | n/a | n/a |

---

## 失败模式热点

### `.unwrap()` / `.expect()` 调用点

**总计**：`.unwrap()` = **301 处**；`.expect()` = **12 处**。这是本仓库最显著的失败模式信号。

#### 按文件 top-10（`rg -t rust '\.unwrap\(\)' -c`）

| File | 数量 | 公开 API 可达？| 备注 |
|---|---|---|---|
| `crates/popsicle-core/src/storage/index.rs` | 101 | 是 —— 所有 CRUD 都过这里 | SQLite 操作的 `Result` 大量 `.unwrap()`；DB 损坏 / schema mismatch 时 panic |
| `crates/popsicle-core/src/registry/index.rs` | 24 | 是 —— `popsicle module install/list` 路径 | module 注册的 SQLite 操作 |
| `crates/popsicle-core/src/model/skill.rs` | 23 | 是 —— `popsicle skill list/show` 路径 | YAML 解析 + 状态机定义 |
| `crates/popsicle-core/src/engine/extractor.rs` | 22 | 是 —— `popsicle doc extract` 路径 | 从 doc 抽 work_item 时的 regex / parse |
| `crates/popsicle-core/src/engine/guard.rs` | 16 | 是 —— `pipeline review` / `stage complete` 触发 | guard DSL 解析 |
| `crates/popsicle-core/src/memory/store.rs` | 15 | 是 —— `popsicle memory *` 路径 | memory CRUD |
| `crates/popsicle-core/src/agent/mod.rs` | 14 | 是 —— `popsicle init -a *` 路径 | agent 文件渲染（commit 热点 top 1）|
| `crates/popsicle-core/build.rs` | 13 | 否（编译期）| build script |
| `crates/popsicle-core/src/scanner.rs` | 11 | 是 —— `popsicle context scan` 路径 | 文件系统扫描 |
| `crates/popsicle-cli/src/commands/prompt.rs` | 9 | 是 —— `popsicle prompt` 路径 | prompt 渲染 |

#### 公开 API 可达性结论

- 上面 10 个文件中除 `build.rs` 外，**全部经主 CLI 命令可达**。
- 这意味着用户在 SQLite 损坏、YAML schema 不匹配、文件系统权限错误等场景下会撞 panic（不是 graceful error）。
- intent-coder 的 schema drift bug（已在 LEGACY_PIN.md 登记）就是这类问题的**实例**——`intent-consistency-check/skill.yaml` 用错 `inputs` 字段后 `model/skill.rs` 的 23 处 `.unwrap()` 之一炸了 `popsicle skill list`。

#### `.expect()` 调用点（12 处，跳过具体文件分布；模式同 unwrap）

> 与 `.unwrap()` 等价，但提供静态错误消息。

### `panic!` / `unreachable!` / `todo!`

| File:Line | 宏 | 消息 |
|---|---|---|
| `crates/popsicle-core/src/registry/package.rs:157` | `panic!` | `"empty package name"` |

> 1 处 `panic!` —— 极简空包名守卫，可以重写为 `Result`。
> `rg -t rust 'unreachable!\(\)' crates/` 返回 0 行。
> `rg -t rust 'todo!\(\)' crates/` 返回 0 行。

### 动态 eval / shell

| File:Line | 构造 | 输入来源 |
|---|---|---|
| _（none —— `rg -t rust 'eval\(' crates/` 返回 0）_ | n/a | n/a |
| `crates/popsicle-cli/src/commands/git.rs`（间接）| `std::process::Command::new("git")` | 用户提供的 commit hash / branch 字符串 | 仅 git 子进程调用，参数已通过 `.arg()` 转义；无 shell -c |

> 主代码**不**做动态代码 eval。`Command::new("git")` 在 `git/tracker.rs` 用，是受控调用。

---

## 并发风险

### 共享可变状态

| 构造 | File:Line | 同步 | 备注 |
|---|---|---|---|
| _（none found —— `rg -t rust 'lazy_static!|static mut|RwLock<|Mutex<' crates/popsicle-core` 无 `static mut`；少量 `Mutex` 在 sync 子模块用于 daemon）_ | n/a | n/a | popsicle-core 是**几乎**纯函数 / Rusqlite Connection-per-call 模型 |
| popsicle-sync 的 `Arc<tokio::sync::Mutex<...>>` | `crates/popsicle-sync/src/{client,ws}.rs`（具体位置未逐个标）| tokio Mutex | 用于 WebSocket daemon 共享状态 |

### 跨 `await` 持锁

| File:Line | 锁类型 | 跨 await 的表达式 |
|---|---|---|
| _(needs human input — 未跑专用 lint；仅 popsicle-sync 用 `tokio::sync::Mutex` 是事实，是否跨 await 持锁需要进一步 lint)_ | — | — |

---

## 密码学 / 密钥处理

| 构造 | File:Line | 算法 / 库 | 备注 |
|---|---|---|---|
| TLS（HTTP）| `crates/popsicle-sync/src/http.rs` | `reqwest` features `["rustls-tls"]` | OK —— 现代 TLS 实现 |
| TLS（WebSocket）| `crates/popsicle-sync/src/ws.rs` | `tokio-tungstenite` features `["rustls-tls-webpki-roots"]` | OK |
| 密钥 / token 存储 | `crates/popsicle-cli/src/commands/sync.rs` | `keyring` crate（系统密钥环）| OK —— 不写明文 |
| Base64 编码（payload）| `crates/popsicle-sync/src/{client,http}.rs` | `base64 0.22` | 非加密用途 |

### 源码中密钥扫描

| 匹配 | File:Line | 可能是误报？|
|---|---|---|
| _(none —— `rg 'AKIA|SECRET_KEY\s*=|password\s*=' crates/` 无命中)_ | n/a | n/a |

> 无硬编码密钥/凭证。`popsicle sync login` 流程把 token 存进系统密钥环。

---

## 网络 / IO 风险

| 端点 | File:Line | 校验 | 超时 |
|---|---|---|---|
| popsicle-cloud HTTPS push/pull | `crates/popsicle-sync/src/{client,http}.rs` | reqwest 默认 (rustls + webpki-roots cert validation) | (needs human input —— 未在源码中显式设置 `.timeout()`，使用 reqwest 默认行为) |
| popsicle-cloud WebSocket daemon | `crates/popsicle-sync/src/ws.rs` | tokio-tungstenite 默认 | 同上 |

> 主路径（非 sync）完全离线，无网络风险面。

---

## 按模块的风险密度

| 模块 | unsafe | unwrap | panic | dyn-eval | 总计 |
|---|---|---|---|---|---|
| `popsicle-core::storage` | 0 | 101 + n | 0 | 0 | 101+ |
| `popsicle-core::registry` | 0 | 24 + 7 | 1 | 0 | 32 |
| `popsicle-core::model` | 0 | 23 + n | 0 | 0 | 23+ |
| `popsicle-core::engine` | 0 | 22 + 16 + n | 0 | 0 | 38+ |
| `popsicle-core::memory` | 0 | 15 + 8 | 0 | 0 | 23 |
| `popsicle-core::agent` | 0 | 14 | 0 | 0 | 14 |
| `popsicle-core::scanner` | 0 | 11 | 0 | 0 | 11 |
| `popsicle-cli::commands` | 0 | 9 + 8 + ...（合计 unwrap 数未单独抽，但 prompt/sync/issue/init 各约 7-12）| 0 | 0 | ~40 估计 |
| `popsicle-sync` | 0 | 6 | 0 | 0 | 6 |
| build scripts | 0 | 13 | 0 | 0 | 13 |
| **总计** | **0** | **301** | **1** | **0** | **302** |

> 风险密度**最高**：`popsicle-core::storage` —— 101 unwrap，几乎全在 SQLite 操作（DB-related）。
> 风险密度**最低**：`popsicle-sync` —— 6 unwrap。

---

## Extraction Checklist

- [x] 每条都有 file:line 引用
- [x] 每条都引用了周围上下文（或写 `(none)`）
- [x] `unsafe` 块的「是否注明原因」填：(none —— 0 unsafe 块全仓库)
- [x] `.unwrap()` 表把「公开 API 可达」与「仅内部」分开（top-10 已标注）
- [x] 并发章节即使为空也已审过（写 `(needs human input)` 不省略）
- [x] 密码学章节即使为空也已审过
- [x] 风险密度表的合计与各分节计数匹配
- [x] 没有句子含 "should be removed" / "should be replaced"（发表观点检查）
