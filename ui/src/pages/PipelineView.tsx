import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Background,
  Controls,
  MiniMap,
  ReactFlow,
  type Edge,
  type Node,
  MarkerType,
} from "@xyflow/react";
import { FileText, ShieldAlert } from "lucide-react";
import {
  completeStage,
  getPipelineStatus,
  type PipelineStatusFull,
  type StageStatusInfo,
} from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import type { Page } from "../App";

interface Props {
  runId: string;
  setPage: (p: Page) => void;
}

const stateBorder: Record<string, string> = {
  completed: "#4ade80",
  in_progress: "#a78bfa",
  ready: "#38bdf8",
  blocked: "#475569",
  error: "#f87171",
};

function layoutStages(stages: StageStatusInfo[]): { nodes: Node[]; edges: Edge[] } {
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

  const byDepth = new Map<number, StageStatusInfo[]>();
  for (const s of stages) {
    const d = depth.get(s.name) ?? 0;
    byDepth.set(d, [...(byDepth.get(d) ?? []), s]);
  }

  const nodes: Node[] = [];
  for (const [d, group] of byDepth) {
    group.forEach((stage, i) => {
      nodes.push({
        id: stage.name,
        position: { x: d * 260, y: i * 120 },
        data: {
          label: `${stage.name}\n${stage.state}`,
          stageName: stage.name,
        },
        style: {
          width: 200,
          padding: 10,
          borderRadius: 8,
          border: `2px solid ${stateBorder[stage.state] ?? stateBorder.blocked}`,
          background: "var(--bg-secondary)",
          color: "var(--text-primary)",
          fontSize: 11,
          whiteSpace: "pre-wrap",
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
        style: { stroke: "var(--text-secondary)" },
      });
    }
  }

  return { nodes, edges };
}

export function PipelineView({ runId, setPage }: Props) {
  const [status, setStatus] = useState<PipelineStatusFull | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [confirming, setConfirming] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(() => {
    getPipelineStatus(runId)
      .then((s) => {
        setStatus(s);
        if (!selected && s.current_stage) setSelected(s.current_stage);
      })
      .catch((e) => setError(String(e)));
  }, [runId]);

  useEffect(() => {
    load();
  }, [runId]);

  const flow = useMemo(
    () => (status ? layoutStages(status.stages) : { nodes: [], edges: [] }),
    [status]
  );

  const stage = status?.stages.find((s) => s.name === selected) ?? null;

  const doComplete = async (confirm: boolean) => {
    if (!stage) return;
    setBusy(true);
    try {
      await completeStage(runId, stage.name, confirm);
      setConfirming(false);
      load();
    } catch (e: unknown) {
      alert(String(e));
    } finally {
      setBusy(false);
    }
  };

  if (error) {
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );
  }
  if (!status) {
    return <div className="text-[var(--text-secondary)]">Loading…</div>;
  }

  return (
    <div className="space-y-4 h-full flex flex-col">
      <div>
        <h2 className="text-2xl font-bold">{status.pipeline_name}</h2>
        <p className="text-sm text-[var(--text-secondary)]">
          {status.issue_key} · {status.run_status} · current:{" "}
          {status.current_stage || "—"}
        </p>
      </div>

      <div className="h-[360px] bg-[var(--bg-secondary)] border border-[var(--border)] rounded-xl overflow-hidden">
        <ReactFlow
          nodes={flow.nodes}
          edges={flow.edges}
          onNodeClick={(_, n) => setSelected(n.id)}
          fitView
          proOptions={{ hideAttribution: true }}
        >
          <Background color="var(--border)" gap={16} />
          <Controls />
          <MiniMap />
        </ReactFlow>
      </div>

      {stage && (
        <div className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-xl p-4">
          <div className="flex items-center justify-between mb-2">
            <h3 className="font-medium">{stage.name}</h3>
            <StatusBadge status={stage.state} />
          </div>
          <p className="text-xs text-[var(--text-secondary)] mb-3">
            {stage.description || stage.skills.join(", ")}
          </p>
          <div className="space-y-1 mb-3">
            {stage.documents.map((doc) => (
              <button
                key={doc.id}
                onClick={() => setPage({ kind: "document", docId: doc.id })}
                className="w-full text-left text-xs px-2 py-1 rounded hover:bg-[var(--bg-tertiary)] flex items-center gap-2"
              >
                <FileText size={12} />
                {doc.title}
                <StatusBadge status={doc.status} />
              </button>
            ))}
          </div>
          {(stage.state === "ready" || stage.state === "in_progress") && (
            <div>
              {!confirming ? (
                <button
                  onClick={() =>
                    stage.requires_approval
                      ? setConfirming(true)
                      : doComplete(false)
                  }
                  disabled={busy}
                  className="text-sm px-3 py-1.5 rounded bg-[var(--accent)]/20 text-[var(--accent)]"
                >
                  Complete stage
                </button>
              ) : (
                <div className="flex items-center gap-3">
                  <span className="text-xs text-yellow-300 flex items-center gap-1">
                    <ShieldAlert size={14} /> Requires approval
                  </span>
                  <button
                    onClick={() => doComplete(true)}
                    disabled={busy}
                    className="text-sm px-3 py-1.5 rounded bg-yellow-500/20 text-yellow-200"
                  >
                    Confirm complete
                  </button>
                  <button
                    onClick={() => setConfirming(false)}
                    className="text-xs text-[var(--text-secondary)]"
                  >
                    Cancel
                  </button>
                </div>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
