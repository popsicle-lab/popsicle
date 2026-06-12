import { useLocale } from "../i18n/LocaleContext";
import { issueTypeLabel } from "../lib/issueLabels";

const TYPE_STYLES: Record<string, string> = {
  product: "issue-type-badge issue-type-product",
  technical: "issue-type-badge issue-type-technical",
  bug: "issue-type-badge issue-type-bug",
  idea: "issue-type-badge issue-type-idea",
};

interface Props {
  type: string;
}

export function IssueTypeBadge({ type }: Props) {
  const { m } = useLocale();
  const cls = TYPE_STYLES[type] ?? "issue-type-badge issue-type-default";
  return <span className={cls}>{issueTypeLabel(type, m)}</span>;
}

export function issueTypeAccentClass(type: string): string {
  switch (type) {
    case "product":
      return "list-item-type-product";
    case "technical":
      return "list-item-type-technical";
    case "bug":
      return "list-item-type-bug";
    case "idea":
      return "list-item-type-idea";
    default:
      return "";
  }
}
