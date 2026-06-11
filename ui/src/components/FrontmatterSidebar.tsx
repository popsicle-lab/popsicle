interface Props {
  data: Record<string, unknown>;
  keys?: string[];
}

function fmt(value: unknown): string {
  if (value === null || value === undefined) return "—";
  if (Array.isArray(value)) return value.map(String).join(", ");
  if (typeof value === "object") return JSON.stringify(value);
  return String(value);
}

const PREFERRED_KEYS = [
  "task_id",
  "title",
  "journey_stage",
  "task_type",
  "audience",
  "involved_features",
  "prerequisites",
  "limits",
  "related_intents",
  "related_next_tasks",
  "last_updated",
  "last_verified",
  "decision_ref",
];

export function FrontmatterSidebar({ data, keys }: Props) {
  const ordered = keys ?? [
    ...PREFERRED_KEYS.filter((k) => k in data),
    ...Object.keys(data).filter((k) => !PREFERRED_KEYS.includes(k)),
  ];

  return (
    <aside className="w-56 shrink-0 space-y-2 text-xs">
      <h3 className="font-medium text-[var(--text-secondary)]">Frontmatter</h3>
      <dl className="space-y-2">
        {ordered.map((key) => (
          <div key={key}>
            <dt className="text-[var(--text-secondary)] font-mono">{key}</dt>
            <dd className="mt-0.5 break-words">{fmt(data[key])}</dd>
          </div>
        ))}
      </dl>
    </aside>
  );
}
