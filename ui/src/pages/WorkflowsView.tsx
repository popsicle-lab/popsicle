import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Background,
  Controls,
  ReactFlow,
} from "@xyflow/react";
import {
  BookOpen,
  CheckCircle2,
  GitBranch,
  ShieldAlert,
  Sparkles,
} from "lucide-react";
import {
  getPipelineStatus,
  getWorkflowCatalog,
  type PipelineCatalogEntry,
  type PipelineStatusFull,
  type SkillCatalogEntry,
  type WorkflowCatalog,
} from "../hooks/useTauri";
import { layoutStageDag } from "../lib/pipelineLayout";
import { LoadingState } from "../components/LoadingState";
import { StatusBadge } from "../components/StatusBadge";
import { useLocale } from "../i18n/LocaleContext";
import type { Page } from "../App";

type Tab = "pipelines" | "skills";

interface Props {
  setPage: (p: Page) => void;
  tab?: Tab;
  pipeline?: string;
  skill?: string;
  contextRunId?: string;
  contextIssueKey?: string;
  highlightStage?: string;
}

const CATEGORY_KEYS = {
  migration: "catMigration",
  feature: "catFeature",
  product: "catProduct",
  doc: "catDoc",
  fix: "catFix",
  arch: "catArch",
  platform: "catPlatform",
  other: "catOther",
} as const;

function categoryLabel(
  category: string,
  m: ReturnType<typeof useLocale>["m"]
): string {
  const key = CATEGORY_KEYS[category as keyof typeof CATEGORY_KEYS];
  return key ? m.workflows.categories[key] : category;
}

