import type { Page } from "../App";

export interface Crumb {
  label: string;
  page?: Page;
}

export interface CrumbLabels {
  issues: string;
  settings: string;
  products: string;
  workflows: string;
  pipeline: string;
  document: string;
}

function issueCrumbs(issueKey: string, labels: CrumbLabels): Crumb[] {
  return [
    { label: labels.issues, page: { kind: "issues" } },
    { label: issueKey, page: { kind: "issue", issueKey } },
  ];
}

export function pageCrumbs(page: Page, labels: CrumbLabels): Crumb[] {
  switch (page.kind) {
    case "settings":
      return [{ label: labels.settings }];
    case "workflows":
      return [{ label: labels.workflows }];
    case "issues":
      return page.selectedKey
        ? [
            { label: labels.issues, page: { kind: "issues" } },
            { label: page.selectedKey },
          ]
        : [{ label: labels.issues }];
    case "issue":
      return issueCrumbs(page.issueKey, labels);
    case "pipeline":
      return [
        { label: labels.issues, page: { kind: "issues" } },
        { label: labels.pipeline },
      ];
    case "document":
      return [
        { label: labels.issues, page: { kind: "issues" } },
        { label: labels.document },
      ];
    case "products":
      return [
        { label: labels.products, page: { kind: "products", tab: "tasks" } },
        ...(page.product ? [{ label: page.product }] : []),
      ];
    case "task": {
      if (page.returnTo?.kind === "issue") {
        return [
          ...issueCrumbs(page.returnTo.issueKey, labels),
          { label: page.taskId },
        ];
      }
      if (
        page.returnTo?.kind === "issues" &&
        page.returnTo.selectedKey
      ) {
        return [
          { label: labels.issues, page: { kind: "issues" } },
          {
            label: page.returnTo.selectedKey,
            page: {
              kind: "issues",
              selectedKey: page.returnTo.selectedKey,
            },
          },
          { label: page.taskId },
        ];
      }
      return [
        { label: labels.products, page: { kind: "products", tab: "tasks" } },
        {
          label: page.product,
          page: { kind: "products", product: page.product, tab: "tasks" },
        },
        { label: page.taskId },
      ];
    }
    case "intent": {
      if (page.returnTo?.kind === "issue") {
        return [
          ...issueCrumbs(page.returnTo.issueKey, labels),
          { label: page.block ?? page.file },
        ];
      }
      if (
        page.returnTo?.kind === "issues" &&
        page.returnTo.selectedKey
      ) {
        return [
          { label: labels.issues, page: { kind: "issues" } },
          {
            label: page.returnTo.selectedKey,
            page: {
              kind: "issues",
              selectedKey: page.returnTo.selectedKey,
            },
          },
          { label: page.block ?? page.file },
        ];
      }
      return [
        { label: labels.products, page: { kind: "products", tab: "intents" } },
        {
          label: page.product,
          page: { kind: "products", product: page.product, tab: "intents" },
        },
        { label: page.block ?? page.file },
      ];
    }
    default:
      return [{ label: "Popsicle" }];
  }
}

export function pageBack(page: Page): Page | null {
  switch (page.kind) {
    case "issues":
      return null;
    case "issue":
      return { kind: "issues" };
    case "pipeline":
    case "document":
      return { kind: "issues" };
    case "products":
      return null;
    case "workflows":
      return null;
    case "task":
      if (page.returnTo) return page.returnTo;
      return {
        kind: "products",
        product: page.product,
        tab: "tasks",
        taskId: page.taskId,
      };
    case "intent":
      if (page.returnTo) return page.returnTo;
      return {
        kind: "products",
        product: page.product,
        tab: "intents",
        intentFile: page.file,
        intentBlock: page.block,
      };
    default:
      return null;
  }
}
