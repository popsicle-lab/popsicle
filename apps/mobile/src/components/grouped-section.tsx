import type { ReactNode } from "react";
import { Pressable, Text, View } from "react-native";
import { Ionicons } from "@expo/vector-icons";

import { colors } from "@/theme/colors";
import { radius, spacing, typography } from "@/theme/tokens";

export function GroupedSection({
  title,
  footer,
  children,
}: {
  title?: string;
  footer?: string;
  children: ReactNode;
}) {
  return (
    <View style={{ gap: spacing.sm }}>
      {title ? (
        <Text
          style={{
            ...typography.sectionHeader,
            color: colors.secondaryLabel as string,
            paddingHorizontal: spacing.lg + 4,
          }}
        >
          {title}
        </Text>
      ) : null}
      <GroupedCard>{children}</GroupedCard>
      {footer ? (
        <Text
          style={{
            ...typography.footnote,
            color: colors.secondaryLabel as string,
            lineHeight: 18,
            paddingHorizontal: spacing.lg + 4,
          }}
        >
          {footer}
        </Text>
      ) : null}
    </View>
  );
}

export function GroupedCard({ children }: { children: ReactNode }) {
  return (
    <View
      style={{
        backgroundColor: colors.secondaryGroupedBackground as string,
        borderRadius: radius.md,
        borderCurve: "continuous",
        overflow: "hidden",
        boxShadow: "0 1px 2px rgba(0, 0, 0, 0.04)",
      }}
    >
      {children}
    </View>
  );
}

export function GroupedSeparator({ inset = spacing.lg }: { inset?: number }) {
  return (
    <View
      style={{
        height: 0.5,
        backgroundColor: colors.separator as string,
        marginLeft: inset,
      }}
    />
  );
}

export function GroupedRow({
  title,
  subtitle,
  leading,
  trailing,
  onPress,
  showChevron = Boolean(onPress),
  accessory,
}: {
  title: string;
  subtitle?: string;
  leading?: ReactNode;
  trailing?: ReactNode;
  onPress?: () => void;
  showChevron?: boolean;
  accessory?: ReactNode;
}) {
  const content = (
    <View
      style={{
        flexDirection: "row",
        alignItems: "center",
        gap: spacing.md,
        paddingHorizontal: spacing.lg,
        paddingVertical: subtitle ? spacing.md : 13,
        minHeight: 44,
      }}
    >
      {leading}
      <View style={{ flex: 1, minWidth: 0, gap: 2 }}>
        <Text
          selectable
          style={{
            ...typography.body,
            fontWeight: "400",
            color: colors.label as string,
          }}
          numberOfLines={1}
        >
          {title}
        </Text>
        {subtitle ? (
          <Text
            selectable
            style={{
              ...typography.footnote,
              color: colors.secondaryLabel as string,
              lineHeight: 18,
            }}
            numberOfLines={2}
          >
            {subtitle}
          </Text>
        ) : null}
      </View>
      {accessory ?? trailing}
      {showChevron && onPress ? (
        <Ionicons
          name="chevron-forward"
          size={16}
          color={colors.quaternaryLabel as string}
        />
      ) : null}
    </View>
  );

  if (!onPress) return content;

  return (
    <Pressable
      onPress={onPress}
      style={({ pressed }) => ({
        backgroundColor: pressed
          ? (colors.fill as string)
          : (colors.secondaryGroupedBackground as string),
      })}
    >
      {content}
    </Pressable>
  );
}

export function GroupedInsetRow({
  label,
  children,
  last,
}: {
  label?: string;
  children: ReactNode;
  last?: boolean;
}) {
  return (
    <View>
      <View style={{ paddingHorizontal: spacing.lg, paddingVertical: spacing.sm, gap: spacing.xs }}>
        {label ? (
          <Text
            style={{
              ...typography.footnote,
              fontWeight: "500",
              color: colors.secondaryLabel as string,
            }}
          >
            {label}
          </Text>
        ) : null}
        {children}
      </View>
      {!last ? <GroupedSeparator inset={spacing.lg} /> : null}
    </View>
  );
}

export function MetricTile({
  label,
  value,
  tone = "default",
}: {
  label: string;
  value: string;
  tone?: "default" | "accent" | "success" | "warning" | "danger";
}) {
  const valueColor =
    tone === "accent"
      ? (colors.systemBlue as string)
      : tone === "success"
        ? (colors.systemGreen as string)
        : tone === "warning"
          ? (colors.systemOrange as string)
          : tone === "danger"
            ? (colors.systemRed as string)
            : (colors.label as string);

  return (
    <View
      style={{
        flex: 1,
        backgroundColor: colors.secondaryGroupedBackground as string,
        borderRadius: radius.md,
        borderCurve: "continuous",
        padding: spacing.lg,
        gap: spacing.xs,
        boxShadow: "0 1px 2px rgba(0, 0, 0, 0.04)",
      }}
    >
      <Text
        style={{
          ...typography.caption2,
          color: colors.secondaryLabel as string,
        }}
      >
        {label}
      </Text>
      <Text
        selectable
        style={{
          ...typography.title3,
          color: valueColor,
          fontVariant: ["tabular-nums"],
        }}
        numberOfLines={1}
      >
        {value}
      </Text>
    </View>
  );
}