export function WorkflowsView({
  setPage,
  tab: initialTab = "pipelines",
  pipeline: initialPipeline,
  skill: initialSkill,
  contextRunId,
  contextIssueKey,
  highlightStage,
}: Props) {
  const { m } = useLocale();
  const [catalog, setCatalog] = useState<WorkflowCatalog | null>(null);
  const [runStatus, setRunStatus] = useState<PipelineStatusFull | null>(null);
  const [tab, setTab] = useState<Tab>(initialTab);
  const [selectedPipeline, setSelectedPipeline] = useState<string | null>(
    initialPipeline ?? null
  );
  const [selectedSkill, setSelectedSkill] = useState<string | null>(
    initialSkill ?? null
  );
  const [error, setError] = useState<string | null>(null);

  const activeStage =
    highlightStage ??
    runStatus?.current_stage ??
    runStatus?.stages.find((s) => s.state === "in_progress")?.name ??
    null;

  const load = useCallback(() => {
    getWorkflowCatalog()
      .then((c) => {
        setCatalog(c);
        const pipelineFromRun = runStatus?.pipeline_name;
        setSelectedPipeline((prev) => {
          if (prev && c.pipelines.some((p) => p.name === prev)) return prev;
          if (initialPipeline && c.pipelines.some((p) => p.name === initialPipeline)) {
            return initialPipeline;
          }
          if (
            pipelineFromRun &&
            c.pipelines.some((p) => p.name === pipelineFromRun)
          ) {
            return pipelineFromRun;
          }
          return c.pipelines[0]?.name ?? null;
        });
        setSelectedSkill((prev) => {
          if (prev && c.skills.some((s) => s.name === prev)) return prev;
          if (initialSkill && c.skills.some((s) => s.name === initialSkill)) {
            return initialSkill;
          }
          return null;
        });
      })
      .catch((e) => setError(String(e)));
  }, [initialPipeline, initialSkill, runStatus?.pipeline_name]);

  useEffect(() => {
    if (!contextRunId) {
      setRunStatus(null);
      return;
    }
    getPipelineStatus(contextRunId)
      .then(setRunStatus)
      .catch(() => setRunStatus(null));
  }, [contextRunId]);

  useEffect(() => {
    load();
  }, [load]);

  useEffect(() => {
    setTab(initialTab);
  }, [initialTab]);

  const pipeline = catalog?.pipelines.find((p) => p.name === selectedPipeline);
  const skill = catalog?.skills.find((s) => s.name === selectedSkill);

  const pipelineFlow = useMemo(() => {
    if (!pipeline) return { nodes: [], edges: [] };
    return layoutStageDag(
      pipeline.stages.map((s) => ({
        name: s.name,
        depends_on: s.depends_on,
        label: s.requires_approval ? `${s.name}\n⚠` : s.name,
        highlight: activeStage === s.name,
      }))
    );
  }, [pipeline, activeStage]);

  if (error) {
    return (
      <div className="page-frame">
        <div className="card border-[rgba(239,68,68,0.25)] bg-[rgba(239,68,68,0.08)] p-4 text-[13px] text-[var(--accent-red)]">
          {error}
        </div>
      </div>
    );
  }

  if (!catalog) {
    return (
      <div className="page-frame">
        <LoadingState label={m.workflows.loading} />
      </div>
    );
  }

  return (
    <div className="page-frame flex h-full flex-col gap-3">
      <div className="shrink-0 flex flex-wrap items-start justify-between gap-3">
        <div>
          <div className="flex items-center gap-2">
            <BookOpen size={18} className="text-[var(--accent)]" />
            <h2 className="text-[15px] font-semibold">{m.workflows.title}</h2>
          </div>
          <p className="mt-0.5 text-[12px] text-[var(--text-muted)]">
            {m.workflows.intro}
          </p>
        </div>
        <div className="flex rounded-[var(--radius-sm)] border border-[var(--border)] p-0.5">
          <button
            type="button"
            className={`btn btn-ghost px-3 py-1.5 text-[12px] ${tab === "pipelines" ? "bg-[var(--bg-hover)]" : ""}`}
            onClick={() => setTab("pipelines")}
          >
            <GitBranch size={14} className="mr-1.5 inline" />
            {m.workflows.tabPipelines}
          </button>
          <button
            type="button"
            className={`btn btn-ghost px-3 py-1.5 text-[12px] ${tab === "skills" ? "bg-[var(--bg-hover)]" : ""}`}
            onClick={() => setTab("skills")}
          >
            <Sparkles size={14} className="mr-1.5 inline" />
            {m.workflows.tabSkills}
          </button>
        </div>
      </div>

      {(contextIssueKey || runStatus) && (
        <div className="card flex flex-wrap items-center gap-3 border-[rgba(59,130,246,0.25)] bg-[rgba(59,130,246,0.08)] px-3 py-2.5 text-[12px]">
          <span className="font-medium text-[var(--text-primary)]">
            {m.workflows.issueContext}
          </span>
          {contextIssueKey && (
            <button
              type="button"
              className="font-mono text-[#93c5fd] hover:underline"
              onClick={() => setPage({ kind: "issue", issueKey: contextIssueKey })}
            >
              {contextIssueKey}
            </button>
          )}
          {runStatus && (
            <>
              <span className="text-[var(--text-muted)]">·</span>
              <span>{runStatus.pipeline_name}</span>
              {activeStage && (
                <>
                  <span className="text-[var(--text-muted)]">·</span>
                  <span>
                    {m.workflows.currentStage}:{" "}
                    <strong className="text-[var(--text-primary)]">
                      {activeStage}
                    </strong>
                  </span>
                  <StatusBadge status={runStatus.run_status} />
                </>
              )}
              <button
                type="button"
                className="ml-auto text-[11px] text-[var(--accent)] hover:underline"
                onClick={() =>
                  setPage({ kind: "pipeline", runId: runStatus.id })
                }
              >
                {m.workflows.openPipelineRun}
              </button>
            </>
          )}
        </div>
      )}

      {tab === "pipelines" ? (
        <div className="explorer-split min-h-0 flex-1">
          <div className="master-panel">
            <div className="master-panel-scroll space-y-0.5 p-2">
              {catalog.pipelines.map((p) => (
                <button
                  key={p.name}
                  type="button"
                  onClick={() => setSelectedPipeline(p.name)}
                  className={`product-task-row w-full text-left ${selectedPipeline === p.name ? "product-task-row-active" : ""}`}
                >
                  <div className="flex items-center gap-2">
                    <span className="min-w-0 flex-1 truncate text-[12px] font-medium">
                      {p.name}
                    </span>
                    {p.recommended && (
                      <span className="shrink-0 rounded bg-[var(--accent-muted)] px-1.5 py-0.5 text-[10px] text-[var(--accent)]">
                        {m.workflows.recommended}
                      </span>
                    )}
                  </div>
                  <p className="mt-0.5 truncate text-[10px] text-[var(--text-muted)]">
                    {categoryLabel(p.category, m)} · {p.stage_count}{" "}
                    {m.workflows.stages}
                  </p>
                </button>
              ))}
            </div>
          </div>

          <div className="detail-panel flex min-h-[320px] min-w-0 flex-col md:min-h-0">
            {pipeline ? (
              <PipelineDetail
                pipeline={pipeline}
                flow={pipelineFlow}
                activeStage={activeStage}
                m={m}
                onSelectSkill={(name) => {
                  setTab("skills");
                  setSelectedSkill(name);
                }}
              />
            ) : (
              <div className="flex flex-1 items-center justify-center text-[12px] text-[var(--text-muted)]">
                {m.workflows.selectPipeline}
              </div>
            )}
          </div>
        </div>
      ) : (
        <div className="explorer-split min-h-0 flex-1">
          <div className="master-panel">
            <div className="master-panel-scroll space-y-0.5 p-2">
              {catalog.skills.map((s) => (
                <button
                  key={s.name}
                  type="button"
                  onClick={() => setSelectedSkill(s.name)}
                  className={`product-task-row w-full text-left ${selectedSkill === s.name ? "product-task-row-active" : ""}`}
                >
                  <span className="block truncate text-[12px] font-medium">
                    {s.name}
                  </span>
                  <p className="mt-0.5 truncate text-[10px] text-[var(--text-muted)]">
                    v{s.version}
                    {s.standalone ? ` · ${m.workflows.standalone}` : ""}
                  </p>
                </button>
              ))}
            </div>
          </div>

          <div className="detail-panel flex min-h-[320px] min-w-0 flex-col md:min-h-0">
            {skill ? (
              <SkillDetail
                skill={skill}
                m={m}
                onSelectPipeline={(name) => {
                  setTab("pipelines");
                  setSelectedPipeline(name);
                }}
              />
            ) : (
              <div className="flex flex-1 items-center justify-center text-[12px] text-[var(--text-muted)]">
                {m.workflows.selectSkill}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function PipelineDetail({
  pipeline,
  flow,
  activeStage,
  m,
  onSelectSkill,
}: {
  pipeline: PipelineCatalogEntry;
  flow: ReturnType<typeof layoutStageDag>;
  activeStage: string | null;
  m: ReturnType<typeof useLocale>["m"];
  onSelectSkill: (name: string) => void;
}) {
  return (
    <div className="detail-panel-scroll flex h-full flex-col gap-4 p-4">
      <div>
        <h3 className="text-[14px] font-semibold">{pipeline.name}</h3>
        <p className="mt-1 text-[12px] leading-relaxed text-[var(--text-muted)]">
          {pipeline.description || m.workflows.noDescription}
        </p>
        <div className="mt-2 flex flex-wrap gap-2 text-[11px] text-[var(--text-secondary)]">
          <span className="rounded border border-[var(--border)] px-2 py-0.5">
            {categoryLabel(pipeline.category, m)}
          </span>
          <span className="rounded border border-[var(--border)] px-2 py-0.5">
            {pipeline.stage_count} {m.workflows.stages}
          </span>
          {pipeline.approval_count > 0 && (
            <span className="flex items-center gap-1 rounded border border-[var(--border)] px-2 py-0.5 text-[var(--accent-yellow)]">
              <ShieldAlert size={12} />
              {pipeline.approval_count} {m.workflows.approvalStages}
            </span>
          )}
          {pipeline.recommended && (
            <span className="flex items-center gap-1 rounded bg-[var(--accent-muted)] px-2 py-0.5 text-[var(--accent)]">
              <CheckCircle2 size={12} />
              {m.workflows.recommendedForProfile}
            </span>
          )}
        </div>
      </div>

      <div className="graph-panel min-h-[200px] flex-1">
        <ReactFlow
          nodes={flow.nodes}
          edges={flow.edges}
          fitView
          fitViewOptions={{ padding: 0.2 }}
          proOptions={{ hideAttribution: true }}
          nodesDraggable={false}
          nodesConnectable={false}
          elementsSelectable={false}
        >
          <Background color="var(--border)" gap={20} />
          <Controls
            showInteractive={false}
            position="bottom-right"
            className="graph-flow-controls"
          />
        </ReactFlow>
      </div>

      <div>
        <h4 className="mb-2 text-[12px] font-semibold">{m.workflows.stageGuide}</h4>
        <div className="space-y-2">
          {pipeline.stages.map((stage) => (
            <div
              key={stage.name}
              className={`rounded-[var(--radius-sm)] border p-3 ${
                activeStage === stage.name
                  ? "border-[rgba(59,130,246,0.45)] bg-[rgba(59,130,246,0.08)]"
                  : "border-[var(--border)]"
              }`}
            >
              <div className="flex items-center justify-between gap-2">
                <span className="text-[12px] font-semibold">{stage.name}</span>
                {stage.requires_approval && (
                  <ShieldAlert size={14} className="text-[var(--accent-yellow)]" />
                )}
              </div>
              <p className="mt-1 text-[11px] leading-relaxed text-[var(--text-muted)]">
                {stage.description || m.workflows.noDescription}
              </p>
              <div className="mt-2 flex flex-wrap gap-1">
                {stage.skills.map((sk) => (
                  <button
                    key={sk}
                    type="button"
                    onClick={() => onSelectSkill(sk)}
                    className="rounded bg-[var(--bg-hover)] px-1.5 py-0.5 text-[11px] text-[var(--accent)] hover:underline"
                  >
                    {sk}
                  </button>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function SkillDetail({
  skill,
  m,
  onSelectPipeline,
}: {
  skill: SkillCatalogEntry;
  m: ReturnType<typeof useLocale>["m"];
  onSelectPipeline: (name: string) => void;
}) {
  return (
    <div className="detail-panel-scroll space-y-4 p-4">
      <div>
        <h3 className="text-[14px] font-semibold">{skill.name}</h3>
        <p className="mt-0.5 text-[11px] text-[var(--text-muted)]">v{skill.version}</p>
        <p className="mt-2 text-[12px] leading-relaxed text-[var(--text-muted)]">
          {skill.description || m.workflows.noDescription}
        </p>
        {skill.standalone && (
          <p className="mt-2 text-[11px] text-[var(--accent)]">{m.workflows.standaloneHint}</p>
        )}
      </div>

      {skill.artifacts.length > 0 && (
        <div>
          <h4 className="mb-2 text-[12px] font-semibold">{m.workflows.artifacts}</h4>
          <ul className="space-y-1 text-[11px]">
            {skill.artifacts.map((a) => (
              <li key={a.artifact_type} className="rounded border border-[var(--border)] px-3 py-2">
                <span className="font-medium">{a.artifact_type}</span>
                {a.description && (
                  <span className="text-[var(--text-muted)]"> — {a.description}</span>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}

      {skill.workflow_states.length > 0 && (
        <div>
          <h4 className="mb-2 text-[12px] font-semibold">{m.workflows.stateMachine}</h4>
          <div className="flex flex-wrap gap-1.5">
            {skill.workflow_states.map((st) => (
              <span
                key={st.name}
                className={`rounded border px-2 py-1 text-[10px] ${
                  st.is_initial
                    ? "border-[var(--accent)] bg-[var(--accent-muted)]"
                    : "border-[var(--border)]"
                }`}
              >
                {st.name}
                {st.requires_approval ? " ⚠" : ""}
                {st.is_final ? " ✓" : ""}
              </span>
            ))}
          </div>
        </div>
      )}

      {skill.used_in_pipelines.length > 0 && (
        <div>
          <h4 className="mb-2 text-[12px] font-semibold">{m.workflows.usedInPipelines}</h4>
          <div className="flex flex-wrap gap-1.5">
            {skill.used_in_pipelines.map((p) => (
              <button
                key={p}
                type="button"
                onClick={() => onSelectPipeline(p)}
                className="rounded bg-[var(--bg-hover)] px-2 py-1 text-[11px] text-[var(--accent)] hover:underline"
              >
                {p}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
