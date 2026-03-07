import { useEffect, useState } from "react";
import {
  getDocument,
  getCommitLinks,
  type DocFull,
  type CommitLinkInfo,
} from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeHighlight from "rehype-highlight";
import {
  FileText,
  Tag,
  Clock,
  GitBranch,
  GitCommit,
  Puzzle,
} from "lucide-react";

interface Props {
  docId: string;
}

export function DocumentView({ docId }: Props) {
  const [doc, setDoc] = useState<DocFull | null>(null);
  const [linkedCommits, setLinkedCommits] = useState<CommitLinkInfo[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getDocument(docId)
      .then((d) => {
        setDoc(d);
        return getCommitLinks({ docId });
      })
      .then(setLinkedCommits)
      .catch((e) => setError(e?.toString()));
  }, [docId]);

  if (error)
    return (
      <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg">
        {error}
      </div>
    );
  if (!doc)
    return <div className="text-[var(--text-secondary)]">Loading...</div>;

  return (
    <div className="flex gap-6 h-full">
      {/* Document Body */}
      <div className="flex-1 min-w-0">
        <h2 className="text-2xl font-bold mb-4">{doc.title}</h2>
        <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)] p-6 prose prose-invert max-w-none prose-pre:my-3 prose-headings:border-b prose-headings:border-[var(--border)] prose-headings:pb-2 prose-h1:text-xl prose-h2:text-lg prose-h3:text-base">
          <Markdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeHighlight]}>
            {doc.body || "*No content yet*"}
          </Markdown>
        </div>
      </div>

      {/* Metadata Panel */}
      <aside className="w-72 shrink-0">
        <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)] sticky top-0">
          <div className="px-4 py-3 border-b border-[var(--border)]">
            <h3 className="font-medium text-sm">Metadata</h3>
          </div>
          <div className="p-4 space-y-4 text-sm">
            <MetaRow icon={<FileText size={14} />} label="Type">
              <code className="text-xs bg-[var(--bg-tertiary)] px-1.5 py-0.5 rounded">
                {doc.doc_type}
              </code>
            </MetaRow>
            <MetaRow icon={<Tag size={14} />} label="Status">
              <StatusBadge status={doc.status} />
            </MetaRow>
            <MetaRow icon={<Puzzle size={14} />} label="Skill">
              {doc.skill_name}
            </MetaRow>
            <MetaRow icon={<GitBranch size={14} />} label="Pipeline Run">
              <span className="font-mono text-xs">
                {doc.pipeline_run_id.slice(0, 8)}
              </span>
            </MetaRow>
            {doc.tags.length > 0 && (
              <MetaRow icon={<Tag size={14} />} label="Tags">
                <div className="flex flex-wrap gap-1">
                  {doc.tags.map((t) => (
                    <span
                      key={t}
                      className="text-xs bg-[var(--bg-tertiary)] px-1.5 py-0.5 rounded"
                    >
                      {t}
                    </span>
                  ))}
                </div>
              </MetaRow>
            )}
            {doc.created_at && (
              <MetaRow icon={<Clock size={14} />} label="Created">
                <span className="text-xs text-[var(--text-secondary)]">
                  {new Date(doc.created_at).toLocaleString()}
                </span>
              </MetaRow>
            )}
            {doc.updated_at && (
              <MetaRow icon={<Clock size={14} />} label="Updated">
                <span className="text-xs text-[var(--text-secondary)]">
                  {new Date(doc.updated_at).toLocaleString()}
                </span>
              </MetaRow>
            )}

            <div className="pt-2 border-t border-[var(--border)]">
              <div className="text-xs text-[var(--text-secondary)] mb-1">
                File
              </div>
              <code className="text-xs break-all text-[var(--accent)]">
                {doc.file_path}
              </code>
            </div>

            <div className="pt-2 border-t border-[var(--border)]">
              <div className="text-xs text-[var(--text-secondary)] mb-1">
                ID
              </div>
              <code className="text-xs break-all font-mono">
                {doc.id}
              </code>
            </div>

            {linkedCommits.length > 0 && (
              <div className="pt-2 border-t border-[var(--border)]">
                <div className="flex items-center gap-1.5 text-[var(--text-secondary)] mb-2">
                  <GitCommit size={14} />
                  <span className="text-xs">
                    Linked Commits ({linkedCommits.length})
                  </span>
                </div>
                <div className="space-y-1.5">
                  {linkedCommits.map((c) => (
                    <div
                      key={c.sha}
                      className="flex items-center gap-1.5 text-xs"
                    >
                      <StatusBadge status={c.review_status} />
                      <code className="font-mono text-[var(--accent)]">
                        {c.short_sha}
                      </code>
                      <span className="truncate text-[var(--text-secondary)]">
                        {c.message}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      </aside>
    </div>
  );
}

function MetaRow({
  icon,
  label,
  children,
}: {
  icon: React.ReactNode;
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <div className="flex items-center gap-1.5 text-[var(--text-secondary)] mb-1">
        {icon}
        <span className="text-xs">{label}</span>
      </div>
      <div className="pl-5">{children}</div>
    </div>
  );
}
