import { useEffect, useId, useState } from "react";
import mermaid from "mermaid";

mermaid.initialize({
  startOnLoad: false,
  theme: "dark",
  securityLevel: "loose",
});

interface Props {
  chart: string;
  className?: string;
}

export function MermaidRenderer({ chart, className }: Props) {
  const id = useId().replace(/:/g, "");
  const [svg, setSvg] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setSvg(null);
    setError(null);
    mermaid
      .render(`mmd-${id}`, chart)
      .then(({ svg: rendered }) => {
        if (!cancelled) setSvg(rendered);
      })
      .catch((e: unknown) => {
        if (!cancelled) setError(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [chart, id]);

  if (error) {
    return (
      <div className={className}>
        <p className="text-[var(--accent-red)] text-sm mb-2">
          Mermaid render failed
        </p>
        <pre className="text-xs bg-[var(--bg-primary)] p-3 rounded border border-[var(--border)] overflow-auto">
          {chart}
        </pre>
      </div>
    );
  }

  if (!svg) {
    return (
      <div className={`text-[var(--text-secondary)] text-sm ${className ?? ""}`}>
        Rendering diagram…
      </div>
    );
  }

  return (
    <div
      className={className}
      dangerouslySetInnerHTML={{ __html: svg }}
    />
  );
}
