# UI Test — Spec 驱动 Playwright UI 测试

铁律：

```
先读 SPEC，再生 CODE。Page Object 统一管理，测试只写用户行为。
```

## When To Use

- 需要生成或更新 `tests/ui/e2e/` 下的 Playwright 测试
- 新 story spec 中有 `how: ui` 的验收条件需要覆盖
- 新增页面需要注册到 page-registry

不使用本技能的场景：

- API 接口测试 → 使用 `api-integration`
- E2E 多步骤链路（API 层）→ 使用 `story-e2e`
- 边界发现 → 使用 `boundary-tdd`

## File Isolation

```
只写: tests/ui/e2e/
不写: tests/acceptance/
不写: crates/
```

## Spec Sources

从 `testsuite/specs/manifest.yaml` 的 `pipelines.ui_test.spec_sources` 获取路径：

- `testsuite/specs/acceptance/stories/*.yaml` — 筛选 `how: ui` 的 scenario
- `testsuite/specs/ui/page-registry.yaml` — Page Object 注册表

## Workflow — 两阶段提交

### Stage 1: Spec Proposal（阻塞，等人类审阅）

1. **读 Spec Sources** — 加载 story specs 和 page-registry
2. **筛选 UI scenario** — 从 story specs 中筛选 `how: ui` 的条目
3. **匹配 Page Object** — 根据 page-registry 确定需要的 Page Object
4. **生成 Proposal** — 输出以下内容供人类审阅：
   - 将覆盖的 Story / AC ID
   - 将生成/更新的测试文件列表
   - 需要新建的 Page Object
   - 预计的用例数量
5. **等待确认** — 人类回复 "approved" / "加 XX 场景" / "reject"

### Stage 2: Generate（需 Stage 1 approved）

1. **生成/更新 Page Object** — 新页面按 page-registry 创建 Page Object 类
2. **生成测试代码** — 按 story 生成 `.spec.ts` 测试文件
   - 遵循 Page Object Model 模式
   - 使用 `data-testid` 作为选择器
   - 在文件头部标注 spec 来源
3. **Self-Consistency Check** — 逐条对照 spec 和生成代码

本 Skill 只生成测试代码，不执行测试。执行由 CI 或人工完成。

## Test Code Conventions

- 文件头部标注 spec 来源：

```typescript
/**
 * Generated from: testsuite/specs/acceptance/stories/{story}.yaml
 * Scenarios: {scenario-ids}
 */
```

- Page Object 放在 `tests/ui/e2e/pages/` 目录
- 使用 Playwright 的 `test.step` 组织多步骤场景
- 失败时自动截图（playwright.config.ts 已配置）

## Output Contract

1. **Stage 1 输出** — Spec Proposal（覆盖的 story + 预计生成量）
2. **Stage 2 输出** — 生成的文件列表 + 一致性报告
3. **隔离确认** — 声明未修改 tests/ui/e2e/ 以外的文件
