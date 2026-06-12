import { TaskDetailPanel } from "../components/TaskDetailPanel";
import type { Page } from "../App";

interface Props {
  product: string;
  taskId: string;
  returnTo?: Page;
  setPage: (p: Page) => void;
}

export function TaskDetailPage({
  product,
  taskId,
  returnTo,
  setPage,
}: Props) {
  return (
    <div className="page-frame mx-auto max-w-5xl">
      <TaskDetailPanel
        product={product}
        taskId={taskId}
        returnTo={returnTo}
        setPage={setPage}
        showBack
      />
    </div>
  );
}
