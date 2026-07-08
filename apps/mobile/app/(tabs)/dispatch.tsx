import { useRouter } from "expo-router";
import { useEffect, useState } from "react";
import { Text, View } from "react-native";

import {
  GroupedInsetRow,
  GroupedSection,
} from "@/components/grouped-section";
import { PipelinePicker } from "@/components/pipeline-picker";
import {
  AppHost,
  ErrorText,
  FormInput,
  Hint,
  Loading,
  MonoText,
  PrimaryButton,
  SuccessBanner,
  WarningBanner,
} from "@/components/layout";
import { useConfig } from "@/hooks/useConfig";
import { useRuntimeStatus } from "@/hooks/useRuntimeStatus";
import { useWorkflows } from "@/hooks/useWorkflows";
import { colors } from "@/theme/colors";
import { spacing } from "@/theme/tokens";
import { shortWorkspace } from "@/utils/format";
import { hapticError, hapticLight, hapticSuccess } from "@/utils/haptics";

function defaultPipelineName(
  pipelines: { name: string }[],
  current: string
): string {
  if (current.trim()) return current;
  const preferred = pipelines.find((p) => p.name === "fix-regression");
  return preferred?.name ?? pipelines[0]?.name ?? "";
}

export default function DispatchScreen() {
  const router = useRouter();
  const { loaded, config, client } = useConfig();
  const { runtimeOnline } = useRuntimeStatus();
  const {
    pipelines,
    loading: workflowsLoading,
    error: workflowsError,
    refresh: refreshWorkflows,
  } = useWorkflows(client, config.workspaceId, loaded);
  const [issueKey, setIssueKey] = useState("");
  const [pipeline, setPipeline] = useState("");
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refreshingWorkflows, setRefreshingWorkflows] = useState(false);

  useEffect(() => {
    setPipeline((current) => defaultPipelineName(pipelines, current));
  }, [pipelines]);

  const onRefreshWorkflows = async () => {
    setRefreshingWorkflows(true);
    try {
      await refreshWorkflows();
    } finally {
      setRefreshingWorkflows(false);
    }
  };

  const submit = async () => {
    const key = issueKey.trim();
    const pipe = pipeline.trim();
    if (!key) {
      setError("请填写 Issue Key（如 PROJ-92）");
      hapticError();
      return;
    }
    if (!pipe) {
      setError("请选择 Pipeline");
      hapticError();
      return;
    }
    setBusy(true);
    setError(null);
    setResult(null);
    hapticLight();
    try {
      const resp = await client.dispatch({
        issue_key: key,
        pipeline: pipe,
      });
      if (resp.accepted) {
        setResult(`已入队 ${key} · ${pipe}`);
        setIssueKey("");
        hapticSuccess();
        router.push("/(tabs)");
      } else {
        setError(resp.reason ?? "派活被拒绝");
        hapticError();
      }
    } catch (e) {
      setError(String(e));
      hapticError();
    } finally {
      setBusy(false);
    }
  };

  return (
    <AppHost
      scroll
      refreshing={refreshingWorkflows}
      onRefresh={onRefreshWorkflows}
    >
      {!runtimeOnline ? (
        <WarningBanner>
          Runtime 当前离线，派活会入队但无法立即执行。请确认本机 Daemon 已启动。
        </WarningBanner>
      ) : null}

      <Hint>提交后由 Server 入队，本机 Daemon 认领并执行 issue start。</Hint>

      <GroupedSection title="工作区">
        <View style={{ padding: spacing.lg, gap: spacing.xs }}>
          <MonoText numberOfLines={1}>{shortWorkspace(config.workspaceId)}</MonoText>
          <Text style={{ fontSize: 13, color: colors.secondaryLabel as string }}>
            {`Runtime · ${config.runtimeId}`}
          </Text>
        </View>
      </GroupedSection>

      <GroupedSection
        title="派活参数"
        footer="Issue Key 对应 popsicle issue；Pipeline 从工作区模板加载，下拉刷新可同步本地变更。"
      >
        <GroupedInsetRow label="Issue Key">
          <FormInput
            placeholder="如 PROJ-92"
            value={issueKey}
            onChangeText={setIssueKey}
            autoCapitalize="none"
            autoCorrect={false}
          />
        </GroupedInsetRow>
        <GroupedInsetRow label="Pipeline" last>
          <PipelinePicker
            pipelines={pipelines}
            value={pipeline}
            onValueChange={(name) => {
              hapticLight();
              setPipeline(name);
            }}
            disabled={busy}
            loading={workflowsLoading}
            placeholder="选择 pipeline"
          />
        </GroupedInsetRow>
      </GroupedSection>

      <View style={{ gap: spacing.md }}>
        {busy ? (
          <Loading label="提交中…" />
        ) : (
          <PrimaryButton
            label="提交派活"
            onPress={submit}
            disabled={!pipeline.trim() || workflowsLoading}
          />
        )}
      </View>

      {result ? <SuccessBanner>{result}</SuccessBanner> : null}
      {workflowsError ? <ErrorText>{workflowsError}</ErrorText> : null}
      {error ? <ErrorText>{error}</ErrorText> : null}
    </AppHost>
  );
}
