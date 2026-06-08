# Fact Extraction Report — {project_name}

> **基线日期**：{YYYY-MM-DD}
> **源 commit**：`{git rev-parse HEAD}`
> **抽取者**：fact-extractor v0.1.0

本报告是 `{project_name}` 事实基的**入口**。它承载执行摘要，并链接到 4 份详细 artifact。这里的每个声明都来自 4 份中的一份；不要引入无法追溯到详细 artifact 的事实。

---

## Summary

| 指标 | 值 | 来源 |
|---|---|---|
| 总 LoC | {n} | tokei（appendix A.1）|
| 主语言 | {Rust 78%, TS 22%} | tokei |
| 公开 crate / package 数 | {n} | dependency-graph.md |
| 直接外部依赖数 | {n} | dependency-graph.md |
| 公开 API 表面（函数 + 类型）| {n} | api-contracts.md |
| `unsafe` 块数 | {n} | unsafe-risk-report.md |
| `.unwrap()` / `.expect()` 调用点 | {n} | unsafe-risk-report.md |
| TODO / FIXME / HACK 总数 | {n} | tech-debt-inventory.md |
| Top-50 commit 热点文件 | 见 §Risk Hotspots | git log |

---

## Bounded Contexts

> 看起来像内聚 bounded context 的顶级目录。每一个都是下游 PRD/RFC writer 的「主题区」候选。

| Context | 路径 | LoC | 主要类型 | 备注 |
|---|---|---|---|---|
| {auth} | `src/auth/` | {1,243} | `User`、`Session`、`Role` | 持有身份与访问控制 |
| {payment} | `src/payment/` | {2,108} | `Charge`、`Refund`、`Receipt` | 调用外部 Stripe 网关 |
| {storage} | `src/storage/` | {876} | `Repo`、`Tx` | 包装 Postgres |
| ... | ... | ... | ... | ... |

如果某个目录无法干净映射到一个 context，列在表底部的 **§Unclassified** —— **不要**强行贴标签。

---

## Domain Glossary

> 在代码、注释、doc string 中反复出现的术语。下游 skill 用它维护统一语言。

| 术语 | 首次出现 | 可能含义 | 置信度 |
|---|---|---|---|
| {Tenant} | `src/multitenancy/lib.rs:1` | 拥有一组 user 的客户组织 | high |
| {Pinpoint} | `docs/internal.md:42` | 带元数据的地理空间事件 | medium |
| {Phase} | `src/lifecycle.rs:18` | 4 种生命周期状态之一：pending/active/sealed/archived | high |

`high`：术语在 struct doc-comment 或 README 中有明确定义；`medium`：含义从使用方式推断；`low`：使用方式前后不一致。

---

## Risk Hotspots

> 同时具备**高 churn + 高风险构造**的文件 / 模块。首个迁移切片的首选候选。

| 文件 | 提交/年 | unsafe 数 | unwrap 数 | TODO 数 | 主要风险 |
|---|---|---|---|---|---|
| `src/payment/process.rs` | {47} | {0} | {12} | {3} | 边界情况下 panic；无入参校验 |
| `src/auth/session.rs` | {38} | {2} | {5} | {1} | unsafe 全局状态 |
| ... | ... | ... | ... | ... | ... |

每一行必须引用 `unsafe-risk-report.md` 或 `tech-debt-inventory.md` 中的具体子节作为支撑证据。

---

## 建议的首个迁移切片

> **建议**，不是决定。最终范围在 arch-debate / rfc-writer 阶段确定。

基于上面的热点表，**风险最低 + 杠杆最高**的首切片是 **{`src/payment/`}**，因为：

1. {高 churn（47 commits/yr）→ 频繁触碰 → invariant 收益高}
2. {`api-contracts.md` 中未见外部 API 消费者 → 重构更容易}
3. {已有 TODO 注释暗示期望的 invariant → spec writer 有 head start}

考虑过的替代切片：

- **{src/auth/}**：同样高 churn，但 `unsafe` 全局状态要求先做 unsafe 移除——范围更大。
- **{src/storage/}**：低 churn、低风险；首切片的投入产出比差。

---

## 详细 Artifact

| Artifact | 文件 | 状态 |
|---|---|---|
| Dependency graph | [{slug}.dependency-graph.md]({slug}.dependency-graph.md) | ✅ |
| API contracts | [{slug}.api-contracts.md]({slug}.api-contracts.md) | ✅ |
| Unsafe / risk report | [{slug}.unsafe-risk-report.md]({slug}.unsafe-risk-report.md) | ✅ |
| Tech-debt inventory | [{slug}.tech-debt-inventory.md]({slug}.tech-debt-inventory.md) | ✅ |

---

## 工具来源

| 工具 | 版本 | 用途 | 状态 |
|---|---|---|---|
| tokei | {12.1} | LoC 计数、语言占比 | ✅ |
| cargo metadata | {1.0} | Rust 依赖图 | ✅ |
| cargo tree | {1.79} | 传递依赖 | ✅ |
| ripgrep | {14.1} | 模式挖掘（unsafe、TODO、public API）| ✅ |
| ast-grep | {0.20} | 结构匹配（可选）| ⚠️ 不可用 |
| git log | — | Churn / 热点数据 | ✅ |

某工具不可用时，本应使用它的章节会被标记 `[reduced fidelity]`。

---

## Extraction Checklist

- [ ] 5 个 artifact 都产出且交叉链接
- [ ] Summary 表中每个值都引用了详细 artifact
- [ ] Bounded contexts 已列（或 `Unclassified` 已填）
- [ ] Domain glossary 含 ≥10 个术语且都带置信度
- [ ] Risk hotspots 表含 ≥5 条且都带证据指针
- [ ] 建议的首迁移切片至少有一个替代被考虑
- [ ] 工具来源表列出了所有实际使用过的工具
- [ ] 报告中没有句子含 "should"、"ought to"、"is bad"、"is good"（发表观点检查）
- [ ] 报告中没有句子凭空发明代码中不存在的需求
- [ ] 每个近似数字要么换成精确值，要么删掉
