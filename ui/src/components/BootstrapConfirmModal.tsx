import { FolderOpen, Layers } from "lucide-react";
import { messagesFor, normalizeLocale, type Locale } from "../i18n/messages";

interface Props {
  path: string;
  locale?: Locale;
  onConfirm: () => void;
  onCancel: () => void;
}

function truncatePath(path: string, max = 48): string {
  if (path.length <= max) return path;
  const parts = path.split("/");
  const name = parts.pop() ?? path;
  if (name.length >= max - 3) return `…/${name.slice(-(max - 4))}`;
  return `…/${name}`;
}

export function BootstrapConfirmModal({
  path,
  locale = normalizeLocale(navigator.language),
  onConfirm,
  onCancel,
}: Props) {
  const t = messagesFor(locale).project.bootstrap;

  return (
    <div
      className="fixed inset-0 z-[100] flex items-center justify-center bg-black/45 p-4"
      role="dialog"
      aria-modal="true"
      aria-labelledby="bootstrap-confirm-title"
    >
      <div className="w-full max-w-md rounded-[var(--radius-md)] border border-[var(--border)] bg-[var(--bg-elevated)] p-5 shadow-[var(--shadow-md)]">
        <div className="mb-4 flex items-start gap-3">
          <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-[var(--radius-sm)] bg-[var(--accent-muted)] text-[var(--accent)]">
            <Layers size={18} />
          </div>
          <div className="min-w-0">
            <h2
              id="bootstrap-confirm-title"
              className="text-[15px] font-semibold tracking-tight"
            >
              {t.title}
            </h2>
            <p className="mt-1.5 text-[13px] leading-relaxed text-[var(--text-secondary)]">
              {t.body}
            </p>
          </div>
        </div>

        <div className="mb-5 flex items-center gap-2 rounded-[var(--radius-sm)] border border-[var(--border)] bg-[var(--bg-primary)] px-3 py-2.5">
          <FolderOpen size={14} className="shrink-0 text-[var(--text-muted)]" />
          <span
            className="min-w-0 truncate font-mono text-[12px] text-[var(--text-muted)]"
            title={path}
          >
            {truncatePath(path)}
          </span>
        </div>

        <ul className="mb-5 space-y-1 text-[12px] text-[var(--text-muted)]">
          {t.items.map((item) => (
            <li key={item} className="flex gap-2">
              <span className="text-[var(--accent)]">·</span>
              <span>{item}</span>
            </li>
          ))}
        </ul>

        <div className="flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
          <button type="button" onClick={onCancel} className="btn btn-secondary">
            {t.cancel}
          </button>
          <button type="button" onClick={onConfirm} className="btn btn-primary">
            {t.confirm}
          </button>
        </div>
      </div>
    </div>
  );
}
