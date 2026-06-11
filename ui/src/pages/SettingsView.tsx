import { useCallback, useEffect, useState } from "react";
import {
  getProjectConfig,
  saveProjectConfig,
  type ProjectConfigDto,
} from "../hooks/useTauri";

interface Props {
  onSaved?: () => void;
}

export function SettingsView({ onSaved }: Props) {
  const [config, setConfig] = useState<ProjectConfigDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const cfg = await getProjectConfig();
      setConfig(cfg);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const update = <K extends keyof ProjectConfigDto>(
    key: K,
    value: ProjectConfigDto[K]
  ) => {
    setConfig((c) => (c ? { ...c, [key]: value } : c));
    setSaved(false);
  };

  const handleSave = async () => {
    if (!config) return;
    setSaving(true);
    setError(null);
    try {
      const next = await saveProjectConfig({
        language: config.language,
        products_dir: config.products_dir,
        default_spec: config.default_spec,
        sync_agents_md: config.sync_agents_md,
        inject_on_run: config.inject_on_run,
        approval_mode: config.approval_mode,
      });
      setConfig(next);
      setSaved(true);
      onSaved?.();
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
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
        {error ?? "无法加载项目配置"}
      </div>
    );
  }

  return (
    <div className="mx-auto flex h-full max-w-2xl flex-col gap-6 overflow-y-auto pb-8">
      <div>
        <h2 className="text-lg font-semibold tracking-tight">项目设置</h2>
        <p className="mt-1 text-[13px] text-[var(--text-muted)]">
          配置会写入{" "}
          <code className="rounded bg-[var(--bg-elevated)] px-1.5 py-0.5 text-[12px]">
            .popsicle/project.yaml
          </code>
          ，并可同步到根目录{" "}
          <code className="rounded bg-[var(--bg-elevated)] px-1.5 py-0.5 text-[12px]">
            AGENTS.md
          </code>
          。
        </p>
      </div>

      <section className="card space-y-5 p-5">
        <div>
          <label className="mb-1.5 block text-[13px] font-medium">
            Agent 回复语言
          </label>
          <select
            className="filter-select w-full max-w-xs"
            value={config.language}
            onChange={(e) => update("language", e.target.value)}
          >
            <option value="zh-CN">简体中文</option>
            <option value="en">English</option>
          </select>
        </div>

        <div>
          <label className="mb-1.5 block text-[13px] font-medium">
            产品文档目录
          </label>
          <input
            type="text"
            className="input w-full max-w-md"
            value={config.products_dir}
            onChange={(e) => update("products_dir", e.target.value)}
            placeholder="products"
          />
          <p className="mt-1 text-[12px] text-[var(--text-muted)]">
            相对仓库根目录，Products 浏览器与 pipeline 文档路径均以此为准。
          </p>
        </div>

        <div>
          <label className="mb-1.5 block text-[13px] font-medium">
            默认 spec（可选）
          </label>
          <input
            type="text"
            className="input w-full max-w-md"
            value={config.default_spec}
            onChange={(e) => update("default_spec", e.target.value)}
            placeholder="slice-3-cli-ux"
          />
        </div>

        <div>
          <label className="mb-1.5 block text-[13px] font-medium">
            Pipeline 审批模式
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
            <option value="manual">必须人工审批</option>
            <option value="auto">全自动</option>
            <option value="delegate-dangerous">
              危险操作需审批（其余代批）
            </option>
          </select>
          <p className="mt-1 text-[12px] text-[var(--text-muted)]">
            控制带 <code className="text-[11px]">requires_approval</code>{" "}
            阶段的完成方式；危险阶段为 cutover、living-docs。
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
            保存时同步到 AGENTS.md
          </label>
          <label className="flex cursor-pointer items-center gap-2.5 text-[13px]">
            <input
              type="checkbox"
              className="h-4 w-4 rounded border-[var(--border-strong)]"
              checked={config.inject_on_run}
              onChange={(e) => update("inject_on_run", e.target.checked)}
            />
            工作流启动 / 创建文档时注入偏好到提示词
          </label>
        </div>
      </section>

      <div className="flex items-center gap-3">
        <button
          type="button"
          className="btn-primary"
          disabled={saving}
          onClick={handleSave}
        >
          {saving ? "保存中…" : "保存配置"}
        </button>
        {saved && (
          <span className="text-[13px] text-[var(--accent-green)]">已保存</span>
        )}
        {error && (
          <span className="text-[13px] text-[var(--accent-red)]">{error}</span>
        )}
      </div>

      <p className="text-[12px] text-[var(--text-muted)]">
        配置文件：{config.config_path}
      </p>
    </div>
  );
}
