import { StyleSheet, View } from "react-native";
import { BlurView } from "expo-blur";

import { colors } from "@/theme/colors";

/** iOS 毛玻璃 Tab Bar 背景（配合 `tabBarStyle: { position: 'absolute', backgroundColor: 'transparent' }`） */
export function TabBarBackground() {
  if (process.env.EXPO_OS !== "ios") {
    return null;
  }

  return (
    <View style={StyleSheet.absoluteFill}>
      <BlurView
        tint="systemChromeMaterial"
        intensity={80}
        style={StyleSheet.absoluteFill}
      />
      <View
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          right: 0,
          height: StyleSheet.hairlineWidth,
          backgroundColor: colors.separator as string,
          opacity: 0.6,
        }}
      />
    </View>
  );
}
