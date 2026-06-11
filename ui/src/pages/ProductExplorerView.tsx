import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Background,
  Controls,
  ReactFlow,
  type Edge,
  type Node,
  MarkerType,
} from "@xyflow/react";
import {
  intentGraphMermaid,
  listProductNames,
  scanIntentGraph,
  scanProductTaskGraph,
  type IntentBlockNode,
  type TaskNode,
} from "../hooks/useTauri";
import { IntentDetailPanel } from "../components/IntentDetailPanel";
import { MermaidRenderer } from "../components/MermaidRenderer";
import { TaskDetailPanel } from "../components/TaskDetailPanel";
import type { Page } from "../App";

const JOURNEY_ORDER = [
  "onboarding",
  "daily-ops",
  "troubleshooting",
  "admin",
  "lifecycle",
];

type Tab = "tasks" | "intents" | "graph";

interface Props {
  setPage: (p: Page) => void;
  product?: string;
  tab?: Tab;
  taskId?: string;
  intentFile?: string;
  intentBlock?: string;
}

function groupByJourney(nodes: TaskNode[]): Map<string, TaskNode[]> {
  const map = new Map<string, TaskNode[]>();
  for (const n of nodes) {
    const stage = n.journey_stage || "other";
    map.set(stage, [...(map.get(stage) ?? []), n]);
  }
  return map;
}

function groupIntentsByFile(blocks: IntentBlockNode[]): Map<string, IntentBlockNode[]> {
  const map = new Map<string, IntentBlockNode[]>();
  for (const b of blocks) {
    map.set(b.file, [...(map.get(b.file) ?? []), b]);
  }
  return map;
}

function layoutProductTasks(nodes: TaskNode[]): { nodes: Node[]; edges: Edge[] } {
  const byStage = groupByJourney(nodes);
  const stages = [
    ...JOURNEY_ORDER.filter((s) => byStage.has(s)),
    ...[...byStage.keys()].filter((s) => !JOURNEY_ORDER.includes(s)),
  ];
  const flowNodes: Node[] = [];
  stages.forEach((stage, si) => {
    (byStage.get(stage) ?? []).forEach((task, i) => {
      flowNodes.push({
        id: task.task_id,
        position: { x: si * 220, y: i * 90 },
        data: { label: `${task.task_id}\n${task.title}`, task },
        style: {
          width: 180,
          padding: 8,
          fontSize: 10,
          borderRadius: 8,
          border: "2px solid var(--accent)",
          background: "var(--bg-secondary)",
          color: "var(--text-primary)",
          whiteSpace: "pre-wrap",
        },
      });
    });
  });
  const edges: Edge[] = [];
  for (const task of nodes) {
    for (const next of task.related_next_tasks) {
      edges.push({
        id: `${task.task_id}->${next}`,
        source: task.task_id,
        target: next,
        markerEnd: { type: MarkerType.ArrowClosed },
      });
    }
  }
  return { nodes: flowNodes, edges };
}

