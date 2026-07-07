import { useRouter } from "expo-router";
import Constants from "expo-constants";
import { useRef, useState } from "react";
import { Platform, Text, View } from "react-native";

import { ChatMarkdown } from "@/components/chat-markdown";
import {
  IntakeMessageInput,
  type IntakeMessageInputRef,
} from "@/components/intake-message-input";
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
import { colors } from "@/theme/colors";
import { radius, spacing, typography } from "@/theme/tokens";
import {
  hapticError,
  hapticLight,
  hapticSuccess,
} from "@/utils/haptics";

export default function IntakeScreen() {
  const router = useRouter();
  const { loaded } = useConfig();
  const { client } = useConfig();
  const { runtimeOnline } = useRuntimeStatus();
  const {
    session,
    messages,
    busy,
    error,
    sendMessage,
    bootstrap,
    resetSession,
  } = useChatSession(client, loaded);
  const inputRef = useRef<IntakeMessageInputRef>(null);
  const [bootResult, setBootResult] = useState<string | null>(null);

  const clearInput = () => {
    inputRef.current?.clear();
  };

  const draftReady =
    session?.status === "ready" ||
    Boolean(session?.draft_title && session?.draft_pipeline);
  const bootstrapped = session?.status === "bootstrapped";

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
    const runId = await bootstrap();
    if (runId) {
      hapticSuccess();
      setBootResult(`已立项 · Run ${runId.slice(0, 8)}…`);
      router.push("/(tabs)");
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

  return (
    <AppHost scroll>
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
                  alignSelf: m.role === "user" ? "flex-end" : "flex-start",
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
        <GroupedSection title="草案" footer="确认后将 issue create + issue start。">
          <GroupedInsetRow label="标题">
            <MonoText numberOfLines={2}>{session.draft_title}</MonoText>
          </GroupedInsetRow>
          <GroupedInsetRow label="Pipeline">
            <MonoText>{session.draft_pipeline ?? "feature-spec"}</MonoText>
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
            disabled={busy || !runtimeOnline}
          />
        ) : null}
        <SecondaryButton label="新会话" onPress={onReset} disabled={busy} />
      </View>

      {bootResult ? <SuccessBanner>{bootResult}</SuccessBanner> : null}
      {error ? <ErrorText>{error}</ErrorText> : null}
    </AppHost>
  );
}
