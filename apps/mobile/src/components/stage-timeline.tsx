import { Text, View } from "react-native";

import type { StageMirror } from "@/api/types";
import { Badge } from "@/components/badge";
import { stageStatusLabel } from "@/utils/format";
import { colors, type Tone } from "@/theme/colors";
import { spacing, typography } from "@/theme/tokens";

function stageTone(status: string, isCurrent: boolean): Tone {
  if (status === "completed") return "success";
  if (status === "in_progress" || isCurrent) return "accent";
  if (status === "skipped") return "muted";
  return "default";
}

function dotColor(status: string, isCurrent: boolean): string {
  if (status === "completed") return colors.systemGreen as string;
  if (status === "in_progress" || isCurrent) return colors.systemBlue as string;
  return colors.quaternaryLabel as string;
}

export function StageTimeline({
  stages,
  currentStage,
}: {
  stages: StageMirror[];
  currentStage: string;
}) {
  if (stages.length === 0) {
    return (
      <Text style={{ ...typography.footnote, color: colors.secondaryLabel as string }}>
        暂无阶段数据
      </Text>
    );
  }

  return (
    <View>
      {stages.map((stage, index) => {
        const isCurrent = stage.name === currentStage;
        const isLast = index === stages.length - 1;
        const tone = stageTone(stage.status, isCurrent);

        return (
          <View key={stage.name} style={{ flexDirection: "row", gap: spacing.md }}>
            <View style={{ alignItems: "center", width: 22 }}>
              <View
                style={{
                  width: isCurrent ? 12 : 10,
                  height: isCurrent ? 12 : 10,
                  borderRadius: 6,
                  backgroundColor: dotColor(stage.status, isCurrent),
                  borderCurve: "continuous",
                  borderWidth: isCurrent ? 2 : 0,
                  borderColor: isCurrent
                    ? `${colors.systemBlue as string}33`
                    : "transparent",
                }}
              />
              {!isLast ? (
                <View
                  style={{
                    width: 2,
                    flex: 1,
                    minHeight: 32,
                    backgroundColor: colors.separator as string,
                    marginTop: 4,
                    borderRadius: 1,
                  }}
                />
              ) : null}
            </View>

            <View
              style={{
                flex: 1,
                paddingBottom: isLast ? 0 : spacing.lg,
                flexDirection: "row",
                alignItems: "center",
                gap: spacing.sm,
              }}
            >
              <Text
                style={{
                  ...typography.subhead,
                  fontWeight: isCurrent ? "600" : "500",
                  color: colors.label as string,
                  flex: 1,
                }}
              >
                {stage.name}
              </Text>
              <Badge label={stageStatusLabel(stage.status)} tone={tone} compact />
            </View>
          </View>
        );
      })}
    </View>
  );
}
