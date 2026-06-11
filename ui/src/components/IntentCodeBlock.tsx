import { useEffect, useRef } from "react";

interface Props {
  code: string;
  highlightBlock?: string;
}

export function IntentCodeBlock({ code, highlightBlock }: Props) {
  const preRef = useRef<HTMLPreElement>(null);

  useEffect(() => {
    if (!highlightBlock || !preRef.current) return;
    const el = preRef.current;
    const text = el.textContent ?? "";
    const idx = text.indexOf(highlightBlock);
    if (idx < 0) return;
    const range = document.createRange();
    const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
    let pos = 0;
    let startNode: Text | null = null;
    let startOff = 0;
    let endNode: Text | null = null;
    let endOff = 0;
    const endIdx = idx + highlightBlock.length;
    while (walker.nextNode()) {
      const node = walker.currentNode as Text;
      const len = node.length;
      if (!startNode && pos + len > idx) {
        startNode = node;
        startOff = idx - pos;
      }
      if (!endNode && pos + len >= endIdx) {
        endNode = node;
        endOff = endIdx - pos;
        break;
      }
      pos += len;
    }
    if (startNode && endNode) {
      range.setStart(startNode, startOff);
      range.setEnd(endNode, endOff);
      const sel = window.getSelection();
      sel?.removeAllRanges();
      sel?.addRange(range);
      const rect = range.getBoundingClientRect();
      if (rect.top) {
        preRef.current.scrollIntoView({ block: "center", behavior: "smooth" });
      }
    }
  }, [code, highlightBlock]);

  return (
    <pre
      ref={preRef}
      className="text-xs font-mono bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-4 overflow-auto whitespace-pre-wrap leading-relaxed max-h-[70vh]"
    >
      {code}
    </pre>
  );
}
