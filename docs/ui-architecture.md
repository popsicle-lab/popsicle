# UI 架构说明

Popsicle 的桌面 UI 采用 **Tauri + React** 技术栈，Tauri 后端代码**内嵌在 CLI crate 中**，通过 feature flag 控制编译。

## 架构决策

UI 的 Tauri 后端（commands、watcher）直接集成在 `crates/popsicle-cli/` 中，而非作为独立的 Tauri crate。这意味着：

- **单一入口**：用户通过 `popsicle ui` 命令启动桌面应用，无需安装额外二进制
- **一次构建**：`cargo build --features ui` 同时编译 CLI 和 UI 后端
- **避免代码重复**：Tauri commands 只维护一份，在 `crates/popsicle-cli/src/ui/` 中

## 目录结构

```
crates/popsicle-cli/
├── src/
│   ├── main.rs              # CLI 入口，#[cfg(feature = "ui")] 启动 Tauri
│   ├── commands/             # CLI 子命令
│   └── ui/                   # Tauri 后端（仅在 feature = "ui" 时编译）
│       ├── mod.rs            # Tauri Builder 配置和启动
│       ├── commands.rs       # Tauri invoke commands
│       └── watcher.rs        # 文件变更监听
├── tauri.conf.json           # Tauri 配置
├── icons/                    # 应用图标
└── Cargo.toml                # ui feature 定义
ui/
├── src/                      # React 前端源码
├── dist/                     # 前端构建产物（嵌入二进制）
└── package.json
```

## 构建方式

```bash
# 构建前端
cd ui && npm install && npm run build && cd ..

# 构建带 UI 的 CLI（包含桌面应用）
cargo build -p popsicle-cli --features ui --release

# 启动桌面应用
./target/release/popsicle ui
```

## 开发注意事项

### 添加新的 Tauri Command

当需要添加新的 Tauri command 时，需要修改两个文件：

1. **`crates/popsicle-cli/src/ui/commands.rs`** — 实现 `#[tauri::command]` 函数
2. **`crates/popsicle-cli/src/ui/mod.rs`** — 在 `invoke_handler` 中注册命令

同时在前端添加对应调用：

3. **`ui/src/hooks/useTauri.ts`** — 添加 TypeScript 接口和 `invoke()` 调用

> **重要**：项目中不应存在 `ui/src-tauri/` 目录。所有 Tauri 后端代码统一维护在
> `crates/popsicle-cli/src/ui/` 中。如果发现 `ui/src-tauri/` 重新出现（例如通过
> `cargo tauri init` 生成），应立即删除并将代码合并到 CLI crate 中。
