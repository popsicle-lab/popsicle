# 领域插件系统

> 核心语言不变，通过插件适配任意领域。

---

## 架构

```
┌──────────────────────────────────────────┐
│            intent-lang core               │
│                                           │
│  type / enum / intent / require / ensure  │
│  invariant / theorem / safety / axiom     │
│                                           │
│  ↑ 永远不变                                │
└─────────┬───────────────────┬────────────┘
          │ Plugin API         │
  ┌───────▼────────┐  ┌───────▼────────┐
  │ finance         │  │ smarthome      │
  └────────────────┘  └────────────────┘
```

---

## 插件的 4 层结构

每个插件包含 4 层，作用于不同引擎模块：

| 层 | 内容 | 作用于 |
|---|---|---|
| **类型层** | 领域数据结构 | Parser + 类型系统 |
| **安全层** | 全局不变量 | VCGen（自动合并到所有 intent） |
| **公理层** | 领域事实 | SMT（注入为前置假设） |
| **函数层** | 便捷辅助函数 | 用户代码 |

---

## 完整示例：智能家居插件

```intent
@plugin("smarthome")
@version("0.1.0")

// ── 类型层 ──
type Device { id: String, on: Bool, room: Room }
type Light extends Device { brightness: Int, color: Color }
type Thermostat extends Device { temperature: Int, mode: ThermoMode, target: Int }
type Sensor { type: SensorType, value: Int, room: Room }
type Room { name: String, devices: Seq<Device>, sensors: Seq<Sensor> }

enum ThermoMode { Heat, Cool, Auto, Off }
enum Color { Warm, Cool, Daylight, Custom }

// ── 安全层（自动附加到所有 intent）──
safety PhysicalConstraints {
  invariant forall t: Thermostat, t.target >= 5 && t.target <= 40
  invariant forall l: Light, l.brightness >= 0 && l.brightness <= 100
  invariant forall l: Light, !l.on ==> l.brightness == 0
}

safety EmergencyOverride {
  invariant smokeDetected ==> forall d: Device, !d.on'
}

// ── 公理层 ──
axiom temp_monotonic {
  forall t: Thermostat,
    t.mode == Heat && t.target > t.temperature ==>
      t.temperature' > t.temperature
}

axiom device_mutex {
  forall r: Room, h: Thermostat, c: Thermostat,
    h.room == r && c.room == r ==>
      !(h.mode == Heat && c.mode == Cool)
}

// ── 函数层 ──
function allLightsOff(rooms: Seq<Room>) -> Bool {
  forall r: Room, l: Light, l.room == r ==> !l.on
}
```

### 用户使用

```intent
import smarthome

intent MovieMode(living: Room) {
  ensure forall l: Light, l.room == living ==> l.brightness' == 20
}
// 验证时自动检查 PhysicalConstraints + EmergencyOverride
```

---

## 更多领域插件

### 金融

```intent
@plugin("finance")

type Currency { code: String, decimals: Int }
type Money { amount: Int, currency: Currency }
type Account { id: String, balance: Money, frozen: Bool }

safety DoubleEntry {
  invariant forall e: LedgerEntry,
    e.debit.balance' + e.amount.amount == e.debit.balance &&
    e.credit.balance' == e.credit.balance + e.amount.amount
}

safety NoOverdraft {
  invariant forall a: Account, a.balance.amount >= 0
}
```

### 医疗

```intent
@plugin("healthcare")

type Patient { weight: Int, age: Int, allergies: Set<String> }
type Medication { name: String, maxDailyDose: Int }
type Prescription { patient: Patient, medication: Medication, dose: Int, frequency: Int }

safety DrugSafety {
  invariant forall p: Prescription,
    p.dose * p.frequency <= p.medication.maxDailyDose
  invariant forall p: Prescription,
    !(p.medication.name in p.patient.allergies)
}
```

### 访问控制

```intent
@plugin("access-control")

type Principal { id: String, roles: Set<Role> }
enum Role { Admin, Manager, Developer, Viewer }

safety SeparationOfDuty {
  invariant forall p: Principal,
    !(Developer in p.roles && Admin in p.roles)
}
```

---

## 插件开发规范

### 目录结构

```
plugins/my-domain/
├── plugin.intent       # 主文件
├── plugin.toml         # 元数据
├── examples/
└── tests/
```

### plugin.toml

