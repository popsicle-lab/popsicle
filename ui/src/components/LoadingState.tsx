interface Props {
  label?: string;
}

export function LoadingState({ label = "Loading…" }: Props) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 py-16 text-[var(--text-muted)]">
      <div className="spinner" aria-hidden />
      <span className="text-[13px]">{label}</span>
    </div>
  );
}
