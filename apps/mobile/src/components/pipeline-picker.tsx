import { Host, Picker } from "@expo/ui";
import { useMemo } from "react";
import { Text, View } from "react-native";

import type { WorkflowPipeline } from "@/api/types";
import { colors } from "@/theme/colors";
import { spacing, typography } from "@/theme/tokens";

const PLACEHOLDER_VALUE = "__pipeline_placeholder__";

export function PipelinePicker({
  pipelines,
  value,
  onValueChange,
  disabled,
  loading,
  placeholder = "待选择…",
}: {
  pipelines: WorkflowPipeline[];
  value: string;
  onValueChange: (name: string) => void;
  disabled?: boolean;
  loading?: boolean;
  placeholder?: string;
}) {
  const options = useMemo(() => {
    if (value && !pipelines.some((p) => p.name === value)) {
      return [
        { name: value, description: "Agent 推荐（当前工作区未扫描到同名模板）" },
        ...pipelines,
      ];
    }
    return pipelines;
  }, [pipelines, value]);

  const selected = options.find((p) => p.name === value);
  const selectedValue = value.trim() || PLACEHOLDER_VALUE;

  if (loading) {
    return (
      <Text style={{ ...typography.footnote, color: colors.secondaryLabel as string }}>
        加载工作流…
      </Text>
    );
  }

  if (options.length === 0) {
    return (
      <Text style={{ ...typography.footnote, color: colors.systemOrange as string }}>
        未获取到可用 pipeline，请确认 Server 可访问工作区
      </Text>
    );
  }

  return (
    <View style={{ gap: spacing.xs }}>
      <Host matchContents style={{ width: "100%", alignSelf: "stretch" }}>
        <Picker
          selectedValue={selectedValue}
          onValueChange={(itemValue) => {
            const next = String(itemValue ?? "").trim();
            if (!next || next === PLACEHOLDER_VALUE) return;
            onValueChange(next);
          }}
          enabled={!disabled}
          appearance="menu"
        >
          <Picker.Item label={placeholder} value={PLACEHOLDER_VALUE} />
          {options.map((pipeline) => (
            <Picker.Item
              key={pipeline.name}
              label={pipeline.name}
              value={pipeline.name}
            />
          ))}
        </Picker>
      </Host>
      {selected?.description ? (
        <Text
          style={{
            ...typography.caption2,
            color: colors.secondaryLabel as string,
            lineHeight: 18,
          }}
        >
          {selected.description}
        </Text>
      ) : null}
    </View>
  );
}
