import { useEffect } from "react";
import { ExternalLink, X } from "lucide-react";
import { IssueDetailView } from "../pages/IssueDetailView";
import type { Page } from "../App";
import { useLocale } from "../i18n/LocaleContext";

interface Props {
  issueKey: string;
  onClose: () => void;
  setPage: (p: Page) => void;
}

export function IssueDetailModal({ issueKey, onClose, setPage }: Props) {
  const { m } = useLocale();

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", onKey);
    const prev = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      document.removeEventListener("keydown", onKey);
      document.body.style.overflow = prev;
    };
  }, [onClose]);

  const openFull = () => {
    onClose();
    setPage({ kind: "issue", issueKey });
  };

  return (
    <div
      className="issue-modal-backdrop"
      role="presentation"
      onClick={onClose}
    >
      <div
        className="issue-modal-shell"
        role="dialog"
        aria-modal="true"
        aria-labelledby="issue-modal-title"
        onClick={(e) => e.stopPropagation()}
      >
        <header className="issue-modal-toolbar">
          <span id="issue-modal-title" className="issue-modal-title">
            {issueKey}
          </span>
          <div className="issue-modal-actions">
            <button type="button" className="btn btn-ghost text-[12px]" onClick={openFull}>
              <ExternalLink size={14} />
              {m.issues.openFullPage}
            </button>
            <button
              type="button"
              className="btn btn-ghost"
              onClick={onClose}
              aria-label={m.issues.closeDetail}
            >
              <X size={18} />
            </button>
          </div>
        </header>
        <div className="issue-modal-body">
          <IssueDetailView
            issueKey={issueKey}
            setPage={(p) => {
              onClose();
              setPage(p);
            }}
            variant="modal"
          />
        </div>
      </div>
    </div>
  );
}
