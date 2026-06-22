/** Sanitize visualizer output: CJK/punctuation node IDs break Mermaid 11. */
export function sanitizeMermaidChart(input: string): string {
  const idMap = new Map<string, string>();
  let next = 0;

  const allocate = (raw: string): string => {
    const hit = idMap.get(raw);
    if (hit) return hit;
    const id = `n${next++}`;
    idMap.set(raw, id);
    return id;
  };

  const isSafeId = (id: string) =>
    /^[A-Za-z_][A-Za-z0-9_]*$/.test(id);

  const rewriteNode = (line: string): string | null => {
    const m = line.match(/^(\s*)(.+)$/);
    if (!m) return null;
    const [, indent, content] = m;
    const shapeIdx = content.search(/[\[(]/);
    if (shapeIdx <= 0) return null;
    const oldId = content.slice(0, shapeIdx).trim();
    if (!oldId) return null;
    const newId = isSafeId(oldId) ? oldId : allocate(oldId);
    return `${indent}${newId}${content.slice(shapeIdx)}`;
  };

  const rewriteEdge = (line: string): string | null => {
    const m = line.match(/^(\s*)(.+)$/);
    if (!m) return null;
    const [, indent, content] = m;
    for (const arrow of ["-.->", "==>", "-->", "---"]) {
      const idx = content.indexOf(arrow);
      if (idx === -1) continue;
      const from = content.slice(0, idx).trim();
      let rest = content.slice(idx + arrow.length).trim();
      let label: string | null = null;
      if (rest.startsWith("|")) {
        const end = rest.indexOf("|", 1);
        if (end === -1) return null;
        label = rest.slice(1, end);
        rest = rest.slice(end + 1).trim();
      }
      const to = rest;
      const fromId = idMap.get(from) ?? from;
      const toId = idMap.get(to) ?? to;
      const mid = label ? ` ${arrow}|${label}| ` : ` ${arrow} `;
      return `${indent}${fromId}${mid}${toId}`;
    }
    return null;
  };

  return input
    .split("\n")
    .map((line) => {
      const t = line.trim();
      if (!t || t.startsWith("graph ") || t.startsWith("classDef")) return line;
      return rewriteNode(line) ?? rewriteEdge(line) ?? line;
    })
    .join("\n");
}
