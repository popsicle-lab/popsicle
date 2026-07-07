import { useRouter } from "expo-router";
import { useState } from "react";
import { Text, View } from "react-native";

import {
  GroupedInsetRow,
  GroupedSection,
} from "@/components/grouped-section";
import {
  AppHost,
  Chip,
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
import { colors } from "@/theme/colors";
import { spacing } from "@/theme/tokens";
import { shortWorkspace } from "@/utils/format";
import {
  hapticError,
  hapticLight,
  hapticSelection,
  hapticSuccess,
} from "@/utils/haptics";

const PIPELINE_PRESETS = [
  "fix-regression",
  "feature-delivery",
  "feature-spec",
  "arch-decision",
] as const;

export default function DispatchScreen() {
  const router = useRouter();
  const { config, client } = useConfig();
  const { runtimeOnline } = useRuntimeStatus();
  const [issueKey, setIssueKey] = useState("");
  const [pipeline, setPipeline] = useState("fix-regression");
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const submit = async () => {
    const key = issueKey.trim();
    const pipe = pipeline.trim() || "fix-regression";
    if (!key) {
      setError("请填写 Issue Key（如 PROJ-92）");
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

  const selectPipeline = (name: string) => {
    hapticSelection();
    setPipeline(name);
  };

  return (
    <AppHost scroll>
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
        footer="Issue Key 对应 popsicle issue；Pipeline 决定编排链路。"
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
          <FormInput
            placeholder="fix-regression"
            value={pipeline}
            onChangeText={setPipeline}
            autoCapitalize="none"
          />
        </GroupedInsetRow>
      </GroupedSection>

      <GroupedSection title="常用 Pipeline">
        <View
          style={{
            flexDirection: "row",
            flexWrap: "wrap",
            gap: spacing.sm,
            padding: spacing.lg,
          }}
        >
          {PIPELINE_PRESETS.map((name) => (
            <Chip
              key={name}
              label={name}
              selected={pipeline === name}
              onPress={() => selectPipeline(name)}
            />
          ))}
        </View>
      </GroupedSection>

      <View style={{ gap: spacing.md }}>
        {busy ? <Loading label="提交中…" /> : <PrimaryButton label="提交派活" onPress={submit} />}
      </View>

      {result ? <SuccessBanner>{result}</SuccessBanner> : null}
      {error ? <ErrorText>{error}</ErrorText> : null}
    </AppHost>
  );
}
