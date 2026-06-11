import { useCallback, useEffect, useState } from "react";
import {
  intentGraphMermaid,
  listProductNames,
  scanIntentGraph,
  type IntentGraph,
} from "../hooks/useTauri";
import { MermaidRenderer } from "../components/MermaidRenderer";

export function IntentGraphView() {
  const [products, setProducts] = useState<string[]>([]);
  const [product, setProduct] = useState("");
  const [graph, setGraph] = useState<IntentGraph | null>(null);
  const [mermaid, setMermaid] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    listProductNames()
      .then((names) => {
        setProducts(names);
        if (names.length > 0) setProduct(names[0]);
      })
      .catch((e) => setError(String(e)));
  }, []);

  const load = useCallback(() => {
    if (!product) return;
    setError(null);
    Promise.all([
      scanIntentGraph(product),
      intentGraphMermaid(product),
    ])
      .then(([g, mm]) => {
        setGraph(g);
        setMermaid(mm);
      })
      .catch((e) => setError(String(e)));
  }, [product]);

  useEffect(() => {
    load();
  }, [load]);

  if (error) {
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between gap-4">
        <div>
          <h2 className="text-2xl font-bold">Intent Graph</h2>
          <p className="text-sm text-[var(--text-secondary)]">
            Source: {graph?.source ?? "—"}
          </p>
        </div>
        <select
          value={product}
          onChange={(e) => setProduct(e.target.value)}
          className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm"
        >
          {products.map((p) => (
            <option key={p} value={p}>
              {p}
            </option>
          ))}
        </select>
      </div>

      {mermaid ? (
        <div className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-xl p-6 overflow-auto">
          <MermaidRenderer chart={mermaid} />
        </div>
      ) : (
        <p className="text-[var(--text-secondary)] text-sm">
          No diagram available. Run `make intent` to install intent-cli.
        </p>
      )}

      {graph && graph.blocks.length > 0 && (
        <div className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-xl p-4">
          <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-2">
            Blocks ({graph.blocks.length})
          </h3>
          <ul className="text-xs space-y-1 max-h-48 overflow-auto">
            {graph.blocks.map((b) => (
              <li key={`${b.file}-${b.name}`} className="font-mono">
                {b.kind}:{b.name}
                {b.task_id ? ` → ${b.task_id}` : ""}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
