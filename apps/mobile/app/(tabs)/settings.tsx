import { useEffect, useState } from "react";
import { View } from "react-native";

import { Badge } from "@/components/badge";
import {
  GroupedInsetRow,
  GroupedRow,
  GroupedSection,
  GroupedSeparator,
} from "@/components/grouped-section";
import {
  AppHost,
  ErrorText,
  FormInput,
  Hint,
  Loading,
  PrimaryButton,
  SuccessBanner,
} from "@/components/layout";
import { useConfig } from "@/hooks/useConfig";
import { useRuntimeStatus } from "@/hooks/useRuntimeStatus";
import { colors, type Tone } from "@/theme/colors";
import { spacing } from "@/theme/tokens";
import { hapticLight, hapticSuccess } from "@/utils/haptics";

export default function SettingsScreen() {
  const { config, loaded, saveConfig } = useConfig();
  const {
    health,
    runtime,
    refreshing,
    refresh,
    serverOk,
    runtimeOnline,
    error: statusError,
  } = useRuntimeStatus();
  const [serverUrl, setServerUrl] = useState(config.serverUrl);
  const [runtimeId, setRuntimeId] = useState(config.runtimeId);
  const [workspaceId, setWorkspaceId] = useState(config.workspaceId);
  const [busy, setBusy] = useState(false);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (loaded) {
      setServerUrl(config.serverUrl);
      setRuntimeId(config.runtimeId);
      setWorkspaceId(config.workspaceId);
    }
  }, [loaded, config]);

  const handleRefresh = () => {
    hapticLight();
    refresh();
  };

  const handleSave = async () => {
    setBusy(true);
    setSaved(false);
    hapticLight();
    try {
      await saveConfig({
        serverUrl: serverUrl.trim(),
        runtimeId: runtimeId.trim() || "default",
        workspaceId: workspaceId.trim(),
      });
      setSaved(true);
      hapticSuccess();
      await refresh({ silent: true });
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <AppHost scroll refreshing={refreshing} onRefresh={handleRefresh}>
      <Hint>
        真机请填开发机局域网 IP（如 http://192.168.1.10:8787），确保手机与电脑在同一 Wi-Fi。
      </Hint>

      <GroupedSection title="连接状态">
        <StatusRow
          title="Agent Server"
          detail={health ? `正常 · ${health.storage}` : "不可达"}
          tone={serverOk ? "success" : "danger"}
        />
        <GroupedSeparator inset={spacing.lg} />
        <StatusRow
          title="Runtime"
          detail={runtime?.state ?? "未知"}
          tone={runtimeOnline ? "success" : "warning"}
        />
      </GroupedSection>

      <GroupedSection
        title="连接配置"
        footer="修改后点保存；Server URL 不要带尾部斜杠。"
      >
        <GroupedInsetRow label="Server URL">
          <FormInput
            placeholder="http://192.168.1.10:8787"
            value={serverUrl}
            onChangeText={(v) => {
              setServerUrl(v);
              setSaved(false);
            }}
            autoCapitalize="none"
            autoCorrect={false}
          />
        </GroupedInsetRow>
        <GroupedInsetRow label="Runtime ID">
          <FormInput
            placeholder="default"
            value={runtimeId}
            onChangeText={(v) => {
              setRuntimeId(v);
              setSaved(false);
            }}
            autoCapitalize="none"
          />
        </GroupedInsetRow>
        <GroupedInsetRow label="Workspace" last>
          <FormInput
            placeholder="/path/to/project"
            value={workspaceId}
            onChangeText={(v) => {
              setWorkspaceId(v);
              setSaved(false);
            }}
            autoCapitalize="none"
          />
        </GroupedInsetRow>
      </GroupedSection>

      <View style={{ gap: spacing.md }}>
        {busy ? <Loading label="保存中…" /> : <PrimaryButton label="保存配置" onPress={handleSave} />}
      </View>

      {saved ? <SuccessBanner>配置已保存</SuccessBanner> : null}
      {error ? <ErrorText>{error}</ErrorText> : null}
      {statusError ? <ErrorText>{statusError}</ErrorText> : null}
    </AppHost>
  );
}

function StatusRow({
  title,
  detail,
  tone,
}: {
  title: string;
  detail: string;
  tone: Tone;
}) {
  return (
    <GroupedRow
      title={title}
      trailing={<Badge label={detail} tone={tone} compact />}
      showChevron={false}
    />
  );
}
