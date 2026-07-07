# Popsicle Agent — Expo 手机 App

Expo SDK **57** + [**@expo/ui**](https://docs.expo.dev/versions/latest/sdk/ui/) universal 组件（SwiftUI / Compose 原生渲染）。对接自托管 `agent-server`（PROJ-92 / T-AR-0002–0004）。

UI 遵循仓库内 `.agents/skills/expo-ui/`：每个屏幕根节点包在 `Host` 内，设置/派活用 `FieldGroup`，列表用 `List`/`ListItem`。

## 功能

| Tab | Task | 组件 |
|---|---|---|
| 进度 | T-AR-0003 | `List` + WebSocket |
| 派活 | T-AR-0002 | `FieldGroup` + `POST /v1/dispatch` |
| 设置 | T-AR-0001 | `FieldGroup` + Server/Runtime 状态 |
| Run 详情 | T-AR-0004 | `ListItem` + 批准 `Button` |

## 前置条件

1. `./deploy/agent-runtime/up.sh`
2. `AGENT_RUNTIME_SERVER_URL=… ./target/debug/popsicle daemon start --foreground`
3. 手机与开发机同一局域网

## 安装与运行

```bash
cd apps/mobile
npm install
npx expo start
```

- **SDK 57**：`@expo/ui` universal 层在 Expo Go 中可直接运行（见 expo-ui skill）
- **iOS 模拟器**：Server URL `http://127.0.0.1:8787`
- **Android 模拟器**：`http://10.0.2.2:8787`
- **真机**：Mac 局域网 IP，如 `http://192.168.x.x:8787`

```bash
npm run lint   # tsc --noEmit
```

## 架构

- 布局：`src/components/layout.tsx`（`AppHost` = `Host` + `Column`）
- 配置：`AsyncStorage`（`popsicle.agent-runtime.mobile.config`）
- Babel：`@expo/ui/babel-plugin` + `react-native-reanimated/plugin`
