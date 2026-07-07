import { Ionicons } from "@expo/vector-icons";
import { SymbolView, type SymbolViewProps } from "expo-symbols";

type SfName = SymbolViewProps["name"];

const ANDROID_FALLBACK: Record<string, keyof typeof Ionicons.glyphMap> = {
  "list.bullet.rectangle": "list",
  "list.bullet.rectangle.fill": "list",
  "paperplane.fill": "paper-plane",
  gear: "settings-outline",
  "tray.full": "file-tray-outline",
  "checkmark.circle.fill": "checkmark-circle",
  "exclamationmark.triangle.fill": "warning",
  "arrow.clockwise": "refresh",
};

function resolveSfName(name: string, focused?: boolean): SfName {
  if (!focused) return name as SfName;
  const override: Record<string, string> = {
    gear: "gearshape.fill",
  };
  if (override[name]) return override[name] as SfName;
  if (name.endsWith(".fill")) return name as SfName;
  return `${name}.fill` as SfName;
}

function AndroidIcon({
  name,
  color,
  size,
}: {
  name: string;
  color: string;
  size: number;
}) {
  const icon = ANDROID_FALLBACK[name] ?? ANDROID_FALLBACK[name.replace(/\.fill$/, "")] ?? "ellipse";
  return <Ionicons name={icon} size={size} color={color} />;
}

export function TabIcon({
  name,
  color,
  focused,
  size = 22,
}: {
  name: string;
  color: string;
  focused?: boolean;
  size?: number;
}) {
  if (process.env.EXPO_OS === "ios") {
    return (
      <SymbolView
        name={resolveSfName(name, focused)}
        tintColor={color}
        size={size}
        weight={focused ? "semibold" : "regular"}
        resizeMode="scaleAspectFit"
        fallback={<AndroidIcon name={name} color={color} size={size} />}
      />
    );
  }

  return <AndroidIcon name={name} color={color} size={size} />;
}

export function AppIcon({
  name,
  color,
  size = 40,
  weight = "regular",
}: {
  name: string;
  color: string;
  size?: number;
  weight?: SymbolViewProps["weight"];
}) {
  if (process.env.EXPO_OS === "ios") {
    return (
      <SymbolView
        name={name as SfName}
        tintColor={color}
        size={size}
        weight={weight}
        resizeMode="scaleAspectFit"
        fallback={<AndroidIcon name={name} color={color} size={size} />}
      />
    );
  }

  return <AndroidIcon name={name} color={color} size={size} />;
}
