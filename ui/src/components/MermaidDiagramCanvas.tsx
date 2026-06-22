import {
  useCallback,
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
} from "react";
import {
  Expand,
  Maximize2,
  Minimize2,
  Minus,
  Plus,
  RotateCcw,
} from "lucide-react";
import mermaid from "mermaid";
import { sanitizeMermaidChart } from "../lib/mermaidSanitize";

let mermaidReady = false;

function ensureMermaidInit() {
  if (mermaidReady) return;
  mermaid.initialize({
    startOnLoad: false,
    theme: "base",
    securityLevel: "loose",
    flowchart: {
      useMaxWidth: false,
      htmlLabels: true,
      curve: "basis",
      padding: 20,
      nodeSpacing: 56,
      rankSpacing: 72,
      diagramPadding: 20,
    },
    themeVariables: {
      darkMode: true,
      background: "transparent",
      primaryColor: "#1e3a8a",
      primaryTextColor: "#f4f4f5",
      primaryBorderColor: "#60a5fa",
      secondaryColor: "#27272a",
      secondaryTextColor: "#f4f4f5",
      secondaryBorderColor: "#71717a",
      tertiaryColor: "#18181b",
      tertiaryTextColor: "#f4f4f5",
      tertiaryBorderColor: "#52525b",
      lineColor: "#a1a1aa",
      textColor: "#f4f4f5",
      mainBkg: "#27272a",
      nodeBorder: "#71717a",
      clusterBkg: "#18181b",
      clusterBorder: "#52525b",
      titleColor: "#fafafa",
      edgeLabelBackground: "#27272a",
      nodeTextColor: "#f4f4f5",
      fontFamily:
        "SF Pro Text, Inter, system-ui, -apple-system, BlinkMacSystemFont, sans-serif",
    },
  });
  mermaidReady = true;
}

function normalizeSvg(svg: string): string {
  return svg.replace(/<svg([^>]*)>/i, (_, attrs: string) => {
    let next = attrs
      .replace(/\swidth="[^"]*"/i, "")
      .replace(/\sheight="[^"]*"/i, "")
      .replace(/\sstyle="[^"]*"/i, "");
    if (!/class=/i.test(next)) {
      next += ' class="mermaid-svg"';
    }
    return `<svg${next}>`;
  });
}

function readSvgNaturalSize(svgEl: SVGSVGElement): { w: number; h: number } {
  const vb = svgEl.viewBox.baseVal;
  if (vb.width > 0 && vb.height > 0) {
    return { w: vb.width, h: vb.height };
  }
  try {
    const bbox = svgEl.getBBox();
    if (bbox.width > 0 && bbox.height > 0) {
      return { w: bbox.width, h: bbox.height };
    }
  } catch {
    /* getBBox may fail before layout */
  }
  const w = parseFloat(svgEl.getAttribute("width") ?? "800");
  const h = parseFloat(svgEl.getAttribute("height") ?? "600");
  return { w: w > 0 ? w : 800, h: h > 0 ? h : 600 };
}

function applySvgScale(
  content: HTMLDivElement,
  scale: number
): { w: number; h: number } | null {
  const svgEl = content.querySelector("svg");
  if (!svgEl) return null;
  const natural = readSvgNaturalSize(svgEl);
  const w = natural.w * scale;
  const h = natural.h * scale;
  svgEl.setAttribute("width", String(w));
  svgEl.setAttribute("height", String(h));
  svgEl.style.width = `${w}px`;
  svgEl.style.height = `${h}px`;
  svgEl.style.maxWidth = "none";
  content.style.width = `${w}px`;
  content.style.height = `${h}px`;
  return natural;
}

interface Props {
  chart: string;
  className?: string;
}

