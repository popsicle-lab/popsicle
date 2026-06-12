import { ClipboardList } from "lucide-react";

const RETRO_STEPS = [
  "在 products/<product>/decisions/pdr/ 写 PDR（含 Intent Mapping）",
  "新增或更新 tasks/ 下 task 文件",
  "在 intents/acceptance.intent 增加 acceptance block",
  "运行 popsicle tool run intent-validate path=products/<product>/intents",
  "可选：living-doc-author --target implementation-status,product-header",
];

interface Props {
  productId: string;
}

export function RetroDocBanner({ productId }: Props) {
  return (
    <div className="card border-[rgba(234,179,8,0.25)] bg-[rgba(234,179,8,0.08)] p-4">
      <div className="mb-2 flex items-center gap-2 text-[13px] font-semibold text-[var(--accent-yellow)]">
        <ClipboardList size={16} />
        Retro 文档债（代码已交付、spec 待补）
      </div>
      <p className="mb-3 text-[12px] text-[var(--text-secondary)]">
        本 Issue 未绑定 pipeline，适合已合并增量的 spec 回补。不要开 slice-spec；
        直接在 <code className="font-mono">products/{productId}/</code> 写活文档。
      </p>
      <ol className="list-decimal space-y-1 pl-4 text-[12px] text-[var(--text-muted)]">
        {RETRO_STEPS.map((step) => (
          <li key={step}>{step.replace("<product>", productId)}</li>
        ))}
      </ol>
    </div>
  );
}
