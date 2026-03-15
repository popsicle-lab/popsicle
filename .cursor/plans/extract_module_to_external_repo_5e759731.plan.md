---
name: Extract module to external repo
overview: 将 Popsicle 的 17 个 Skills 和 5 个 Pipelines 提取到外部 Git 仓库 `popsclice-spec-development`，重命名 adr/prd/rfc 为 adr-writer/prd-writer/rfc-writer，然后从 Popsicle 主仓库移除内置模块，更新代码使 init 不再依赖内置文件，最后编译并测试模块安装流程。
todos:
  - id: ext-repo-init
    content: Clone 外部仓库，复制 skills/pipelines/module.yaml，重命名 adr/prd/rfc 目录和 skill name
    status: completed
  - id: ext-repo-refs
    content: 更新外部仓库中所有 from_skill、pipeline skill、prompt、guide.md 中的引用
    status: completed
  - id: ext-repo-push
    content: 提交并推送到 github:curtiseng/popsclice-spec-development
    status: completed
  - id: popsicle-remove-builtin
    content: 删除 popsicle 主仓库的 skills/、pipelines/、module.yaml
    status: completed
  - id: popsicle-update-code
    content: 更新 init.rs（移除内置假设）、scaffold.rs（处理空数组）、module.rs（builtin 分支提示）
    status: completed
  - id: popsicle-update-docs
    content: 更新 README.md、module-guide.md、skill-guide.md 中的内置引用和 skill 名
    status: completed
  - id: build-test
    content: cargo build + cargo test + 手动测试 init -> module install -> skill/pipeline list
    status: completed
isProject: false
---

# 提取 Module 到外部仓库并重命名 Skills

## Phase 1: 创建外部仓库 `popsclice-spec-development`

### 1.1 初始化仓库结构

```
popsclice-spec-development/
├── module.yaml          # name: spec-development, version: 0.1.0
├── skills/              # 17 个 skill（3 个重命名）
│   ├── adr-writer/      # 原 adr
│   ├── prd-writer/      # 原 prd
│   ├── rfc-writer/      # 原 rfc
│   ├── arch-debate/
│   ├── implementation/
│   ├── ... (其余 12 个 skill 不变)
└── pipelines/           # 5 个 pipeline（更新引用）
```

### 1.2 Skill 重命名（仅改 skill name，不改 artifact_type）


| 原名  | 新名         | artifact_type（不变） |
| --- | ---------- | ----------------- |
| adr | adr-writer | adr               |
| prd | prd-writer | prd               |
| rfc | rfc-writer | rfc               |


保留原有 artifact_type 以最小化变更范围——`from_skill` 改名即可，`artifact_type` 无需改。

### 1.3 更新所有 `from_skill` 引用（8 个 skill.yaml）


| 文件                                     | 变更                                                                   |
| -------------------------------------- | -------------------------------------------------------------------- |
| `skills/adr-writer/skill.yaml`         | `name: adr-writer`, `from_skill: prd` -> `prd-writer`                |
| `skills/rfc-writer/skill.yaml`         | `name: rfc-writer`, `from_skill: prd` -> `prd-writer`                |
| `skills/prd-writer/skill.yaml`         | `name: prd-writer`                                                   |
| `skills/arch-debate/skill.yaml`        | `from_skill: prd` -> `prd-writer`                                    |
| `skills/implementation/skill.yaml`     | `from_skill: rfc` -> `rfc-writer`, `from_skill: adr` -> `adr-writer` |
| `skills/priority-test-spec/skill.yaml` | `from_skill: adr` -> `adr-writer`                                    |
| `skills/api-test-spec/skill.yaml`      | `from_skill: adr` -> `adr-writer`                                    |
| `skills/e2e-test-spec/skill.yaml`      | `from_skill: prd` -> `prd-writer`, `from_skill: adr` -> `adr-writer` |
| `skills/ui-test/skill.yaml`            | `from_skill: prd` -> `prd-writer`                                    |


### 1.4 更新 prompt 中的 skill 名引用

- `adr-writer/skill.yaml` prompt: `popsicle prompt adr` -> `popsicle prompt adr-writer`
- `prd-writer/skill.yaml` prompt: `popsicle prompt prd` -> `popsicle prompt prd-writer`
- `rfc-writer/skill.yaml` prompt: `popsicle prompt rfc` -> `popsicle prompt rfc-writer`

