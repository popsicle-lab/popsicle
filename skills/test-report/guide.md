# Test Report — 测试报告解析与闭环

铁律：`每个 FAIL 必须登记 Bug，每个 XPASS 必须标记复查。`

## When To Use

- CI 执行完成后需要分析测试结果
- 用户贴入 cargo test / pytest / playwright 终端输出
- 用户提供 `reports/` 目录下的报告文件
- 需要更新验收覆盖追踪或 xfail 注册表

不适用：生成测试代码（→ 各测试生成 Skill）、手动记录单个 Bug（→ `bug-tracker`）

## Spec Sources

从 `testsuite/specs/manifest.yaml` 的 `pipelines.report.spec_sources` 获取：

- `xfail_registry` → `docs/test/xfail-registry.md`
- `acceptance_map` → `docs/test/acceptance-criteria-map.md`

## Supported Report Formats

| 框架 | 结构化格式 | 文本格式 |
|------|-----------|---------|
| pytest | `testsuite/reports/acceptance.xml`（JUnit XML） | 终端输出 |
| cargo test | — | `testsuite/reports/unit.log` 或终端输出 |
| playwright | `testsuite/reports/playwright.json` | 终端输出 |

优先解析结构化格式，退化为终端文本解析。

## Workflow

### Step 1: Collect — 收集报告

1. 检查 `testsuite/reports/` 目录，识别可用报告文件
2. 若用户贴入终端输出，自动识别框架类型（pytest/cargo/playwright）
3. 确定报告时间戳和部署 profile

### Step 2: Parse — 解析报告

调用解析脚本将报告标准化：

```bash
python3 .cursor/skills/test-report/scripts/parse_report.py \
  --junit testsuite/reports/api-integration.xml \
  --cargo-log testsuite/reports/unit.log \
  --playwright-json testsuite/reports/playwright.json
```

输出统一 JSON，包含 summary（total/pass/fail/skip/xfail/xpass）和 failures 列表。

对终端文本输出，Agent 直接解析提取 FAIL 用例，无需脚本。

### Step 3: Bug Registration — 失败用例登记

对每个 FAIL 用例，调用 bug-tracker：

1. `bug_ops.py match` — 去重检查
2. 无匹配 → `bug_ops.py add` — 登记新 Bug
3. 有匹配 → 状态回退 pending，追加 evidence

严重等级推断：`p0` 标记 → critical，`p1` → high，`p2` → medium，无标记 → high。

### Step 4: XPASS Detection — XPASS 检测

检测 XPASS 结果（预期失败但实际通过），建议：
- 更新 `docs/test/xfail-registry.md` 中对应条目状态
- 移除测试代码中的 `@pytest.mark.xfail` 或 `#[should_panic]` 标记

### Step 5: Summary Report — 汇总报告

1. 输出测试金字塔汇总（各框架 pass/fail/skip 统计）
2. 更新 `docs/test/acceptance-criteria-map.md` 覆盖率
3. 列出新增/复用的 Bug 列表
4. 列出 XPASS 待复查项

## Output Contract

1. 解析摘要 — 各框架 total/pass/fail/skip/xfail/xpass
2. Bug 登记结果 — 新增数/复用数/bug_id 列表
3. XPASS 清单 — 待移除 xfail 的条目
4. 覆盖率更新 — acceptance-criteria-map 变更摘要

## File Isolation

```
只读: testsuite/reports/, testsuite/specs/manifest.yaml
只写: docs/test/acceptance-criteria-map.md, docs/test/xfail-registry.md
调用: .cursor/skills/bug-tracker/scripts/bug_ops.py
不写: tests/, crates/, specs/boundaries/
```
