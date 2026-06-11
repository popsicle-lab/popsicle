import { IntentDetailPanel } from "../components/IntentDetailPanel";
import type { Page } from "../App";

interface Props {
  product: string;
  file: string;
  block?: string;
  returnTo?: Page;
  setPage: (p: Page) => void;
}

export function IntentDetailPage({
  product,
  file,
  block,
  returnTo,
  setPage,
}: Props) {
  return (
    <IntentDetailPanel
      product={product}
      file={file}
      block={block}
      returnTo={returnTo}
      setPage={setPage}
      showBack
    />
  );
}
