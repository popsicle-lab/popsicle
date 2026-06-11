import { useCallback, useEffect, useMemo, useState } from "react";
import { ArrowRight, ArrowUpDown, Plus, X } from "lucide-react";
import {
  createIssue,
  getCreateIssueFormOptions,
  listIssues,
  type CreateIssueFormOptions,
  type IssueInfo,
} from "../hooks/useTauri";
import {
  IssueTypeBadge,
  issueTypeAccentClass,
} from "../components/IssueTypeBadge";
import { StatusBadge } from "../components/StatusBadge";
import { useWideLayout } from "../hooks/useWideLayout";
import {
  ISSUE_SORT_OPTIONS,
  sortIssues,
  type IssueSortKey,
} from "../lib/issueSort";
import { IssueDetailView } from "./IssueDetailView";
import type { Page } from "../App";

interface Props {
  setPage: (p: Page) => void;
  initialSelectedKey?: string;
}

const TYPE_FILTERS = [
  { value: "all", label: "All" },
  { value: "product", label: "Product" },
  { value: "technical", label: "Technical" },
  { value: "bug", label: "Bug" },
  { value: "idea", label: "Idea" },
] as const;

const STATUS_OPTIONS = [
  { value: "all", label: "All statuses" },
  { value: "backlog", label: "Backlog" },
  { value: "ready", label: "Ready" },
  { value: "in_progress", label: "In progress" },
  { value: "done", label: "Done" },
];

