import { useEffect, useMemo, useState } from "react";
import { Pressable, Text, View } from "react-native";
import { Ionicons } from "@expo/vector-icons";

import type { RunLogEntry } from "@/api/types";
import { TabIcon } from "@/components/tab-icon";
import { Badge } from "@/components/badge";
import { colors, type Tone } from "@/theme/colors";
import { radius, spacing, typography } from "@/theme/tokens";
import {
  buildAgentTimeline,
  toolKindIcon,
  toolKindLabel,
  toolTitle,
  type ToolTimelineItem,
} from "@/utils/agent-timeline";
import { formatLogTime } from "@/utils/run-logs";

export function AgentToolTimeline({
  logs,
  loading,
}: {
  logs: RunLogEntry[];
  loading?: boolean;
}) {
  const timeline = useMemo(() => buildAgentTimeline(logs), [logs]);
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});

  useEffect(() => {
    const running = timeline.tools.find((t) => t.status === "running");
    if (!running) return;
    setExpanded((prev) => {
      if (prev[running.id]) return prev;
      return { ...prev, [running.id]: true };
    });
  }, [timeline.tools]);

  if (loading && logs.length === 0) {
    return (
      <Text style={{ ...typography.footnote, color: colors.secondaryLabel as string }}>
        加载工具时间线…
      </Text>
    );
  }

  if (!timeline.hasStructuredAgentOutput || timeline.tools.length === 0) {
    return (
      <Text
        style={{
          ...typography.footnote,
          color: colors.secondaryLabel as string,
          lineHeight: 20,
        }}
      >
        暂无工具调用。Daemon 使用 stream-json 派活后，读取/写入/命令会显示在这里。
      </Text>
    );
  }

  const runningCount = timeline.tools.filter((t) => t.status === "running").length;
  const doneCount = timeline.tools.filter((t) => t.status === "done").length;

  return (
    <View style={{ gap: spacing.md }}>
      <View style={{ flexDirection: "row", flexWrap: "wrap", gap: spacing.sm }}>
        {timeline.session.model ? (
          <Badge label={timeline.session.model} tone="muted" compact />
        ) : null}
        <Badge label={`${doneCount} 完成`} tone="success" compact />
        {runningCount > 0 ? (
          <Badge label={`${runningCount} 进行中`} tone="accent" compact />
        ) : null}
        {timeline.session.durationMs != null ? (
          <Badge
            label={`${(timeline.session.durationMs / 1000).toFixed(1)}s`}
            tone="default"
            compact
          />
        ) : null}
      </View>

      <View>
        {timeline.tools.map((tool, index) => (
          <ToolTimelineRow
            key={tool.id}
            tool={tool}
            isLast={index === timeline.tools.length - 1}
            expanded={expanded[tool.id] ?? false}
            onToggle={() =>
              setExpanded((prev) => ({ ...prev, [tool.id]: !prev[tool.id] }))
            }
          />
        ))}
      </View>

      {timeline.assistantLines.length > 0 ? (
        <AssistantSnippet lines={timeline.assistantLines} />
      ) : null}
    </View>
  );
}

