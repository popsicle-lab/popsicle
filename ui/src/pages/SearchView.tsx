import { useState } from "react";
import {
  searchDocuments,
  type SearchDocResult,
} from "../hooks/useTauri";
import { StatusBadge } from "../components/StatusBadge";
import {
  Search,
  FileText,
  Filter,
  Hash,
  ArrowRight,
  Loader2,
} from "lucide-react";
import type { Page } from "../App";

interface Props {
  setPage: (p: Page) => void;
}

export function SearchView({ setPage }: Props) {
  const [query, setQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState("");
  const [skillFilter, setSkillFilter] = useState("");
  const [results, setResults] = useState<SearchDocResult[]>([]);
  const [searched, setSearched] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const doSearch = async () => {
    if (!query.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const r = await searchDocuments({
        query: query.trim(),
        status: statusFilter || undefined,
        skill: skillFilter || undefined,
        limit: 30,
      });
      setResults(r);
      setSearched(true);
    } catch (e: any) {
      setError(e?.toString());
    } finally {
      setLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") doSearch();
  };

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">Document Search</h2>
      <p className="text-sm text-[var(--text-secondary)]">
        Search across all pipeline runs using full-text search (FTS5 + BM25 ranking).
      </p>

      {/* Search Bar */}
      <div className="flex gap-3">
        <div className="flex-1 relative">
          <Search
            size={16}
            className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-secondary)]"
          />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search documents by title, summary, or tags..."
            className="w-full pl-10 pr-4 py-2.5 bg-[var(--bg-secondary)] border border-[var(--border)] rounded-lg text-sm focus:outline-none focus:border-[var(--accent)] transition-colors"
          />
        </div>
        <button
          onClick={doSearch}
          disabled={loading || !query.trim()}
          className="px-5 py-2.5 bg-[var(--accent)] text-white rounded-lg text-sm font-medium hover:opacity-90 transition-opacity disabled:opacity-40"
        >
          {loading ? <Loader2 size={16} className="animate-spin" /> : "Search"}
        </button>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-3">
        <Filter size={14} className="text-[var(--text-secondary)]" />
        <select
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
          className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-lg px-3 py-1.5 text-sm"
        >
          <option value="">All Statuses</option>
          <option value="active">Active</option>
          <option value="final">Final</option>
        </select>
        <input
          type="text"
          value={skillFilter}
          onChange={(e) => setSkillFilter(e.target.value)}
          placeholder="Filter by skill..."
          className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-lg px-3 py-1.5 text-sm w-48"
        />
        {searched && (
          <span className="text-sm text-[var(--text-secondary)] ml-auto">
            {results.length} result{results.length !== 1 ? "s" : ""}
          </span>
        )}
      </div>

      {error && (
        <div className="text-[var(--accent-red)] p-4 bg-red-500/10 rounded-lg text-sm">
          {error}
        </div>
      )}

      {/* Results */}
      {searched && results.length === 0 && !loading && (
        <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)] p-8 text-center text-[var(--text-secondary)]">
          No documents found for "{query}".
        </div>
      )}

      {results.length > 0 && (
        <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)]">
          <div className="px-4 py-3 border-b border-[var(--border)] flex items-center gap-2">
            <FileText size={16} />
            <h3 className="font-medium text-sm">Search Results</h3>
          </div>
          <div className="divide-y divide-[var(--border)]">
            {results.map((doc) => (
              <button
                key={doc.id}
                onClick={() => setPage({ kind: "document", docId: doc.id })}
                className="w-full px-4 py-4 hover:bg-[var(--bg-tertiary)] transition-colors text-left"
              >
                <div className="flex items-start justify-between gap-4">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="font-medium text-sm">{doc.title}</span>
                      <StatusBadge status={doc.status} />
                      <code className="text-xs bg-[var(--bg-tertiary)] px-1.5 py-0.5 rounded text-[var(--text-secondary)]">
                        {doc.doc_type}
                      </code>
                    </div>

                    {doc.summary && (
                      <p className="text-xs text-[var(--text-secondary)] leading-relaxed mb-2 line-clamp-2">
                        {doc.summary}
                      </p>
                    )}

                    <div className="flex items-center gap-3 text-xs text-[var(--text-secondary)]">
                      <span>{doc.skill_name}</span>
                      <span className="font-mono">
                        run:{doc.pipeline_run_id.slice(0, 8)}
                      </span>
                      {doc.doc_tags.length > 0 && (
                        <span className="inline-flex items-center gap-1">
                          <Hash size={10} />
                          {doc.doc_tags.slice(0, 5).join(", ")}
                          {doc.doc_tags.length > 5 && ` +${doc.doc_tags.length - 5}`}
                        </span>
                      )}
                    </div>
                  </div>

                  <div className="flex items-center gap-3 shrink-0">
                    <span
                      className="text-xs font-mono px-2 py-0.5 rounded"
                      style={{
                        color: scoreColor(doc.bm25_score),
                        background: `color-mix(in srgb, ${scoreColor(doc.bm25_score)} 10%, transparent)`,
                      }}
                    >
                      {doc.bm25_score.toFixed(1)}
                    </span>
                    <ArrowRight
                      size={14}
                      className="text-[var(--text-secondary)]"
                    />
                  </div>
                </div>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Help */}
      {!searched && (
        <div className="bg-[var(--bg-secondary)] rounded-xl border border-[var(--border)] p-6">
          <h3 className="text-sm font-medium mb-3">Search Tips</h3>
          <ul className="space-y-2 text-xs text-[var(--text-secondary)]">
            <li>• Search by keywords from document titles, summaries, or semantic tags</li>
            <li>• Use status filter to find only approved/accepted documents</li>
            <li>• BM25 scores indicate relevance — higher is better</li>
            <li>
              • CLI equivalent:{" "}
              <code className="bg-[var(--bg-tertiary)] px-1.5 py-0.5 rounded text-[var(--accent)]">
                popsicle context search "query"
              </code>
            </li>
          </ul>
        </div>
      )}
    </div>
  );
}

function scoreColor(score: number): string {
  if (score >= 5) return "var(--accent-green)";
  if (score >= 2) return "var(--accent-yellow)";
  return "var(--text-secondary)";
}
