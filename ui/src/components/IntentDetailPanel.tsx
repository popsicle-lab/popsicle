import { useEffect, useState } from "react";
import { ArrowLeft } from "lucide-react";
import {
  intentGraphMermaid,
  readIntentFile,
  type IntentFileFull,
} from "../hooks/useTauri";
import { IntentCodeBlock } from "./IntentCodeBlock";
import { MermaidRenderer } from "./MermaidRenderer";
import type { Page } from "../App";

interface Props {
  product: string;
  file: string;
  block?: string;
  returnTo?: Page;
  setPage: (p: Page) => void;
  showBack?: boolean;
}

function backLabel(returnTo?: Page): string {
  if (returnTo?.kind === "issue") return `Back to ${returnTo.issueKey}`;
  if (returnTo?.kind === "issues" && returnTo.selectedKey) {
    return `Back to ${returnTo.selectedKey}`;
  }
  return "Back to Products";
}

export function IntentDetailPanel({
  product,
  file,
  block,
  returnTo,
  setPage,
  showBack,
}: Props) {
  const [intent, setIntent] = useState<IntentFileFull | null>(null);
  const [mermaid, setMermaid] = useState<string | null>(null);
  const [viewFull, setViewFull] = useState(!block);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setError(null);
    Promise.all([
      readIntentFile(product, file),
      intentGraphMermaid(product).catch(() => null),
    ])
      .then(([f, mm]) => {
        setIntent(f);
        setMermaid(mm);
      })
      .catch((e) => setError(String(e)));
  }, [product, file]);

  useEffect(() => {
    setViewFull(!block);
  }, [block, file]);

  if (error) {
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );
  }
  if (!intent) {
    return <div className="text-[var(--text-secondary)]">Loading…</div>;
  }

  const selected = block
    ? intent.blocks.find((b) => b.name === block)
    : undefined;
  const displayCode = viewFull || !selected ? intent.content : selected.snippet;

  return (
    <div className="space-y-4 h-full flex flex-col">
      {showBack && (
        <button
          type="button"
          onClick={() =>
            setPage(
              returnTo ?? {
                kind: "products",
                product,
                tab: "intents",
                intentFile: file,
                intentBlock: block,
              }
            )
          }
          className="btn btn-ghost w-fit gap-1.5 px-0"
        >
          <ArrowLeft size={15} /> {backLabel(returnTo)}
        </button>
      )}
      <div className="flex items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold font-mono">{intent.file}</h2>
          {selected && (
            <p className="text-sm text-[var(--accent)]">
              {selected.kind} {selected.name}
            </p>
          )}
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => setViewFull(false)}
            disabled={!block}
            className={`px-3 py-1 rounded text-xs ${!viewFull ? "bg-[var(--accent)]/20 text-[var(--accent)]" : "bg-[var(--bg-tertiary)]"}`}
          >
            Block
          </button>
          <button
            onClick={() => setViewFull(true)}
            className={`px-3 py-1 rounded text-xs ${viewFull ? "bg-[var(--accent)]/20 text-[var(--accent)]" : "bg-[var(--bg-tertiary)]"}`}
          >
            Full file
          </button>
        </div>
      </div>
      {mermaid && (
        <div className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-xl p-4 overflow-auto max-h-48">
          <MermaidRenderer chart={mermaid} />
        </div>
      )}
      <IntentCodeBlock
        code={displayCode}
        highlightBlock={!viewFull && selected ? selected.name : undefined}
      />
    </div>
  );
}
