import { useLocale } from "../i18n/LocaleContext";
import { issueStatusLabel } from "../lib/issueLabels";

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
  const { m } = useLocale();
  const variant = statusVariant[status] ?? "badge-neutral";
  return (
    <span className={`badge ${variant}`}>{issueStatusLabel(status, m)}</span>
  );
}
