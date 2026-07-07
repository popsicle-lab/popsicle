import type { RunLogEntry } from "@/api/types";

export type ToolCallStatus = "running" | "done";

export interface ToolTimelineItem {
  id: string;
  kind: string;
  detail: string;
  resultDetail?: string;
  status: ToolCallStatus;
  startedTs: number;
  completedTs?: number;
  startedMessage: string;
  completedMessage?: string;
}

export interface AgentSessionMeta {
  startedTs?: number;
  model?: string;
  sessionId?: string;
  finishedTs?: number;
  durationMs?: number;
  isError?: boolean;
}

export interface AgentTimeline {
  session: AgentSessionMeta;
  tools: ToolTimelineItem[];
  assistantLines: { ts: number; text: string }[];
  hasStructuredAgentOutput: boolean;
}

const TOOL_START = /^agent: tool ▶ (\S+) (.+)$/;
const TOOL_DONE = /^agent: tool ✓ (\S+) (.+)$/;
const AGENT_START = /^agent: start model=(.+?) session=(.+)$/;
const AGENT_FINISH =
  /^agent: finished subtype=(\S+) duration_ms=(\d+) is_error=(true|false)$/;

export function toolKindLabel(kind: string): string {
  switch (kind) {
    case "read":
      return "读取";
    case "write":
      return "写入";
    case "shell":
      return "命令";
    default:
      return kind;
  }
}

export function toolKindIcon(kind: string): string {
  switch (kind) {
    case "read":
      return "doc.text";
    case "write":
      return "square.and.pencil";
    case "shell":
      return "terminal";
    default:
      return "wrench.and.screwdriver";
  }
}

export function detailKey(detail: string): string {
  const pathMatch = detail.match(/^path=(\S+)/);
  if (pathMatch) return `path:${pathMatch[1]}`;
  const cmdMatch = detail.match(/^cmd=(.+)/);
  if (cmdMatch) return `cmd:${cmdMatch[1]}`;
  return detail;
}

export function toolTitle(item: ToolTimelineItem): string {
  const pathMatch = item.detail.match(/^path=(\S+)/);
  if (pathMatch) return pathMatch[1];
  const cmdMatch = item.detail.match(/^cmd=(.+)/);
  if (cmdMatch) {
    const cmd = cmdMatch[1];
    return cmd.length > 48 ? `${cmd.slice(0, 45)}…` : cmd;
  }
  return item.detail;
}

export function buildAgentTimeline(logs: RunLogEntry[]): AgentTimeline {
  const session: AgentSessionMeta = {};
  const tools: ToolTimelineItem[] = [];
  const assistantLines: { ts: number; text: string }[] = [];
  let hasStructuredAgentOutput = false;
  let toolSeq = 0;

  for (const entry of logs) {
    const message = entry.message.trim();
    if (!message) continue;

    const startMatch = message.match(TOOL_START);
    if (startMatch) {
      hasStructuredAgentOutput = true;
      toolSeq += 1;
      tools.push({
        id: `tool-${toolSeq}`,
        kind: startMatch[1],
        detail: startMatch[2],
        status: "running",
        startedTs: entry.ts,
        startedMessage: message,
      });
      continue;
    }

    const doneMatch = message.match(TOOL_DONE);
    if (doneMatch) {
      hasStructuredAgentOutput = true;
      const kind = doneMatch[1];
      const detail = doneMatch[2];
      const key = detailKey(detail);
      for (let i = tools.length - 1; i >= 0; i -= 1) {
        const candidate = tools[i];
        if (
          candidate.status === "running" &&
          candidate.kind === kind &&
          detailKey(candidate.detail) === key
        ) {
          candidate.status = "done";
          candidate.completedTs = entry.ts;
          candidate.completedMessage = message;
          if (detail.length > candidate.detail.length) {
            candidate.resultDetail = detail.slice(candidate.detail.length).trim();
          }
          break;
        }
      }
      continue;
    }

    const agentStart = message.match(AGENT_START);
    if (agentStart) {
      hasStructuredAgentOutput = true;
      session.startedTs = entry.ts;
      session.model = agentStart[1];
      session.sessionId = agentStart[2];
      continue;
    }

    const agentFinish = message.match(AGENT_FINISH);
    if (agentFinish) {
      hasStructuredAgentOutput = true;
      session.finishedTs = entry.ts;
      session.durationMs = Number(agentFinish[2]);
      session.isError = agentFinish[3] === "true";
      continue;
    }

    if (message.startsWith("›")) {
      assistantLines.push({
        ts: entry.ts,
        text: message.replace(/^›\s*/, ""),
      });
    }
  }

  return { session, tools, assistantLines, hasStructuredAgentOutput };
}
