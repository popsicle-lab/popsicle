# ADR-012 · self-host usability completion

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux
> **Generated-by**: cutover-author（PROJ-24）
> **Source-Baseline**: `docs/baseline/2026-06-11/cli-ux-usability/`
> **Closes**: ADR-011 follow-ups D-101 / O-102 · PDR-001 `doc check` 承诺 · D4 issue 命令族 close 缺口

## Context

ADR-011 对齐了命令面,但日常可用性仍有四个缺口:PDR-001 承诺的 `doc check`
(checklist 替代物)未实现;issue 生命周期没有 close,run 完成后 issue 永远
`in_progress`;issue 类型默认管线指向从未打包的模板(D-101);workflow smoke
测试在真实工作区堆积残留(O-102,清理时已累计 19 个 smoke issue)。

## Decision

1. **`doc check <doc_id>` 落地**:校验 frontmatter(id/doc_type/title)、正文
   是否有标题之外的实文、占位符扫描、checkbox 统计;未通过时 `status: failed`
   且进程退出码 1(`main.rs` 对所有 failed 响应统一非零退出)。
2. **`issue close <key>` 落地**:存在未完成 run 时返回 `active-run:` 可行动
   错误;run 完成后置 `done` 并持久化。生命周期闭环:create → start → stages
   → completed → close。
3. **默认管线重映射(根治 D-101)**:product→`greenfield-product-spec`、
   technical→`tech-decision`、bug→`bugfix`(新增最小两阶段模板,无审批)、
   idea→`tech-decision`;并以 golden 强制"默认名必须 bundled"。
4. **模板自愈**:`load_pipeline_def` 找不到模板时,若属 bundled 集合则按需
   安装进 `.popsicle/pipelines/`,旧工作区无需手动升级。
5. **smoke 隔离(根治 O-102)**:smoke 的全部变更操作迁入临时工作区,真实
   工作区零增量(golden-004 断言);存量 19 个 smoke issue/run/session/artifact
   已清理(备份保留)。

## Divergences / Deferred

- **O-201**:doc check 占位符扫描不豁免反引号/代码块字面量(首次 dogfood 即
  误报一次)。Phase 2 优化。
- **O-202**:state.tsv 手工清理有误删风险(本次发生过一次,靠备份恢复);
  需要 `admin prune` 类命令替代手写脚本。
- 字段命名不一致(doc create 返回 `id`,doc check 返回 `doc_id`)→ Phase 2 统一。
- PROJ-11 SQLite Phase 2 不变。

## Compliance

| 门禁 | 证据 | 结果 |
|---|---|---|
| Intent Z3 | `tool run intent-validate path=products` exit 0 | pass |
| Golden ≥5 | usability run-all 18 scripts(8+5 回归 + 5 新)| pass |
| cargo test | cli-ux + skill-runtime + storage + artifact-system 全绿 | pass |
| Dogfood | doc-48/doc-49 经 `doc check` 通过;本 run 全程用新命令推进 | pass |

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-24 slice-delivery cutover stage(user 授权 agent `--confirm`,见会话记录)
- **Approval date**: 2026-06-11
