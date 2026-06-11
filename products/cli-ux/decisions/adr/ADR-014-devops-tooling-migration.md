# ADR-014 · legacy devops tooling migration

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux
> **Generated-by**: cutover-author（PROJ-26）
> **Source-Baseline**: `docs/baseline/2026-06-11/cli-ux-devops/`
> **Legacy-Pin**: `legacy/popsicle` @ c76d729（见 LEGACY_PIN.md）

## Context

新工作区自托管闭环已可用(ADR-010~013),但 DevOps 面是空的:无 Makefile、
无安装脚本、无 git hooks、无 CI/Release 工作流。legacy popsicle 在 pin 的
c76d729 上有一套完整的工具脚本,需要按新架构事实(纯 Rust、无 UI、
completions deferred)迁移而非照搬。

## Decision

1. **Makefile**:保留 legacy 的 `check = fmt + clippy + test`(均
   `-Dwarnings`)与 `install-hooks`;删除 `build-ui`;新增 IDD 专属目标
   `golden`(全链基线)与 `intent`(Z3 校验)、`fmt-fix`、`install`。
2. **scripts/install.sh**:保留 prefix/cargo-install 双路径、PATH 检查、
   卸载;**移除 UI 构建**(新架构无 UI)与 **completions 安装**
   (`completions` 是 deferred 命令,裁决落地后再恢复);新增工作区
   provenance 提示(开发工作区内优先 `./target/debug/popsicle`)。
3. **hooks/pre-commit**:三件套原样迁移。
4. **.github/workflows/ci.yml**:删 node/npm/webkit 工具链;保留
   fmt/clippy/test 三道门。cargo test 已覆盖 golden_001-011、双后端测试与
   隔离 e2e smoke;shell golden 链与 intent-validate 需 intent-lang,留在
   本地 `make golden` / `make intent`(O-401 follow-up:intent-lang 进 CI)。
5. **.github/workflows/release.yml**:4 平台矩阵保留(darwin x86_64/aarch64、
   linux x86_64、windows x86_64),删 UI 步骤与 Linux webkit 依赖;
   rusqlite bundled 仅需 runner 自带 cc。
6. **前置整备**:迁移 CI 前清零 fmt/clippy 历史欠账(2 处 fmt 漂移 +
   3 处 clippy:bool_assert_comparison / ptr_arg + collapsible_str_replace /
   cmp_owned),保证三件套即刻可绿。

## Divergences / Deferred

- **D-401**:completions 安装不迁移(依赖 deferred 命令)。
- **D-402**:UI 相关一切不迁移(新架构无 UI)。
- **O-401**:CI 不含 shell golden 链 / intent-validate(intent-lang 工具链
  未进 CI);本地 `make golden` / `make intent` 承担。
- hooks 安装是手动 opt-in(`make install-hooks`),与 legacy 一致。

## Compliance

| 门禁 | 证据 | 结果 |
|---|---|---|
| Intent Z3 | `tool run intent-validate path=products` exit 0 | pass |
| Golden | 全链 27/27(23 回归 + devops 4)| pass |
| make check | fmt + clippy + test(-Dwarnings)全绿 | pass |
| Dogfood | install.sh 真实安装→运行→卸载闭环;install-hooks 落位;本提交过 pre-commit hook | pass |

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-26 slice-delivery cutover stage(user 授权 agent `--confirm`,见会话记录)
- **Approval date**: 2026-06-11