### 1.5 更新 Pipeline YAML（3 个文件）

- [pipelines/full-sdlc.pipeline.yaml](pipelines/full-sdlc.pipeline.yaml): `skill: prd` -> `prd-writer`, `- rfc` -> `rfc-writer`, `- adr` -> `adr-writer`
- [pipelines/tech-sdlc.pipeline.yaml](pipelines/tech-sdlc.pipeline.yaml): `skills: [rfc, adr]` -> `[rfc-writer, adr-writer]`
- [pipelines/design-only.pipeline.yaml](pipelines/design-only.pipeline.yaml): `skill: prd` -> `prd-writer`, `skills: [rfc, adr]` -> `[rfc-writer, adr-writer]`
- `impl-test` 和 `test-only` 无直接引用，无需修改

### 1.6 更新 guide.md 中的引用

- `skills/arch-debate/guide.md` 和 `references/output-templates.md`：`adr` skill -> `adr-writer`
- `skills/product-debate/guide.md` 和 `references/output-templates.md`：`prd` skill -> `prd-writer`
- `skills/adr-writer/guide.md`：更新 skill 名引用
- `skills/prd-writer/guide.md`：更新 skill 名引用

### 1.7 提交并推送

```bash
git add . && git commit -m "Initial module: spec-development with renamed skills"
git push origin main
```

---

## Phase 2: 更新 Popsicle 主仓库

### 2.1 删除内置模块文件

- 删除 `skills/` 目录（全部 17 个 skill）
- 删除 `pipelines/` 目录（全部 5 个 pipeline）
- 删除 `module.yaml`

### 2.2 更新 `init.rs` — 移除内置模块假设

文件：[crates/popsicle-cli/src/commands/init.rs](crates/popsicle-cli/src/commands/init.rs)

- 默认 `config.toml` 移除 `[module]` 段（不再写入 `name = "official"`, `source = "builtin"`）
- 技能加载从硬编码 `"official"` 路径改为调用 `popsicle_core::helpers::load_registry()`

```rust
// Before (lines 120-136): hardcoded "official"
let module_skills = project_root.join(".popsicle").join("modules").join("official").join("skills");

// After: use helpers
let registry = popsicle_core::helpers::load_registry(&project_root).unwrap_or_default();
```

### 2.3 更新 `scaffold.rs` — 优雅处理空内置

文件：[crates/popsicle-core/src/scaffold.rs](crates/popsicle-core/src/scaffold.rs)

- `upgrade_builtins()`: 如果 `BUILTIN_FILES` 为空，提前返回而非创建 "0.0.0" 版本
- `embedded_module_version()`: 空数组时返回 `None`（已满足）

### 2.4 更新 `module.rs` upgrade 命令

文件：[crates/popsicle-cli/src/commands/module.rs](crates/popsicle-cli/src/commands/module.rs)

- `upgrade_module` 中 `source == "builtin"` 分支：如果没有嵌入文件，提示用户使用 `module install`

### 2.5 更新 README.md

- Quick Start 添加 `popsicle module install github:curtiseng/popsclice-spec-development`
- "Built-in Skills" 表格改为说明需安装模块
- "Built-in Pipelines" 同上
- Pipeline 流程图中 `prd` -> `prd-writer`, `rfc` -> `rfc-writer`, `adr` -> `adr-writer`

### 2.6 更新文档

- [docs/module-guide.md](docs/module-guide.md): 更新内置模块说明，添加新仓库作为示例
- [docs/skill-guide.md](docs/skill-guide.md): 更新引用的 skill 名示例

---

## Phase 3: 编译 + 测试

1. `cargo build -p popsicle-cli --release` — 确认编译通过
2. `cargo test` — 确认所有测试通过（测试中的 "prd"/"rfc"/"adr" 是测试数据，与实际 skill 名无关）
3. 在一个临时项目中测试完整流程：
  - `popsicle init` — 不安装任何模块
  - `popsicle module install github:curtiseng/popsclice-spec-development` — 安装外部模块
  - `popsicle skill list` — 验证 17 个 skill（含 adr-writer/prd-writer/rfc-writer）
  - `popsicle pipeline list` — 验证 5 个 pipeline
  - `popsicle module show` — 验证元数据
  - `popsicle pipeline run full-sdlc --title "test"` — 验证 pipeline 可启动
  - `popsicle pipeline next` — 验证 advisor 能推荐下一步