```toml
[plugin]
name = "smarthome"
version = "0.1.0"
description = "Smart home device control and safety rules"
```

### ⚠️ 公理安全

错误的公理会让验证不可靠：

```intent
// 危险！矛盾公理导致一切可证
axiom unsound {
  forall x: Int, x > 0 && x < 0
}
```

建议：
1. 公理必须经领域专家审核
2. `intent check --audit-axioms` 检查公理一致性
3. 插件发布需要签名 + review

---

## 引擎如何处理插件

| 阶段 | 处理 |
|------|------|
| 解析 | `import X` → 加载类型/函数到符号表 |
| 类型检查 | 插件类型与用户代码统一检查 |
| VCGen | 插件 safety 自动合并到所有 intent |
| SMT | 插件 axiom 作为 `(assert ...)` 注入 |
| 报告 | 违反 safety 时标注来源插件 |

---

## 大型项目组织

### 插件 vs 文件：何时用哪个

| 需求 | 方式 | 理由 |
|------|------|------|
| 行业通用的物理/法规约束 | **插件** | safety/axiom 自动注入，跨项目复用 |
| 项目内多人协作拆分文件 | **文件导入** | 不引入隐式约束，符号引入显式可控 |
| 跨领域共享类型定义 | **文件导入** | 类型共享不需要 safety 自动合并 |

### 推荐目录结构

```
my-platform/
├── intent.toml                      # 项目元数据 + 插件依赖
│
├── domains/                         # 按业务领域拆分
│   ├── payment/
│   │   ├── types.intent             # type Account, Money, Ledger
│   │   ├── safety.intent            # safety DoubleEntry, NoOverdraft
│   │   └── transfer.intent          # intent TransferSafe(...)
│   ├── inventory/
│   │   ├── types.intent             # type Product, Stock, Warehouse
│   │   └── stock.intent             # intent Restock(...), Deduct(...)
│   └── shipping/
│       ├── types.intent             # type Order, Delivery, Route
│       └── delivery.intent          # intent Ship(...), Cancel(...)
│
├── cross-domain/                    # 跨领域意图（多 import）
│   ├── checkout.intent              # import payment + inventory + shipping
│   └── refund.intent                # import payment + inventory
│
└── theorems/                        # 全局性质证明
    └── system-wide.intent           # 跨领域不变量
```

### intent.toml

```toml
[project]
name = "my-platform"
version = "0.1.0"

[plugins]
finance = "0.2.0"           # 金融行业插件（DoubleEntry 等）
access-control = "0.1.0"    # 访问控制插件

[check]
entry = ["domains/**/*.intent", "cross-domain/**/*.intent", "theorems/**/*.intent"]
```

### 示例：跨领域意图

```intent
// cross-domain/checkout.intent

import finance                                         // 插件：自动注入 DoubleEntry safety
import "./domains/payment/types.intent" as payment     // 文件：引入项目内的 Account 定义
import "./domains/inventory/types.intent" as inventory  // 文件：引入项目内的 Product, Stock
import "./domains/shipping/types.intent" as shipping    // 文件：引入项目内的 Order

intent Checkout(buyer: payment.Account, item: inventory.Product,
                stock: inventory.Stock, order: shipping.Order) {
  require buyer.balance >= item.price
  require stock.quantity >= order.quantity

  ensure buyer.balance' == buyer.balance - item.price * order.quantity
  ensure stock.quantity' == stock.quantity - order.quantity
  ensure order.status' == Confirmed

  invariant stock.quantity' >= 0
}
// DoubleEntry safety 由 finance 插件自动验证
```

### 名字冲突处理

大型项目中不同领域可能定义同名类型。intent-lang 通过限定名解决：

```intent
import "./domains/payment/types.intent" as payment    // 定义了 Account
import "./domains/user/types.intent" as user           // 也定义了 Account

// 编译器报错（无 as 别名时）：Account is ambiguous, use qualified name
// 修正：
intent Withdraw(wallet: payment.Account, profile: user.Account) {
  require profile.verified
  require wallet.balance >= amount
  ensure wallet.balance' == wallet.balance - amount
}
```

**限定名规则：**

- 插件导入：`import finance` → 限定名前缀为 `finance`
- 文件导入：默认前缀为文件名（不含 `.intent`），同名时用 `as` 指定别名
- 无冲突时限定名可省略，有冲突时必须使用
