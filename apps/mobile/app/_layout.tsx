import "react-native-gesture-handler";

import { Stack } from "expo-router";
import { StatusBar } from "expo-status-bar";

import { ConfigProvider } from "@/hooks/useConfig";
import { colors } from "@/theme/colors";

export default function RootLayout() {
  return (
    <ConfigProvider>
      <StatusBar style="auto" />
      <Stack
        screenOptions={{
          headerLargeTitle: false,
          headerShadowVisible: false,
          headerStyle: {
            backgroundColor: colors.groupedBackground as string,
          },
          headerTitleStyle: {
            fontWeight: "600",
          },
          contentStyle: {
            backgroundColor: colors.groupedBackground as string,
          },
        }}
      >
        <Stack.Screen
          name="(tabs)"
          options={{
            headerShown: false,
            title: "进度",
          }}
        />
        <Stack.Screen
          name="run/[id]"
          options={{
            title: "Run 详情",
            presentation: "card",
            headerBackTitle: "进度",
          }}
        />
      </Stack>
    </ConfigProvider>
  );
}
