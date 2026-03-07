# Release Guide

本文档说明如何发布 Popsicle 的新版本。

## 发布流程

### 1. 确认代码就绪

```bash
# 确保在 main 分支且代码是最新的
git checkout main
git pull origin main

# 本地验证构建通过
cargo build -p popsicle-cli --release
cargo test
```

### 2. 更新版本号

修改 `Cargo.toml` 根工作区中的版本号：

```toml
[workspace.package]
version = "0.2.0"  # 更新为新版本
```

提交版本变更：

```bash
git add Cargo.toml
git commit -m "chore: bump version to v0.2.0"
git push origin main
```

### 3. 创建标签并推送

```bash
git tag v0.2.0
git push origin v0.2.0
```

推送标签后，GitHub Actions 会自动触发 Release 工作流。

### 4. 监控构建

前往 [GitHub Actions](https://github.com/curtiseng/popsicle/actions) 页面查看构建状态。

工作流会为以下 4 个平台并行构建：

| 平台 | Target | Runner | 产物格式 |
|------|--------|--------|----------|
| macOS (Apple Silicon) | `aarch64-apple-darwin` | `macos-14` | `.tar.gz` |
| macOS (Intel) | `x86_64-apple-darwin` | `macos-14` (交叉编译) | `.tar.gz` |
| Linux (x86_64) | `x86_64-unknown-linux-gnu` | `ubuntu-22.04` | `.tar.gz` |
| Windows (x86_64) | `x86_64-pc-windows-msvc` | `windows-latest` | `.zip` |

### 5. 验证 Release

构建完成后，前往 [Releases](https://github.com/curtiseng/popsicle/releases) 页面确认：

- Release 已创建，标题为 tag 名称（如 `v0.2.0`）
- Release notes 已自动生成
- 4 个平台的构建产物均已上传

## CI 工作流详解

工作流文件位于 `.github/workflows/release.yml`。

### 触发条件

```yaml
on:
  push:
    tags:
      - "v*"
```

仅当推送以 `v` 开头的标签时触发（如 `v0.1.0`、`v1.0.0-beta`）。

### build Job

并行构建 4 个平台的二进制文件。每个平台的步骤：

1. **Checkout** — 检出代码
2. **Install Rust** — 安装 stable Rust 工具链 + 目标平台 target
3. **Install Node.js** — 安装 Node.js 22（构建前端需要）
4. **Install Linux deps** — 仅 Linux：安装 Tauri 所需的系统库（WebKitGTK 等）
5. **npm ci** — 安装前端依赖（使用 `package-lock.json` 锁定版本）
6. **Build frontend** — 执行 `tsc -b && vite build`，产物输出到 `ui/dist/`
7. **Generate Windows icon** — 仅 Windows：将 `icon.png` 转换为 `icon.ico`（Tauri Windows 构建需要）
8. **Build binary** — `cargo build -p popsicle-cli --features ui --release --target <target>`
9. **Package** — Unix 平台打包为 `.tar.gz`，Windows 打包为 `.zip`
10. **Upload artifact** — 上传到 GitHub Actions artifacts（供 release job 使用）

### release Job

等待所有 build job 完成后执行：

1. **Download artifacts** — 下载所有平台的构建产物
2. **Create Release** — 使用 `softprops/action-gh-release` 创建 GitHub Release，自动生成 release notes，附带所有构建产物

## 手动操作

### 删除标签重新发布

如果构建失败需要修复后重新发布同一版本：

```bash
# 删除本地和远程标签
git tag -d v0.2.0
git push origin --delete v0.2.0

# 修复问题后重新提交
git add .
git commit -m "fix: resolve build issue"
git push origin main

# 重新创建标签
git tag v0.2.0
git push origin v0.2.0
```

注意：如果 Release 已经创建，需要先在 GitHub Releases 页面手动删除旧的 Release。

### 本地构建 Release 包

不依赖 CI，在本地构建发布包：

```bash
# 构建前端
cd ui && npm install && npm run build && cd ..

# 构建 release 二进制（当前平台）
cargo build -p popsicle-cli --features ui --release

# 二进制位于
ls -la target/release/popsicle
```

### 交叉编译（macOS 上为两个架构构建）

```bash
# 添加 target
rustup target add x86_64-apple-darwin aarch64-apple-darwin

# 构建 Intel 版本
cargo build -p popsicle-cli --features ui --release --target x86_64-apple-darwin

# 构建 Apple Silicon 版本
cargo build -p popsicle-cli --features ui --release --target aarch64-apple-darwin
```

## 版本号规范

遵循 [Semantic Versioning](https://semver.org/)：

- **MAJOR** (`v1.0.0` → `v2.0.0`) — 不兼容的 API 变更
- **MINOR** (`v1.0.0` → `v1.1.0`) — 向后兼容的功能新增
- **PATCH** (`v1.0.0` → `v1.0.1`) — 向后兼容的 bug 修复
- **Pre-release** — `v1.0.0-alpha`、`v1.0.0-beta.1`、`v1.0.0-rc.1`
