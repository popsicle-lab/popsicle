import { ScrollView, Text, View } from "react-native";

import type { RunLogEntry } from "@/api/types";
import { colors } from "@/theme/colors";
import { radius, spacing, typography } from "@/theme/tokens";
import { displayLogMessage, formatLogTime, logLevelTone } from "@/utils/run-logs";

export function RunLogPanel({
  logs,
  loading,
}: {
  logs: RunLogEntry[];
  loading?: boolean;
}) {
  return (
    <View
      style={{
        backgroundColor: colors.terminalBackground as string,
        borderRadius: radius.md,
        borderCurve: "continuous",
        overflow: "hidden",
        boxShadow: "0 2px 8px rgba(0, 0, 0, 0.12)",
      }}
    >
      <View
        style={{
          flexDirection: "row",
          alignItems: "center",
          justifyContent: "space-between",
          paddingHorizontal: spacing.md,
          paddingVertical: spacing.sm,
          borderBottomWidth: 0.5,
          borderBottomColor: "rgba(255,255,255,0.08)",
        }}
      >
        <Text
          style={{
            ...typography.caption2,
            color: "rgba(255,255,255,0.55)",
          }}
        >
          AGENT 输出
        </Text>
        <Text
          style={{
            ...typography.caption2,
            color: "rgba(255,255,255,0.35)",
            fontVariant: ["tabular-nums"],
          }}
        >
          {logs.length} 行
        </Text>
      </View>

      {loading && logs.length === 0 ? (
        <Text
          style={{
            ...typography.footnote,
            color: "rgba(255,255,255,0.45)",
            padding: spacing.lg,
          }}
        >
          加载日志…
        </Text>
      ) : null}

      {!loading && logs.length === 0 ? (
        <Text
          style={{
            ...typography.footnote,
            color: "rgba(255,255,255,0.45)",
            padding: spacing.lg,
            lineHeight: 20,
          }}
        >
          暂无输出。点「恢复执行」后，Daemon 编排与 cursor-agent 日志会显示在这里。
        </Text>
      ) : (
        <ScrollView
          style={{ maxHeight: 300 }}
          contentContainerStyle={{ padding: spacing.md, gap: 4 }}
        >
          {logs.map((entry, index) => (
            <LogLine key={`${entry.ts}-${index}`} entry={entry} />
          ))}
        </ScrollView>
      )}
    </View>
  );
}

function LogLine({ entry }: { entry: RunLogEntry }) {
  const tone = logLevelTone(entry.level);
  const color =
    tone === "danger"
      ? "#ff6b6b"
      : tone === "accent"
        ? "#7ec8ff"
        : "rgba(255,255,255,0.72)";
  const isAgent =
    displayLogMessage(entry).startsWith("›") ||
    displayLogMessage(entry).startsWith("✗");

  return (
    <View style={{ flexDirection: "row", gap: spacing.sm, alignItems: "flex-start" }}>
      <Text
        style={{
          fontSize: 10,
          fontFamily: "Menlo",
          color: "rgba(255,255,255,0.35)",
          minWidth: 48,
          fontVariant: ["tabular-nums"],
        }}
      >
        {formatLogTime(entry.ts)}
      </Text>
      <Text
        selectable
        style={{
          flex: 1,
          fontSize: isAgent ? 11 : 12,
          fontFamily: isAgent ? "Menlo" : undefined,
          color,
          lineHeight: 17,
        }}
      >
        {displayLogMessage(entry)}
      </Text>
    </View>
  );
}
