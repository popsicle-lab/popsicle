---
id: 631e4c6d-45cb-4785-8a28-4b7344d5f400
doc_type: arch-debate-record
title: cli-ux architecture debate io shell boundary
status: final
skill_name: arch-debate
pipeline_run_id: 82936830-374d-4ac4-b0f6-5639a679bfc8
spec_id: 308c231f-0e91-4922-87f0-22b63be857b6
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T10:36:05.000000Z
updated_at: 2026-06-09T10:36:05.000000Z
---

# cli-ux Architecture Debate — IO shell boundary

## Setup Checklist

- [x] 技术议题已用一句话表达
- [x] 边界已绑定（cli-ux / crates/cli-ux）
- [x] 事实基状态已记录
- [x] 技术角色阵容确定（ARCH / OPS / DEV / SEC）
- [x] 用户置信度已设置
- [x] 已展示 setup 摘要并取得 `start` 确认

## Topic

`crates/cli-ux` 是否只做 IO shell，还是保留 legacy CLI 中混杂的业务编排逻辑。

## Participants

| 角色 | 关注 |
|---|---|
| ARCH | 依赖方向和模块边界 |
| OPS | 错误输出、回滚、可诊断性 |
| DEV | 实现范围和测试 |
| SEC | CLI 是否泄露不该有的状态 |

## Phase 1

问题定义：`skill-runtime` 与 `artifact-system` 已经成为新主路径；cli-ux 如果再复制状态机/guard/parser，就会重新制造旧 core 的耦合。

质量属性：可诊断性、可测试性、低耦合、agent-friendly JSON 输出。

## Phase 2

| 方案 | 描述 | 结论 |
|---|---|---|
| A | CLI 持有 command-specific domain logic | 否决 |
| B | CLI 只做 argv/format/error/file IO shell | 采纳 |
| C | CLI 内部再建 facade service 层 | 延后，MVP 不需要 |

## Phase 3

| 角色 | 评审 |
|---|---|
| ARCH | B 保持 `cli-ux -> skill-runtime/artifact-system/storage` 单向依赖 |
| OPS | B 允许统一错误 envelope 和 next-step hint |
| DEV | B 让 golden 可以按 command semantic 输出测试 |
| SEC | B 避免 CLI 私有缓存 domain 状态 |

## Phase 4

决策：采纳 B。ADR-007 固化 `crates/cli-ux` 的职责：parse argv、dispatch domain services、format text/json、write files through storage-facing APIs、produce actionable errors。

## Decision

`cli-ux` is an IO shell. It delegates pipeline/session semantics to `skill-runtime`, document/guard/context semantics to `artifact-system`, and persistence row/file handling to `storage`.

## Phase Coverage

- [x] Phase 1：技术问题 + NFR 优先级 + 硬约束已明确
- [x] Phase 2：2-3 个差异化架构方案，各含模块边界/数据流
- [x] Phase 3：全部角色（含 SEC / OPS）已评审
- [x] Phase 4：收敛到推荐方案 + 用户决策 + 每个声明标了 intent 层/ADR-CADR
- [x] 至少 4 个用户交互暂停点

## Output Checklist

- [x] arch-debate-record 含 Phase 1-4 全部小结
- [x] 标注了「用户决策覆盖」（无）
- [x] rfc-draft 每个核心技术声明标了 intent 层（contracts）
- [x] rfc-draft 列出 ADR 候选 / CADR 候选清单
- [x] 每个数字/LoC/模块名引用可追溯到 fact-extraction-report / api-contracts
- [x] tech-decision-matrix 维度由角色提出且权重明示
- [x] 三份 artifact 的 Topic 一致
- [x] 已展示三份产出并取得 `approve` 确认
