import { TaskDetailPanel } from "../components/TaskDetailPanel";
import type { Page } from "../App";

interface Props {
  product: string;
  taskId: string;
  setPage: (p: Page) => void;
}

export function TaskDetailPage({ product, taskId, setPage }: Props) {
  return (
    <TaskDetailPanel
      product={product}
      taskId={taskId}
      setPage={setPage}
      showBack
    />
  );
}
