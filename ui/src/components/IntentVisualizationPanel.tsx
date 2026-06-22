import { useEffect, useMemo, useState } from "react";
import { RefreshCw } from "lucide-react";
import type { IntentBlockNode, IntentDiagramView } from "../hooks/useTauri";
import { MermaidDiagramCanvas } from "./MermaidDiagramCanvas";

const LEGEND = [
  { kind: "goal", label: "Goal", color: "#e1f5ff", stroke: "#01579b" },
  { kind: "safety", label: "Safety", color: "#fff3e0", stroke: "#e65100" },
  { kind: "intent", label: "Intent", color: "#f3e5f5", stroke: "#4a148c" },
  { kind: "theorem", label: "Theorem", color: "#e8f5e9", stroke: "#1b5e20" },
  { kind: "axiom", label: "Axiom", color: "#fce4ec", stroke: "#880e4f" },
] as const;

const KIND_FOR_DIAGRAM: Record<string, string[]> = {
  "goal-graph": ["goal", "safety", "intent", "theorem", "axiom"],
  "intent-graph": ["intent"],
  "safety-network": ["safety"],
  "coverage-matrix": ["goal", "intent"],
};

interface Props {
  diagrams: IntentDiagramView[];
  blocks: IntentBlockNode[];
  source: string;
  parseError: string | null;
  onRefresh?: () => void;
  onSelectBlock: (file: string, block: string) => void;
}

export function IntentVisualizationPanel({
  diagrams,
  blocks,
  source,
  parseError,
  onRefresh,
  onSelectBlock,
}: Props) {
  const [activeId, setActiveId] = useState(diagrams[0]?.id ?? "goal-graph");
  const [blockFilter, setBlockFilter] = useState("");
  const [diagramEpoch, setDiagramEpoch] = useState(0);

  const handleRefresh = () => {
    setDiagramEpoch((n) => n + 1);
    onRefresh?.();
  };

  useEffect(() => {
    if (diagrams.length === 0) return;
    if (!diagrams.some((d) => d.id === activeId)) {
      setActiveId(diagrams[0].id);
    }
  }, [diagrams, activeId]);

  const active =
    diagrams.find((d) => d.id === activeId) ?? diagrams[0] ?? null;

  const filteredBlocks = useMemo(() => {
    const kinds = KIND_FOR_DIAGRAM[active?.id ?? ""] ?? [];
    const q = blockFilter.trim().toLowerCase();
    return blocks.filter((b) => {
      if (kinds.length > 0 && !kinds.includes(b.kind)) return false;
      if (!q) return true;
      return (
        b.name.toLowerCase().includes(q) ||
        b.file.toLowerCase().includes(q) ||
        b.kind.toLowerCase().includes(q)
      );
    });
  }, [blocks, active?.id, blockFilter]);

  if (diagrams.length === 0) {
    return (
      <div className="intent-viz-empty card flex flex-col items-center justify-center gap-2 p-8 text-center">
        <p className="text-[13px] font-medium text-[var(--text-secondary)]">
          无法生成 intent 关系图
        </p>
        {parseError && (
          <p className="max-w-md text-[12px] text-[var(--text-muted)]">{parseError}</p>
        )}
        {onRefresh && (
          <button type="button" onClick={handleRefresh} className="btn btn-ghost mt-2 gap-1.5">
            <RefreshCw size={14} /> 重新加载
          </button>
        )}
      </div>
    );
  }

  return (
    <div className="intent-viz-panel flex min-h-0 flex-1 flex-col">
      <div className="intent-viz-shell card min-h-0 flex-1 overflow-hidden">
        <div className="intent-viz-tabs" role="tablist" aria-label="Intent 图类型">
          {diagrams.map((d) => (
            <button
              key={d.id}
              type="button"
              role="tab"
              aria-selected={active?.id === d.id}
              onClick={() => setActiveId(d.id)}
              className={`intent-viz-tab ${active?.id === d.id ? "intent-viz-tab-active" : ""}`}
            >
              {d.label}
            </button>
          ))}
          <div className="intent-viz-tabs-spacer" />
          <span className="intent-viz-source-badge">
            {source === "visualizer" ? "intent-lang-visualizer" : source}
          </span>
          {onRefresh && (
            <button
              type="button"
              onClick={handleRefresh}
              className="intent-viz-tab intent-viz-tab-icon"
              title="重新加载并重置视图"
            >
              <RefreshCw size={14} />
            </button>
          )}
        </div>

        <div className="intent-viz-body">
          <aside className="intent-viz-sidebar">
            <p className="intent-viz-sidebar-title">声明块</p>
            <input
              type="search"
              placeholder="筛选…"
              value={blockFilter}
              onChange={(e) => setBlockFilter(e.target.value)}
              className="intent-viz-search"
            />
            <div className="intent-viz-block-list">
              {filteredBlocks.length === 0 ? (
                <p className="empty-state px-2 py-4 text-[12px]">无匹配块</p>
              ) : (
                filteredBlocks.map((b) => (
                  <button
                    key={`${b.file}-${b.name}`}
                    type="button"
                    onClick={() => onSelectBlock(b.file, b.name)}
                    className="intent-viz-block-row"
                  >
                    <span className="intent-viz-block-kind">{b.kind}</span>
                    <span className="intent-viz-block-name">{b.name}</span>
                    <span className="intent-viz-block-file">{b.file}</span>
                  </button>
                ))
              )}
            </div>
          </aside>

          <div className="intent-viz-main">
            {active && (
              <div className="intent-viz-section">
                <h3 className="intent-viz-section-title">{active.label}</h3>
                <p className="intent-viz-section-desc">{active.description}</p>

                {parseError && source === "fallback" && (
                  <div className="intent-viz-warn">
                    解析失败，已回退到简化图：{parseError}
                  </div>
                )}

                <div className="intent-viz-legend">
                  {LEGEND.map((item) => (
                    <span key={item.kind} className="intent-viz-legend-item">
                      <span
                        className="intent-viz-legend-swatch"
                        style={{
                          background: item.color,
                          borderColor: item.stroke,
                        }}
                      />
                      {item.label}
                    </span>
                  ))}
                </div>

                <MermaidDiagramCanvas
                  key={`${active.id}-${diagramEpoch}`}
                  chart={active.mermaid}
                  className="intent-viz-diagram"
                />
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
