import type { ReactNode } from "react";
import { Platform } from "react-native";
import {
  ActivityIndicator,
  Pressable,
  RefreshControl,
  ScrollView,
  Text,
  TextInput,
  View,
} from "react-native";
import Animated, { FadeIn, FadeOut } from "react-native-reanimated";
import { useSafeAreaInsets } from "react-native-safe-area-context";

import { colors, toneColors, type Tone } from "@/theme/colors";
import { radius, spacing, typography } from "@/theme/tokens";

const TAB_BAR_CLEARANCE = Platform.OS === "ios" ? 49 : 0;

export function AppHost({
  children,
  scroll = false,
  refreshing = false,
  onRefresh,
}: {
  children: ReactNode;
  scroll?: boolean;
  refreshing?: boolean;
  onRefresh?: () => void;
}) {
  const insets = useSafeAreaInsets();
  const bottomPad = spacing.xxl + spacing.sm + insets.bottom + TAB_BAR_CLEARANCE;

  const contentStyle = {
    paddingHorizontal: spacing.lg,
    paddingTop: spacing.sm,
    paddingBottom: bottomPad,
    gap: spacing.lg,
  };

  if (scroll) {
    return (
      <View style={{ flex: 1, backgroundColor: colors.groupedBackground as string }}>
        <ScrollView
          contentInsetAdjustmentBehavior="automatic"
          refreshControl={
            onRefresh ? (
              <RefreshControl
                refreshing={refreshing}
                onRefresh={onRefresh}
                tintColor={colors.systemBlue as string}
              />
            ) : undefined
          }
          contentContainerStyle={contentStyle}
          keyboardShouldPersistTaps="handled"
          showsVerticalScrollIndicator={false}
        >
          {children}
        </ScrollView>
      </View>
    );
  }

  return (
    <View
      style={{
        flex: 1,
        backgroundColor: colors.groupedBackground as string,
        paddingHorizontal: spacing.lg,
        paddingTop: spacing.sm,
        paddingBottom: bottomPad,
        gap: spacing.lg,
      }}
    >
      {children}
    </View>
  );
}

/** @deprecated Use GroupedSection from grouped-section.tsx */
export function SectionCard({ children }: { children: ReactNode }) {
  return (
    <View
      style={{
        backgroundColor: colors.secondaryGroupedBackground as string,
        borderRadius: radius.md,
        borderCurve: "continuous",
        padding: spacing.lg,
        gap: spacing.md,
        boxShadow: "0 1px 2px rgba(0, 0, 0, 0.04)",
      }}
    >
      {children}
    </View>
  );
}

export function SectionLabel({ children }: { children: string }) {
  return (
    <Text
      style={{
        ...typography.sectionHeader,
        color: colors.secondaryLabel as string,
      }}
    >
      {children}
    </Text>
  );
}

export function Hint({ children }: { children: string }) {
  return (
    <Text
      style={{
        ...typography.footnote,
        color: colors.secondaryLabel as string,
        lineHeight: 18,
        paddingHorizontal: spacing.xs,
      }}
    >
      {children}
    </Text>
  );
}

export function MonoText({
  children,
  numberOfLines,
}: {
  children: string;
  numberOfLines?: number;
}) {
  return (
    <Text
      selectable
      numberOfLines={numberOfLines}
      style={{
        ...typography.caption1,
        fontFamily: "Menlo",
        color: colors.tertiaryLabel as string,
        lineHeight: 18,
      }}
    >
      {children}
    </Text>
  );
}

export function StatusLabel({
  label,
  tone = "default",
}: {
  label: string;
  tone?: Tone;
}) {
  return (
    <Text
      style={{
        ...typography.footnote,
        fontWeight: "600",
        color: toneColors[tone],
      }}
    >
      {label}
    </Text>
  );
}

export function PrimaryButton({
  label,
  onPress,
  disabled,
}: {
  label: string;
  onPress?: () => void;
  disabled?: boolean;
}) {
  return (
    <Pressable
      onPress={onPress}
      disabled={disabled}
      style={({ pressed }) => ({
        borderRadius: radius.md,
        borderCurve: "continuous",
        paddingVertical: 14,
        paddingHorizontal: spacing.lg,
        alignItems: "center",
        backgroundColor: colors.systemBlue as string,
        opacity: disabled ? 0.45 : pressed ? 0.88 : 1,
      })}
    >
      <Text style={{ ...typography.headline, color: "#fff" }}>{label}</Text>
    </Pressable>
  );
}

