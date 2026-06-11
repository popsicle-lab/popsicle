import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Background,
  Controls,
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
import { LoadingState } from "../components/LoadingState";
import { StatusBadge } from "../components/StatusBadge";
import type { Page } from "../App";

interface Props {
  runId: string;
  setPage: (p: Page) => void;
}

const stateBorder: Record<string, string> = {
  completed: "#22c55e",
  in_progress: "#3b82f6",
  ready: "#60a5fa",
  blocked: "#52525b",
  error: "#ef4444",
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
        position: { x: d * 240, y: i * 100 },
        data: {
          label: `${stage.name}\n${stage.state}`,
          stageName: stage.name,
        },
        style: {
          width: 180,
          padding: 10,
          borderRadius: 8,
          border: `2px solid ${stateBorder[stage.state] ?? stateBorder.blocked}`,
          background: "var(--bg-elevated)",
          color: "var(--text-primary)",
          fontSize: 11,
          whiteSpace: "pre-wrap",
          transition: "box-shadow 0.15s ease",
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
        setSelected((prev) => prev ?? s.current_stage ?? null);
      })
      .catch((e) => setError(String(e)));
  }, [runId]);

  useEffect(() => {
    load();
  }, [load]);

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
      <div className="page-frame">
        <div className="card border-[rgba(239,68,68,0.25)] bg-[rgba(239,68,68,0.08)] p-4 text-[13px] text-[var(--accent-red)]">
          {error}
        </div>
      </div>
    );
  }
  if (!status) {
    return (
      <div className="page-frame">
        <LoadingState label="Loading pipeline…" />
      </div>
    );
  }

  return (
    <div className="page-frame flex h-full flex-col gap-3">
      <div className="shrink-0">
        <p className="text-[13px] font-semibold">{status.pipeline_name}</p>
        <p className="text-[12px] text-[var(--text-muted)]">
          {status.issue_key} · {status.run_status}
          {status.current_stage ? ` · ${status.current_stage}` : ""}
        </p>
      </div>

      <div className="pipeline-split min-h-0 flex-1">
        <div className="graph-panel">
          <ReactFlow
            nodes={flow.nodes}
            edges={flow.edges}
            onNodeClick={(_, n) => setSelected(n.id)}
            fitView
            proOptions={{ hideAttribution: true }}
          >
            <Background color="var(--border)" gap={20} />
            <Controls showInteractive={false} />
          </ReactFlow>
        </div>

        <aside className="detail-panel flex min-h-[240px] flex-col md:min-h-0">
          {stage ? (
            <div className="detail-panel-scroll space-y-3">
              <div className="flex items-center justify-between gap-2">
                <h3 className="text-[13px] font-semibold">{stage.name}</h3>
                <StatusBadge status={stage.state} />
              </div>
              <p className="text-[12px] leading-relaxed text-[var(--text-muted)]">
                {stage.description || stage.skills.join(", ")}
              </p>
              <div className="space-y-0.5">
                {stage.documents.map((doc) => (
                  <button
                    key={doc.id}
                    type="button"
                    onClick={() => setPage({ kind: "document", docId: doc.id })}
                    className="flex w-full items-center gap-2 rounded-[var(--radius-sm)] px-2 py-2 text-left text-[12px] transition-colors hover:bg-[var(--bg-hover)]"
                  >
                    <FileText size={13} className="shrink-0 text-[var(--text-muted)]" />
                    <span className="min-w-0 flex-1 truncate">{doc.title}</span>
                    <StatusBadge status={doc.status} />
                  </button>
                ))}
              </div>
              {(stage.state === "ready" || stage.state === "in_progress") && (
                <div className="border-t border-[var(--border)] pt-3">
                  {!confirming ? (
                    <button
                      type="button"
                      onClick={() =>
                        stage.requires_approval
                          ? setConfirming(true)
                          : doComplete(false)
                      }
                      disabled={busy}
                      className="btn btn-primary w-full"
                    >
                      Complete stage
                    </button>
                  ) : (
                    <div className="space-y-2">
                      <p className="flex items-center gap-1.5 text-[11px] text-[var(--accent-yellow)]">
                        <ShieldAlert size={13} /> Requires approval
                      </p>
                      <button
                        type="button"
                        onClick={() => doComplete(true)}
                        disabled={busy}
                        className="btn btn-primary w-full"
                      >
                        Confirm complete
                      </button>
                      <button
                        type="button"
                        onClick={() => setConfirming(false)}
                        className="btn btn-secondary w-full"
                      >
                        Cancel
                      </button>
                    </div>
                  )}
                </div>
              )}
            </div>
          ) : (
            <div className="flex flex-1 items-center justify-center p-6 text-center text-[12px] text-[var(--text-muted)]">
              Select a stage in the graph
            </div>
          )}
        </aside>
      </div>
    </div>
  );
}
