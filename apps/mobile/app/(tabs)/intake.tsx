import { useRouter, type Href } from "expo-router";
import Constants from "expo-constants";
import { useEffect, useRef, useState } from "react";
import { Platform, Text, View } from "react-native";

import { ChatMarkdown } from "@/components/chat-markdown";
import {
  IntakeMessageInput,
  type IntakeMessageInputRef,
} from "@/components/intake-message-input";
import { PipelinePicker } from "@/components/pipeline-picker";
import {
  GroupedInsetRow,
  GroupedSection,
} from "@/components/grouped-section";
import {
  AppHost,
  ErrorText,
  Hint,
  Loading,
  MonoText,
  PrimaryButton,
  SecondaryButton,
  SuccessBanner,
  WarningBanner,
} from "@/components/layout";
import { useChatSession } from "@/hooks/useChatSession";
import { useConfig } from "@/hooks/useConfig";
import { useRuntimeStatus } from "@/hooks/useRuntimeStatus";
import { useWorkflows } from "@/hooks/useWorkflows";
import { colors } from "@/theme/colors";
import { radius, spacing, typography } from "@/theme/tokens";
import {
  hapticError,
  hapticLight,
  hapticSuccess,
} from "@/utils/haptics";

export default function IntakeScreen() {
  const router = useRouter();
  const { loaded, config, client } = useConfig();
  const { runtimeOnline } = useRuntimeStatus();
  const {
    pipelines,
    loading: workflowsLoading,
    error: workflowsError,
    refresh: refreshWorkflows,
  } = useWorkflows(client, config.workspaceId, loaded);
  const {
    session,
    messages,
    busy,
    error,
    sendMessage,
    bootstrap,
    resetSession,
    updateDraftPipeline,
  } = useChatSession(client, loaded);
  const inputRef = useRef<IntakeMessageInputRef>(null);
  const [bootResult, setBootResult] = useState<string | null>(null);
  const [pipelineInput, setPipelineInput] = useState("");
  const [savingPipeline, setSavingPipeline] = useState(false);
  const [refreshingWorkflows, setRefreshingWorkflows] = useState(false);

  useEffect(() => {
    setPipelineInput(session?.draft_pipeline?.trim() ?? "");
  }, [session?.draft_pipeline, session?.id]);

  const clearInput = () => {
    inputRef.current?.clear();
  };

  const draftReady =
    session?.status === "ready" ||
    Boolean(session?.draft_title && session?.draft_pipeline);
  const bootstrapped = session?.status === "bootstrapped";
  const draftEditable = Boolean(session?.draft_title) && !bootstrapped && !busy;

  const savePipeline = async (value: string) => {
    const trimmed = value.trim();
    if (!trimmed || trimmed === (session?.draft_pipeline?.trim() ?? "")) return;
    setSavingPipeline(true);
    hapticLight();
    await updateDraftPipeline(trimmed);
    setSavingPipeline(false);
  };

  const onSelectPipeline = async (name: string) => {
    hapticLight();
    setPipelineInput(name);
    await savePipeline(name);
  };

  const onSend = async () => {
    const text = (inputRef.current?.getText() ?? "").trim();
    if (!text || busy) return;
    hapticLight();
    clearInput();
    await sendMessage(text);
  };

  const onBootstrap = async () => {
    hapticLight();
    setBootResult(null);
    const result = await bootstrap();
    if (result) {
      hapticSuccess();
      setBootResult(
        result.issueKey
          ? `已立项 ${result.issueKey} · 请在「进度」查看`
          : `已立项 · Run ${result.runId.slice(0, 8)}…`
      );
      router.push("/(tabs)/index" as Href);
    } else {
      hapticError();
    }
  };

  const onReset = async () => {
    hapticLight();
    setBootResult(null);
    clearInput();
    await resetSession();
  };

  const onRefreshWorkflows = async () => {
    setRefreshingWorkflows(true);
    try {
      await refreshWorkflows();
    } finally {
      setRefreshingWorkflows(false);
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
          Runtime 离线时无法发送 Chat 消息。请启动本机 Daemon。
        </WarningBanner>
      ) : null}

      <Hint>
        用自然语言描述需求，Agent 会澄清并生成草案；确认后自动创建 Issue 并启动
        pipeline。
      </Hint>

      <GroupedSection title="对话">
        <View style={{ padding: spacing.lg, gap: spacing.md }}>
          {messages.length === 0 ? (
            <Text style={{ color: colors.secondaryLabel as string, fontSize: 15 }}>
              例如：「给 Mobile 加一个需求 Chat，走 feature-spec pipeline」
            </Text>
          ) : (
            messages.map((m) => (
              <View
                key={m.id}
                style={{
                  alignSelf: m.role === "user" ? "flex-end" : "stretch",
                  width: m.role === "user" ? undefined : "100%",
                  maxWidth: "92%",
                  backgroundColor:
                    m.role === "user"
                      ? (colors.systemBlue as string)
                      : (colors.secondaryFill as string),
                  borderRadius: 16,
                  borderCurve: "continuous",
                  paddingHorizontal: spacing.md,
                  paddingVertical: spacing.sm,
                }}
              >
                {m.role === "user" ? (
                  <Text
                    selectable
                    style={{
                      color: "#fff",
                      fontSize: 15,
                      lineHeight: 21,
                    }}
                  >
                    {m.content}
                  </Text>
                ) : (
                  <ChatMarkdown content={m.content} variant="assistant" />
                )}
              </View>
            ))
          )}
          {busy ? <Loading label="Agent 思考中…" /> : null}
        </View>
      </GroupedSection>

      {session?.draft_title ? (
        <GroupedSection
          title="草案"
          footer="确认前可修改 Pipeline；立项前须选定 pipeline。"
        >
          <GroupedInsetRow label="标题">
            <MonoText numberOfLines={2}>{session.draft_title}</MonoText>
          </GroupedInsetRow>
          <GroupedInsetRow label="Pipeline" last={!session.draft_description}>
            <PipelinePicker
              pipelines={pipelines}
              value={pipelineInput}
              onValueChange={onSelectPipeline}
              disabled={!draftEditable || savingPipeline}
              loading={workflowsLoading}
              placeholder="待 Agent 推荐或手动选择"
            />
            {!session.draft_pipeline ? (
              <Text
                style={{
                  ...typography.caption2,
                  color: colors.systemOrange as string,
                }}
              >
                尚未选定 pipeline，请从下拉列表选择或继续与 Agent 澄清
              </Text>
            ) : null}
          </GroupedInsetRow>
          {session.draft_description ? (
            <View style={{ padding: spacing.lg }}>
              <ChatMarkdown content={session.draft_description} variant="assistant" />
            </View>
          ) : null}
        </GroupedSection>
      ) : null}

      <View style={{ gap: spacing.sm }}>
        <Text
          style={{
            ...typography.sectionHeader,
            color: colors.secondaryLabel as string,
            paddingHorizontal: spacing.lg + 4,
          }}
        >
          输入
        </Text>
        <View
          style={{
            backgroundColor: colors.secondaryGroupedBackground as string,
            borderRadius: radius.md,
            borderCurve: "continuous",
            paddingHorizontal: spacing.lg,
            paddingVertical: spacing.sm,
            gap: spacing.xs,
            overflow: "visible",
          }}
        >
          <Text
            style={{
              ...typography.footnote,
              fontWeight: "500",
              color: colors.secondaryLabel as string,
            }}
          >
            消息
          </Text>
          <IntakeMessageInput
            ref={inputRef}
            placeholder="描述你的需求…"
            editable={!busy && !bootstrapped}
          />
        </View>
        {Platform.OS === "ios" && Constants.appOwnership === "expo" ? (
          <Hint>
            {`Expo Go 对中文拼音支持不稳定。若无法选字，请在 apps/mobile 目录执行 npx expo run:ios 安装开发构建后再试。`}
          </Hint>
        ) : null}
      </View>

      <View style={{ gap: spacing.md }}>
        <PrimaryButton
          label="发送"
          onPress={onSend}
          disabled={busy || !runtimeOnline || bootstrapped}
        />
        {draftReady && !bootstrapped ? (
          <PrimaryButton
            label="确认并立项"
            onPress={onBootstrap}
            disabled={busy || !runtimeOnline || !session?.draft_pipeline?.trim()}
          />
        ) : null}
        <SecondaryButton label="新会话" onPress={onReset} disabled={busy} />
      </View>

      {bootstrapped && session?.linked_issue_key ? (
        <GroupedSection title="已立项">
          <GroupedInsetRow label="Issue">
            <MonoText>{session.linked_issue_key}</MonoText>
          </GroupedInsetRow>
          {session.linked_run_id ? (
            <GroupedInsetRow label="Run" last>
              <SecondaryButton
                label="在进度页查看"
                onPress={() => {
                  hapticLight();
                  router.push(`/run/${session.linked_run_id}`);
                }}
              />
            </GroupedInsetRow>
          ) : null}
        </GroupedSection>
      ) : null}

      {bootResult ? <SuccessBanner>{bootResult}</SuccessBanner> : null}
      {workflowsError ? <ErrorText>{workflowsError}</ErrorText> : null}
      {error ? <ErrorText>{error}</ErrorText> : null}
    </AppHost>
  );
}