export function ProductExplorerView({
  setPage,
  product: initialProduct,
  tab: initialTab = "tasks",
  taskId: initialTaskId,
  intentFile: initialIntentFile,
  intentBlock: initialIntentBlock,
}: Props) {
  const [products, setProducts] = useState<string[]>([]);
  const [product, setProduct] = useState(initialProduct ?? "");
  const [tab, setTab] = useState<Tab>(initialTab);
  const [tasks, setTasks] = useState<TaskNode[]>([]);
  const [intentBlocks, setIntentBlocks] = useState<IntentBlockNode[]>([]);
  const [mermaid, setMermaid] = useState<string | null>(null);
  const [selectedTaskId, setSelectedTaskId] = useState(initialTaskId ?? "");
  const [selectedIntentFile, setSelectedIntentFile] = useState(
    initialIntentFile ?? ""
  );
  const [selectedIntentBlock, setSelectedIntentBlock] = useState(
    initialIntentBlock ?? ""
  );
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    listProductNames()
      .then((names) => {
        setProducts(names);
        if (!product && names.length > 0) {
          setProduct(initialProduct ?? names[0]);
        }
      })
      .catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    if (initialProduct) setProduct(initialProduct);
    if (initialTab) setTab(initialTab);
    if (initialTaskId) setSelectedTaskId(initialTaskId);
    if (initialIntentFile) setSelectedIntentFile(initialIntentFile);
    if (initialIntentBlock) setSelectedIntentBlock(initialIntentBlock);
  }, [initialProduct, initialTab, initialTaskId, initialIntentFile, initialIntentBlock]);

  const load = useCallback(() => {
    if (!product) return;
    setError(null);
    Promise.all([
      scanProductTaskGraph(product),
      scanIntentGraph(product),
      intentGraphMermaid(product).catch(() => null),
    ])
      .then(([tg, ig, mm]) => {
        setTasks(tg.nodes);
        setIntentBlocks(ig.blocks);
        setMermaid(mm);
      })
      .catch((e) => setError(String(e)));
  }, [product]);

  useEffect(() => {
    load();
  }, [load]);

  const journeyGroups = useMemo(() => groupByJourney(tasks), [tasks]);
  const intentFileGroups = useMemo(
    () => groupIntentsByFile(intentBlocks),
    [intentBlocks]
  );
  const taskFlow = useMemo(() => layoutProductTasks(tasks), [tasks]);

  const sortedStages = [
    ...JOURNEY_ORDER.filter((s) => journeyGroups.has(s)),
    ...[...journeyGroups.keys()].filter((s) => !JOURNEY_ORDER.includes(s)),
  ];

  if (error) {
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col gap-4">
      <div className="flex items-center justify-between gap-4 flex-wrap">
        <h2 className="text-2xl font-bold">Products</h2>
        <select
          value={product}
          onChange={(e) => {
            setProduct(e.target.value);
            setSelectedTaskId("");
            setSelectedIntentFile("");
            setSelectedIntentBlock("");
          }}
          className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm"
        >
          {products.map((p) => (
            <option key={p} value={p}>
              {p}
            </option>
          ))}
        </select>
      </div>

      <div className="flex gap-1 p-1 bg-[var(--bg-secondary)] rounded-lg w-fit border border-[var(--border)]">
        {(["tasks", "intents", "graph"] as Tab[]).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`px-4 py-1.5 rounded-md text-sm capitalize ${
              tab === t
                ? "bg-[var(--accent)]/20 text-[var(--accent)]"
                : "text-[var(--text-secondary)]"
            }`}
          >
            {t}
          </button>
        ))}
      </div>

      {tab === "tasks" && (
        <div className="flex gap-4 flex-1 min-h-0">
          <div className="w-72 shrink-0 overflow-auto border border-[var(--border)] rounded-xl bg-[var(--bg-secondary)] p-2 space-y-3">
            {sortedStages.map((stage) => (
              <div key={stage}>
                <p className="text-xs font-medium text-[var(--text-secondary)] uppercase px-2 mb-1">
                  {stage}
                </p>
                {(journeyGroups.get(stage) ?? []).map((t) => (
                  <button
                    key={t.task_id}
                    onClick={() => setSelectedTaskId(t.task_id)}
                    className={`w-full text-left px-2 py-2 rounded-lg text-sm mb-1 ${
                      selectedTaskId === t.task_id
                        ? "bg-[var(--accent)]/15 text-[var(--accent)]"
                        : "hover:bg-[var(--bg-tertiary)]"
                    }`}
                  >
                    <span className="font-mono text-xs block">{t.task_id}</span>
                    <span className="line-clamp-2">{t.title}</span>
                  </button>
                ))}
              </div>
            ))}
          </div>
          <div className="flex-1 min-w-0 overflow-auto">
            {selectedTaskId ? (
              <TaskDetailPanel
                product={product}
                taskId={selectedTaskId}
                setPage={setPage}
              />
            ) : (
              <p className="text-[var(--text-secondary)] text-sm">
                选择左侧 task 查看完整内容
              </p>
            )}
          </div>
        </div>
      )}

      {tab === "intents" && (
        <div className="flex gap-4 flex-1 min-h-0">
          <div className="w-72 shrink-0 overflow-auto border border-[var(--border)] rounded-xl bg-[var(--bg-secondary)] p-2 space-y-3">
            {[...intentFileGroups.keys()].sort().map((file) => (
              <div key={file}>
                <p className="text-xs font-medium text-[var(--accent)] font-mono px-2 mb-1">
                  {file}
                </p>
                {(intentFileGroups.get(file) ?? []).map((b) => (
                  <button
                    key={`${file}-${b.name}`}
                    onClick={() => {
                      setSelectedIntentFile(file);
                      setSelectedIntentBlock(b.name);
                    }}
                    className={`w-full text-left px-2 py-1.5 rounded text-xs mb-1 ${
                      selectedIntentFile === file &&
                      selectedIntentBlock === b.name
                        ? "bg-[var(--accent)]/15 text-[var(--accent)]"
                        : "hover:bg-[var(--bg-tertiary)]"
                    }`}
                  >
                    <span className="text-[var(--text-secondary)]">{b.kind}</span>{" "}
                    {b.name}
                  </button>
                ))}
                <button
                  onClick={() => {
                    setSelectedIntentFile(file);
                    setSelectedIntentBlock("");
                  }}
                  className="text-xs text-[var(--text-secondary)] px-2 py-1 hover:underline"
                >
                  View full file
                </button>
              </div>
            ))}
          </div>
          <div className="flex-1 min-w-0 overflow-auto">
            {selectedIntentFile ? (
              <IntentDetailPanel
                product={product}
                file={selectedIntentFile}
                block={selectedIntentBlock || undefined}
                setPage={setPage}
              />
            ) : (
              <p className="text-[var(--text-secondary)] text-sm">
                选择 intent 块或文件查看内容
              </p>
            )}
          </div>
        </div>
      )}

      {tab === "graph" && (
        <div className="space-y-4 flex-1 min-h-0">
          <div className="h-64 border border-[var(--border)] rounded-xl overflow-hidden bg-[var(--bg-secondary)]">
            <ReactFlow
              nodes={taskFlow.nodes}
              edges={taskFlow.edges}
              onNodeClick={(_, n) => {
                setTab("tasks");
                setSelectedTaskId(n.id);
              }}
              fitView
              proOptions={{ hideAttribution: true }}
            >
              <Background color="var(--border)" gap={16} />
              <Controls />
            </ReactFlow>
          </div>
          {mermaid && (
            <div className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-xl p-4 overflow-auto">
              <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-2">
                Intent diagram
              </h3>
              <MermaidRenderer chart={mermaid} />
            </div>
          )}
        </div>
      )}
    </div>
  );
}