export function MermaidDiagramCanvas({ chart, className }: Props) {
  const id = useId().replace(/:/g, "");
  const rootRef = useRef<HTMLDivElement>(null);
  const viewportRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const [svg, setSvg] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [scale, setScale] = useState(1);
  const [fitScale, setFitScale] = useState(1);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const dragRef = useRef<{ x: number; y: number; px: number; py: number } | null>(
    null
  );

  const safeChart = useMemo(() => sanitizeMermaidChart(chart), [chart]);

  const computeFitScale = useCallback((): number => {
    const viewport = viewportRef.current;
    const content = contentRef.current;
    if (!viewport || !content) return 1;
    const svgEl = content.querySelector("svg");
    if (!svgEl) return 1;
    const natural = readSvgNaturalSize(svgEl);
    const pad = 32;
    const vw = Math.max(viewport.clientWidth - pad, 200);
    const vh = Math.max(viewport.clientHeight - pad, 160);
    if (natural.w <= 0 || natural.h <= 0) return 1;
    return Number(Math.min(1.5, vw / natural.w, vh / natural.h).toFixed(3));
  }, []);

  const fitView = useCallback(() => {
    const next = computeFitScale();
    setFitScale(next);
    setScale(next);
    setPan({ x: 0, y: 0 });
    if (viewportRef.current) {
      viewportRef.current.scrollTop = 0;
      viewportRef.current.scrollLeft = 0;
    }
  }, [computeFitScale]);

  useEffect(() => {
    let cancelled = false;
    setSvg(null);
    setError(null);
    setScale(1);
    setFitScale(1);
    setPan({ x: 0, y: 0 });
    ensureMermaidInit();
    mermaid
      .render(`mmd-${id}-${Date.now()}`, safeChart)
      .then(({ svg: rendered }) => {
        if (!cancelled) setSvg(normalizeSvg(rendered));
      })
      .catch((e: unknown) => {
        if (!cancelled) setError(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [safeChart, id]);

  useEffect(() => {
    if (!svg || !contentRef.current) return;
    applySvgScale(contentRef.current, scale);
    const t = window.requestAnimationFrame(() => {
      const next = computeFitScale();
      setFitScale(next);
      setScale(next);
    });
    return () => window.cancelAnimationFrame(t);
  }, [svg, computeFitScale]);

  useEffect(() => {
    if (!svg) return;
    if (contentRef.current) {
      applySvgScale(contentRef.current, scale);
    }
  }, [scale, svg]);

  useEffect(() => {
    if (!svg) return;
    const ro = new ResizeObserver(() => {
      const next = computeFitScale();
      setFitScale(next);
    });
    if (viewportRef.current) ro.observe(viewportRef.current);
    return () => ro.disconnect();
  }, [svg, computeFitScale]);

  useEffect(() => {
    const onFs = () => {
      setIsFullscreen(document.fullscreenElement === rootRef.current);
      window.requestAnimationFrame(() => fitView());
    };
    document.addEventListener("fullscreenchange", onFs);
    return () => document.removeEventListener("fullscreenchange", onFs);
  }, [fitView]);

  const zoomIn = () =>
    setScale((s) => Math.min(3, +(s + 0.2).toFixed(2)));
  const zoomOut = () =>
    setScale((s) => Math.max(0.2, +(s - 0.2).toFixed(2)));
  const resetZoom = () => {
    setScale(fitScale || 1);
    setPan({ x: 0, y: 0 });
  };

  const toggleFullscreen = async () => {
    const el = rootRef.current;
    if (!el) return;
    try {
      if (document.fullscreenElement === el) {
        await document.exitFullscreen();
      } else {
        await el.requestFullscreen();
      }
    } catch {
      fitView();
    }
  };

  const onPointerDown = (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    dragRef.current = {
      x: e.clientX,
      y: e.clientY,
      px: pan.x,
      py: pan.y,
    };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  };

  const onPointerMove = (e: React.PointerEvent) => {
    const drag = dragRef.current;
    if (!drag) return;
    setPan({
      x: drag.px + (e.clientX - drag.x),
      y: drag.py + (e.clientY - drag.y),
    });
  };

  const onPointerUp = (e: React.PointerEvent) => {
    dragRef.current = null;
    (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
  };

  const onWheel = (e: React.WheelEvent) => {
    if (!e.ctrlKey && !e.metaKey) return;
    e.preventDefault();
    const delta = e.deltaY > 0 ? -0.12 : 0.12;
    setScale((s) =>
      Number(Math.min(3, Math.max(0.2, s + delta)).toFixed(2))
    );
  };

  if (error) {
    return (
      <div className={`mermaid-canvas-error ${className ?? ""}`}>
        <p className="text-[var(--accent-red)] text-sm mb-2">Mermaid 渲染失败</p>
        <pre className="mermaid-canvas-error-pre">{chart}</pre>
      </div>
    );
  }

  return (
    <div
      ref={rootRef}
      className={`mermaid-canvas ${isFullscreen ? "mermaid-canvas-fullscreen" : ""} ${className ?? ""}`}
    >
      <div className="mermaid-canvas-toolbar">
        <span className="mermaid-canvas-zoom-label">
          {Math.round(scale * 100)}%
        </span>
        <button
          type="button"
          className="mermaid-canvas-tool"
          onClick={zoomOut}
          title="缩小"
          aria-label="缩小"
        >
          <Minus size={14} />
        </button>
        <button
          type="button"
          className="mermaid-canvas-tool"
          onClick={zoomIn}
          title="放大"
          aria-label="放大"
        >
          <Plus size={14} />
        </button>
        <button
          type="button"
          className="mermaid-canvas-tool"
          onClick={resetZoom}
          title="重置缩放"
          aria-label="重置缩放"
        >
          <RotateCcw size={14} />
        </button>
        <button
          type="button"
          className="mermaid-canvas-tool"
          onClick={fitView}
          title="适应窗口"
          aria-label="适应窗口"
        >
          <Maximize2 size={14} />
        </button>
        <button
          type="button"
          className="mermaid-canvas-tool"
          onClick={toggleFullscreen}
          title={isFullscreen ? "退出全屏" : "全屏"}
          aria-label={isFullscreen ? "退出全屏" : "全屏"}
        >
          {isFullscreen ? <Minimize2 size={14} /> : <Expand size={14} />}
        </button>
        <span className="mermaid-canvas-hint">拖拽平移 · Ctrl+滚轮缩放</span>
      </div>
      <div
        ref={viewportRef}
        className="mermaid-canvas-viewport"
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerLeave={onPointerUp}
        onWheel={onWheel}
      >
        {!svg ? (
          <div className="mermaid-canvas-loading">正在渲染关系图…</div>
        ) : (
          <div
            className="mermaid-canvas-pan-layer"
            style={{ transform: `translate(${pan.x}px, ${pan.y}px)` }}
          >
            <div
              ref={contentRef}
              className="mermaid-canvas-content"
              dangerouslySetInnerHTML={{ __html: svg }}
            />
          </div>
        )}
      </div>
    </div>
  );
}
