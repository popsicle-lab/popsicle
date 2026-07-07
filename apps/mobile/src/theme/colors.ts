import { Platform } from "react-native";
import { Color } from "expo-router";

export const colors = {
  label: Platform.select({
    ios: Color.ios.label,
    android: Color.android.dynamic.onSurface,
    default: "#1c1c1e",
  })!,
  secondaryLabel: Platform.select({
    ios: Color.ios.secondaryLabel,
    android: Color.android.dynamic.onSurfaceVariant,
    default: "#6b7280",
  })!,
  tertiaryLabel: Platform.select({
    ios: Color.ios.tertiaryLabel,
    android: Color.android.dynamic.outline,
    default: "#9ca3af",
  })!,
  quaternaryLabel: Platform.select({
    ios: Color.ios.quaternaryLabel,
    android: Color.android.dynamic.outline,
    default: "#c7c7cc",
  })!,
  separator: Platform.select({
    ios: Color.ios.separator,
    android: Color.android.dynamic.outlineVariant,
    default: "#e5e7eb",
  })!,
  opaqueSeparator: Platform.select({
    ios: Color.ios.opaqueSeparator,
    android: Color.android.dynamic.outlineVariant,
    default: "#c6c6c8",
  })!,
  systemBackground: Platform.select({
    ios: Color.ios.systemBackground,
    android: Color.android.dynamic.surface,
    default: "#ffffff",
  })!,
  secondaryBackground: Platform.select({
    ios: Color.ios.secondarySystemBackground,
    android: Color.android.dynamic.surfaceContainer,
    default: "#f9fafb",
  })!,
  tertiaryBackground: Platform.select({
    ios: Color.ios.tertiarySystemBackground,
    android: Color.android.dynamic.surfaceContainerHigh,
    default: "#ffffff",
  })!,
  groupedBackground: Platform.select({
    ios: Color.ios.systemGroupedBackground,
    android: Color.android.dynamic.surfaceContainerLow,
    default: "#f2f2f7",
  })!,
  secondaryGroupedBackground: Platform.select({
    ios: Color.ios.secondarySystemGroupedBackground,
    android: Color.android.dynamic.surfaceContainer,
    default: "#ffffff",
  })!,
  fill: Platform.select({
    ios: Color.ios.systemFill,
    android: Color.android.dynamic.surfaceVariant,
    default: "rgba(120, 120, 128, 0.2)",
  })!,
  secondaryFill: Platform.select({
    ios: Color.ios.secondarySystemFill,
    android: Color.android.dynamic.surfaceVariant,
    default: "rgba(120, 120, 128, 0.16)",
  })!,
  systemBlue: Platform.select({
    ios: Color.ios.systemBlue,
    android: Color.android.dynamic.primary,
    default: "#007aff",
  })!,
  systemGreen: Platform.select({
    ios: Color.ios.systemGreen,
    android: Color.android.dynamic.tertiary,
    default: "#34c759",
  })!,
  systemOrange: Platform.select({
    ios: Color.ios.systemOrange,
    android: Color.android.dynamic.secondary,
    default: "#ff9500",
  })!,
  systemRed: Platform.select({
    ios: Color.ios.systemRed,
    android: Color.android.dynamic.error,
    default: "#ff3b30",
  })!,
  systemIndigo: Platform.select({
    ios: Color.ios.systemIndigo,
    default: "#5856d6",
  })!,
  terminalBackground: Platform.select({
    ios: "#1c1c1e",
    android: "#121212",
    default: "#1c1c1e",
  })!,
};

export type Tone = "default" | "success" | "warning" | "danger" | "accent" | "muted";

export const toneColors: Record<Tone, string> = {
  default: colors.secondaryLabel as string,
  success: colors.systemGreen as string,
  warning: colors.systemOrange as string,
  danger: colors.systemRed as string,
  accent: colors.systemBlue as string,
  muted: colors.tertiaryLabel as string,
};

export const toneFills: Record<Tone, string> = {
  default: colors.secondaryFill as string,
  success: "rgba(52, 199, 89, 0.16)",
  warning: "rgba(255, 149, 0, 0.16)",
  danger: "rgba(255, 59, 48, 0.14)",
  accent: "rgba(0, 122, 255, 0.14)",
  muted: "rgba(120, 120, 128, 0.12)",
};
