import { Text, View } from "react-native";

import type { Tone } from "@/theme/colors";
import { toneColors, toneFills } from "@/theme/colors";
import { radius, typography } from "@/theme/tokens";

export function Badge({
  label,
  tone = "default",
  compact,
}: {
  label: string;
  tone?: Tone;
  compact?: boolean;
}) {
  return (
    <View
      style={{
        backgroundColor: toneFills[tone],
        paddingHorizontal: compact ? 6 : 8,
        paddingVertical: compact ? 2 : 4,
        borderRadius: radius.sm,
        borderCurve: "continuous",
      }}
    >
      <Text
        style={{
          ...typography.caption2,
          color: toneColors[tone],
        }}
      >
        {label}
      </Text>
    </View>
  );
}
