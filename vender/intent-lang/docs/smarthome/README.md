# 智能家居场景

> intent-lang 在智能家居中的价值：**让你确信"灯该关的时候一定会关，不该关的时候一定不会关"，并且能解释为什么。**

---

## 现有平台的做法

### 架构对比

| | Alexa | 米家 | Alice | HomeKit |
|---|---|---|---|---|
| 用户写什么 | action list | if-then + actions | action list | 目标状态 (Scene) |
| "意图"含义 | NLU 槽位填充 | 场景名称 | 场景名称 | Scene 名称 |
| 执行方式 | 顺序执行 | 顺序执行 | 顺序执行 | 目标状态设置 |
| 安全验证 | ❌ | ❌ | ❌ | ❌ |
| 冲突检测 | ❌ | ⚠️ 重复触发警告 | ❌ | ❌ |

> HomeKit 的 Scene 最接近 intent-lang —— 它声明目标状态（灯:关, 温度:22°C），而不是动作列表。但没有 require/invariant/冲突检测/形式化验证。

### NLU 技术现状

所有平台都是 **传统 NLU 兜底 + 大模型增强**（混合架构）：

| 平台 | 技术 |
|------|------|
| Alexa | 自研 NLU + Alexa LLM |
| 米家/小爱 | 自研 NLU + MiLM 大模型 |
| Alice | Yandex NLU + YandexGPT |
| Siri | Apple NLU + Apple Intelligence |

原因：智能家居要求**快**（500ms 响应）、**确定**（"关灯"不能理解错）、**离线**（网断了也得能用）。

**intent-lang 不做 NLU**——它接收 NLU 的结构化输出，负责后续的验证→规划→执行→确认。

---

## 5 个核心痛点 & intent-lang 的解法

### ① 安全性 —— 物理后果不可逆

```intent
// 用户新建了 PartyMode
intent PartyMode(home: Home) {
  ensure forall l: Light, l.on' == true
  ensure home.frontDoor.locked' == false   // 方便朋友进
}
```

```bash
$ intent check
  ❌ PartyMode 违反 HomeSafety:
     如果执行后人离开，门没锁！
     建议: 添加 require home.occupied
```

传统平台：规则直接生效 → 某天出门忘关 → 门整晚没锁。

### ② 可解释性 —— 为什么灯没关？

```bash
$ intent why "为什么灯没关？"

  活跃意图:
    GoodNight → 要求关灯
    SecurityAlert → 要求开灯（安全警报）

  冲突解决:
    SecurityAlert 是 safety 规则，优先级更高

  触发链:
    22:45 动作传感器 → SecurityAlert 激活 → 覆盖 GoodNight

  结论: 灯亮是因为安全警报，不是 bug。
```

传统平台：翻 50 条自动化规则慢慢找。

### ③ 冲突检测 —— 部署前发现

```intent
intent EnergySaver(room: Room) {
  require !room.hasMotion
  ensure forall l: Light, l.on' == false     // 没人关灯
}

intent HallwayLight(hall: Room) {
  require hall.name == "hallway"
  ensure exists l: Light, l.on' == true       // 走廊常亮
}
```

```bash
$ intent check --conflicts
  ⚠️ EnergySaver vs HallwayLight: 走廊无人时冲突
     建议: 添加优先级或排除走廊
```

传统平台：两条规则冲突 → 运行时才撞。

### ④ 声明式简洁

```yaml
# Home Assistant: 15 行 YAML
automation:
  - alias: "离家模式"
    trigger: { platform: state, entity_id: person.curtis, to: "not_home" }
    action:
      - service: light.turn_off
        target: { area_id: living_room }
      - service: light.turn_off
        target: { area_id: bedroom }
      - service: climate.set_hvac_mode
        data: { hvac_mode: "off" }
      - service: lock.lock
        target: { entity_id: lock.front_door }
```

```intent
// intent-lang: 5 行
intent LeaveHome(home: Home) {
  require home.occupied
  ensure home.occupied' == false
  ensure forall l: Light, l.on' == false     // 加新灯不需要改这里
  ensure home.thermostat.mode' == Off
  ensure home.frontDoor.locked' == true
}
```

### ⑤ 可组合

```intent
intent MovieNight = ArriveHome && DimLights && CloseCurtains
// 加新设备？只添加 action，不修改 intent
```

---

## 从意图到执行：4 层架构

> 详细设计见 [docs/architecture/EXECUTION.md](../architecture/EXECUTION.md)

```
Layer 1: Intent     声明目标状态，SMT 验证安全性
    │ desired state
    ▼
Layer 2: Planner    从"目标"推导"动作序列"
    │ action plan
    ▼
Layer 3: Executor   将动作映射到具体设备协议 (MQTT/HTTP/Zigbee)
    │ device commands
    ▼
Layer 4: Verifier   读取真实状态，对比 ensure 条件
    │
    ✅ 意图达成 / ❌ 触发补偿
```

### Planner 示例

```
当前: { light1: on, door: open, door: unlocked }
目标: { light1: off, door: locked }

推理:
  Lock(door) 需要 door.open == false
  → 先 Close(door)

计划:
  并行: TurnOff(light1), Close(door)
  串行: Lock(door)   // 依赖 Close
```

### 完整时序

```
14:00:00  "晚安"
14:00:00  NLU → GoodNight
14:00:00  SMT 预检查 → ✅ 安全
14:00:01  Planner → [TurnOff ∥ TurnOff, Lock]
14:00:01  Executor → MQTT light1 OFF ✅, MQTT light2 OFF ✅, HTTP door LOCK ✅
14:00:03  Verifier → 所有 ensure 满足 ✅
14:00:03  "晚安，灯已关闭，门已锁好 🌙"
```

---

## 值得借鉴的现有设计

| 平台 | 借鉴点 | intent-lang 如何吸收 |
|------|--------|---------------------|
| **米家 MIoT** | 设备建模 (siid/piid/aiid) | 插件的 type/action |
| **Alexa** | Directive 标准化 | action 库 |
| **HomeKit** | Scene = 目标状态 | ensure 是增强版 Scene |
| **Home Assistant** | 本地执行 + 丰富集成 | Executor 层可作为 HA integration |

---

## intent-lang 的定位

```
         NLU 层（小爱/Alexa/Siri）
               │
               ▼ 结构化意图名称
      ┌────────────────────┐
      │    intent-lang      │  ← 验证 + 规划 + 确认
      └────────┬───────────┘
               ▼ 标准化执行计划
         设备协议层 (MQTT/Zigbee/HTTP)
```

不替代现有平台，而是**嵌入其中作为"安全验证 + 智能规划"中间层**。
