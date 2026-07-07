import type { RunLogEntry } from "@/api/types";

export function formatLogTime(ts: number): string {
  const epochMs = ts < 1_000_000_000_000 ? ts * 1000 : ts;
  return new Date(epochMs).toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

export function logLevelTone(level: string): "default" | "accent" | "danger" {
  if (level === "error") return "danger";
  if (level === "agent") return "accent";
  return "default";
}

export function isAgentLine(message: string): boolean {
  return message.startsWith("› ") || message.startsWith("✗ ");
}

export function displayLogMessage(entry: RunLogEntry): string {
  return entry.message;
}
