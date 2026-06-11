import type { Page } from "../App";

export interface Crumb {
  label: string;
  page?: Page;
}

function issueCrumbs(issueKey: string): Crumb[] {
  return [
    { label: "Issues", page: { kind: "issues" } },
    { label: issueKey, page: { kind: "issue", issueKey } },
  ];
}

export function pageCrumbs(page: Page): Crumb[] {
  switch (page.kind) {
    case "settings":
      return [{ label: "Settings" }];
    case "issues":
      return page.selectedKey
        ? [
            { label: "Issues", page: { kind: "issues" } },
            { label: page.selectedKey },
          ]
        : [{ label: "Issues" }];
    case "issue":
      return issueCrumbs(page.issueKey);
    case "pipeline":
      return [
        { label: "Issues", page: { kind: "issues" } },
        { label: "Pipeline" },
      ];
    case "document":
      return [
        { label: "Issues", page: { kind: "issues" } },
        { label: "Document" },
      ];
    case "products":
      return [
        { label: "Products", page: { kind: "products", tab: "tasks" } },
        ...(page.product ? [{ label: page.product }] : []),
      ];
    case "task": {
      if (page.returnTo?.kind === "issue") {
        return [
          ...issueCrumbs(page.returnTo.issueKey),
          { label: page.taskId },
        ];
      }
      if (
        page.returnTo?.kind === "issues" &&
        page.returnTo.selectedKey
      ) {
        return [
          { label: "Issues", page: { kind: "issues" } },
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
        { label: "Products", page: { kind: "products", tab: "tasks" } },
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
          ...issueCrumbs(page.returnTo.issueKey),
          { label: page.block ?? page.file },
        ];
      }
      if (
        page.returnTo?.kind === "issues" &&
        page.returnTo.selectedKey
      ) {
        return [
          { label: "Issues", page: { kind: "issues" } },
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
        { label: "Products", page: { kind: "products", tab: "intents" } },
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
