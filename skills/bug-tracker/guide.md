# Bug Tracker — 测试驱动的 Bug 追踪器

铁律：`发现 BUG → 去重 → 记录或复用 → 跳过 → 继续。绝不因单个 BUG 中断整个测试流程。`

## When To Use

- CI 或人工执行测试后发现失败用例，需记录为 Bug
- 用户要求查看/管理 Bug 列表或生成报告
- 用户提供测试执行结果，需提取并登记 Bug

不适用：xfail 预期失败（由各测试 Skill 管理）、纯需求讨论

**重要区分**：
- **发现新 bug** → 使用 `popsicle bug create` 或 `popsicle bug record`（写入数据库，在 UI 中展示）
- **修复 bug 后记录经验** → 使用 `popsicle memory save --type bug`（写入记忆，供未来会话参考）

## Workflow

### Step 1: Detect — 识别 Bug

满足以下任一条件时识别为 Bug：
- 断言失败且非 xfail 标记
- 返回错误码与预期不符
- 安全不变量被违反

### Step 2: Dedup — 去重

使用 `popsicle bug list` 检查已有 bug：

```bash
popsicle bug list --status open --format json
```

匹配规则（满足任一即视为同一 Bug）：
- 相同的文件位置和错误信息
- 相同模块 + 相似标题
- 相同的关联测试用例

匹配到 → 不创建新 bug，继续下一个用例；无匹配 → Step 3 创建。

### Step 3: Record — 记录新 Bug

**从测试失败自动创建（推荐）**：

```bash
popsicle bug record --from-test \
  --error "assertion failed: expected 200, got 500" \
  --run <pipeline-run-id>
```

`bug record` 会自动去重：如果已有同一 TestCase 的 Open bug，不会重复创建。

**手动创建**：

```bash
popsicle bug create \
  --title "Login endpoint returns 500 on valid credentials" \
  --severity major \
  --description "When posting valid credentials to /api/login, server returns 500" \
  --steps "1. POST /api/login with valid user\n2. Observe 500 response" \
  --expected "200 OK with JWT token" \
  --actual "500 Internal Server Error" \
  --run <pipeline-run-id>
```

### Step 4: Skip & Continue

输出 `[BUG-XXXX] 已记录/已存在`，不中断测试，继续后续用例。

### Step 5: Link Fix Commit

修复 bug 后，关联修复 commit：

```bash
popsicle bug link <bug-key> --commit <sha>
popsicle bug update <bug-key> --status fixed
```

### Step 6: Report

```bash
popsicle bug list --format json
popsicle bug show <bug-key>
```

## Bug Status Lifecycle

```
open → confirmed → in_progress → fixed → verified → closed
                                           ↘ wont_fix
测试中再次触发已关闭 Bug → 重新创建（回归 bug）
```

## Integration — 触发方式

| 来源 | 触发时机 | 命令 |
|------|---------|------|
| CI 报告 | 用户贴入 CI 失败日志 | `popsicle bug record --from-test --error "..."` |
| 人工执行 | 用户贴入 cargo test / pytest 输出 | `popsicle bug create --title "..." --severity ...` |
| 文档提取 | bug-report 文档完成后 | `popsicle extract bugs --from-doc <doc-id>` |
| Code Review | 审阅代码时发现潜在缺陷 | `popsicle bug create --title "..." --source manual` |

## Output Contract

1. 去重结果 — 新增 / 已存在
2. Bug 列表 — `popsicle bug list` 展示 key、title、severity、status
3. 跳过确认 — 已跳过故障点，继续测试
4. 汇总统计 — 新增数 / 已存在数
5. 持久化确认 — bug 已写入数据库，可在 Desktop UI 的 Bugs 页面查看
