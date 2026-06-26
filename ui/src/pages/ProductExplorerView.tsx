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
  getProductHealth,
  listProductNames,
  scanIntentGraph,
  scanProductTaskGraph,
  useRefresh,
  type IntentBlockNode,
  type ProductHealthReport,
  type TaskNode,
} from "../hooks/useTauri";
import { IntentDetailPanel } from "../components/IntentDetailPanel";
import { IntentVisualizationPanel } from "../components/IntentVisualizationPanel";
import { ProductHealthPanel } from "../components/ProductHealthPanel";
import { TaskDetailPanel } from "../components/TaskDetailPanel";
import type { Page } from "../App";

const JOURNEY_ORDER = [
  "onboarding",
  "daily-ops",
  "troubleshooting",
  "admin",
  "lifecycle",
];

const JOURNEY_LABELS: Record<string, string> = {
  onboarding: "Onboarding",
  "daily-ops": "Daily ops",
  troubleshooting: "Troubleshooting",
  admin: "Admin",
  lifecycle: "Lifecycle",
};

type Tab = "tasks" | "intents" | "graph";
type GraphSubTab = "tasks" | "intents";

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
  const [graphSubTab, setGraphSubTab] = useState<GraphSubTab>("intents");
  const [tasks, setTasks] = useState<TaskNode[]>([]);
  const [intentGraph, setIntentGraph] = useState<
    Awaited<ReturnType<typeof scanIntentGraph>> | null
  >(null);
  const [selectedTaskId, setSelectedTaskId] = useState(initialTaskId ?? "");
  const [selectedIntentFile, setSelectedIntentFile] = useState(
    initialIntentFile ?? ""
  );
  const [selectedIntentBlock, setSelectedIntentBlock] = useState(
    initialIntentBlock ?? ""
  );
  const [error, setError] = useState<string | null>(null);
  const [health, setHealth] = useState<ProductHealthReport | null>(null);

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
      getProductHealth(product).catch(() => null),
    ])
      .then(([tg, ig, h]) => {
        setTasks(tg.nodes);
        setIntentGraph(ig);
        setHealth(h);
      })
      .catch((e) => setError(String(e)));
  }, [product]);

  useEffect(() => {
    load();
  }, [load]);

  useRefresh(load);

  const intentBlocks = intentGraph?.blocks ?? [];
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

  const handleSelectIntentBlock = useCallback((file: string, block: string) => {
    setTab("intents");
    setSelectedIntentFile(file);
    setSelectedIntentBlock(block);
  }, []);

  const tabHint =
    tab === "tasks"
      ? "按旅程阶段浏览 task 文档"
      : tab === "intents"
        ? "按 intent 文件浏览块，点击查看源码"
        : "任务依赖图与 intent-lang 形式化关系图";

  if (error) {
    return (
      <div className="card border-[rgba(239,68,68,0.25)] bg-[rgba(239,68,68,0.08)] p-4 text-[13px] text-[var(--accent-red)]">
        {error}
      </div>
    );
  }

  return (
    <div className="page-frame flex h-full min-h-0 flex-col gap-3">
      <div className="product-explorer-header shrink-0 space-y-3">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h2 className="text-[15px] font-semibold tracking-tight">Products</h2>
            <p className="mt-0.5 text-[12px] text-[var(--text-muted)]">{tabHint}</p>
          </div>
          <div className="flex items-center gap-3 text-[12px] text-[var(--text-muted)]">
            <span>{tasks.length} tasks</span>
            <span>{intentBlocks.length} intent blocks</span>
            {intentGraph?.diagrams.length ? (
              <span>{intentGraph.diagrams.length} 图</span>
            ) : null}
          </div>
        </div>
        {health && <ProductHealthPanel health={health} />}
        <div className="flex flex-wrap gap-2" role="tablist" aria-label="Products">
          {products.map((p) => (
            <button
              key={p}
              type="button"
              role="tab"
              aria-selected={product === p}
              onClick={() => {
                setProduct(p);
                setSelectedTaskId("");
                setSelectedIntentFile("");
                setSelectedIntentBlock("");
              }}
              className={`product-pill ${product === p ? "product-pill-active" : ""}`}
            >
              {p}
            </button>
          ))}
        </div>
        <div className="tab-group w-fit">
          {(["tasks", "intents", "graph"] as Tab[]).map((t) => (
            <button
              key={t}
              type="button"
              onClick={() => setTab(t)}
              className={`tab-btn capitalize ${tab === t ? "tab-btn-active" : ""}`}
            >
              {t === "graph" ? "关系图" : t}
            </button>
          ))}
        </div>
      </div>

      {tab === "tasks" && (
        <div className="explorer-split min-h-0 flex-1">
          <div className="master-panel">
            <div className="master-panel-scroll space-y-3 p-2">
              {sortedStages.map((stage) => (
                <div key={stage} className="product-journey-card">
                  <p className="section-label">
                    {JOURNEY_LABELS[stage] ?? stage}
                  </p>
                  {(journeyGroups.get(stage) ?? []).map((t) => (
                    <button
                      key={t.task_id}
                      type="button"
                      onClick={() => setSelectedTaskId(t.task_id)}
                      className={`product-task-row ${
                        selectedTaskId === t.task_id ? "product-task-row-active" : ""
                      }`}
                    >
                      <span className="font-mono text-[11px] text-[#93c5fd]">
                        {t.task_id}
                      </span>
                      <span className="mt-0.5 line-clamp-2 block text-[var(--text-primary)]">
                        {t.title}
                      </span>
                    </button>
                  ))}
                </div>
              ))}
              {tasks.length === 0 && (
                <p className="empty-state">No tasks in this product.</p>
              )}
            </div>
          </div>
          <div className="detail-panel">
            <div className="detail-panel-scroll">
              {selectedTaskId ? (
                <TaskDetailPanel
                  product={product}
                  taskId={selectedTaskId}
                  setPage={setPage}
                />
              ) : (
                <div className="flex h-full flex-col items-center justify-center gap-2 p-8 text-center">
                  <p className="text-[13px] font-medium text-[var(--text-secondary)]">
                    选择 task 查看详情
                  </p>
                  <p className="max-w-xs text-[12px] text-[var(--text-muted)]">
                    左侧按旅程阶段分组；点击行在右侧预览 Markdown 与关联 intent。
                  </p>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {tab === "intents" && (
        <div className="explorer-split min-h-0 flex-1">
          <div className="master-panel">
            <div className="master-panel-scroll space-y-3 p-2">
              {[...intentFileGroups.keys()].sort().map((file) => (
                <div key={file} className="product-journey-card">
                  <p className="section-label font-mono text-[#93c5fd]">{file}</p>
                  {(intentFileGroups.get(file) ?? []).map((b) => (
                    <button
                      key={`${file}-${b.name}`}
                      onClick={() => {
                        setSelectedIntentFile(file);
                        setSelectedIntentBlock(b.name);
                      }}
                      type="button"
                      className={`product-task-row ${
                        selectedIntentFile === file &&
                        selectedIntentBlock === b.name
                          ? "product-task-row-active"
                          : ""
                      }`}
                    >
                      <span className="text-[11px] text-[var(--text-muted)]">
                        {b.kind}
                      </span>
                      <span className="block text-[var(--text-primary)]">{b.name}</span>
                    </button>
                  ))}
                  <button
                    type="button"
                    onClick={() => {
                      setSelectedIntentFile(file);
                      setSelectedIntentBlock("");
                    }}
                    className="w-full px-3 py-2 text-left text-[11px] text-[var(--text-muted)] transition-colors hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]"
                  >
                    查看完整文件
                  </button>
                </div>
              ))}
              {intentBlocks.length === 0 && (
                <p className="empty-state">No intent blocks found.</p>
              )}
            </div>
          </div>
          <div className="detail-panel">
            <div className="detail-panel-scroll">
              {selectedIntentFile ? (
                <IntentDetailPanel
                  product={product}
                  file={selectedIntentFile}
                  block={selectedIntentBlock || undefined}
                  setPage={setPage}
                  onOpenGraph={() => {
                    setTab("graph");
                    setGraphSubTab("intents");
                  }}
                />
              ) : (
                <div className="flex h-full flex-col items-center justify-center gap-2 p-8 text-center">
                  <p className="text-[13px] font-medium text-[var(--text-secondary)]">
                    选择 intent 块或文件
                  </p>
                  <button
                    type="button"
                    className="btn btn-ghost mt-2 text-[12px]"
                    onClick={() => {
                      setTab("graph");
                      setGraphSubTab("intents");
                    }}
                  >
                    打开关系图
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {tab === "graph" && (
        <div className="flex min-h-0 flex-1 flex-col gap-3">
          <div className="tab-group w-fit">
            <button
              type="button"
              onClick={() => setGraphSubTab("intents")}
              className={`tab-btn ${graphSubTab === "intents" ? "tab-btn-active" : ""}`}
            >
              Intent 关系
            </button>
            <button
              type="button"
              onClick={() => setGraphSubTab("tasks")}
              className={`tab-btn ${graphSubTab === "tasks" ? "tab-btn-active" : ""}`}
            >
              任务依赖
            </button>
          </div>

          {graphSubTab === "tasks" ? (
            <div className="graph-panel min-h-[320px] flex-1">
              <ReactFlow
                nodes={taskFlow.nodes}
                edges={taskFlow.edges}
                onNodeClick={(_, n) => {
                  setTab("tasks");
                  setSelectedTaskId(n.id);
                }}
                fitView
                fitViewOptions={{ padding: 0.2 }}
                proOptions={{ hideAttribution: true }}
              >
                <Background color="var(--border)" gap={16} />
                <Controls
                  showInteractive={false}
                  position="bottom-right"
                  className="graph-flow-controls"
                />
              </ReactFlow>
            </div>
          ) : (
            <IntentVisualizationPanel
              diagrams={intentGraph?.diagrams ?? []}
              blocks={intentBlocks}
              source={intentGraph?.source ?? "parsed"}
              parseError={intentGraph?.parse_error ?? null}
              onRefresh={load}
              onSelectBlock={handleSelectIntentBlock}
            />
          )}
        </div>
      )}
    </div>
  );
}
