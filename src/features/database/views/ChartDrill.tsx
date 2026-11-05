// The list of rows behind a bar, a slice, a point or the number of a chart (like Notion: click a part of a chart to see its data).
// A small dialog: the group name as the title, a chip with the condition, a close button and a table of the matching rows.

import { Dialog } from "radix-ui";
import { Loader2, X } from "lucide-react";
import type { DbRow, Property } from "@/lib/database-types";
import { cn } from "@/lib/utils";
import CellDisplay from "../cells/CellDisplay";
import { RowTitle } from "./shared";

export interface Drill {
  /** The group name. */
  title: string;
  /** What the rows have in common, as short sentences ("Status is Done"). */
  chips: string[];
  rows: DbRow[];
  /** Properties shown next to the title. */
  cols: Property[];
  loading: boolean;
  error: string | null;
}

export default function ChartDrill({ drill, onClose, onOpenRow }: { drill: Drill; onClose: () => void; onOpenRow: (id: string) => void }) {
  return (
    <Dialog.Root open onOpenChange={(o) => !o && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-black/45 backdrop-blur-sm" />
        <Dialog.Content
          aria-describedby={undefined}
          data-chart-drill
          className="fixed left-1/2 top-1/2 z-50 flex max-h-[80vh] w-[min(720px,94vw)] -translate-x-1/2 -translate-y-1/2 flex-col rounded-xl border border-line bg-elevated shadow-float outline-none"
        >
          <div className="flex items-start gap-3 border-b border-line px-5 pb-3 pt-4">
            <div className="min-w-0 flex-1">
              <Dialog.Title dir="auto" className="truncate text-lg font-semibold tracking-tight">{drill.title || "No value"}</Dialog.Title>
              <div className="mt-1.5 flex flex-wrap items-center gap-1.5">
                {drill.chips.map((c, i) => (
                  <span key={i} dir="auto" className="rounded-md bg-accent-soft px-2 py-0.5 text-[0.75rem] text-accent-strong" data-drill-chip>{c}</span>
                ))}
                {!drill.loading && !drill.error ? <span className="text-[0.75rem] text-muted">{drill.rows.length} {drill.rows.length === 1 ? "row" : "rows"}</span> : null}
              </div>
            </div>
            <Dialog.Close aria-label="Close" className="rounded-lg p-1.5 text-muted hover:bg-surface-2 hover:text-fg">
              <X size={16} />
            </Dialog.Close>
          </div>
          <div className="min-h-0 flex-1 overflow-auto px-2 pb-3 pt-1">
            {drill.loading ? (
              <div className="grid place-items-center py-12 text-muted"><Loader2 className="animate-spin" size={18} /></div>
            ) : drill.error ? (
              <p role="alert" className="px-3 py-8 text-center text-sm text-danger">{drill.error}</p>
            ) : drill.rows.length === 0 ? (
              <p className="px-3 py-8 text-center text-sm text-muted">No rows here.</p>
            ) : (
              <table className="w-full table-fixed border-collapse text-[0.82rem]" data-drill-table>
                <thead>
                  <tr className="text-muted">
                    <th scope="col" className="sticky top-0 z-[1] w-[34%] bg-elevated px-3 py-1.5 text-start font-medium">Name</th>
                    {drill.cols.map((p) => (
                      <th key={p.id} scope="col" className="sticky top-0 z-[1] bg-elevated px-3 py-1.5 text-start font-medium"><span dir="auto" className="block truncate">{p.name}</span></th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {drill.rows.map((r) => (
                    <tr key={r.id} tabIndex={0} role="button" aria-label={`Open ${r.title || "Untitled"}`} onClick={() => onOpenRow(r.id)} onKeyDown={(e) => (e.key === "Enter" || e.key === " ") && (e.preventDefault(), onOpenRow(r.id))} className={cn("cursor-pointer border-t border-line/60 hover:bg-surface-2")} data-drill-row>
                      <td className="px-3 py-1.5"><RowTitle row={r} /></td>
                      {drill.cols.map((p) => (
                        <td key={p.id} className="min-w-0 px-3 py-1.5"><div className="min-w-0 overflow-hidden"><CellDisplay prop={p} value={r.values[p.id]} row={r} /></div></td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
