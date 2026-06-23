import { useCallback, useEffect, useState } from "react";
import {
  getProjectConfig,
  getProjectContextMd,
  saveProjectConfig,
  saveProjectContextMd,
  type ProjectConfigDto,
  type ProjectContextDto,
} from "../hooks/useTauri";
import { normalizeLocale, useLocale } from "../i18n/LocaleContext";
import type { Page } from "../App";

interface Props {
  setPage?: (p: Page) => void;
  onSaved?: () => void;
}

export function SettingsView({ setPage, onSaved }: Props) {
  const { m, setLocale } = useLocale();
  const [config, setConfig] = useState<ProjectConfigDto | null>(null);
  const [projectContext, setProjectContext] = useState<ProjectContextDto | null>(
    null
  );
  const [contextDraft, setContextDraft] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [savingContext, setSavingContext] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const [contextSaved, setContextSaved] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [cfg, ctx] = await Promise.all([
        getProjectConfig(),
        getProjectContextMd(),
      ]);
      setConfig(cfg);
      setProjectContext(ctx);
      setContextDraft(ctx.content);
      setLocale(normalizeLocale(cfg.language));
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [setLocale]);

  useEffect(() => {
    load();
  }, [load]);

  const update = <K extends keyof ProjectConfigDto>(
    key: K,
    value: ProjectConfigDto[K]
  ) => {
    setConfig((c) => (c ? { ...c, [key]: value } : c));
    setSaved(false);
    if (key === "language" && typeof value === "string") {
      setLocale(normalizeLocale(value));
    }
  };

  const handleSave = async () => {
    if (!config) return;
    setSaving(true);
    setError(null);
    try {
      const next = await saveProjectConfig({
        language: config.language,
        products_dir: config.products_dir,
        default_product: config.default_product,
        workflow_profile: config.workflow_profile,
        sync_agents_md: config.sync_agents_md,
        inject_on_run: config.inject_on_run,
        approval_mode: config.approval_mode,
      });
      setConfig(next);
      setLocale(normalizeLocale(next.language));
      setSaved(true);
      onSaved?.();
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  const handleSaveContext = async () => {
    setSavingContext(true);
    setError(null);
    try {
      const next = await saveProjectContextMd(contextDraft);
      setProjectContext(next);
      setContextDraft(next.content);
      setContextSaved(true);
    } catch (e) {
      setError(String(e));
    } finally {
      setSavingContext(false);
    }
  };

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center text-[var(--text-muted)]">
        <div className="spinner" aria-hidden />
      </div>
    );
  }

  if (!config) {
    return (
      <div className="card border-[rgba(239,68,68,0.25)] bg-[rgba(239,68,68,0.08)] p-6 text-sm text-[var(--accent-red)]">
        {error ?? m.settings.loadError}
      </div>
    );
  }

  return (
    <div className="mx-auto flex h-full max-w-2xl flex-col gap-6 overflow-y-auto pb-8">
      <div>
        <h2 className="text-lg font-semibold tracking-tight">{m.settings.title}</h2>
        <p className="mt-1 text-[13px] text-[var(--text-muted)]">
          {m.settings.intro}
        </p>
      </div>

      <section className="card space-y-5 p-5">
        <div>
          <label className="mb-1.5 block text-[13px] font-medium">
            {m.settings.language}
          </label>
          <select
            className="filter-select w-full max-w-xs"
            value={config.language}
            onChange={(e) => update("language", e.target.value)}
          >
            <option value="zh-CN">简体中文</option>
            <option value="en">English</option>
          </select>
          <p className="mt-1 text-[12px] text-[var(--text-muted)]">
            {m.settings.languageHint}
          </p>
        </div>

        <div>
          <label className="mb-1.5 block text-[13px] font-medium">
            {m.settings.productsDir}
          </label>
          <input
            type="text"
            className="input w-full max-w-md"
            value={config.products_dir}
            onChange={(e) => update("products_dir", e.target.value)}
            placeholder="products"
          />
          <p className="mt-1 text-[12px] text-[var(--text-muted)]">
            {m.settings.productsDirHint}
          </p>
        </div>

        <div>
          <label className="mb-1.5 block text-[13px] font-medium">
            {m.settings.defaultProduct}
          </label>
          <select
            className="filter-select w-full max-w-md"
            value={config.default_product}
            onChange={(e) => update("default_product", e.target.value)}
          >
            <option value="">{m.settings.defaultProductNone}</option>
            {config.product_options.map((p) => (
              <option key={p} value={p}>
                {p}
              </option>
            ))}
          </select>
          <p className="mt-1 text-[12px] text-[var(--text-muted)]">
            {m.settings.defaultProductHint}
          </p>
        </div>

        <div>
          <label className="mb-1.5 block text-[13px] font-medium">
            {m.settings.workflowProfile}
          </label>
          <select
            className="filter-select w-full max-w-md"
            value={config.workflow_profile}
            onChange={(e) => update("workflow_profile", e.target.value)}
          >
            <option value="daily-dev">{m.settings.profileDailyDev}</option>
            <option value="migration">{m.settings.profileMigration}</option>
            <option value="pm-spec-only">{m.settings.profilePmSpec}</option>
            <option value="opc-full">{m.settings.profileOpcFull}</option>
          </select>
          <p className="mt-1 text-[12px] text-[var(--text-muted)]">
            {m.settings.workflowProfileHint}
          </p>
          {setPage && (
            <button
              type="button"
              className="mt-2 text-[12px] text-[var(--accent)] hover:underline"
              onClick={() => setPage({ kind: "workflows", tab: "pipelines" })}
            >
              {m.settings.browseWorkflows}
            </button>
          )}
        </div>

        <div>
          <label className="mb-1.5 block text-[13px] font-medium">
            {m.settings.approvalMode}
          </label>
          <select
            className="filter-select w-full max-w-md"
            value={config.approval_mode}
            onChange={(e) =>
              update(
                "approval_mode",
                e.target.value as ProjectConfigDto["approval_mode"]
              )
            }
          >
            <option value="manual">{m.settings.approvalManual}</option>
            <option value="auto">{m.settings.approvalAuto}</option>
            <option value="delegate-dangerous">
              {m.settings.approvalDelegate}
            </option>
          </select>
          <p className="mt-1 text-[12px] text-[var(--text-muted)]">
            {m.settings.approvalHint}
          </p>
        </div>

        <div className="space-y-3 border-t border-[var(--border)] pt-4">
          <label className="flex cursor-pointer items-center gap-2.5 text-[13px]">
            <input
              type="checkbox"
              className="h-4 w-4 rounded border-[var(--border-strong)]"
              checked={config.sync_agents_md}
              onChange={(e) => update("sync_agents_md", e.target.checked)}
            />
            {m.settings.syncAgents}
          </label>
          <label className="flex cursor-pointer items-center gap-2.5 text-[13px]">
            <input
              type="checkbox"
              className="h-4 w-4 rounded border-[var(--border-strong)]"
              checked={config.inject_on_run}
              onChange={(e) => update("inject_on_run", e.target.checked)}
            />
            {m.settings.injectOnRun}
          </label>
        </div>
      </section>

      <section className="card space-y-4 p-5">
        <div>
          <h3 className="text-[14px] font-semibold">{m.settings.projectContext}</h3>
          <p className="mt-1 text-[12px] text-[var(--text-muted)]">
            {m.settings.projectContextHint}
          </p>
        </div>
        <textarea
          className="input min-h-[280px] w-full font-mono text-[12px] leading-relaxed"
          value={contextDraft}
          onChange={(e) => {
            setContextDraft(e.target.value);
            setContextSaved(false);
          }}
          spellCheck={false}
        />
        <div className="flex items-center gap-3">
          <button
            type="button"
            className="btn-primary"
            disabled={savingContext}
            onClick={handleSaveContext}
          >
            {savingContext
              ? m.settings.projectContextSaving
              : m.settings.projectContextSave}
          </button>
          {contextSaved && (
            <span className="text-[13px] text-[var(--accent-green)]">
              {m.settings.projectContextSaved}
            </span>
          )}
        </div>
        {projectContext && (
          <p className="text-[12px] text-[var(--text-muted)]">
            {m.settings.projectContextPath}：{projectContext.path}
          </p>
        )}
      </section>

      <div className="flex items-center gap-3">
        <button
          type="button"
          className="btn-primary"
          disabled={saving}
          onClick={handleSave}
        >
          {saving ? m.settings.saving : m.settings.save}
        </button>
        {saved && (
          <span className="text-[13px] text-[var(--accent-green)]">
            {m.settings.saved}
          </span>
        )}
        {error && (
          <span className="text-[13px] text-[var(--accent-red)]">{error}</span>
        )}
      </div>

      <p className="text-[12px] text-[var(--text-muted)]">
        {m.settings.configPath}：{config.config_path}
      </p>
    </div>
  );
}
