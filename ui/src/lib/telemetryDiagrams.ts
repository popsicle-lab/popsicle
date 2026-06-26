import type { TelemetrySpan } from "../hooks/useTauri";

const AGENT_SPANS = new Set([
  "gen_ai.chat",
  "popsicle.run.score",
  "popsicle.decision",
]);

export function isAgentSpan(name: string): boolean {
  return AGENT_SPANS.has(name) || name.startsWith("gen_ai.");
}

export function spanKindLabel(span: string): string {
  switch (span) {
    case "popsicle.run.start":
      return "Run 启动";
    case "popsicle.stage.complete":
      return "Stage 完成";
    case "popsicle.doc.check":
      return "Doc check";
    case "gen_ai.chat":
      return "Agent LLM";
    case "popsicle.run.score":
      return "Stage 自评";
    case "popsicle.decision":
      return "关键决策";
    default:
      return span;
  }
}

export function formatDurationMs(ms: number | null | undefined): string {
  if (ms == null || Number.isNaN(ms)) return "—";
  if (ms < 1000) return `${ms} ms`;
  if (ms < 60_000) return `${(ms / 1000).toFixed(1)} s`;
  const m = Math.floor(ms / 60_000);
  const s = Math.round((ms % 60_000) / 1000);
  return `${m}m ${s}s`;
}

function escapeMermaid(text: string): string {
  return text.replace(/"/g, "'").replace(/[<>]/g, "").replace(/\n/g, " ");
}

function spanAttributes(span: TelemetrySpan): Record<string, string> {
  const out: Record<string, string> = {};
  for (const [k, v] of Object.entries(span)) {
    if (k !== "ts" && k !== "span") out[k] = v;
  }
  return out;
}

function spanCaption(span: TelemetrySpan): string {
  const attrs = spanAttributes(span);
  const stage = attrs["popsicle.stage"];
  const skill = attrs["popsicle.skill"];
  const doc = attrs["popsicle.doc_id"] ?? attrs.doc;
  const parts = [spanKindLabel(span.span)];
  if (stage) parts.push(stage);
  if (skill) parts.push(skill);
  if (doc) parts.push(doc.slice(0, 12));
  const dur = attrs["popsicle.duration_ms"];
  if (dur) parts.push(formatDurationMs(Number(dur)));
  return parts.join(" · ");
}

export function buildSpanSequenceMermaid(spans: TelemetrySpan[]): string | null {
  if (spans.length === 0) return null;
  const lines = [
    "sequenceDiagram",
    "  autonumber",
    "  participant P as Popsicle",
    "  participant A as Agent",
  ];
  for (const span of spans) {
    const label = escapeMermaid(spanCaption(span));
    if (isAgentSpan(span.span)) {
      lines.push(`  A->>A: ${label}`);
    } else {
      lines.push(`  P->>P: ${label}`);
    }
  }
  return lines.join("\n");
}

export function buildCoverageFlowMermaid(coverage: {
  gen_ai_chat: boolean;
  run_score: boolean;
  decision: boolean;
}): string {
  const mark = (ok: boolean) => (ok ? "✓" : "✗");
  return [
    "flowchart LR",
    `  genai["gen_ai.chat ${mark(coverage.gen_ai_chat)}"]`,
    `  score["run.score ${mark(coverage.run_score)}"]`,
    `  dec["decision ${mark(coverage.decision)}"]`,
    coverage.gen_ai_chat
      ? "  class genai ok"
      : "  class genai miss",
    coverage.run_score ? "  class score ok" : "  class score miss",
    coverage.decision ? "  class dec ok" : "  class dec miss",
    "  classDef ok fill:#14532d,stroke:#22c55e,color:#ecfdf5",
    "  classDef miss fill:#450a0a,stroke:#ef4444,color:#fef2f2",
  ].join("\n");
}

export function pickDisplayAttributes(
  span: TelemetrySpan
): [string, string][] {
  const attrs: Record<string, string> = {};
  for (const [k, v] of Object.entries(span)) {
    if (k !== "ts" && k !== "span") attrs[k] = v;
  }
  const priority = [
    "popsicle.stage",
    "popsicle.skill",
    "popsicle.doc_id",
    "popsicle.duration_ms",
    "popsicle.doc_check.passed",
    "model",
    "input_tokens",
    "output_tokens",
    "score",
    "rubric",
    "summary",
  ];
  const entries = Object.entries(attrs).filter(
    ([k]) => !k.startsWith("popsicle.trace_id")
  );
  entries.sort((a, b) => {
    const ai = priority.indexOf(a[0]);
    const bi = priority.indexOf(b[0]);
    const ar = ai === -1 ? 999 : ai;
    const br = bi === -1 ? 999 : bi;
    return ar - br || a[0].localeCompare(b[0]);
  });
  return entries.slice(0, 8);
}
