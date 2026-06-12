import { useCallback, useEffect, useMemo, useState } from "react";
import {
  ArrowUpDown,
  Box,
  ChevronDown,
  ChevronRight,
  ChevronsDownUp,
  ChevronsUpDown,
  ClipboardList,
  Layers,
  Plus,
  Search,
  X,
} from "lucide-react";
import {
  createIssue,
  getCreateIssueFormOptions,
  listIssues,
  scanProductTaskGraph,
  type CreateIssueFormOptions,
  type IssueInfo,
  type TaskNode,
} from "../hooks/useTauri";
import { sortIssues, type IssueSortKey } from "../lib/issueSort";
import { buildTaskMetaMap, groupIssues, type IssueGroup } from "../lib/issueGroup";
import { computeIssueStats, filterIssues } from "../lib/issueListUtils";
import { formatIssuesMarkdownBrief } from "../lib/issueExportMarkdown";
import {
  issueSortOptions,
  issueTypeLabel,
  workflowProfileLabel,
} from "../lib/issueLabels";
import { IssueBoardCard } from "../components/IssueBoardCard";
import { IssueDetailModal } from "../components/IssueDetailModal";
import { useLocale } from "../i18n/LocaleContext";
import type { Page } from "../App";

interface Props {
  setPage: (p: Page) => void;
  initialSelectedKey?: string;
}

type LayoutMode = "product" | "task";

const TYPE_FILTER_VALUES = [
  "all",
  "product",
  "technical",
  "bug",
  "idea",
] as const;

const JOURNEY_LABELS: Record<string, string> = {
  onboarding: "Onboarding",
  "daily-ops": "Daily ops",
  troubleshooting: "Troubleshooting",
  admin: "Admin",
  lifecycle: "Lifecycle",
};

const JOURNEY_ORDER = [
  "onboarding",
  "daily-ops",
  "troubleshooting",
  "admin",
  "lifecycle",
];

const UNLINKED_KEY = "__unlinked__";

function sortMosaicGroups(groups: IssueGroup[], mode: LayoutMode): IssueGroup[] {
  return [...groups].sort((a, b) => {
    if (mode === "task") {
      if (a.key === UNLINKED_KEY) return 1;
      if (b.key === UNLINKED_KEY) return -1;
      const ja = JOURNEY_ORDER.indexOf(a.journeyStage ?? "");
      const jb = JOURNEY_ORDER.indexOf(b.journeyStage ?? "");
      if (ja !== jb) return ja - jb;
    }
    return (a.subtitle ?? a.label).localeCompare(b.subtitle ?? b.label);
  });
}

function enrichTaskPanels(
  groups: IssueGroup[],
  taskMeta: ReturnType<typeof buildTaskMetaMap>
): IssueGroup[] {
  const keys = new Set(groups.map((g) => g.key));
  const extra: IssueGroup[] = [];
  for (const [tid, meta] of taskMeta) {
    if (keys.has(tid)) continue;
    extra.push({
      key: tid,
      label: meta.title,
      subtitle: tid,
      epicTaskId: tid,
      journeyStage: meta.journey_stage,
      issues: [],
      doneCount: 0,
    });
  }
  return [...groups, ...extra];
}

