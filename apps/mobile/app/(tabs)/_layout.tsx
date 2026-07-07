import { Platform } from "react-native";
import { Tabs } from "expo-router";

import { TabBarBackground } from "@/components/tab-bar-background";
import { TabIcon } from "@/components/tab-icon";
import { colors } from "@/theme/colors";

const iosTabBarStyle = Platform.OS === "ios";

export default function TabsLayout() {
  return (
    <Tabs
      initialRouteName="intake"
      screenOptions={{
        headerShown: true,
        headerShadowVisible: false,
        headerStyle: {
          backgroundColor: colors.groupedBackground as string,
        },
        headerTitleStyle: {
          fontWeight: "700",
          fontSize: 17,
        },
        tabBarActiveTintColor: colors.systemBlue as string,
        tabBarInactiveTintColor: colors.tertiaryLabel as string,
        tabBarBackground: iosTabBarStyle ? () => <TabBarBackground /> : undefined,
        tabBarStyle: iosTabBarStyle
          ? {
              position: "absolute",
              backgroundColor: "transparent",
              borderTopWidth: 0,
              elevation: 0,
            }
          : {
              backgroundColor: colors.systemBackground as string,
              borderTopColor: colors.separator as string,
              borderTopWidth: 0.5,
            },
        tabBarLabelStyle: {
          fontSize: 11,
          fontWeight: "500",
        },
      }}
    >
      <Tabs.Screen
        name="intake"
        options={{
          title: "需求",
          tabBarLabel: "需求",
          headerTitle: "需求",
          tabBarIcon: ({ color, focused }) => (
            <TabIcon
              name="bubble.left.and.bubble.right.fill"
              color={color as string}
              focused={focused}
            />
          ),
        }}
      />
      <Tabs.Screen
        name="dispatch"
        options={{
          title: "派活",
          tabBarLabel: "派活",
          headerTitle: "派活",
          tabBarIcon: ({ color, focused }) => (
            <TabIcon
              name="paperplane.fill"
              color={color as string}
              focused={focused}
            />
          ),
        }}
      />
      <Tabs.Screen
        name="index"
        options={{
          title: "进度",
          tabBarLabel: "进度",
          headerTitle: "进度",
          tabBarIcon: ({ color, focused }) => (
            <TabIcon
              name="list.bullet.rectangle"
              color={color as string}
              focused={focused}
            />
          ),
        }}
      />
      <Tabs.Screen
        name="settings"
        options={{
          title: "设置",
          tabBarLabel: "设置",
          headerTitle: "设置",
          tabBarIcon: ({ color, focused }) => (
            <TabIcon name="gear" color={color as string} focused={focused} />
          ),
        }}
      />
    </Tabs>
  );
}
