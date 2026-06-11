# Legacy Pin

> 这份文件记录 `legacy/popsicle/` submodule 锁定的具体 commit、跟踪分支、所有已知的 patch
> 与限制。所有后续 fact-extractor / 等价性测试 / 切流操作都以这里记录的 SHA 为 ground truth。

## Pinning

| 字段 | 值 |
|---|---|
| Submodule | `legacy/popsicle/` |
| Upstream URL | `https://github.com/popsicle-lab/popsicle.git` |
| Tracked branch | `backup-v0.5`（`.gitmodules` `branch = backup-v0.5`）|
| Pinned commit | `6f7be399b4b86d9baf6165589f66229f03a3ad19` |
| Commit message | `feat(intent-coder): Improve intent coder.` |
| Pin reason | 迁移前 **legacy 全量 popsicle**（`backup-v0.5`）；含 `popsicle-core` / `popsicle-cli` / `popsicle-sync`、Tauri UI、`intent-coder/`、`vender/intent-lang/` |
| License | Apache-2.0（与 popsicle MIT 兼容 —— MIT 可消费 Apache-2.0 内容）|

### 历史 pin（仅供 baseline 追溯）

| 日期 | SHA | 说明 |
|---|---|---|
| 2026-06-08 bootstrap | `c76d729` | 当日 `main` HEAD；`intent-coder` / `vender` 当时未 commit。早期 `docs/baseline/2026-06-08/` 事实基基于此 SHA。|

## ⚠️ Known limitations of this pin

1. **与 `main` 分支不同源**：submodule 跟踪 `backup-v0.5`（legacy 单体），父仓库 `main` 是 IDD 迁移后的新 crate 布局。golden 对账时 legacy 侧应读 submodule 内路径，不要混用父仓库根目录代码。

2. **submodule 更新**：`git submodule update --remote legacy/popsicle` 会拉取 `backup-v0.5` 最新 tip；重 pin 后必须刷新本文件并评估是否重跑 `docs/baseline/`。

3. **`c76d729` baseline 仍有效**：2026-06-08 fact-extraction 与部分 golden 脚本锚定在旧 SHA；追溯时以各 baseline README 头部注明的 commit 为准，不要求 submodule 回退。

## Known patches

> 迁移期间对 legacy `popsicle/` 的"为让 legacy 能跑"必要修复——**不是**改进。任何
> patch 必须在这里登记，且解释为什么不能等到下一次 pin 升级。

| 日期 | 文件（在 legacy popsicle 中的路径）| 修复 | 理由 |
|---|---|---|---|
| — | — | （无活跃 patch）| `backup-v0.5` 已含修正后的 `intent-consistency-check/skill.yaml`（`from_skill` + `artifact_type`）|

## How to update this pin

1. 在 upstream `backup-v0.5` 合并所需变更并推送
2. 在 `legacy/popsicle/` 跑 `git fetch origin backup-v0.5 && git checkout backup-v0.5 && git pull`
3. 在仓库根目录 `git add legacy/popsicle .gitmodules && git commit -m "chore(legacy): re-pin backup-v0.5 to <new-sha>"`
4. 刷新本文件 "Pinning" 表；已上游的 patch 从 "Known patches" 移除
5. 受影响的 baseline 必须重跑（`docs/baseline/` 下的产物作废）