export function IssuesView({ setPage, initialSelectedKey }: Props) {
  const wide = useWideLayout();
  const [allIssues, setAllIssues] = useState<IssueInfo[]>([]);
  const [selectedKey, setSelectedKey] = useState<string | null>(
    initialSelectedKey ?? null
  );
  const [typeFilter, setTypeFilter] = useState("all");
  const [statusFilter, setStatusFilter] = useState("all");
  const [sortKey, setSortKey] = useState<IssueSortKey>("key_desc");
  const [showCreate, setShowCreate] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (initialSelectedKey) setSelectedKey(initialSelectedKey);
  }, [initialSelectedKey]);

  const load = useCallback(() => {
    listIssues()
      .then(setAllIssues)
      .catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const issues = useMemo(() => {
    const filtered = allIssues.filter((i) => {
      if (typeFilter !== "all" && i.issue_type !== typeFilter) return false;
      if (statusFilter !== "all" && i.status !== statusFilter) return false;
      return true;
    });
    return sortIssues(filtered, sortKey);
  }, [allIssues, typeFilter, statusFilter, sortKey]);

  useEffect(() => {
    if (allIssues.length === 0) return;
    if (selectedKey && !allIssues.some((i) => i.key === selectedKey)) {
      setSelectedKey(null);
    }
  }, [allIssues, selectedKey]);

  const openIssue = (key: string) => {
    if (wide) {
      setSelectedKey(key);
    } else {
      setPage({ kind: "issue", issueKey: key });
    }
  };

  const filterBar = (
    <div className="issues-filter-bar shrink-0">
      <div className="issues-filter-row">
        <span className="filter-label">Type</span>
        <div className="filter-chips" role="group" aria-label="Filter by type">
          {TYPE_FILTERS.map((t) => (
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
      </div>
      <div className="issues-filter-row issues-filter-controls">
        <label className="filter-field">
          <span className="filter-label">Status</span>
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
            className="filter-select"
          >
            {STATUS_OPTIONS.map((o) => (
              <option key={o.value} value={o.value}>
                {o.label}
              </option>
            ))}
          </select>
        </label>
        <label className="filter-field">
          <span className="filter-label">
            <ArrowUpDown size={12} className="inline" /> Sort
          </span>
          <select
            value={sortKey}
            onChange={(e) => setSortKey(e.target.value as IssueSortKey)}
            className="filter-select"
          >
            {ISSUE_SORT_OPTIONS.map((o) => (
              <option key={o.value} value={o.value}>
                {o.label}
              </option>
            ))}
          </select>
        </label>
        <span className="filter-count">
          {issues.length} issue{issues.length === 1 ? "" : "s"}
        </span>
        <button
          type="button"
          onClick={() => setShowCreate(true)}
          className="btn btn-primary ml-auto shrink-0"
        >
          <Plus size={15} />
          New issue
        </button>
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
      <div className="page-frame flex flex-col gap-3">
        {filterBar}
        <div className="card border-[rgba(239,68,68,0.25)] bg-[rgba(239,68,68,0.08)] p-4 text-[13px] text-[var(--accent-red)]">
          {error}
        </div>
      </div>
    );
  }

  const listPane = (
    <div className={`flex min-h-0 flex-1 flex-col ${wide ? "master-panel" : ""}`}>
      <div className={wide ? "master-panel-scroll" : "space-y-2"}>
        {issues.map((issue) => {
          const active = wide && selectedKey === issue.key;
          const typeAccent = issueTypeAccentClass(issue.issue_type);
          const inner = (
            <div className="flex items-start justify-between gap-2">
              <div className="min-w-0">
                <div className="mb-1 flex flex-wrap items-center gap-1.5">
                  <span className="font-mono text-[12px] text-[var(--text-primary)]">
                    {issue.key}
                  </span>
                  <IssueTypeBadge type={issue.issue_type} />
                </div>
                <h3 className="line-clamp-2 text-[13px] font-medium leading-snug">
                  {issue.title}
                </h3>
                <div className="mt-1.5 flex items-center gap-2">
                  <StatusBadge status={issue.status} />
                  <span className="truncate text-[11px] text-[var(--text-muted)]">
                    {issue.product_id}
                  </span>
                </div>
              </div>
              {!wide && (
                <ArrowRight
                  size={16}
                  className="mt-0.5 shrink-0 text-[var(--text-muted)]"
                />
              )}
            </div>
          );

          if (wide) {
            return (
              <button
                key={issue.key}
                type="button"
                onClick={() => setSelectedKey(issue.key)}
                className={`list-item ${typeAccent} ${active ? "list-item-active" : ""}`}
              >
                {inner}
              </button>
            );
          }

          return (
            <button
              key={issue.key}
              type="button"
              onClick={() => openIssue(issue.key)}
              className={`card card-interactive w-full p-3.5 text-left issue-card-${issue.issue_type}`}
            >
              {inner}
            </button>
          );
        })}
        {issues.length === 0 && (
          <p className="empty-state">No issues match the current filters.</p>
        )}
      </div>
    </div>
  );

  if (!wide) {
    return (
      <div className="page-frame flex flex-col gap-3">
        {filterBar}
        {listPane}
      </div>
    );
  }

  return (
    <div className="page-frame flex h-full min-h-0 flex-col gap-3">
      {filterBar}
      <div className="master-detail min-h-0 flex-1">
        {listPane}
        <div className="detail-panel">
          {selectedKey ? (
            <IssueDetailView
              issueKey={selectedKey}
              setPage={setPage}
              variant="panel"
            />
          ) : (
            <div className="flex flex-1 flex-col items-center justify-center gap-2 p-8 text-center">
              <p className="text-[13px] font-medium text-[var(--text-secondary)]">
                Select an issue
              </p>
              <p className="max-w-xs text-[12px] text-[var(--text-muted)]">
                Use filters above, then pick a row to preview details and
                guidance.
              </p>
            </div>
          )}
        </div>
      </div>
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
  const [formOptions, setFormOptions] = useState<CreateIssueFormOptions | null>(
    null
  );
  const [issueType, setIssueType] = useState("technical");
  const [title, setTitle] = useState("");
  const [productId, setProductId] = useState("");
  const [pipeline, setPipeline] = useState("");
  const [description, setDescription] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    getCreateIssueFormOptions()
      .then((opts) => {
        setFormOptions(opts);
        setProductId(opts.default_product);
        setPipeline(defaultPipelineForType("technical", opts));
      })
      .catch(() => {
        setProductId("cli-ux");
      });
  }, []);

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
        pipeline,
        description,
      });
      onCreated(issue);
    } catch (err: unknown) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="card mt-2 p-3">
      <div className="mb-3 flex items-center justify-between">
        <h3 className="text-[13px] font-semibold">Create issue</h3>
        <button
          type="button"
          onClick={onClose}
          className="btn btn-ghost text-[var(--text-muted)]"
        >
          <X size={16} />
        </button>
      </div>
      <form onSubmit={submit} className="grid gap-2.5">
        <div className="grid grid-cols-2 gap-2">
          <label className="text-[12px] text-[var(--text-secondary)]">
            Type
            <select
              value={issueType}
              onChange={(e) => handleTypeChange(e.target.value)}
              className="input mt-1"
            >
              <option value="product">Product</option>
              <option value="technical">Technical</option>
              <option value="bug">Bug</option>
              <option value="idea">Idea</option>
            </select>
          </label>
          <label className="text-[12px] text-[var(--text-secondary)]">
            Pipeline
            <select
              value={pipeline}
              onChange={(e) => setPipeline(e.target.value)}
              className="input mt-1"
              required
            >
              {(formOptions?.pipeline_options ?? []).map((p) => (
                <option key={p} value={p}>
                  {p}
                </option>
              ))}
            </select>
          </label>
        </div>
        <label className="text-[12px] text-[var(--text-secondary)]">
          Title
          <input
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            required
            className="input mt-1"
          />
        </label>
        <label className="text-[12px] text-[var(--text-secondary)]">
          所属产品
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
            对应 <code className="text-[10px]">products/&lt;name&gt;/</code> 目录；Guidance 与文档路径均以此为准。
          </span>
        </label>
        <label className="text-[12px] text-[var(--text-secondary)]">
          Description
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
          disabled={
            loading || !title.trim() || !productId.trim() || !pipeline.trim()
          }
          className="btn btn-primary w-fit"
        >
          {loading ? "Creating…" : "Create"}
        </button>
      </form>
    </div>
  );
}
