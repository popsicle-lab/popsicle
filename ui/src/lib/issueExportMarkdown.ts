import type { IssueInfo } from "../hooks/useTauri";
import { getLinkedTaskIds } from "./issueListUtils";

export interface IssueExportContext {
  layoutMode: "product" | "task";
  typeFilter: string;
  statusFilter: string;
  searchQuery: string;
}

function linkedTaskIds(issue: IssueInfo): string[] {
  const fromLinks = getLinkedTaskIds(issue);
  if (fromLinks.length > 0) return fromLinks;
  if (issue.epic_task_id) return [issue.epic_task_id];
  return [];
}

export function formatIssuesMarkdownBrief(
  issues: IssueInfo[],
  ctx: IssueExportContext
): string {
  const lines: string[] = [];
  const exportedAt = new Date().toISOString().slice(0, 19).replace("T", " ");

  lines.push("# Issue 列表简报");
  lines.push("");
  lines.push(`> 导出时间: ${exportedAt}`);
  lines.push(
    `> 视图: ${ctx.layoutMode === "product" ? "按产品" : "按 Task"}`
  );
  const filters: string[] = [];
  if (ctx.typeFilter !== "all") filters.push(`类型=${ctx.typeFilter}`);
  if (ctx.statusFilter !== "all") filters.push(`状态=${ctx.statusFilter}`);
  if (ctx.searchQuery.trim()) filters.push(`搜索=${ctx.searchQuery.trim()}`);
  lines.push(`> 筛选: ${filters.length ? filters.join(", ") : "无"}`);
  lines.push(`> 条数: ${issues.length}`);
  lines.push("");

  if (issues.length === 0) {
    lines.push("_（当前筛选下无 Issue）_");
    return lines.join("\n");
  }

  lines.push("| Key | Title | Type | Status | Pipeline | Product | Tasks |");
  lines.push("|-----|-------|------|--------|----------|---------|-------|");

  for (const issue of issues) {
    const tasks = linkedTaskIds(issue).join(", ") || "—";
    const pipeline = issue.pipeline ?? "—";
    const title = issue.title.replace(/\|/g, "\\|");
    lines.push(
      `| ${issue.key} | ${title} | ${issue.issue_type} | ${issue.status} | ${pipeline} | ${issue.product_id} | ${tasks} |`
    );
  }

  lines.push("");
  lines.push("## 明细");
  lines.push("");

  for (const issue of issues) {
    lines.push(`### ${issue.key}`);
    lines.push("");
    lines.push(`- **标题**: ${issue.title}`);
    lines.push(`- **类型**: ${issue.issue_type}`);
    lines.push(`- **状态**: ${issue.status}`);
    lines.push(`- **产品**: ${issue.product_id}`);
    lines.push(`- **Pipeline**: ${issue.pipeline ?? "—"}`);
    const tasks = linkedTaskIds(issue);
    if (tasks.length) {
      lines.push(`- **Linked tasks**: ${tasks.join(", ")}`);
    }
    const proposed = issue.task_links.filter((l) => l.role === "proposed");
    if (proposed.length) {
      for (const p of proposed) {
        lines.push(
          `- **Proposed task**: ${p.proposed_title ?? "—"} (${p.journey_stage ?? "—"})`
        );
      }
    }
    if (issue.active_run_id) {
      lines.push(`- **Active run**: \`${issue.active_run_id}\``);
    }
    lines.push("");
  }

  return lines.join("\n");
}
