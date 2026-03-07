# Bug Tracker — 测试驱动的 Bug 追踪器

铁律：`发现 BUG → 去重 → 记录或复用 → 跳过 → 继续。绝不因单个 BUG 中断整个测试流程。`

## When To Use

- CI 或人工执行测试后发现失败用例，需记录为 Bug
- 用户要求查看/管理 Bug 列表或生成报告
- 用户提供测试执行结果，需提取并登记 Bug

不适用：xfail 预期失败（由各测试 Skill 管理）、纯需求讨论（→ `feature-submission`）

## Workflow

### Step 1: Detect — 识别 Bug

满足以下任一条件时识别为 Bug：
- 断言失败且非 xfail 标记
- 返回错误码与 `testsuite/specs/boundaries/*.yaml` 中 assertions 定义不符
- `testsuite/specs/references/security-invariants.md` 中安全不变量被违反

### Step 2: Dedup — 去重（核心）

```bash
python3 .cursor/skills/bug-tracker/scripts/bug_ops.py match \
  --module "{module}" --code-location "{file}:{line_range}"
```

匹配规则（满足任一即视为同一 Bug）：
- `code_location` 完全相同
- `module` + `title` 相似度 > 80%
- `module` + `test_name` 相同

匹配到 → 不创建，状态回退 `pending`，追加 evidence；无匹配 → Step 3 创建。

### Step 3: Record — 记录新 Bug

```bash
python3 .cursor/skills/bug-tracker/scripts/bug_ops.py add \
  --module "{module}" --title "{title}" --severity "{critical|high|medium|low}" \
  --test-file "{test_file}" --test-name "{test_name}" \
  --steps "{steps}" --expected "{expected}" --actual "{actual}" \
  --log-snippet "{log}" --code-location "{file}:{lines}" --tags "{tag1,tag2}"
```

### Step 4: Skip & Continue

输出 `[BUG-XXXX] 已记录/已存在`，不中断测试，继续后续用例。

### Step 5: Report

```bash
python3 .cursor/skills/bug-tracker/scripts/bug_ops.py report --output docs/bugs/bug-registry.md
```

## Bug Status Lifecycle

```
pending → confirmed → fixing → resolved → verified → closed
                                    ↘ wontfix
测试中再次触发已关闭 Bug → 状态回退 pending（回归）
```

## Integration — 触发方式

| 来源 | 触发时机 |
|------|---------|
| CI 报告 | 用户贴入 CI 失败日志，Agent 解析后逐条 match → add |
| 人工执行 | 用户贴入 cargo test / pytest / playwright 输出，Agent 提取 FAIL |
| Code Review | 审阅代码时发现潜在缺陷，手动触发记录 |

## Output Contract

1. 去重结果 — 新增 / 已存在（回退 pending）
2. Bug 列表 — bug_id、title、module、severity
3. 跳过确认 — 已跳过故障点
4. 汇总统计 — 新增数 / 复用数 / pending 总数
5. 持久化确认 — bugs.json 已更新

## File Isolation

```
只写: .cursor/skills/bug-tracker/state/bugs.json, docs/bugs/bug-registry.md
不写: 任何测试文件、源码文件、其他 Skill state
```
