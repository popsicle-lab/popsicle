import { useState } from "react";
import { Pressable, ScrollView, Text, View } from "react-native";

import type { RunLogEntry } from "@/api/types";
import { AgentToolTimeline } from "@/components/agent-tool-timeline";
import { colors } from "@/theme/colors";
import { radius, spacing, typography } from "@/theme/tokens";
import { displayLogMessage, formatLogTime, logLevelTone } from "@/utils/run-logs";

type LogTab = "tools" | "raw";

export function RunLogPanel({
  logs,
  loading,
}: {
  logs: RunLogEntry[];
  loading?: boolean;
}) {
  const [tab, setTab] = useState<LogTab>("tools");

  return (
    <View
      style={{
        backgroundColor:
          tab === "raw"
            ? (colors.terminalBackground as string)
            : (colors.secondaryGroupedBackground as string),
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
          borderBottomColor:
            tab === "raw" ? "rgba(255,255,255,0.08)" : (colors.separator as string),
        }}
      >
        <Text
          style={{
            ...typography.caption2,
            color:
              tab === "raw"
                ? "rgba(255,255,255,0.55)"
                : (colors.secondaryLabel as string),
          }}
        >
          AGENT 输出
        </Text>
        <View style={{ flexDirection: "row", gap: spacing.xs }}>
          <TabChip
            label="工具"
            active={tab === "tools"}
            onPress={() => setTab("tools")}
            dark={false}
          />
          <TabChip
            label="原始"
            active={tab === "raw"}
            onPress={() => setTab("raw")}
            dark={tab === "raw"}
          />
        </View>
      </View>

      {tab === "tools" ? (
        <View style={{ padding: spacing.lg }}>
          <AgentToolTimeline logs={logs} loading={loading} />
        </View>
      ) : (
        <RawLogView logs={logs} loading={loading} />
      )}
    </View>
  );
}

function TabChip({
  label,
  active,
  onPress,
  dark,
}: {
  label: string;
  active: boolean;
  onPress: () => void;
  dark?: boolean;
}) {
  return (
    <Pressable
      onPress={onPress}
      style={{
        paddingHorizontal: spacing.sm,
        paddingVertical: 4,
        borderRadius: radius.pill,
        backgroundColor: active
          ? dark
            ? "rgba(255,255,255,0.14)"
            : (colors.systemBlue as string)
          : dark
            ? "rgba(255,255,255,0.06)"
            : (colors.secondaryFill as string),
      }}
    >
      <Text
        style={{
          ...typography.caption2,
          color: active
            ? dark
              ? "#fff"
              : "#fff"
            : dark
              ? "rgba(255,255,255,0.55)"
              : (colors.secondaryLabel as string),
        }}
      >
        {label}
      </Text>
    </Pressable>
  );
}

function RawLogView({
  logs,
  loading,
}: {
  logs: RunLogEntry[];
  loading?: boolean;
}) {
  return (
    <>
      <View
        style={{
          flexDirection: "row",
          justifyContent: "flex-end",
          paddingHorizontal: spacing.md,
          paddingTop: spacing.xs,
        }}
      >
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
          style={{ maxHeight: 320 }}
          contentContainerStyle={{ padding: spacing.md, gap: 4 }}
        >
          {logs.map((entry, index) => (
            <LogLine key={`${entry.ts}-${index}`} entry={entry} />
          ))}
        </ScrollView>
      )}
    </>
  );
}

function LogLine({ entry }: { entry: RunLogEntry }) {
  const message = displayLogMessage(entry);
  const tone = logLevelTone(entry.level);
  const color =
    tone === "danger"
      ? "#ff6b6b"
      : tone === "accent"
        ? "#7ec8ff"
        : message.startsWith("agent:")
          ? "#9cdcfe"
          : "rgba(255,255,255,0.72)";
  const isMono =
    message.startsWith("›") ||
    message.startsWith("✗") ||
    message.startsWith("agent:");

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
          fontSize: isMono ? 11 : 12,
          fontFamily: isMono ? "Menlo" : undefined,
          color,
          lineHeight: 17,
        }}
      >
        {message}
      </Text>
    </View>
  );
}
