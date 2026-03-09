const statusColors: Record<string, string> = {
  draft: "bg-gray-500/20 text-gray-300",
  review: "bg-yellow-500/20 text-yellow-300",
  discussion: "bg-yellow-500/20 text-yellow-300",
  proposed: "bg-yellow-500/20 text-yellow-300",
  approved: "bg-green-500/20 text-green-300",
  accepted: "bg-green-500/20 text-green-300",
  completed: "bg-green-500/20 text-green-300",
  done: "bg-green-500/20 text-green-300",
  ready: "bg-blue-500/20 text-blue-300",
  in_progress: "bg-purple-500/20 text-purple-300",
  blocked: "bg-red-500/20 text-red-300",
  rejected: "bg-red-500/20 text-red-300",
  planning: "bg-indigo-500/20 text-indigo-300",
  code_review: "bg-orange-500/20 text-orange-300",
  superseded: "bg-gray-500/20 text-gray-400",
  skipped: "bg-gray-500/20 text-gray-400",
  backlog: "bg-gray-500/20 text-gray-300",
  cancelled: "bg-red-500/20 text-red-400",
};

export function StatusBadge({ status }: { status: string }) {
  const color = statusColors[status] || "bg-gray-500/20 text-gray-300";
  return (
    <span
      className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${color}`}
    >
      {status}
    </span>
  );
}
