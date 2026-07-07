import type { ReactNode } from "react";
import { Text, View } from "react-native";

import { AppIcon } from "@/components/tab-icon";
import { colors } from "@/theme/colors";
import { radius, spacing, typography } from "@/theme/tokens";

export function EmptyState({
  symbol,
  title,
  message,
  action,
}: {
  symbol?: string;
  title: string;
  message: string;
  action?: ReactNode;
}) {
  return (
    <View
      style={{
        alignItems: "center",
        paddingVertical: spacing.xxl * 2,
        paddingHorizontal: spacing.xl,
        gap: spacing.md,
      }}
    >
      {symbol ? (
        <View
          style={{
            width: 72,
            height: 72,
            borderRadius: 36,
            borderCurve: "continuous",
            backgroundColor: colors.secondaryFill as string,
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          <AppIcon
            name={symbol}
            color={colors.tertiaryLabel as string}
            size={32}
          />
        </View>
      ) : null}
      <Text
        style={{
          ...typography.title3,
          color: colors.label as string,
          textAlign: "center",
        }}
      >
        {title}
      </Text>
      <Text
        style={{
          ...typography.subhead,
          color: colors.secondaryLabel as string,
          lineHeight: 22,
          textAlign: "center",
          maxWidth: 280,
        }}
      >
        {message}
      </Text>
      {action ? <View style={{ marginTop: spacing.sm, width: "100%" }}>{action}</View> : null}
    </View>
  );
}
