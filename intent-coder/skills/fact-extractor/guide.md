# fact-extractor —— 编写指南

> 读者：负责产出 fact-extractor 五个 artifact 的 AI agent（也就是你）。开工写任何 artifact 前先读完本指南，以及
> **`../../tools/mermaid-diagram/`**（`dependency-graph.md` 写完后
> `popsicle tool run mermaid-diagram action=validate path=…`）。

## 任务

把一份遗留代码库变成**事实基**，让下游 IDD skill 可以引用。你是带笔记本和卷尺的考古学家，**不是**设计师，**也不是**评论家。

## 三个会毁掉事实基的反模式

下面三种失败模式会让下游 PRD/RFC writer 产生幻觉。**虔诚地**回避它们。

### 1. **发表观点**

❌ "auth 模块设计糟糕，关注点混杂。"
✅ "模块 `auth`（src/auth/，1,243 LoC）import 自 `db`、`crypto`、`http`。它导出 14 个公开函数；其中 7 个通过 `lazy_static` 修改全局状态（src/auth/state.rs:22）。"

第一句是装成事实的观点。第二句是事实。

### 2. **推断意图**

❌ "函数 `process_payment` 应该在扣款前校验金额。"
✅ "函数 `process_payment(amount: u64) -> Result<Receipt, Error>`（src/payment/process.rs:108）：对 `amount` 无前置检查；首个调用是第 115 行的 `gateway.charge(amount)`。其上方 TODO 注释写：`// TODO: validate amount > 0 before going to prod`。"

第一句凭空发明了一条需求。第二句记录代码所做+TODO 暗示的缺口，但不声称这个 TODO 是对的。

### 3. **数字近似**

❌ "大约 30% 的代码在 `core` crate 里。"
✅ "tokei 报告 `core` crate 4,127 LoC，总 13,508 LoC（30.5%）；见 appendix A.1。"

近似会剥夺下游 skill 量化范围所需的精度。

## 结构化事实基 `facts.yaml` 是唯一真相源（feedback #11）

**先写 `facts.yaml`，再把 5 份 markdown 当它的渲染写**。下游（debate/prd/rfc/intent-spec/golden）
用 `fact_id` 精确引用，不再用「§ store」这类散文锚点。

- 每条事实必带 `fact_id`（`F-<ABBREV>-<KIND>-NNN`）、`kind`（api/dep/risk/debt/behavior）、
  `source`（`legacy@<pinned-sha>:<路径>#L<n>` 可复验溯源）、`evidence`。缺 `source` 就删该条。
- **API 契约按签名结构化**（解析 `.proto`/trait/`pub fn`），同一份既喂 `contracts.intent`
  也喂 golden 骨架——不要只写文字表。
- **必须收割行为事实（`kind: behavior`）**：从 legacy 现有测试 / 可观测 I/O / 可推时序抽取，
  尽量标 `golden_candidate`。静态结构（依赖 / LoC / unwrap 计数）不够——缺行为事实，
  下游的 `@asis` intent 与 golden 对账都无从下手（见 intent-spec-writer @asis 方法论，#13）。
- **漂移检测**：换 pinned commit 重跑 → diff `facts.yaml` 的 fact_id/source，即得过期的
  intent/task/golden 清单。事实基因此从一次性快照变成**活契约**。

## 每个 artifact 的作用（决定你要写到什么深度）

| Artifact | 谁消费 | 含义 |
|---|---|---|
| `facts.yaml` | **所有下游（机器）** | 唯一真相源；fact_id + file:line 溯源 + behavior 事实 |
| `dependency-graph.md` | rfc-writer（设计新模块边界）| 必须**机器可读** —— 含邻接表，不只是图 |
| `api-contracts.md` | prd-writer（写「这个 product 今天到底做什么」）| 必须按 bounded context 分组，不是按文件 |
| `unsafe-risk-report.md` | safety-spec、invariant-spec | 每条目要有 file:line + 周围 1 行注释（如有）|
| `tech-debt-inventory.md` | adr-writer（记录「为什么我们有这笔债」）、bug-tracker | 每条要有估计年龄（用 git blame）|
| `fact-extraction-report.md` | 上面所有的入口点 | 交叉链接到 facts.yaml + 4 份详细 artifact；承载 executive summary |

## 引用代码

引用代码时，使用标注真实语言的代码围栏，并**在围栏上方写 file:line**：

````markdown
**src/auth/login.rs:42-50**
```rust
pub fn login(user: &str, pass: &str) -> Result<Token, Error> {
    let hash = HASHES.lock().unwrap().get(user).cloned();
    // ...
}
```
````

每段最多引用 10 行。超过的，链到文件而不是粘进来。

## 当你不知道某件事时

**逐字**写 **`(unknown — needs human input)`**。**不要**猜。下游 skill 把这串字符串当 flag，会去问人。比如：

- 某个 API 端点的预期 SLA
- 某个 `panic!` 在生产中是否可达
- 某个魔法数字的业务含义

## 当某个工具失败时

某些工具不会在每个环境都可用（如 `cargo metadata` 需要 Rust toolchain）。当一个工具失败：

1. 在报告中记录失败（如 `tool: cargo metadata —— 不可用，dependency graph 仅靠解析 Cargo.toml 得到`）。
2. 降级到精度更低的方法（如直接解析 Cargo.toml）。
3. 把对应章节标记为 `[reduced fidelity]`。

**永远不要**悄悄地用另一个工具替换——下游消费者需要知道他们在用什么质量的数据。

## 迭代

`fact-extractor` 设计为代码库重大变化时跑一次。它产出的 artifact 是**带日期的基线**，**不是**活文档（那是 `living-doc-author` 的活）。要不要包含某项内容时，问自己：「3 个月后有人读到这一条，还会觉得它作为基线参考有用吗？」是 → 写；否 → 删。

## 输出格式约定

每个 artifact 必须以 `## Extraction Checklist` 章节结尾。顶层 `fact-extraction-report.md` 的 checklist 是 workflow guard 会检查的那一份。其余 4 份详细 artifact 有各自的 checklist 用于自检。
