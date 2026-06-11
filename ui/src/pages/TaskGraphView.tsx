import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Background,
  Controls,
  ReactFlow,
  type Edge,
  type Node,
  MarkerType,
} from "@xyflow/react";
import { scanTaskGraph, type TaskNode } from "../hooks/useTauri";

const stageColors: Record<string, string> = {
  onboarding: "#38bdf8",
  "daily-ops": "#4ade80",
  troubleshooting: "#fbbf24",
  admin: "#a78bfa",
  lifecycle: "#f472b6",
};

function layoutTasks(nodes: TaskNode[]): { nodes: Node[]; edges: Edge[] } {
  const byStage = new Map<string, TaskNode[]>();
  for (const n of nodes) {
    const stage = n.journey_stage || "other";
    byStage.set(stage, [...(byStage.get(stage) ?? []), n]);
  }

  const flowNodes: Node[] = [];
  let stageIndex = 0;
  for (const [stage, group] of byStage) {
    group.forEach((task, i) => {
      flowNodes.push({
        id: task.task_id,
        position: { x: stageIndex * 240, y: i * 100 },
        data: { label: `${task.task_id}\n${task.title}`, task },
        style: {
          width: 200,
          padding: 8,
          borderRadius: 8,
          border: `2px solid ${stageColors[stage] ?? "#475569"}`,
          background: "var(--bg-secondary)",
          color: "var(--text-primary)",
          fontSize: 10,
          whiteSpace: "pre-wrap",
        },
      });
    });
    stageIndex += 1;
  }

  const edges: Edge[] = [];
  for (const task of nodes) {
    for (const next of task.related_next_tasks) {
      edges.push({
        id: `${task.task_id}->${next}`,
        source: task.task_id,
        target: next,
        markerEnd: { type: MarkerType.ArrowClosed },
        style: { stroke: "var(--accent)" },
      });
    }
  }

  return { nodes: flowNodes, edges };
}

export function TaskGraphView() {
  const [graph, setGraph] = useState<TaskNode[]>([]);
  const [selected, setSelected] = useState<TaskNode | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(() => {
    scanTaskGraph()
      .then((g) => setGraph(g.nodes))
      .catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const flow = useMemo(() => layoutTasks(graph), [graph]);

  if (error) {
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );
  }

  return (
    <div className="space-y-4 h-full flex flex-col">
      <div>
        <h2 className="text-2xl font-bold">Task Graph</h2>
        <p className="text-sm text-[var(--text-secondary)]">
          {graph.length} tasks from products/*/tasks
        </p>
      </div>
      <div className="flex-1 min-h-[400px] bg-[var(--bg-secondary)] border border-[var(--border)] rounded-xl overflow-hidden">
        <ReactFlow
          nodes={flow.nodes}
          edges={flow.edges}
          onNodeClick={(_, n) =>
            setSelected((n.data as { task: TaskNode }).task)
          }
          fitView
          proOptions={{ hideAttribution: true }}
        >
          <Background color="var(--border)" gap={16} />
          <Controls />
        </ReactFlow>
      </div>
      {selected && (
        <div className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-xl p-4 text-sm">
          <h3 className="font-medium">
            {selected.task_id}: {selected.title}
          </h3>
          <p className="text-[var(--text-secondary)] mt-1">
            {selected.product} · {selected.journey_stage}
          </p>
          {selected.related_intents.length > 0 && (
            <p className="mt-2">
              Intents: {selected.related_intents.join(", ")}
            </p>
          )}
          <p className="mt-1 font-mono text-xs text-[var(--text-secondary)]">
            {selected.file_path}
          </p>
        </div>
      )}
    </div>
  );
}
