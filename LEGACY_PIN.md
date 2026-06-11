# Legacy Pin

> 这份文件记录 `legacy/popsicle/` submodule 锁定的具体 commit、所有已知的 patch
> 与限制。所有后续 fact-extractor / 等价性测试 / 切流操作都以这里记录的 SHA 为 ground truth。

## Pinning

| 字段 | 值 |
|---|---|
| Submodule | `legacy/popsicle/` |
| Upstream URL | `https://github.com/popsicle-lab/popsicle.git` |
| Pinned commit | `c76d729db91c59009f0fa8f7c6f1e499eb0c7eb1` |
| Commit message | `[feat] 重新迁移组织仓库` |
| Pin reason | bootstrap 当日父仓库 HEAD；含已 commit 的 3 个工程 crate（core / cli / sync）、UI、Cargo workspace |
| License | Apache-2.0（与 popsicle-new MIT 兼容 —— MIT 可消费 Apache-2.0 内容；注：本字段 init stage 错记为 MIT，fact-extractor 2026-06-08 从 `legacy/popsicle/LICENSE` + 根 `Cargo.toml` 验证后修正）|

## ⚠️ Known limitations of this pin

1. **未 commit 资产不在 submodule 里**：父仓库 popsicle 在 `c76d729` 时有大量 untracked
   文件未提交，包括：
   - `intent-coder/` 整个 v0.3.0 技能包（10 个 skill + 1 pipeline + 1 tool）
   - `vender/intent-lang/`（intent-lang Rust 实现，含 `intent-cli` / `intent-core` / `intent-syntax`）
   - `intent-devlopment/` 旧版技能包（注：拼写错误的目录名，是 intent-coder 的早期版本，将被淘汰）
   - 多份 `docs/*.md`

   **影响**：fact-extractor 在 submodule 内只看 commit 过的代码（crates/、ui/、Makefile 等）。
   对未 commit 的资产，popsicle-new 通过相对路径 `../intent-coder` / `../vender/intent-lang`
   临时引用——见 [`Cargo.toml`](Cargo.toml) workspace 声明 / [`.popsicle/config.toml`](.popsicle/config.toml) module 路径。

   **长期解决**：用户应把这些资产在父 popsicle 提交并打新 tag，本文件 pin 升级到新 SHA。

2. **submodule URL 公开可达，但部分历史可能未推送**：`c76d729` 已确认在 `origin/main`
   （`git branch -r --contains` 校验通过）。任何重 pin 操作必须重新验证。

## Known patches

> 迁移期间对 legacy `popsicle/` 的"为让 legacy 能跑"必要修复——**不是**改进。任何
> patch 必须在这里登记，且解释为什么不能等到下一次 pin 升级。

| 日期 | 文件（在 legacy popsicle 中的路径）| 修复 | 理由 |
|---|---|---|---|
| 2026-06-08 | `intent-coder/skills/intent-consistency-check/skill.yaml` | `inputs[0]` 字段格式 `type:` → `from_skill: + artifact_type:` | 阻塞性 bug：popsicle CLI 加载 module 时 schema 校验失败，导致 10 个 skill 全部加载不到。属于"让 legacy 能跑"的例外修复。fact-extractor 应把这条 schema drift 风险写进 `unsafe-risk-report`。 |

## How to update this pin

1. 在父 popsicle 把待提交资产 commit 并推到 `origin/main`
2. 在 `popsicle-new/legacy/popsicle/` 跑 `git fetch && git checkout <new-sha>`
3. 在 popsicle-new 根目录 `git add legacy/popsicle && git commit -m "chore(legacy): re-pin to <new-sha>"`
4. 本文件的 "Pinning" 表 + "Known patches" 表都要刷新（patch 已上游则移除条目）
5. 受影响的 baseline 必须重跑（`docs/baseline/` 下的产物作废）
