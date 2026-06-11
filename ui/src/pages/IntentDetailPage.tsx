import { IntentDetailPanel } from "../components/IntentDetailPanel";
import type { Page } from "../App";

interface Props {
  product: string;
  file: string;
  block?: string;
  setPage: (p: Page) => void;
}

export function IntentDetailPage({ product, file, block, setPage }: Props) {
  return (
    <IntentDetailPanel
      product={product}
      file={file}
      block={block}
      setPage={setPage}
      showBack
    />
  );
}
