---
artifact: verification-report
slug: {slug}
generated_by: verifier
slice: {slice-name}
last_updated: {date}
verifier_identity: "<验收者身份：须不同于实现者；人名 / 独立 agent id>"
verdict: reject            # accept | reject —— 复算全过才 accept
query_anchors:
  - "这个切片的门禁是真过了还是被编造的？"
  - "独立复算的结果是什么？"
---

# 独立验收报告 — {slug}

> 由 `verifier` 产出，**执行/验收分离**（feedback H6）。本报告的每个结论来自
> 验收者**亲自复算**，不采信实现者手填的数字。verdict=reject 直到全部复算通过。

## Recomputed Gates

> 每行：门禁 / 我跑的命令 / 我观察到的结果 / 与实现者声称是否一致。

| 门禁 | 我跑的命令 | 我观察到的 | 实现者声称 | 一致？ |
|---|---|---|---|---|
| cargo test | `cargo test` | exit 0 | 全绿 | ✅ |
| golden_pass 重算 | 数 baseline.yaml goldens[status=pass] | 5 | 5 | ✅ |
| golden_pass>=5 | — | 5>=5 | pass | ✅ |
| legacy_pin 真实 | `git -C legacy/… rev-parse HEAD` | `<sha>` | `<sha>` | ✅ |
| realized_by 解析 | `popsicle tool run intent-validate path=products/<p>/intents` | 0 未解析 | pass | ✅ |

```
（粘贴你实跑的命令与原始输出——这是报告的证据基，不能省）
```

## Divergence Audit

> equivalence-report 的每条 Divergence ↔ Accepted ADR 双向对齐核对。

| Divergence | 对应 ADR | ADR 状态 | 对齐？ |
|---|---|---|---|
| D-001 | ADR-0003 | Accepted | ✅ |

- ADR 声明但 report 未列的有意分叉：（无）
- report 列出但无 ADR 的分叉：（无）→ 有则 **reject**

## Verdict

- **结论**：accept / reject
- **理由**：……
- 若 reject：逐条列「实现者声称 X，我复算得 Y」，指明修复方向（回实现者，不由 verifier 改）。

## Verification Checklist

- [ ] 验收者身份 ≠ 实现者（H6 硬要求）
- [ ] 每条门禁都**亲自跑过**，粘贴了命令与原始输出
- [ ] golden_pass 从明细**自己数**过，与 summary 比对
- [ ] legacy_pin 真实且 == submodule HEAD
- [ ] 每条 divergence ↔ Accepted ADR 双向对齐
- [ ] rewrite 切片：fixture sha256 复算一致
- [ ] verdict 与理由明确；reject 时给了差异与修复方向
