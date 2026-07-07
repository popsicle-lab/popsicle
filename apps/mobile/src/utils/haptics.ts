import * as Haptics from "expo-haptics";

async function run(fn: () => Promise<void>) {
  if (process.env.EXPO_OS !== "ios") return;
  try {
    await fn();
  } catch {
    // haptics are best-effort
  }
}

export function hapticLight() {
  return run(() => Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light));
}

export function hapticSuccess() {
  return run(() =>
    Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success)
  );
}

export function hapticError() {
  return run(() =>
    Haptics.notificationAsync(Haptics.NotificationFeedbackType.Error)
  );
}

export function hapticSelection() {
  return run(() => Haptics.selectionAsync());
}
