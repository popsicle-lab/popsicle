# port 使用指南

`shadow-implementer` 在 verbatim 平移下退化为「填 coverage 表」（代码是拷的，没有「实现」）。
S3 把它拆成两个诚实的分工：

| skill | 用于 | 干什么 | golden |
|---|---|---|---|
| **port**（本 skill）| `migration_mode: verbatim` | 逐字节/逐行平移 legacy → new，登记覆盖，不改逻辑 | characterization（快照自证）|
| **shadow-implementer** | rewrite / greenfield | 真写新逻辑，intent→fn/test 覆盖 | 需真实对账（rewrite 配 golden-capture）|

先读切片 `task` / `equivalence-report` 的 `migration_mode` 决定用哪个。

## 为什么分开（#18/#22/S3）

verbatim 平移里，new 代码**直接是主路径**，没有影子并行、没有流量对比——「shadow / strangler-fig」
叙事不成立；golden 只能是 characterization。硬把它塞进 shadow-implementer 会诱导「假装做了等价差分」。
`port` 让 verbatim 成为一等公民：诚实登记「哪段 legacy 搬到了哪」，并显式声明等价是平凡的。

## 定位

| 做 | 不做 |
|---|---|
| 逐段平移 + 登记 legacy↔new 覆盖 | 改逻辑（那是 rewrite，走 shadow-implementer）|
| 标 verbatim / adapted，adapted 说明适配点 | adapted 悄悄改可观测行为 |
| `cargo build` 通过再交 | 平移不完整就提交 |
| 声明 golden=characterization | 假装跑了 legacy↔new 差分 |

## 与 pipeline 的接法

`migration-preserve` / `migration-slice-delivery` 的 `implement` stage 默认挂 `shadow-implementer`。
verbatim 切片可在该 stage 用 `port` 履行实现职责（产 `port-coverage` 而非 `implementation-coverage`）——
二者是同一「实现」阶段的两种模式，按 `migration_mode` 选。下游 equivalence-baseline 对 verbatim 建
characterization golden（无需 `golden-capture`）；对 rewrite 才需 `golden-capture` 录 legacy 做差分。