export function SecondaryButton({
  label,
  onPress,
  disabled,
}: {
  label: string;
  onPress?: () => void;
  disabled?: boolean;
}) {
  return (
    <Pressable
      onPress={onPress}
      disabled={disabled}
      style={({ pressed }) => ({
        borderRadius: radius.md,
        borderCurve: "continuous",
        paddingVertical: 14,
        paddingHorizontal: spacing.lg,
        alignItems: "center",
        backgroundColor: pressed
          ? (colors.fill as string)
          : (colors.secondaryGroupedBackground as string),
        borderWidth: 1,
        borderColor: colors.separator as string,
        opacity: disabled ? 0.45 : 1,
      })}
    >
      <Text
        style={{
          ...typography.headline,
          color: colors.systemBlue as string,
        }}
      >
        {label}
      </Text>
    </Pressable>
  );
}

export function TextButton({
  label,
  onPress,
  disabled,
}: {
  label: string;
  onPress?: () => void;
  disabled?: boolean;
}) {
  return (
    <Pressable
      onPress={onPress}
      disabled={disabled}
      style={{ paddingVertical: spacing.sm, alignItems: "center" }}
    >
      <Text
        style={{
          ...typography.body,
          color: colors.systemBlue as string,
          opacity: disabled ? 0.45 : 1,
        }}
      >
        {label}
      </Text>
    </Pressable>
  );
}

export function Loading({ label = "加载中…" }: { label?: string }) {
  return (
    <View
      style={{
        flexDirection: "row",
        alignItems: "center",
        justifyContent: "center",
        gap: spacing.sm,
        paddingVertical: spacing.xl,
      }}
    >
      <ActivityIndicator color={colors.systemBlue as string} />
      <Text style={{ ...typography.subhead, color: colors.secondaryLabel as string }}>
        {label}
      </Text>
    </View>
  );
}

function BannerShell({
  children,
  backgroundColor,
  textColor,
}: {
  children: string;
  backgroundColor: string;
  textColor: string;
}) {
  return (
    <View
      style={{
        backgroundColor,
        padding: spacing.md,
        borderRadius: radius.md,
        borderCurve: "continuous",
        flexDirection: "row",
        alignItems: "flex-start",
        gap: spacing.sm,
      }}
    >
      <Text
        selectable
        style={{
          ...typography.subhead,
          color: textColor,
          lineHeight: 20,
          flex: 1,
        }}
      >
        {children}
      </Text>
    </View>
  );
}

export function ErrorText({ children }: { children: string }) {
  return (
    <Animated.View entering={FadeIn.duration(220)} exiting={FadeOut.duration(160)}>
      <BannerShell backgroundColor="rgba(255, 59, 48, 0.1)" textColor={colors.systemRed as string}>
        {children}
      </BannerShell>
    </Animated.View>
  );
}

export function SuccessBanner({ children }: { children: string }) {
  return (
    <Animated.View entering={FadeIn.duration(220)} exiting={FadeOut.duration(160)}>
      <BannerShell
        backgroundColor="rgba(52, 199, 89, 0.12)"
        textColor={colors.systemGreen as string}
      >
        {children}
      </BannerShell>
    </Animated.View>
  );
}

export function WarningBanner({ children }: { children: string }) {
  return (
    <BannerShell
      backgroundColor="rgba(255, 149, 0, 0.12)"
      textColor={colors.systemOrange as string}
    >
      {children}
    </BannerShell>
  );
}

export function FormInput({
  value,
  defaultValue,
  onChangeText,
  placeholder,
  autoCapitalize,
  autoCorrect,
}: {
  value?: string;
  defaultValue?: string;
  onChangeText?: (text: string) => void;
  placeholder?: string;
  autoCapitalize?: "none" | "sentences" | "words" | "characters";
  autoCorrect?: boolean;
}) {
  return (
    <TextInput
      value={value}
      defaultValue={defaultValue}
      onChangeText={onChangeText}
      placeholder={placeholder}
      placeholderTextColor={colors.tertiaryLabel as string}
      autoCapitalize={autoCapitalize}
      autoCorrect={autoCorrect}
      style={{
        ...typography.body,
        color: colors.label as string,
        paddingVertical: spacing.sm,
        minHeight: 36,
      }}
    />
  );
}

export function Chip({
  label,
  selected,
  onPress,
}: {
  label: string;
  selected?: boolean;
  onPress?: () => void;
}) {
  return (
    <Pressable
      onPress={onPress}
      style={({ pressed }) => ({
        paddingHorizontal: spacing.md,
        paddingVertical: spacing.sm,
        borderRadius: radius.pill,
        borderCurve: "continuous",
        backgroundColor: selected
          ? (colors.systemBlue as string)
          : pressed
            ? (colors.fill as string)
            : (colors.secondaryFill as string),
      })}
    >
      <Text
        style={{
          ...typography.footnote,
          fontWeight: "600",
          color: selected ? "#fff" : (colors.label as string),
        }}
      >
        {label}
      </Text>
    </Pressable>
  );
}

export { toneColors };
