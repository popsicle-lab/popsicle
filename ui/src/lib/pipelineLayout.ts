import type { Edge, Node } from "@xyflow/react";
import { MarkerType } from "@xyflow/react";

export interface StageLayoutInput {
  name: string;
  depends_on: string[];
  label?: string;
  highlight?: boolean;
}

export function layoutStageDag(
  stages: StageLayoutInput[],
  options?: { xGap?: number; yGap?: number; nodeWidth?: number }
): { nodes: Node[]; edges: Edge[] } {
  const xGap = options?.xGap ?? 240;
  const yGap = options?.yGap ?? 100;
  const nodeWidth = options?.nodeWidth ?? 180;

  const depth = new Map<string, number>();
  const visiting = new Set<string>();

  const resolveDepth = (name: string): number => {
    if (depth.has(name)) return depth.get(name)!;
    if (visiting.has(name)) return 0;
    visiting.add(name);
    const stage = stages.find((s) => s.name === name);
    const deps = stage?.depends_on ?? [];
    const d =
      deps.length === 0
        ? 0
        : Math.max(...deps.map((dep) => resolveDepth(dep))) + 1;
    visiting.delete(name);
    depth.set(name, d);
    return d;
  };

  for (const s of stages) resolveDepth(s.name);

  const byDepth = new Map<number, StageLayoutInput[]>();
  for (const s of stages) {
    const d = depth.get(s.name) ?? 0;
    byDepth.set(d, [...(byDepth.get(d) ?? []), s]);
  }

  const nodes: Node[] = [];
  for (const [d, group] of byDepth) {
    group.forEach((stage, i) => {
      const border = stage.highlight
        ? "2px solid #3b82f6"
        : "2px solid var(--border)";
      nodes.push({
        id: stage.name,
        position: { x: d * xGap, y: i * yGap },
        data: { label: stage.label ?? stage.name, stageName: stage.name },
        style: {
          width: nodeWidth,
          padding: 10,
          borderRadius: 8,
          border,
          background: stage.highlight
            ? "rgba(59,130,246,0.12)"
            : "var(--bg-elevated)",
          color: "var(--text-primary)",
          fontSize: 11,
          whiteSpace: "pre-wrap",
          boxShadow: stage.highlight ? "0 0 0 1px rgba(59,130,246,0.35)" : undefined,
        },
      });
    });
  }

  const edges: Edge[] = [];
  for (const stage of stages) {
    for (const dep of stage.depends_on) {
      edges.push({
        id: `${dep}->${stage.name}`,
        source: dep,
        target: stage.name,
        markerEnd: { type: MarkerType.ArrowClosed },
        style: { stroke: "var(--text-muted)" },
      });
    }
  }

  return { nodes, edges };
}