export function IssuesView({ setPage, initialSelectedKey }: Props) {
  const { m } = useLocale();
  const [allIssues, setAllIssues] = useState<IssueInfo[]>([]);
  const [modalKey, setModalKey] = useState<string | null>(initialSelectedKey ?? null);
  const [layoutMode, setLayoutMode] = useState<LayoutMode>("product");
  const [typeFilter, setTypeFilter] = useState("all");
  const [statusFilter, setStatusFilter] = useState("all");
  const [sortKey, setSortKey] = useState<IssueSortKey>("key_desc");
  const [showCreate, setShowCreate] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [taskMeta, setTaskMeta] = useState(buildTaskMetaMap([]));
  const [searchQuery, setSearchQuery] = useState("");
  const [exportNotice, setExportNotice] = useState<string | null>(null);
  const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(new Set());

  const typeFilters = useMemo(
    () =>
      TYPE_FILTER_VALUES.map((value) => ({
        value,
        label:
          value === "all" ? m.issues.typeAll : issueTypeLabel(value, m),
      })),
    [m]
  );

  const groupLabels = useMemo(
    () => ({
      unlinkedEpic: m.issues.unlinkedEpic,
      noPipeline: m.issues.noPipeline,
      pipelinePrefix: m.issues.pipelinePrefix,
    }),
    [m]
  );

  useEffect(() => {
    if (initialSelectedKey) setModalKey(initialSelectedKey);
  }, [initialSelectedKey]);

  const load = useCallback(() => {
    listIssues()
      .then(setAllIssues)
      .catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  useEffect(() => {
    const products = [
      ...new Set(allIssues.map((i) => i.product_id).filter(Boolean)),
    ];
    if (products.length === 0) {
      setTaskMeta(buildTaskMetaMap([]));
      return;
    }
    Promise.all(
      products.map((p) =>
        scanProductTaskGraph(p).catch(() => ({ nodes: [] as TaskNode[] }))
      )
    ).then((graphs) => {
      const nodes = graphs.flatMap((g) => g.nodes);
      setTaskMeta(buildTaskMetaMap(nodes));
    });
  }, [allIssues]);

  const issues = useMemo(() => {
    const filtered = allIssues.filter((i) => {
      if (typeFilter !== "all" && i.issue_type !== typeFilter) return false;
      if (statusFilter !== "all" && i.status !== statusFilter) return false;
      return true;
    });
    return sortIssues(
      filterIssues(filtered, { search: searchQuery, taskId: null }),
      sortKey
    );
  }, [allIssues, typeFilter, statusFilter, sortKey, searchQuery]);

  const stats = useMemo(() => computeIssueStats(issues), [issues]);

  const mosaicGroups = useMemo(() => {
    const base =
      layoutMode === "product"
        ? groupIssues(issues, "product", taskMeta, groupLabels)
        : enrichTaskPanels(
            groupIssues(issues, "epic", taskMeta, groupLabels),
            taskMeta
          );
    return sortMosaicGroups(base, layoutMode);
  }, [issues, layoutMode, taskMeta, groupLabels]);

  const mosaicGroupKeys = useMemo(
    () => mosaicGroups.map((g) => g.key),
    [mosaicGroups]
  );

  useEffect(() => {
    setCollapsedGroups(new Set());
  }, [layoutMode]);

  const toggleGroup = useCallback((key: string) => {
    setCollapsedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }, []);

  const expandAllGroups = () => setCollapsedGroups(new Set());
  const collapseAllGroups = () => setCollapsedGroups(new Set(mosaicGroupKeys));

  const openIssue = (key: string) => setModalKey(key);

  const renderMosaicGroup = (
    group: IssueGroup,
    opts: { showProduct?: boolean; taskMode?: boolean }
  ) => {
    const collapsed = collapsedGroups.has(group.key);
    const panelClass = opts.taskMode
      ? `issues-mosaic-panel issues-mosaic-panel-column ${
          group.key === UNLINKED_KEY ? "issues-mosaic-panel-unlinked" : ""
        }`
      : "issues-mosaic-panel";

    return (
      <section key={group.key} className={panelClass}>
        <button
          type="button"
          className="issues-mosaic-panel-head"
          onClick={() => toggleGroup(group.key)}
          aria-expanded={!collapsed}
        >
          <span className="issues-mosaic-panel-chevron" aria-hidden>
            {collapsed ? <ChevronRight size={16} /> : <ChevronDown size={16} />}
          </span>
          <div className="min-w-0 flex-1 text-left">
            {opts.taskMode && group.subtitle && (
              <span className="issues-mosaic-task-id">{group.subtitle}</span>
            )}
            <h2 className="issues-mosaic-panel-title">{group.label}</h2>
            {opts.taskMode && group.journeyStage ? (
              <span className="issues-mosaic-journey">
                {JOURNEY_LABELS[group.journeyStage] ?? group.journeyStage}
              </span>
            ) : (
              <p className="issues-mosaic-panel-sub">
                {group.doneCount}/{group.issues.length} {m.issues.statDone}
              </p>
            )}
          </div>
          <span className="issues-mosaic-panel-count">{group.issues.length}</span>
        </button>
        {!collapsed && (
          <div className="issues-mosaic-panel-body">
            <div
              className={`issues-mosaic-grid ${opts.taskMode ? "issues-mosaic-grid-task" : ""}`}
            >
              {group.issues.length === 0 ? (
                <p className="issues-mosaic-empty">{m.issues.emptyPanel}</p>
              ) : (
                group.issues.map((issue) => (
                  <IssueBoardCard
                    key={`${group.key}-${issue.key}`}
                    issue={issue}
                    onClick={() => openIssue(issue.key)}
                    showProduct={opts.showProduct}
                  />
                ))
              )}
            </div>
          </div>
        )}
      </section>
    );
  };

  const handleExportMarkdown = useCallback(async () => {
    const md = formatIssuesMarkdownBrief(issues, {
      layoutMode,
      typeFilter,
      statusFilter,
      searchQuery,
    });
    try {
      await navigator.clipboard.writeText(md);
      setExportNotice(m.issues.exportCopied);
    } catch {
      setExportNotice(m.issues.exportFailed);
    }
    window.setTimeout(() => setExportNotice(null), 3000);
  }, [issues, layoutMode, typeFilter, statusFilter, searchQuery, m]);

  const toolbar = (
    <div className="issues-mosaic-toolbar shrink-0">
      <div className="issues-mosaic-toolbar-row">
        <div className="issues-layout-switch" role="tablist">
          <button
            type="button"
            role="tab"
            aria-selected={layoutMode === "product"}
            className={`issues-layout-tab ${layoutMode === "product" ? "issues-layout-tab-active" : ""}`}
            onClick={() => setLayoutMode("product")}
          >
            <Box size={15} />
            {m.issues.viewByProduct}
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={layoutMode === "task"}
            className={`issues-layout-tab ${layoutMode === "task" ? "issues-layout-tab-active" : ""}`}
            onClick={() => setLayoutMode("task")}
          >
            <Layers size={15} />
            {m.issues.viewByTask}
          </button>
        </div>
        <div className="issues-search-wrap">
          <Search size={15} className="issues-search-icon" />
          <input
            type="search"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder={m.issues.searchPlaceholder}
            className="issues-search-input"
          />
          {searchQuery && (
            <button
              type="button"
              className="issues-search-clear"
              onClick={() => setSearchQuery("")}
              aria-label="Clear"
            >
              <X size={14} />
            </button>
          )}
        </div>
        <button
          type="button"
          onClick={() => void handleExportMarkdown()}
          className="btn btn-secondary shrink-0"
          title={m.issues.exportMarkdown}
        >
          <ClipboardList size={15} />
          {m.issues.exportMarkdown}
        </button>
        {exportNotice && (
          <span className="issues-export-notice" role="status">
            {exportNotice}
          </span>
        )}
        <button
          type="button"
          onClick={() => setShowCreate(true)}
          className="btn btn-primary shrink-0"
        >
          <Plus size={15} />
          {m.issues.createIssue}
        </button>
      </div>
      <div className="issues-mosaic-toolbar-row issues-mosaic-toolbar-sub">
        <div className="issues-stat-strip" role="group">
          <button
            type="button"
            className={`issues-stat-pill ${statusFilter === "all" ? "issues-stat-pill-active" : ""}`}
            onClick={() => setStatusFilter("all")}
          >
            <span className="issues-stat-value">{stats.total}</span>
            <span className="issues-stat-label">{m.issues.statTotal}</span>
          </button>
          <button
            type="button"
            className={`issues-stat-pill ${statusFilter === "in_progress" ? "issues-stat-pill-active" : ""}`}
            onClick={() =>
              setStatusFilter((s) => (s === "in_progress" ? "all" : "in_progress"))
            }
          >
            <span className="issues-stat-value">{stats.inProgress}</span>
            <span className="issues-stat-label">{m.issues.statInProgress}</span>
          </button>
          <button
            type="button"
            className={`issues-stat-pill ${statusFilter === "done" ? "issues-stat-pill-active" : ""}`}
            onClick={() => setStatusFilter((s) => (s === "done" ? "all" : "done"))}
          >
            <span className="issues-stat-value">{stats.done}</span>
            <span className="issues-stat-label">{m.issues.statDone}</span>
          </button>
        </div>
        <div className="filter-chips" role="group">
          {typeFilters.map((t) => (
            <button
              key={t.value}
              type="button"
              onClick={() => setTypeFilter(t.value)}
              className={`filter-chip ${typeFilter === t.value ? "filter-chip-active" : ""} ${
                t.value !== "all" ? `filter-chip-${t.value}` : ""
              }`}
            >
              {t.label}
            </button>
          ))}
        </div>
        <label className="filter-field filter-field-inline">
          <span className="filter-label">
            <ArrowUpDown size={12} className="inline" /> {m.issues.filterSort}
          </span>
          <select
            value={sortKey}
            onChange={(e) => setSortKey(e.target.value as IssueSortKey)}
            className="filter-select"
          >
            {issueSortOptions(m).map((o) => (
              <option key={o.value} value={o.value}>
                {o.label}
              </option>
            ))}
          </select>
        </label>
        {mosaicGroupKeys.length > 0 && (
          <div className="issues-group-actions">
            <button
              type="button"
              className="btn btn-ghost text-[11px]"
              onClick={expandAllGroups}
            >
              <ChevronsDownUp size={14} />
              {m.issues.expandAll}
            </button>
            <button
              type="button"
              className="btn btn-ghost text-[11px]"
              onClick={collapseAllGroups}
            >
              <ChevronsUpDown size={14} />
              {m.issues.collapseAll}
            </button>
          </div>
        )}
        <span className="filter-count">
          {m.issues.filterCount.replace("{n}", String(issues.length))}
        </span>
      </div>
      {showCreate && (
        <CreateIssueForm
          onClose={() => setShowCreate(false)}
          onCreated={(issue) => {
            setShowCreate(false);
            load();
            openIssue(issue.key);
          }}
        />
      )}
    </div>
  );

  if (error) {
    return (
      <div className="page-frame issues-mosaic-page">
        {toolbar}
        <div className="card border-[rgba(239,68,68,0.25)] bg-[rgba(239,68,68,0.08)] p-4 text-[13px] text-[var(--accent-red)]">
          {error}
        </div>
      </div>
    );
  }

  return (
    <div className="page-frame issues-mosaic-page">
      {toolbar}
      <div className="issues-mosaic-scroll">
        {issues.length === 0 ? (
          <p className="empty-state">{m.issues.emptyFiltered}</p>
        ) : layoutMode === "product" ? (
          <div className="issues-mosaic-product-stack">
            {mosaicGroups.map((group) => renderMosaicGroup(group, {}))}
          </div>
        ) : (
          <div className="issues-mosaic-task-grid">
            {mosaicGroups.map((group) =>
              renderMosaicGroup(group, { showProduct: true, taskMode: true })
            )}
          </div>
        )}
      </div>
      {modalKey && (
        <IssueDetailModal
          issueKey={modalKey}
          onClose={() => setModalKey(null)}
          setPage={setPage}
        />
      )}
    </div>
  );
}

function defaultPipelineForType(
  issueType: string,
  options: CreateIssueFormOptions | null
): string {
  if (!options) return "";
  return options.default_pipeline_by_type[issueType] ?? "";
}

function CreateIssueForm({
  onClose,
  onCreated,
}: {
  onClose: () => void;
  onCreated: (issue: IssueInfo) => void;
}) {
  const { m } = useLocale();
  const [formOptions, setFormOptions] = useState<CreateIssueFormOptions | null>(
    null
  );
  const [taskOptions, setTaskOptions] = useState<TaskNode[]>([]);
  const [issueType, setIssueType] = useState("technical");
  const [title, setTitle] = useState("");
  const [productId, setProductId] = useState("");
  const [pipeline, setPipeline] = useState("");
  const [description, setDescription] = useState("");
  const [linkedTaskIds, setLinkedTaskIds] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    getCreateIssueFormOptions()
      .then((opts) => {
        setFormOptions(opts);
        setProductId(opts.default_product);
        setPipeline(defaultPipelineForType("technical", opts));
        setTaskOptions(
          opts.task_options.map((t) => ({
            task_id: t.task_id,
            title: t.title,
            journey_stage: t.journey_stage,
            product: opts.default_product,
            file_path: "",
            related_next_tasks: [],
            related_intents: [],
          }))
        );
      })
      .catch(() => {
        setProductId("cli-ux");
      });
  }, []);

  useEffect(() => {
    if (!productId) return;
    scanProductTaskGraph(productId)
      .then((g) => setTaskOptions(g.nodes))
      .catch(() => setTaskOptions([]));
    setLinkedTaskIds([]);
  }, [productId]);

  const toggleLinkedTask = (taskId: string) => {
    setLinkedTaskIds((prev) =>
      prev.includes(taskId)
        ? prev.filter((id) => id !== taskId)
        : [...prev, taskId]
    );
  };

  const handleTypeChange = (nextType: string) => {
    setIssueType(nextType);
    setPipeline(defaultPipelineForType(nextType, formOptions));
  };

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      const issue = await createIssue({
        issueType,
        title,
        productId,
        pipeline: pipeline || undefined,
        description,
        linkedTaskIds: linkedTaskIds.length > 0 ? linkedTaskIds : undefined,
      });
      onCreated(issue);
    } catch (err: unknown) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const issueTypes = ["product", "technical", "bug", "idea"] as const;

  return (
    <div className="card mt-2 p-3">
      <div className="mb-3 flex items-center justify-between">
        <h3 className="text-[13px] font-semibold">{m.issues.createTitle}</h3>
        <button
          type="button"
          onClick={onClose}
          className="btn btn-ghost text-[var(--text-muted)]"
        >
          <X size={16} />
        </button>
      </div>
      {formOptions?.workflow_profile && (
        <p className="mb-2 text-[11px] text-[var(--text-muted)]">
          {m.issues.workflowProfile}:{" "}
          <span className="text-[var(--text-secondary)]">
            {workflowProfileLabel(formOptions.workflow_profile, m)}
          </span>
        </p>
      )}
      <form onSubmit={submit} className="grid gap-2.5">
        <div className="grid grid-cols-2 gap-2">
          <label className="text-[12px] text-[var(--text-secondary)]">
            {m.issues.createType}
            <select
              value={issueType}
              onChange={(e) => handleTypeChange(e.target.value)}
              className="input mt-1"
            >
              {issueTypes.map((t) => (
                <option key={t} value={t}>
                  {issueTypeLabel(t, m)}
                </option>
              ))}
            </select>
          </label>
          <label className="text-[12px] text-[var(--text-secondary)]">
            {m.issues.createPipeline}
            <select
              value={pipeline}
              onChange={(e) => setPipeline(e.target.value)}
              className="input mt-1"
            >
              <option value="">{m.issues.createPipelineNone}</option>
              {(formOptions?.pipeline_options ?? []).map((p) => (
                <option key={p} value={p}>
                  {p}
                </option>
              ))}
            </select>
          </label>
        </div>
        <label className="text-[12px] text-[var(--text-secondary)]">
          {m.issues.createTitleLabel}
          <input
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            required
            className="input mt-1"
          />
        </label>
        <label className="text-[12px] text-[var(--text-secondary)]">
          {m.issues.createProduct}
          <select
            value={productId}
            onChange={(e) => setProductId(e.target.value)}
            className="input mt-1"
            required
          >
            {(formOptions?.product_options ?? [productId].filter(Boolean)).map(
              (p) => (
                <option key={p} value={p}>
                  {p}
                </option>
              )
            )}
          </select>
          <span className="mt-1 block text-[11px] text-[var(--text-muted)]">
            {m.issues.createProductHint}
          </span>
        </label>
        <div className="text-[12px] text-[var(--text-secondary)]">
          <span>{m.issues.createTasks}</span>
          <span className="mt-1 block text-[11px] text-[var(--text-muted)]">
            {m.issues.createTasksHint}
          </span>
          <div className="mt-1 max-h-32 space-y-1 overflow-y-auto rounded-[var(--radius-sm)] border border-[var(--border)] p-2">
            {taskOptions.length === 0 ? (
              <p className="text-[11px] text-[var(--text-muted)]">
                {m.issues.createEpicNone}
              </p>
            ) : (
              taskOptions.map((t) => (
                <label
                  key={t.task_id}
                  className="flex cursor-pointer items-center gap-2 rounded px-1 py-0.5 hover:bg-[var(--bg-hover)]"
                >
                  <input
                    type="checkbox"
                    checked={linkedTaskIds.includes(t.task_id)}
                    onChange={() => toggleLinkedTask(t.task_id)}
                  />
                  <span className="font-mono text-[11px] text-[#93c5fd]">
                    {t.task_id}
                  </span>
                  <span className="min-w-0 flex-1 truncate text-[12px]">
                    {t.title}
                  </span>
                </label>
              ))
            )}
          </div>
        </div>
        <label className="text-[12px] text-[var(--text-secondary)]">
          {m.issues.createDescription}
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={2}
            className="input mt-1 resize-y"
          />
        </label>
        {error && (
          <p className="text-[13px] text-[var(--accent-red)]">{error}</p>
        )}
        <button
          type="submit"
          disabled={loading || !title.trim() || !productId.trim()}
          className="btn btn-primary w-fit"
        >
          {loading ? m.issues.creating : m.issues.createSubmit}
        </button>
      </form>
    </div>
  );
}
