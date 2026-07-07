import type { RunMirror } from "@/api/types";
import { isValidIssueKey } from "@/utils/format";

export function sanitizeRunMirror(
  mirror: RunMirror,
  previous?: RunMirror
): RunMirror {
  const issue_key = isValidIssueKey(mirror.issue_key)
    ? mirror.issue_key
    : isValidIssueKey(previous?.issue_key)
      ? previous!.issue_key!
      : null;
  return { ...mirror, issue_key };
}
