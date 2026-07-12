# verifier 使用指南

破解 H6「运动员=裁判」：auto 模式下 agent 自建 issue、自写 spec、自实现、自声明门禁通过、自关 issue——
当执行者兼验收者，IDD 的「验收」在语义上坍缩。`verifier` 是**独立验收轴**：由不同于实现者的
agent（或人）**复算**门禁，产出独立 verdict。

```
shadow-implementer / equivalence-baseline（实现者自证）
    → verifier（本 skill，独立复算）        ← 身份必须不同于实现者
    → cutover-author（据 verdict 决策）
```

## 与引擎 gate 的关系（互补，非重复）

- **引擎 gate**（ROADMAP §10，`stage complete` 时）：无人值守的机器闸，`auto` 不可绕。快、确定、但只查白名单谓词。
- **verifier**（本 skill）：一个**独立主体**的复算 + 判断，能查引擎 gate 覆盖不到的语义项
  （divergence↔ADR 是否对齐、coverage 表是否名副其实、fixture 是否被事后篡改）。
- 二者叠加 = 双保险：verifier 出 accept，引擎在 cutover 仍会独立再跑白名单 gate。

## 铁律

1. **身份分离**：`verifier_identity` 必须 ≠ 实现者。同一 agent 连跑实现与验收 = 没验收。
2. **亲自复算**：报告里粘贴**你自己**跑的命令与原始输出，不转述实现者结论。
3. **只判不改**：verifier 不改代码、不改门禁数字；发现问题回给实现者，重跑。
4. **抓造假**：`summary.golden_pass` 与逐项复算不符、pin 是占位符、divergence 无 ADR → 一律 reject。

## 建议接法

在 `migration-slice-delivery` / `migration-preserve` 的 `cutover` 之前插一个 **gate-only** 的 verifier stage
（P5，无 artifact 亦可，或产 verification-report），或在项目 `approval_mode` 里把 cutover 的人验交给独立 verifier。
参见 ROADMAP §10 的 gate/approval 正交轴。
