import type { Messages } from "../i18n/messages";
import type { IssueSortKey } from "./issueSort";

export function issueTypeLabel(type: string, m: Messages): string {
  const map: Record<string, string> = {
    product: m.issues.typeProduct,
    technical: m.issues.typeTechnical,
    bug: m.issues.typeBug,
    idea: m.issues.typeIdea,
  };
  return map[type] ?? type;
}

export function issueStatusLabel(status: string, m: Messages): string {
  const map: Record<string, string> = {
    backlog: m.issues.statusBacklog,
    ready: m.issues.statusReady,
    in_progress: m.issues.statusInProgress,
    done: m.issues.statusDone,
  };
  return map[status] ?? status.replace(/_/g, " ");
}

export function workflowProfileLabel(profile: string, m: Messages): string {
  const map: Record<string, string> = {
    "daily-dev": m.settings.profileDailyDev,
    migration: m.settings.profileMigration,
    "pm-spec-only": m.settings.profilePmSpec,
    "opc-full": m.settings.profileOpcFull,
  };
  return map[profile] ?? profile;
}

export function issueSortOptions(m: Messages): { value: IssueSortKey; label: string }[] {
  return [
    { value: "key_desc", label: m.issues.sortKeyDesc },
    { value: "key_asc", label: m.issues.sortKeyAsc },
    { value: "title_asc", label: m.issues.sortTitleAsc },
    { value: "title_desc", label: m.issues.sortTitleDesc },
    { value: "type", label: m.issues.sortType },
    { value: "status", label: m.issues.sortStatus },
  ];
}
