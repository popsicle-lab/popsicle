import { useEffect, useState } from "react";
import { listSkills, type SkillInfo } from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import { ChevronDown, ChevronRight, ArrowRight } from "lucide-react";

export function SkillsView() {
  const [skills, setSkills] = useState<SkillInfo[]>([]);
  const [expanded, setExpanded] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    listSkills()
      .then(setSkills)
      .catch((e) => setError(e?.toString()));
  }, []);

  if (error)
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">Skills Registry</h2>

      <div className="grid gap-3">
        {skills.map((skill) => {
          const isOpen = expanded === skill.name;
          return (
            <div
              key={skill.name}
              className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)] overflow-hidden"
            >
              <button
                onClick={() => setExpanded(isOpen ? null : skill.name)}
                className="w-full px-4 py-3 flex items-center gap-3 hover:bg-[var(--bg-tertiary)]/50 transition-colors text-left"
              >
                {isOpen ? (
                  <ChevronDown size={16} />
                ) : (
                  <ChevronRight size={16} />
                )}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="font-medium">{skill.name}</span>
                    <span className="text-xs text-[var(--text-secondary)]">
                      v{skill.version}
                    </span>
                  </div>
                  <div className="text-sm text-[var(--text-secondary)] truncate">
                    {skill.description}
                  </div>
                </div>
                <div className="flex gap-1">
                  {skill.artifact_types.map((t) => (
                    <code
                      key={t}
                      className="text-xs bg-[var(--bg-tertiary)] px-1.5 py-0.5 rounded"
                    >
                      {t}
                    </code>
                  ))}
                </div>
              </button>

              {isOpen && (
                <div className="px-4 pb-4 pt-1 border-t border-[var(--border)] space-y-4">
                  {skill.inputs.length > 0 && (
                    <div>
                      <h4 className="text-xs font-medium text-[var(--text-secondary)] mb-2">
                        Inputs
                      </h4>
                      {skill.inputs.map((input, i) => (
                        <div
                          key={i}
                          className="flex items-center gap-2 text-sm"
                        >
                          <code className="text-xs">{input.artifact_type}</code>
                          <span className="text-[var(--text-secondary)]">
                            from
                          </span>
                          <code className="text-xs text-[var(--accent)]">
                            {input.from_skill}
                          </code>
                          {input.required && (
                            <span className="text-xs text-[var(--accent-red)]">
                              required
                            </span>
                          )}
                        </div>
                      ))}
                    </div>
                  )}

                  <div>
                    <h4 className="text-xs font-medium text-[var(--text-secondary)] mb-2">
                      Workflow (initial: {skill.workflow_initial})
                    </h4>
                    <div className="flex flex-wrap items-center gap-2">
                      {skill.workflow_states.map((ws) => (
                        <div key={ws.name} className="flex items-center gap-1">
                          <StatusBadge status={ws.name} />
                          {ws.transitions.map((t, i) => (
                            <span
                              key={i}
                              className="flex items-center gap-1 text-xs text-[var(--text-secondary)]"
                            >
                              <ArrowRight size={10} />
                              <span>{t.action}</span>
                              <ArrowRight size={10} />
                              <StatusBadge status={t.to} />
                            </span>
                          ))}
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
