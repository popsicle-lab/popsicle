export type Locale = "zh-CN" | "en";

export function normalizeLocale(raw: string): Locale {
  const s = raw.trim().toLowerCase();
  if (s === "zh" || s === "zh-cn" || s === "zh_cn" || s === "chinese") {
    return "zh-CN";
  }
  return "en";
}

export type Messages = {
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
    workflowProfile: string;
    workflowProfileHint: string;
    profileDailyDev: string;
    profileMigration: string;
    profilePmSpec: string;
    profileOpcFull: string;
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
    filterType: string;
    filterStatus: string;
    filterGroup: string;
    filterSort: string;
    filterCount: string;
    typeAll: string;
    typeProduct: string;
    typeTechnical: string;
    typeBug: string;
    typeIdea: string;
    statusAll: string;
    statusBacklog: string;
    statusReady: string;
    statusInProgress: string;
    statusDone: string;
    groupFlat: string;
    groupProduct: string;
    groupPipeline: string;
    groupEpic: string;
    sortKeyDesc: string;
    sortKeyAsc: string;
    sortTitleAsc: string;
    sortTitleDesc: string;
    sortType: string;
    sortStatus: string;
    createTitle: string;
    createType: string;
    createPipeline: string;
    createPipelineNone: string;
    createTitleLabel: string;
    createProduct: string;
    createProductHint: string;
    createEpic: string;
    createEpicNone: string;
    createTasks: string;
    createTasksHint: string;
    createDescription: string;
    creating: string;
    createSubmit: string;
    workflowProfile: string;
    unlinkedEpic: string;
    unlinkedEpicHint: string;
    noPipeline: string;
    pipelinePrefix: string;
    epicProgress: string;
    emptyFiltered: string;
    activeRun: string;
    searchPlaceholder: string;
    filterTask: string;
    filterTaskAll: string;
    statTotal: string;
    statInProgress: string;
    statDone: string;
    statActiveRuns: string;
    expandAll: string;
    collapseAll: string;
    colKey: string;
    colTitle: string;
    colTasks: string;
    colStatus: string;
    colPipeline: string;
    viewByProduct: string;
    viewByTask: string;
    openFullPage: string;
    closeDetail: string;
    emptyPanel: string;
    selectHintMosaic: string;
    exportMarkdown: string;
    exportCopied: string;
    exportFailed: string;
  };
  project: {
    bootstrap: {
      title: string;
      body: string;
      items: string[];
      confirm: string;
      cancel: string;
    };
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
    workflowProfile: "工作流画像",
    workflowProfileHint:
      "影响新建 Issue 的默认 pipeline 与 UI 侧重点（非权限控制）。",
    profileDailyDev: "日常开发",
    profileMigration: "大型迁移",
    profilePmSpec: "产品经理 / Spec",
    profileOpcFull: "OPC 全流程",
    approvalMode: "Pipeline 审批模式",
    approvalManual: "必须人工审批",
    approvalAuto: "全自动",
    approvalDelegate: "危险操作需审批（其余代批）",
    approvalHint:
      "控制带 requires_approval 阶段的完成方式；危险阶段为 cutover、living-docs。",
    syncAgents: "保存时同步到 AGENTS.md",
    injectOnRun: "创建 Issue / 启动 run / 创建文档时向 CLI 注入 agent_context",
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
    filterType: "类型",
    filterStatus: "状态",
    filterGroup: "分组",
    filterSort: "排序",
    filterCount: "{n} 条 Issue",
    typeAll: "全部",
    typeProduct: "产品",
    typeTechnical: "技术",
    typeBug: "缺陷",
    typeIdea: "想法",
    statusAll: "全部状态",
    statusBacklog: "待办",
    statusReady: "就绪",
    statusInProgress: "进行中",
    statusDone: "已完成",
    groupFlat: "平铺列表",
    groupProduct: "按产品",
    groupPipeline: "按 Pipeline",
    groupEpic: "按 Task",
    sortKeyDesc: "编号 ↓",
    sortKeyAsc: "编号 ↑",
    sortTitleAsc: "标题 A–Z",
    sortTitleDesc: "标题 Z–A",
    sortType: "类型",
    sortStatus: "状态",
    createTitle: "新建 Issue",
    createType: "类型",
    createPipeline: "Pipeline",
    createPipelineNone: "（无 — 仅跟踪 / Retro）",
    createTitleLabel: "标题",
    createProduct: "所属产品",
    createProductHint:
      "对应 products/<name>/ 目录；Guidance 与文档路径均以此为准。",
    createEpic: "关联 Epic（Task）",
    createEpicNone: "（不关联）",
    createTasks: "关联 Task（可多选）",
    createTasksHint: "语义关联已有 task；Agent 创建 Issue 请用 issue-author skill",
    createDescription: "描述",
    creating: "创建中…",
    createSubmit: "创建",
    workflowProfile: "工作流画像",
    unlinkedEpic: "未关联 Task",
    unlinkedEpicHint: "未绑定 task 的 Issue",
    noPipeline: "（无 Pipeline）",
    pipelinePrefix: "Pipeline",
    epicProgress: "{done}/{total} 已完成",
    emptyFiltered: "没有符合筛选条件的 Issue。",
    activeRun: "运行中",
    searchPlaceholder: "搜索编号、标题、Task、Pipeline…",
    filterTask: "Task",
    filterTaskAll: "全部 Task",
    statTotal: "全部",
    statInProgress: "进行中",
    statDone: "已完成",
    statActiveRuns: "有 Run",
    expandAll: "展开全部分组",
    collapseAll: "收起全部分组",
    colKey: "编号",
    colTitle: "标题",
    colTasks: "Task",
    colStatus: "状态",
    colPipeline: "Pipeline",
    viewByProduct: "按产品",
    viewByTask: "按 Task",
    openFullPage: "全屏打开",
    closeDetail: "关闭",
    emptyPanel: "暂无 Issue",
    selectHintMosaic: "点击卡片查看详情；支持全屏打开。",
    exportMarkdown: "导出 Markdown",
    exportCopied: "已复制到剪贴板",
    exportFailed: "复制失败，请检查浏览器权限",
  },
  project: {
    bootstrap: {
      title: "初始化 Popsicle 工作区？",
      body: "该目录尚未配置 Popsicle。确认后将创建轻量运行时环境，便于 Agent 与 pipeline 开工。",
      items: [
        "创建 .popsicle/ 与内置 pipelines",
        "安装 intent-coder 模块",
        "写入 project.yaml 与 AGENTS.md 片段",
      ],
      confirm: "初始化并打开",
      cancel: "取消",
    },
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
    workflowProfile: "Workflow profile",
    workflowProfileHint:
      "Default pipelines for new issues and UI emphasis (not RBAC).",
    profileDailyDev: "Daily development",
    profileMigration: "Large migration",
    profilePmSpec: "PM / spec authoring",
    profileOpcFull: "OPC full pipeline",
    approvalMode: "Pipeline approval mode",
    approvalManual: "Manual approval required",
    approvalAuto: "Fully automatic",
    approvalDelegate: "Dangerous stages need approval (delegate others)",
    approvalHint:
      "How requires_approval stages complete; dangerous: cutover, living-docs.",
    syncAgents: "Sync to AGENTS.md on save",
    injectOnRun: "Inject agent_context on issue create, issue start, and doc create",
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
    filterType: "Type",
    filterStatus: "Status",
    filterGroup: "Group",
    filterSort: "Sort",
    filterCount: "{n} issue(s)",
    typeAll: "All",
    typeProduct: "Product",
    typeTechnical: "Technical",
    typeBug: "Bug",
    typeIdea: "Idea",
    statusAll: "All statuses",
    statusBacklog: "Backlog",
    statusReady: "Ready",
    statusInProgress: "In progress",
    statusDone: "Done",
    groupFlat: "Flat list",
    groupProduct: "By product",
    groupPipeline: "By pipeline",
    groupEpic: "By task",
    sortKeyDesc: "Key ↓",
    sortKeyAsc: "Key ↑",
    sortTitleAsc: "Title A–Z",
    sortTitleDesc: "Title Z–A",
    sortType: "Type",
    sortStatus: "Status",
    createTitle: "Create issue",
    createType: "Type",
    createPipeline: "Pipeline",
    createPipelineNone: "(none — tracking / retro)",
    createTitleLabel: "Title",
    createProduct: "Product",
    createProductHint:
      "Maps to products/<name>/; guidance and doc paths use this product.",
    createEpic: "Epic (task)",
    createEpicNone: "(unlinked)",
    createTasks: "Linked tasks (multi-select)",
    createTasksHint: "Link existing tasks; agents should use issue-author skill",
    createDescription: "Description",
    creating: "Creating…",
    createSubmit: "Create",
    workflowProfile: "Workflow profile",
    unlinkedEpic: "Unlinked tasks",
    unlinkedEpicHint: "Issues without linked tasks",
    noPipeline: "(no pipeline)",
    pipelinePrefix: "Pipeline",
    epicProgress: "{done}/{total} done",
    emptyFiltered: "No issues match the current filters.",
    activeRun: "Active run",
    searchPlaceholder: "Search key, title, task, pipeline…",
    filterTask: "Task",
    filterTaskAll: "All tasks",
    statTotal: "All",
    statInProgress: "In progress",
    statDone: "Done",
    statActiveRuns: "Active runs",
    expandAll: "Expand all",
    collapseAll: "Collapse all",
    colKey: "Key",
    colTitle: "Title",
    colTasks: "Tasks",
    colStatus: "Status",
    colPipeline: "Pipeline",
    viewByProduct: "By product",
    viewByTask: "By task",
    openFullPage: "Open full page",
    closeDetail: "Close",
    emptyPanel: "No issues",
    selectHintMosaic: "Click a card for details, or open full page.",
    exportMarkdown: "Export Markdown",
    exportCopied: "Copied to clipboard",
    exportFailed: "Copy failed — check browser permissions",
  },
  project: {
    bootstrap: {
      title: "Initialize Popsicle workspace?",
      body: "This folder has no Popsicle setup yet. Confirm to create a lightweight runtime so agents and pipelines can start.",
      items: [
        "Create .popsicle/ and bundled pipelines",
        "Install the intent-coder module",
        "Write project.yaml and an AGENTS.md snippet",
      ],
      confirm: "Initialize and open",
      cancel: "Cancel",
    },
  },
};

export function messagesFor(locale: Locale): Messages {
  return locale === "zh-CN" ? zhCN : en;
}
