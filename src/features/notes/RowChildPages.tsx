import { FileText } from "lucide-react";
import { PageIcon } from "@/lib/pageLook";
import type { PageMeta } from "@/lib/types";

export default function RowChildPages({ parentId, pages, onOpen }: {
  parentId: string;
  pages: PageMeta[];
  onOpen: (id: string) => void;
}) {
  const children = pages.filter((p) => p.parent === parentId && p.trashedAt === null);
  if (!children.length) return null;
  return (
    <ul aria-label="Child pages" className="mb-4 space-y-1 text-sm">
      {children.map((p) => (
        <li key={p.id}>
          <button type="button" onClick={() => onOpen(p.id)}
            className="inline-flex max-w-full items-center gap-2 rounded px-2 py-1 text-muted hover:bg-surface-2 hover:text-fg">
            {p.icon ? <PageIcon icon={p.icon} size={14} /> : <FileText size={14} className="shrink-0" aria-hidden />}
            <span className="truncate" dir="auto">{p.title || "Untitled"}</span>
          </button>
        </li>
      ))}
    </ul>
  );
}
