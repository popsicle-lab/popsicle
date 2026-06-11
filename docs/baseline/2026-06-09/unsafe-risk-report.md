# Unsafe / Risk Report — artifact-system scope @ popsicle c76d729

> **Scope**: slice-2 (artifact-system) 8 模块。源 commit `c76d729`。
> **方法**: `rg` 计数 + 直读分类（生产代码 vs `#[cfg(test)]`）。每条 cite file:line。
> **记录员声明**: 只记失败模式构造与位置，不评判「该不该」。

## 失败模式计数（8 模块合计）

| 构造 | 计数 | 备注 |
|---|---|---|
| `unsafe` 块 | **0** | 全范围无 unsafe |
| `.unwrap()` | 44（总）| 其中**生产代码 19，测试代码 25**（见下表）|
| `.expect(` | 0 | |
| `panic!()` | 0 | |
| `todo!()` / `unimplemented!()` / `unreachable!()` | 0 / 0 / 0 | |

## `.unwrap()` 分布（按模块，区分 test/prod）

| 模块 | unwrap 总 | `#[cfg(test)]` 起始行 | 生产 unwrap | 测试 unwrap |
|---|---|---|---|---|
| `engine/extractor.rs` | 22 | 260 | **19**（行 21,25,26,31,73,74,75,82,129,130,131,132,133,140,183,188,210,211,216）| 3（298,330,361）|
| `engine/guard.rs` | 16 | 284 | 0 | 16（≥309）|
| `model/document.rs` | 3 | （test 模块）| 0 | 3（159,169,170）|
| `engine/markdown.rs` | 2 | 224 | 0 | 2（321,322）|
| `model/namespace.rs` | 1 | 74 | 0 | 1（89）|
| `engine/context.rs` | 0 | — | 0 | 0 |
| `engine/context_layer.rs` | 0 | — | 0 | 0 |
| `model/work_item.rs` | 0 | — | 0 | 0 |
| **合计** | **44** | | **19** | **25** |

## 生产 unwrap 的构造分类（extractor.rs，全部 19 处）

| 构造 | 行 | 事实描述 |
|---|---|---|
| `Regex::new(<字面量>).unwrap()` | 21,25,26,73,74,75,129,130,131,132,133,183,210,211（等编译期常量正则）| 正则模式是源码字面量；编译失败仅在模式本身有误时发生 |
| `<re>.captures(m.as_str()).unwrap()` | 31,82,140,188,216 | 在 `find_iter` 命中的子串上再 `captures`；命中串与同一正则匹配 |

> 事实：extractor.rs 的全部生产 unwrap 都属上述两类（常量正则编译 + 对已命中子串的二次 captures）。本报告不判断其是否「安全」——仅记录构造与位置，留给下游 RFC/ADR 评估。

## 失败模式热点排序（生产代码）

| 文件 | 生产 unwrap | 主路径 | 证据 |
|---|---|---|---|
| `engine/extractor.rs` | 19 | `popsicle doc extract`（从 PRD/test-spec 抽 WorkItem）| 行号见上表 |
| 其余 7 模块 | 0 | — | — |

## 错误传播观察（事实层）

| 模块 | 错误处置 | 证据 |
|---|---|---|
| `model/document.rs` | 用 `crate::error::Result<T>` + `?`；缺 frontmatter 返回 `InvalidDocumentFormat` | `model/document.rs:108,114,126-129` |
| `engine/guard.rs` | 返回 `Result<GuardResult>`；未知 guard 返回 `InvalidSkillDef`（不 panic）| `engine/guard.rs:33,92-95` |
| `engine/extractor.rs` | 返回 `Vec<WorkItem>`（无 Result）；解析不到段落返回空 Vec（早返回）| `engine/extractor.rs:16-19` |
| `engine/context.rs` / `context_layer.rs` | 纯函数装配，无 `?`/panic | 全文件 |

## Checklist

- [x] unsafe/unwrap/expect/panic/todo 计数完整且区分 test/prod
- [x] 每个生产 unwrap 都给了 file:line 并归类构造
- [x] 失败模式热点已排序（生产口径）
- [x] 错误传播方式已记录（Result vs 空集合 vs 纯函数）
- [x] 无 "should/is bad" 等观点句；未对 unwrap 安全性下结论
