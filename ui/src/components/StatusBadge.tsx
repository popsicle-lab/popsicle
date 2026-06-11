const statusVariant: Record<string, string> = {
  draft: "badge-neutral",
  review: "badge-warning",
  approved: "badge-success",
  completed: "badge-success",
  done: "badge-success",
  ready: "badge-accent",
  in_progress: "badge-accent",
  blocked: "badge-danger",
  error: "badge-danger",
  backlog: "badge-neutral",
  cancelled: "badge-danger",
};

export function StatusBadge({ status }: { status: string }) {
  const variant = statusVariant[status] ?? "badge-neutral";
  return <span className={`badge ${variant}`}>{status.replace(/_/g, " ")}</span>;
}
