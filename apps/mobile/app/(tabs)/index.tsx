import { useRouter } from "expo-router";
import { View } from "react-native";
import { Ionicons } from "@expo/vector-icons";

import type { RunMirror } from "@/api/types";
import { Badge } from "@/components/badge";
import { EmptyState } from "@/components/empty-state";
import {
  GroupedCard,
  GroupedRow,
  GroupedSection,
  GroupedSeparator,
  MetricTile,
} from "@/components/grouped-section";
import {
  AppHost,
  ErrorText,
  Hint,
  Loading,
  SecondaryButton,
  WarningBanner,
} from "@/components/layout";
import { useRuntimeStatus } from "@/hooks/useRuntimeStatus";
import { useRuns } from "@/hooks/useRuns";
import { colors, type Tone } from "@/theme/colors";
import { spacing } from "@/theme/tokens";
import { formatRelativeTime, runStatusLabel, displayRunTitle } from "@/utils/format";
import { hapticLight } from "@/utils/haptics";

function runTone(status: string): Tone {
  if (status === "completed") return "success";
  if (status === "in_progress") return "accent";
  if (status === "failed") return "danger";
  return "default";
}

function RunLeadingIcon({ status }: { status: string }) {
  const tone = runTone(status);
  const icon =
    status === "completed"
      ? "checkmark-circle"
      : status === "failed"
        ? "close-circle"
        : status === "in_progress"
          ? "play-circle"
          : "ellipse-outline";
  const color =
    tone === "success"
      ? (colors.systemGreen as string)
      : tone === "danger"
        ? (colors.systemRed as string)
        : tone === "accent"
          ? (colors.systemBlue as string)
          : (colors.tertiaryLabel as string);

  return (
    <View
      style={{
        width: 32,
        height: 32,
        borderRadius: 8,
        borderCurve: "continuous",
        backgroundColor: colors.secondaryFill as string,
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <Ionicons name={icon} size={18} color={color} />
    </View>
  );
}

export default function RunsScreen() {
  const router = useRouter();
  const { runs, loading, refreshing, error, refresh } = useRuns();
  const { runtimeOnline, health } = useRuntimeStatus();

  const activeCount = runs.filter((r) => r.run_status === "in_progress").length;
  const completedCount = runs.filter((r) => r.run_status === "completed").length;

  const handleRefresh = () => {
    hapticLight();
    refresh();
  };

  const openRun = (runId: string) => {
    hapticLight();
    router.push(`/run/${runId}`);
  };

  return (
    <AppHost scroll refreshing={refreshing} onRefresh={handleRefresh}>
      {!runtimeOnline ? (
        <WarningBanner>
          Runtime 离线，派活将无法执行。请在本机启动 Daemon 后下拉刷新。
        </WarningBanner>
      ) : null}

      <View style={{ flexDirection: "row", gap: spacing.md }}>
        <MetricTile
          label="进行中"
          value={String(activeCount)}
          tone={activeCount > 0 ? "accent" : "default"}
        />
        <MetricTile
          label="已完成"
          value={String(completedCount)}
          tone="success"
        />
        <MetricTile
          label="存储"
          value={health?.storage === "postgres" ? "PG" : health?.storage === "sqlite" ? "SQL" : "—"}
          tone="default"
        />
      </View>

      <Hint>
        {activeCount > 0
          ? `${activeCount} 个任务进行中 · 下拉刷新`
          : "Daemon 同步后自动更新 · 下拉刷新"}
      </Hint>

      {loading && runs.length === 0 ? <Loading /> : null}
      {error ? <ErrorText>{error}</ErrorText> : null}

      {runs.length === 0 && !loading && !error ? (
        <GroupedCard>
          <EmptyState
            symbol="tray.full"
            title="暂无 Pipeline Run"
            message="在「需求」页确认立项，或在「派活」页提交已有 Issue；本机 Daemon 同步后出现在此列表。"
            action={
              <SecondaryButton
                label="去派活"
                onPress={() => {
                  hapticLight();
                  router.push("/(tabs)/dispatch");
                }}
              />
            }
          />
        </GroupedCard>
      ) : runs.length > 0 ? (
        <GroupedSection
          title="全部 Run"
          footer="点按查看阶段进度、Agent 输出与远程批准。"
        >
          {runs.map((run, index) => {
            const title = displayRunTitle(run);
            const stageHint =
              run.run_status === "completed"
                ? "已完成"
                : run.current_stage
                  ? `阶段 · ${run.current_stage}`
                  : "等待启动";
            const canResume = run.run_status === "in_progress";
            const subtitle = `${run.pipeline} · ${stageHint} · ${formatRelativeTime(run.updated_at)}${canResume ? " · 可恢复" : ""}`;

            return (
              <View key={run.run_id}>
                <GroupedRow
                  title={title}
                  subtitle={subtitle}
                  leading={<RunLeadingIcon status={run.run_status} />}
                  trailing={
                    <Badge
                      label={runStatusLabel(run.run_status)}
                      tone={runTone(run.run_status)}
                      compact
                    />
                  }
                  onPress={() => openRun(run.run_id)}
                />
                {index < runs.length - 1 ? <GroupedSeparator inset={64} /> : null}
              </View>
            );
          })}
        </GroupedSection>
      ) : null}
    </AppHost>
  );
}
