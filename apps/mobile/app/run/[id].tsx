import {
  useLocalSearchParams,
  useNavigation,
  useRouter,
} from "expo-router";
import { useCallback, useEffect, useLayoutEffect, useState } from "react";
import { Alert, Pressable, Text, View } from "react-native";

import type { RunMirror } from "@/api/types";
import { DANGEROUS_STAGES } from "@/api/types";
import { Badge } from "@/components/badge";
import { GroupedSection } from "@/components/grouped-section";
import { RunLogPanel } from "@/components/run-log-panel";
import { StageTimeline } from "@/components/stage-timeline";
import { TabIcon } from "@/components/tab-icon";
import {
  AppHost,
  ErrorText,
  Hint,
  Loading,
  MonoText,
  PrimaryButton,
  SuccessBanner,
  TextButton,
} from "@/components/layout";
import { useConfig } from "@/hooks/useConfig";
import { useRunLogs } from "@/hooks/useRunLogs";
import { colors, type Tone } from "@/theme/colors";
import { spacing, typography } from "@/theme/tokens";
import { runStatusLabel, displayRunTitle } from "@/utils/format";
import { sanitizeRunMirror } from "@/utils/run-mirror";
import { resumeReasonLabel } from "@/utils/resume";
import {
  hapticError,
  hapticLight,
  hapticSuccess,
} from "@/utils/haptics";

