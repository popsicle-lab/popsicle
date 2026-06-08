# 4 层执行架构

> 从"意图验证通过"到"物理世界执行"的完整桥接。

---

## 问题

`ensure light.on' == false` 是一个逻辑命题。它不知道灯的 IP 地址、用 MQTT 还是 Zigbee、离线了怎么办。

---

## 架构总览

```
Layer 1: Intent       声明目标状态 + SMT 验证安全性
    │ desired state
    ▼
Layer 2: Planner      从"目标"推导"动作序列"+ SMT 验证计划正确性
    │ action plan
    ▼
Layer 3: Executor     将动作映射到设备协议 + 执行 + 失败处理
    │ device commands
    ▼
Layer 4: Verifier     读取真实状态 + 对比 ensure 条件
    │
    ✅ 达成 / ❌ 补偿
```

---

## Layer 2: Planner

### 设备能力声明

```intent
action TurnOff(light: Light) {
  require light.on == true
  effect light.on' == false
  timeout 5s
  on failure retry(3) then alert("灯关闭失败")
}

action Lock(door: Door) {
  require door.open == false       // 门必须先关上
  effect door.locked' == true
  timeout 10s
  on failure alert("门锁失败！")
}

action Close(door: Door) {
  require door.open == true
  effect door.open' == false
}
```

### 规划过程

```
输入:
  当前: { light1.on=true, door.open=true, door.locked=false }
  目标: { light1.on=false, door.locked=true }    ← 从 ensure 提取

推理:
  Lock(door) 需要 door.open==false → 先 Close(door)

输出:
  并行组 1: TurnOff(light1), Close(door)
  并行组 2: Lock(door)                    // 依赖组 1
```

计划本身也可被 SMT 验证——确认 action 的 effect 组合满足 intent 的 ensure。

---

## Layer 3: Executor

### 设备绑定

```toml
# devices.toml
[[devices]]
id = "light1"
type = "Light"
protocol = "mqtt"
topic = "zigbee2mqtt/0x00158d0001a2b3c4/set"
commands = { turnOff = '{"state":"OFF"}' }
state_topic = "zigbee2mqtt/0x00158d0001a2b3c4"

[[devices]]
id = "front_door"
type = "Door"
protocol = "http"
endpoint = "http://192.168.1.50/api"
commands = { lock = "POST /lock" }
```

### 执行策略

- **并行**：无依赖的动作并行发送
- **重试**：每个 action 可配置 timeout + retry
- **降级**：retry → 补偿 → 告警
- **部分成功**：记录已完成动作，支持回滚

---

## Layer 4: Verifier

### 从 intent 自动生成运行时验证

```bash
$ intent export GoodNight --format rust-assert
```

```rust
fn verify_good_night(home: &HomeState) -> VerifyResult {
    let mut failures = vec![];
    for light in &home.lights {
        if light.on { failures.push(format!("{} still on", light.id)); }
    }
    if !home.front_door.locked {
        failures.push("door not locked".into());
    }
    match failures.is_empty() {
        true => VerifyResult::Satisfied,
        false => VerifyResult::Violated(failures),
    }
}
```

### 失败处理

```
验证失败 → 可重试？→ 重新执行 → 再验证
         → 部分满足？→ 报告哪些 ensure 未满足
         → 完全失败？→ 补偿 → 恢复安全状态 → 告警
```

---

## 新增语言构造

| 构造 | 用途 | 层 |
|------|------|---|
| `action` | 声明设备能力（require + effect） | Planner |
| `effect` | action 的状态变更 | Planner |
| `bind` | 抽象类型 → 具体设备 | Executor |
| `on failure` | 失败处理策略 | Executor |

---

## 跨领域通用性

| 层 | 智能家居 | 软件开发 | 金融 |
|---|---------|---------|------|
| Intent | ensure light.off | ensure order.paid | ensure balance >= 0 |
| Planner | 推导设备动作 | 推导 API 调用链 | 推导交易步骤 |
| Executor | MQTT / HTTP | REST / gRPC | SWIFT / FIX |
| Verifier | 读传感器 | 读数据库 | 读对账系统 |
