export type Locale = "zh-CN" | "en";

export function normalizeLocale(raw: string): Locale {
  const s = raw.trim().toLowerCase();
  if (s === "zh" || s === "zh-cn" || s === "zh_cn" || s === "chinese") {
    return "zh-CN";
  }
  return "en";
}

type Messages = {
  nav: { issues: string; products: string; settings: string; tagline: string; footer: string };
  header: {
    expandSidebar: string;
    collapseSidebar: string;
    back: string;
    binaryMismatch: string;
  };
  crumbs: {
    issues: string;
    settings: string;
    products: string;
    pipeline: string;
    document: string;
  };
  settings: {
    title: string;
    intro: string;
    language: string;
    languageHint: string;
    productsDir: string;
    productsDirHint: string;
    defaultProduct: string;
    defaultProductHint: string;
    defaultProductNone: string;
    approvalMode: string;
    approvalManual: string;
    approvalAuto: string;
    approvalDelegate: string;
    approvalHint: string;
    syncAgents: string;
    injectOnRun: string;
    save: string;
    saving: string;
    saved: string;
    configPath: string;
    loadError: string;
  };
  issues: {
    selectIssue: string;
    selectHint: string;
    createIssue: string;
  };
};

const zhCN: Messages = {
  nav: {
    issues: "Issues",
    products: "Products",
    settings: "设置",
    tagline: "Spec 工作区",
    footer: "Spec-driven development",
  },
  header: {
    expandSidebar: "展开侧栏",
    collapseSidebar: "收起侧栏",
    back: "返回",
    binaryMismatch: "二进制不匹配",
  },
  crumbs: {
    issues: "Issues",
    settings: "设置",
    products: "Products",
    pipeline: "Pipeline",
    document: "Document",
  },
  settings: {
    title: "项目设置",
    intro: "配置会写入 .popsicle/project.yaml，并可同步到根目录 AGENTS.md。",
    language: "界面 / Agent 语言",
    languageHint: "控制 Popsicle 桌面端与 CLI 帮助文案，以及 Agent 回复语言。",
    productsDir: "产品文档目录",
    productsDirHint: "相对仓库根目录，Products 浏览器与 pipeline 文档路径均以此为准。",
    defaultProduct: "默认产品（可选）",
    defaultProductHint: "新建 Issue 时预选的 products/<name>/ 产品。",
    defaultProductNone: "（无）",
    approvalMode: "Pipeline 审批模式",
    approvalManual: "必须人工审批",
    approvalAuto: "全自动",
    approvalDelegate: "危险操作需审批（其余代批）",
    approvalHint:
      "控制带 requires_approval 阶段的完成方式；危险阶段为 cutover、living-docs。",
    syncAgents: "保存时同步到 AGENTS.md",
    injectOnRun: "工作流启动 / 创建文档时注入偏好到提示词",
    save: "保存配置",
    saving: "保存中…",
    saved: "已保存",
    configPath: "配置文件",
    loadError: "无法加载项目配置",
  },
  issues: {
    selectIssue: "选择一条 Issue",
    selectHint: "使用上方筛选，点击列表行查看详情与 Guidance。",
    createIssue: "新建 Issue",
  },
};

const en: Messages = {
  nav: {
    issues: "Issues",
    products: "Products",
    settings: "Settings",
    tagline: "Spec workspace",
    footer: "Spec-driven development",
  },
  header: {
    expandSidebar: "Expand sidebar",
    collapseSidebar: "Collapse sidebar",
    back: "Back",
    binaryMismatch: "Binary mismatch",
  },
  crumbs: {
    issues: "Issues",
    settings: "Settings",
    products: "Products",
    pipeline: "Pipeline",
    document: "Document",
  },
  settings: {
    title: "Project settings",
    intro:
      "Saved to .popsicle/project.yaml and optionally synced to AGENTS.md.",
    language: "UI / agent language",
    languageHint:
      "Controls Popsicle desktop UI, CLI help text, and agent reply language.",
    productsDir: "Products directory",
    productsDirHint:
      "Relative to repo root; used by Products explorer and pipeline doc paths.",
    defaultProduct: "Default product (optional)",
    defaultProductHint: "Pre-selected product when creating a new issue.",
    defaultProductNone: "(none)",
    approvalMode: "Pipeline approval mode",
    approvalManual: "Manual approval required",
    approvalAuto: "Fully automatic",
    approvalDelegate: "Dangerous stages need approval (delegate others)",
    approvalHint:
      "How requires_approval stages complete; dangerous: cutover, living-docs.",
    syncAgents: "Sync to AGENTS.md on save",
    injectOnRun: "Inject preferences when starting workflows / creating docs",
    save: "Save",
    saving: "Saving…",
    saved: "Saved",
    configPath: "Config file",
    loadError: "Failed to load project config",
  },
  issues: {
    selectIssue: "Select an issue",
    selectHint: "Use filters above, then pick a row to preview details and guidance.",
    createIssue: "Create issue",
  },
};

export function messagesFor(locale: Locale): Messages {
  return locale === "zh-CN" ? zhCN : en;
}
