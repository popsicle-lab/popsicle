import { useEffect, useState } from "react";
import { getUserStory, type UserStoryFull } from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import { BookOpen, ArrowLeft, CheckCircle2, Circle } from "lucide-react";
import type { Page } from "../App";

interface Props {
  storyKey: string;
  setPage: (p: Page) => void;
}

export function StoryDetailView({ storyKey, setPage }: Props) {
  const [story, setStory] = useState<UserStoryFull | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getUserStory(storyKey)
      .then(setStory)
      .catch((e) => setError(e?.toString()));
  }, [storyKey]);

  if (error)
    return (
      <div className="text-red-400 p-4 bg-red-500/10 rounded-lg">{error}</div>
    );
  if (!story) return <div className="text-[var(--text-secondary)]">Loading...</div>;

  const verified = story.acceptance_criteria.filter((ac) => ac.verified).length;
  const total = story.acceptance_criteria.length;

  return (
    <div className="space-y-6 max-w-4xl">
      <button
        onClick={() => setPage({ kind: "stories" })}
        className="flex items-center gap-2 text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
      >
        <ArrowLeft size={16} /> Back to User Stories
      </button>

      <div className="flex items-start gap-4">
        <BookOpen size={28} className="text-blue-400 mt-1 shrink-0" />
        <div className="min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <span className="font-mono text-sm text-[var(--accent)]">{story.key}</span>
            <StatusBadge status={story.status} />
          </div>
          <h2 className="text-2xl font-bold mt-1">{story.title}</h2>
        </div>
      </div>

      {(story.persona || story.goal || story.benefit) && (
        <div className="bg-[var(--bg-secondary)] rounded-xl p-5 border border-[var(--border)]">
          <p className="text-sm leading-relaxed">
            {story.persona && (
              <>
                <span className="text-[var(--text-secondary)]">As a </span>
                <span className="font-medium text-blue-300">{story.persona}</span>
              </>
            )}
            {story.goal && (
              <>
                <span className="text-[var(--text-secondary)]"> I want to </span>
                <span className="font-medium text-green-300">{story.goal}</span>
              </>
            )}
            {story.benefit && (
              <>
                <span className="text-[var(--text-secondary)]"> so that </span>
                <span className="font-medium text-purple-300">{story.benefit}</span>
              </>
            )}
          </p>
        </div>
      )}

      <div className="grid grid-cols-2 gap-4">
        <InfoCard label="Priority" value={story.priority} />
        <InfoCard label="Status" value={story.status} />
        <InfoCard label="Created" value={new Date(story.created_at).toLocaleString()} />
        <InfoCard label="Updated" value={new Date(story.updated_at).toLocaleString()} />
      </div>

      {story.description && (
        <Section title="Description">
          <p className="text-sm text-[var(--text-secondary)] whitespace-pre-wrap">{story.description}</p>
        </Section>
      )}

      <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
        <div className="px-4 py-3 border-b border-[var(--border)] flex items-center justify-between">
          <h3 className="text-sm font-medium">Acceptance Criteria</h3>
          <span className="text-xs text-[var(--text-secondary)]">
            {verified}/{total} verified
          </span>
        </div>
        {total === 0 ? (
          <div className="p-4 text-sm text-[var(--text-secondary)]">
            No acceptance criteria defined.
          </div>
        ) : (
          <div className="divide-y divide-[var(--border)]">
            {story.acceptance_criteria.map((ac) => (
              <div key={ac.id} className="px-4 py-3 flex items-start gap-3">
                {ac.verified ? (
                  <CheckCircle2 size={18} className="text-green-400 mt-0.5 shrink-0" />
                ) : (
                  <Circle size={18} className="text-[var(--text-secondary)] mt-0.5 shrink-0" />
                )}
                <div className="min-w-0 flex-1">
                  <p className="text-sm">{ac.description}</p>
                  {ac.test_case_ids.length > 0 && (
                    <div className="flex gap-1.5 mt-1 flex-wrap">
                      {ac.test_case_ids.map((tcId) => (
                        <span
                          key={tcId}
                          className="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-mono bg-blue-500/10 text-blue-300"
                        >
                          {tcId}
                        </span>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
        {total > 0 && (
          <div className="px-4 py-2 border-t border-[var(--border)]">
            <div className="h-1.5 bg-[var(--bg-tertiary)] rounded-full overflow-hidden">
              <div
                className="h-full rounded-full transition-all"
                style={{
                  width: `${(verified / total) * 100}%`,
                  background: verified === total ? "var(--accent-green)" : "var(--accent-yellow)",
                }}
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function InfoCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-[var(--bg-secondary)] rounded-lg p-3 border border-[var(--border)]">
      <div className="text-xs text-[var(--text-secondary)]">{label}</div>
      <div className="text-sm font-medium mt-0.5">{value}</div>
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="bg-[var(--bg-secondary)] rounded-xl p-4 border border-[var(--border)]">
      <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-2">{title}</h3>
      {children}
    </div>
  );
}
