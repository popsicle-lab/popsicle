# `{product_name}` Tasks Index

> **本文件由 prd-writer / living-doc-author 自动维护，请勿手工编辑**。
> 重新生成命令：`popsicle skill start living-doc-author --target tasks-index`
> Last-Generated: {YYYY-MM-DD}

本目录按**用户旅程阶段**组织 task 文件。`{product_name}` 的所有用户可见行为都
被拆解为 task chunk，每个 chunk 可被 AI 独立召回回答用户问题。

---

## 索引

### onboarding/ — 首次接触到首次成功

| Task | 用户问句锚点 | Audience | Last-Updated |
|---|---|---|---|
| [T-0001 我第一次跑 intent check 直到拿到 PASS](onboarding/T-0001-first-time-verify.md) | 怎么第一次跑校验？/ 5 秒没结果是卡死了吗？ | new-user | 2026-05-13 |
| [T-0002 拿到 FAIL 后看反例并调整 spec](onboarding/T-0002-handle-fail-counterexample.md) | FAIL 怎么看？/ 反例怎么读？ | new-user | 2026-05-13 |

### daily-ops/ — 日常使用

| Task | 用户问句锚点 | Audience | Last-Updated |
|---|---|---|---|
| [T-0010 批量验证一组 .intent 文件](daily-ops/T-0010-batch-verify.md) | 怎么一次校验一堆？ | end-user | 2026-05-13 |
| [T-0011 把 spec 分享给团队成员](daily-ops/T-0011-share-spec-with-team.md) | 怎么发给同事？ | end-user | 2026-05-13 |

### troubleshooting/ — 故障排查

| Task | 用户问句锚点 | Audience | Last-Updated |
|---|---|---|---|
| [T-0020 验证超时怎么办](troubleshooting/T-0020-timeout-recovery.md) | 一直转圈怎么办？/ 卡 30 秒是网络问题吗？ | end-user | 2026-05-13 |
| [T-0021 invariants 互相矛盾的报错](troubleshooting/T-0021-conflicting-invariants.md) | invariants 冲突怎么办？ | end-user | 2026-05-13 |

### admin/ — 管理类（配额 / 权限 / 审计）

| Task | 用户问句锚点 | Audience | Last-Updated |
|---|---|---|---|
| [T-0030 给团队成员开通验证额度](admin/T-0030-quota-management.md) | 怎么加额度？ | admin | 2026-05-13 |
| [T-0031 导出 30 天审计日志](admin/T-0031-audit-log-export.md) | 怎么看谁跑过哪些验证？ | admin / 合规 | 2026-05-13 |

### lifecycle/ — 终止 / 迁出 / 续费

| Task | 用户问句锚点 | Audience | Last-Updated |
|---|---|---|---|
| [T-0040 注销账号前导出所有数据](lifecycle/T-0040-export-and-delete.md) | 不用了怎么导数据？ | end-user / admin | 2026-05-13 |

---

## 健康度统计

> 由 living-doc-author 在重跑时刷新。本统计是「文档腐烂预警」的核心信号。

| 旅程阶段 | Task 数 | 平均行数 | 上次更新最久的 task | 未引用的 task |
|---|---|---|---|---|
| onboarding | 2 | 87 | T-0002（30 天前）| 无 |
| daily-ops | 2 | 102 | T-0011（45 天前）| 无 |
| troubleshooting | 2 | 145 | T-0020（12 天前）| 无 |
| admin | 2 | 76 | T-0031（60 天前）| **T-0031**（无任何反向引用）|
| lifecycle | 1 | 64 | T-0040（90 天前）| **T-0040**（无任何反向引用）|

⚠️ **未引用的 task** 是 AI 反馈闭环的输入：超 90 天无反向引用的 task 进入「归档
评审」流程，由 PM 决定是否真的有用户在用。

---

## 跨 Product 旅程

涉及本 product 的跨 product 旅程：

- [J-0001 新员工入职当天开通所有权限](../../../docs/user-journeys/J-0001-new-employee-onboarding.md)
  —— 本 product 承担 Stage 1 / 3

---

## 维护规则

1. **新增 task** 由 `prd-writer` skill 通过 PDR 引入
2. **修改 task** 必须有新 PDR（charter 第 3 条铁律）
3. **删除 task** 必须有标注 `Supersedes` 的 PDR 显式废止
4. **重新分类 task**（在 5 个目录之间移动）算修改，需要 PDR
5. **重命名 slug**（不改 task_id）算小改，不需要新 PDR，但要更新 Last-Updated