function ToolTimelineRow({
  tool,
  isLast,
  expanded,
  onToggle,
}: {
  tool: ToolTimelineItem;
  isLast: boolean;
  expanded: boolean;
  onToggle: () => void;
}) {
  const tone: Tone = tool.status === "running" ? "accent" : "success";
  const dotColor =
    tool.status === "running"
      ? (colors.systemBlue as string)
      : (colors.systemGreen as string);

  return (
    <View style={{ flexDirection: "row", gap: spacing.md }}>
      <View style={{ alignItems: "center", width: 28 }}>
        <View
          style={{
            width: tool.status === "running" ? 12 : 10,
            height: tool.status === "running" ? 12 : 10,
            borderRadius: 6,
            backgroundColor: dotColor,
            borderCurve: "continuous",
            borderWidth: tool.status === "running" ? 2 : 0,
            borderColor:
              tool.status === "running"
                ? `${colors.systemBlue as string}33`
                : "transparent",
          }}
        />
        {!isLast ? (
          <View
            style={{
              width: 2,
              flex: 1,
              minHeight: expanded ? 56 : 36,
              backgroundColor: colors.separator as string,
              marginTop: 4,
              borderRadius: 1,
            }}
          />
        ) : null}
      </View>

      <View style={{ flex: 1, paddingBottom: isLast ? 0 : spacing.md, gap: spacing.xs }}>
        <Pressable
          onPress={onToggle}
          style={({ pressed }) => ({
            backgroundColor: pressed
              ? (colors.fill as string)
              : (colors.tertiaryBackground as string),
            borderRadius: radius.sm,
            borderCurve: "continuous",
            paddingHorizontal: spacing.md,
            paddingVertical: spacing.sm,
            gap: spacing.xs,
          })}
        >
          <View
            style={{
              flexDirection: "row",
              alignItems: "center",
              gap: spacing.sm,
            }}
          >
            <TabIcon
              name={toolKindIcon(tool.kind)}
              color={
                tool.status === "running"
                  ? (colors.systemBlue as string)
                  : (colors.systemGreen as string)
              }
              size={18}
            />
            <View style={{ flex: 1, minWidth: 0, gap: 2 }}>
              <Text
                style={{
                  ...typography.subhead,
                  fontWeight: "600",
                  color: colors.label as string,
                }}
                numberOfLines={1}
              >
                {toolKindLabel(tool.kind)}
              </Text>
              <Text
                selectable
                style={{
                  ...typography.footnote,
                  color: colors.secondaryLabel as string,
                  fontFamily: "Menlo",
                }}
                numberOfLines={expanded ? undefined : 1}
              >
                {toolTitle(tool)}
              </Text>
            </View>
            <Badge
              label={tool.status === "running" ? "进行中" : "完成"}
              tone={tone}
              compact
            />
            <Ionicons
              name={expanded ? "chevron-up" : "chevron-down"}
              size={16}
              color={colors.quaternaryLabel as string}
            />
          </View>

          <Text
            style={{
              ...typography.caption2,
              color: colors.tertiaryLabel as string,
              fontVariant: ["tabular-nums"],
            }}
          >
            {formatLogTime(tool.startedTs)}
            {tool.completedTs ? ` → ${formatLogTime(tool.completedTs)}` : ""}
          </Text>
        </Pressable>

        {expanded ? (
          <View
            style={{
              marginLeft: spacing.xs,
              paddingLeft: spacing.md,
              borderLeftWidth: 2,
              borderLeftColor: colors.separator as string,
              gap: spacing.xs,
            }}
          >
            <DetailLine label="开始" value={tool.startedMessage} />
            {tool.completedMessage ? (
              <DetailLine label="完成" value={tool.completedMessage} />
            ) : null}
            {tool.resultDetail ? (
              <DetailLine label="结果" value={tool.resultDetail} />
            ) : null}
          </View>
        ) : null}
      </View>
    </View>
  );
}

function DetailLine({ label, value }: { label: string; value: string }) {
  return (
    <View style={{ gap: 2 }}>
      <Text
        style={{
          ...typography.caption2,
          color: colors.tertiaryLabel as string,
        }}
      >
        {label}
      </Text>
      <Text
        selectable
        style={{
          fontSize: 11,
          fontFamily: "Menlo",
          color: colors.secondaryLabel as string,
          lineHeight: 16,
        }}
      >
        {value}
      </Text>
    </View>
  );
}

function AssistantSnippet({
  lines,
}: {
  lines: { ts: number; text: string }[];
}) {
  const [open, setOpen] = useState(false);
  const preview = lines[lines.length - 1]?.text ?? "";

  return (
    <View
      style={{
        backgroundColor: colors.secondaryFill as string,
        borderRadius: radius.sm,
        borderCurve: "continuous",
        overflow: "hidden",
      }}
    >
      <Pressable
        onPress={() => setOpen((v) => !v)}
        style={({ pressed }) => ({
          paddingHorizontal: spacing.md,
          paddingVertical: spacing.sm,
          backgroundColor: pressed ? (colors.fill as string) : "transparent",
          flexDirection: "row",
          alignItems: "center",
          gap: spacing.sm,
        })}
      >
        <TabIcon name="text.bubble" color={colors.secondaryLabel as string} size={16} />
        <Text
          style={{
            ...typography.footnote,
            color: colors.secondaryLabel as string,
            flex: 1,
          }}
        >
          Agent 文本 ({lines.length})
        </Text>
        <Ionicons
          name={open ? "chevron-up" : "chevron-down"}
          size={16}
          color={colors.quaternaryLabel as string}
        />
      </Pressable>
      {open ? (
        <View style={{ paddingHorizontal: spacing.md, paddingBottom: spacing.sm, gap: 6 }}>
          {lines.map((line, index) => (
            <Text
              key={`${line.ts}-${index}`}
              selectable
              style={{
                fontSize: 11,
                fontFamily: "Menlo",
                color: colors.label as string,
                lineHeight: 16,
              }}
            >
              {line.text}
            </Text>
          ))}
        </View>
      ) : (
        <Text
          numberOfLines={2}
          style={{
            fontSize: 11,
            fontFamily: "Menlo",
            color: colors.tertiaryLabel as string,
            lineHeight: 16,
            paddingHorizontal: spacing.md,
            paddingBottom: spacing.sm,
          }}
        >
          {preview}
        </Text>
      )}
    </View>
  );
}