export default function RunDetailScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const router = useRouter();
  const navigation = useNavigation();
  const { client } = useConfig();
  const { logs, loading: logsLoading } = useRunLogs(id);
  const [run, setRun] = useState<RunMirror | null>(null);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [approving, setApproving] = useState(false);
  const [resuming, setResuming] = useState(false);
  const [resumeMsg, setResumeMsg] = useState<string | null>(null);

  const load = useCallback(
    async (opts?: { silent?: boolean }) => {
      if (!id) return;
      if (!opts?.silent) setLoading(true);
      setError(null);
      try {
        const mirror = await client.getRun(id);
        setRun((prev) => sanitizeRunMirror(mirror, prev ?? undefined));
      } catch (e) {
        setError(String(e));
      } finally {
        if (!opts?.silent) setLoading(false);
      }
    },
    [client, id]
  );

  const refresh = useCallback(async () => {
    setRefreshing(true);
    hapticLight();
    await load({ silent: true });
    setRefreshing(false);
  }, [load]);

  useEffect(() => {
    load();
  }, [load]);

  useEffect(() => {
    const disconnect = client.connectEvents((event) => {
      if (event.type === "run_updated" && event.mirror?.run_id === id) {
        setRun((prev) => sanitizeRunMirror(event.mirror!, prev ?? undefined));
      }
    });
    return disconnect;
  }, [client, id]);

  useEffect(() => {
    if (run) {
      navigation.setOptions({ title: displayRunTitle(run) });
    }
  }, [run, navigation]);

  useLayoutEffect(() => {
    navigation.setOptions({
      headerBackTitle: "进度",
      headerRight: () => (
        <Pressable
          onPress={refresh}
          hitSlop={12}
          style={{ paddingHorizontal: 8 }}
        >
          <TabIcon name="arrow.clockwise" color={colors.systemBlue as string} />
        </Pressable>
      ),
    });
  }, [navigation, refresh]);

  const resumeRun = () => {
    if (!id) return;
    hapticLight();
    Alert.alert(
      "恢复执行",
      "Daemon 将跳过 issue start，直接从当前阶段继续 pipeline 编排。请确保本机 Daemon 已启动。",
      [
        { text: "取消", style: "cancel" },
        {
          text: "恢复",
          onPress: async () => {
            setResuming(true);
            setResumeMsg(null);
            setError(null);
            try {
              const resp = await client.resume(id);
              if (resp.accepted) {
                setResumeMsg("已入队恢复任务，等待 Daemon 认领后继续 pipeline。");
                hapticSuccess();
              } else {
                const reason = resumeReasonLabel(resp.reason ?? "恢复被拒绝");
                setError(reason);
                hapticError();
              }
            } catch (e) {
              setError(String(e));
              hapticError();
            } finally {
              setResuming(false);
            }
          },
        },
      ]
    );
  };

  const approveStage = (stage: string) => {
    const dangerous = DANGEROUS_STAGES.has(stage);
    const message = dangerous
      ? `「${stage}」为危险阶段，等价于 pipeline stage complete --confirm。确认批准？`
      : `批准阶段「${stage}」？Daemon 将执行 stage complete --confirm。`;

    hapticLight();
    Alert.alert("远程批准", message, [
      { text: "取消", style: "cancel" },
      {
        text: "批准",
        style: dangerous ? "destructive" : "default",
        onPress: async () => {
          if (!id) return;
          setApproving(true);
          try {
            await client.approve(id, stage);
            hapticSuccess();
            Alert.alert("已入队", "confirm 任务已创建，等待 Daemon 认领。");
          } catch (e) {
            hapticError();
            Alert.alert("失败", String(e));
          } finally {
            setApproving(false);
          }
        },
      },
    ]);
  };

  if (loading && !run) {
    return (
      <AppHost>
        <Loading />
      </AppHost>
    );
  }

  if (!run) {
    return (
      <AppHost>
        <ErrorText>{error ?? "Run 不存在"}</ErrorText>
        <TextButton label="返回" onPress={() => router.back()} />
      </AppHost>
    );
  }

  const statusTone: Tone =
    run.run_status === "completed"
      ? "success"
      : run.run_status === "in_progress"
        ? "accent"
        : run.run_status === "failed"
          ? "danger"
          : "default";

  const pendingStage = run.stages.find(
    (s) =>
      run.run_status === "in_progress" &&
      (s.status === "in_progress" || s.name === run.current_stage)
  );

  return (
    <AppHost scroll refreshing={refreshing} onRefresh={refresh}>
      <View style={{ gap: spacing.sm }}>
        <View style={{ flexDirection: "row", alignItems: "center", gap: spacing.sm }}>
          <Badge label={runStatusLabel(run.run_status)} tone={statusTone} />
          <Text style={{ ...typography.subhead, color: colors.secondaryLabel as string }}>
            {run.pipeline}
          </Text>
        </View>
        <MonoText>{run.run_id}</MonoText>
      </View>

      <GroupedSection title="阶段进度">
        <View style={{ padding: spacing.lg }}>
          <StageTimeline
            stages={run.stages}
            currentStage={
              run.run_status === "completed" ? "" : run.current_stage
            }
          />
        </View>
      </GroupedSection>

      <RunLogPanel logs={logs} loading={logsLoading} />

      {run.run_status === "in_progress" ? (
        <GroupedSection
          title="恢复执行"
          footer="Daemon 曾关闭或编排中断时使用，不会创建新 run。"
        >
          <View style={{ padding: spacing.lg, gap: spacing.md }}>
            <Hint>
              Daemon 曾关闭或编排中断时，点此让 Agent 从当前阶段继续 pipeline。
            </Hint>
            <PrimaryButton
              label={resuming ? "提交中…" : "恢复执行"}
              disabled={resuming}
              onPress={resumeRun}
            />
          </View>
        </GroupedSection>
      ) : null}

      {pendingStage ? (
        <GroupedSection title="待审批">
          <View style={{ padding: spacing.lg, gap: spacing.md }}>
            <Hint>
              {DANGEROUS_STAGES.has(pendingStage.name)
                ? `「${pendingStage.name}」为危险阶段，批准前请确认产物已审阅。`
                : `当前可进行远程批准：${pendingStage.name}`}
            </Hint>
            <PrimaryButton
              label={approving ? "提交中…" : "批准当前阶段"}
              disabled={approving}
              onPress={() => approveStage(pendingStage.name)}
            />
          </View>
        </GroupedSection>
      ) : null}

      {resumeMsg ? <SuccessBanner>{resumeMsg}</SuccessBanner> : null}
      {error ? <ErrorText>{error}</ErrorText> : null}
    </AppHost>
  );
}
