# Priority Test Gate

铁律：`NO P0 FEATURE WITHOUT UNIT TESTS. UNCHANGED + PASSING = SKIP.`

## When To Use

- 用户要求执行 P0 测试或检查功能测试覆盖
- 大批量代码变更后的回归验证 / 新功能合入后的质量门禁
- 用户指定某个文件或 Feature ID 进行测试

不适用：纯文档变更、前端 UI 测试（→ `ui-test`）、边界安全测试（→ `boundary-tdd`）

## Spec Sources

从 `testsuite/specs/manifest.yaml` 的 `pipelines.unit.spec_sources` 获取：

- `feature_map` → 产品功能全景图
- `tech_map` → 技术需求表
- `crate_mapping` → `testsuite/specs/unit/crate-mapping.yaml`（Feature ID → crate 映射 + priority_rules）

## State Architecture

状态拆分为两个文件，职责分离：

| 文件 | 路径 | Git | 用途 |
|------|------|-----|------|
| Coverage Registry | `testsuite/state/priority-test-gate.json` | 提交 | 团队共享的覆盖率记录：feature 元信息、test_status、dimensions、conflicts |
| Local Cache | `state/local-cache.json`（相对于本 skill 目录） | 忽略 | 本地增量缓存：source_hash、last_run |

### Coverage Registry 字段

```jsonc
{
  "version": 4,
  "updated_at": "...",
  "features": {
    "<FEATURE_ID>": {
      "name": "...",
      "priority": "P0",
      "crate": "crates/...",
      "source_files": ["..."],      // 可选，增强条目才有
      "test_locations": ["..."],     // 可选
      "test_count": 8,              // 可选
      "test_status": "pass",
      "dimensions": { "happy": true, "boundary": true, "error": true },
      "conflicts": []               // 可选
    }
  }
}
```

### Local Cache 字段

```jsonc
{
  "version": 4,
  "features": {
    "<FEATURE_ID>": {
      "source_hash": "f6837be2...",  // SHA256 前 16 位
      "last_run": "2026-02-21T..."
    }
  }
}
```

## Workflow

### Step 1: Auto-Discover — 自动发现并分级

1. 读取 feature_map 和 tech_map，提取 `id`, `name`, `status`, `test_coverage`
2. 按 `crate-mapping.yaml` 的 `priority_rules` 自动分级 P0/P1/P2
3. 按 Feature ID 前缀匹配目标 crate

**Stage 1 阻塞** — 展示分级摘要 + 预计测试数量，等待人类确认。

### Step 2: Diff State — 增量判定

1. 读取 Local Cache `state/local-cache.json`（首次为空，全部视为新增）
2. 计算关联源文件 SHA256，与缓存比对判定动作：

| 条件 | 动作 |
|------|------|
| 新增 P0（Cache 无记录） | `generate` |
| source_hash 变化 | `generate` |
| 未变且 Registry test_status=pass | `skip` |

### Step 3: Validate + Generate — 反馈补偿 + 测试生成

对非 skip 的 feature：

1. 从 Spec 构建**预期行为模型**，从源码构建**实际能力模型**
2. 交叉验证标记置信度：`[SPEC]` / `[CONFLICT]` / `[INFERRED]` / `[REVIEW:LOW]`
3. `[CONFLICT]` 向用户三选一：A 以 Spec 为准 / B 以 Code 为准 / C 搁置
4. 生成三维度测试：
   - **Happy Path**（`_happy_*`）— 正常输入验证返回值
   - **Boundary Cases**（`_boundary_*`）— 边界值输入
   - **Error Handling**（`_error_*`）— 异常注入
5. 断言强度按标记分层：`[SPEC]` 强 / `[INFERRED]` 中 / `[REVIEW:LOW]` 弱
6. 任何维度 < 1 条则补充后才算生成完毕

测试写入对应 crate 的 `#[cfg(test)] mod tests`。

### Step 4: Report + Persist

1. 更新 Coverage Registry `testsuite/state/priority-test-gate.json`（test_status、dimensions、conflicts）
2. 更新 Local Cache `state/local-cache.json`（source_hash、last_run）
3. 输出汇总：P0 总数 / 生成数 / 跳过数 / 三维度计数 / CONFLICT 明细

## Output Contract

1. 分级摘要（P0/P1/P2 数量）
2. Action Plan（skip/generate）
3. Validation 报告（CONFLICT 明细）
4. 三维度计数（happy / boundary / error）
5. 双文件更新确认

本 Skill 只生成测试代码，不执行测试。执行由 CI 或人工完成。

## File Isolation

```
只读: testsuite/specs/*, docs/*
只写: crates/*/src/**/#[cfg(test)], testsuite/state/priority-test-gate.json, state/local-cache.json
不写: tests/acceptance/, tests/ui/, testsuite/specs/
```
