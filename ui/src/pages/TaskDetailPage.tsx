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
    <TaskDetailPanel
      product={product}
      taskId={taskId}
      returnTo={returnTo}
      setPage={setPage}
      showBack
    />
  );
}
