// Gallery view: a grid of cards with an optional cover picture (the first image of a files property),
// the title and the visible properties. Groups become sections. The last card is "+ New".

import { memo, useMemo, useState } from "react";
import { ChevronDown, ChevronRight, Image as ImageIcon, LayoutGrid, Plus } from "lucide-react";
import type { DbRow, GroupBy, GroupOut, Property } from "@/lib/database-types";
import type { ViewProps } from "../viewTypes";
import { cn } from "@/lib/utils";
import { EmptyState, GroupChip, NewTitleInput, PropLines, RowTitle, firstImage, groupPatch, groupProp, imageUrl, propOf, setCollapsed, titleMap, visibleProps } from "./shared";

const MIN_W: Record<string, number> = { small: 168, medium: 236, large: 320 };
const COVER_H: Record<string, number> = { small: 92, medium: 132, large: 190 };

export default function GalleryView({ db, view, result, rowById, openRow, createRow, updateView }: ViewProps) {
  const size = MIN_W[view.cardSize] ? view.cardSize : "medium";
  const g = view.group;
  const gp = groupProp(db, g);
  const coverProp = propOf(db, view.cardCover);
  const cover = coverProp?.kind === "files" ? coverProp.id : "";
  const wantsCover = view.cardCover !== "none";
  const shown = useMemo(() => visibleProps(db, view, [g?.prop]), [db, view, g?.prop]);
  const titles = useMemo(() => titleMap(result.rows), [result.rows]);
  const [adding, setAdding] = useState<string | null>(null);

  const cellsFor = (gr: GroupOut | null, gg: GroupBy | null): Record<string, unknown> => {
    if (!gr || !gg) return {};
    const patch = groupPatch(gp, gg, gr.key, gr.label) ?? {};
    return Object.fromEntries(Object.entries(patch).filter(([, v]) => v !== null));
  };

  const grid = (ids: DbRow[], key: string, gr: GroupOut | null) => (
    <div className="grid gap-3" style={{ gridTemplateColumns: `repeat(auto-fill, minmax(${MIN_W[size]}px, 1fr))` }} data-grid={key}>
      {ids.map((row) => (
        <Card key={row.id} row={row} cover={cover} wantsCover={wantsCover} fit={view.cardFit} coverH={COVER_H[size]} props={shown} titles={titles} onOpen={openRow} />
      ))}
      {adding === key ? (
        <div className="flex min-h-[5rem] items-start rounded-xl border border-dashed border-accent p-2">
          <NewTitleInput keepOpen onSubmit={(t) => void createRow(cellsFor(gr, g), t)} onCancel={() => setAdding((a) => (a === key ? null : a))} />
        </div>
      ) : (
        <button
          type="button"
          onClick={() => setAdding(key)}
          data-new-card
          className="flex min-h-[5rem] items-center justify-center gap-1.5 rounded-xl border border-dashed border-line text-[0.82rem] text-muted transition-colors hover:border-accent hover:bg-surface hover:text-fg"
          style={{ minHeight: wantsCover ? COVER_H[size] / 2 + 48 : undefined }}
        >
          <Plus size={15} />
          New
        </button>
      )}
    </div>
  );

  if (!result.rows.length && !(result.groups && result.groups.some((x) => x.count))) {
    return (
      <div className="h-full overflow-y-auto px-4 pb-8 pt-2">
        <EmptyState icon={<LayoutGrid size={28} />} title="No rows yet">
          <span>Add a row to start the gallery.</span>
        </EmptyState>
        {grid([], "all", null)}
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto px-4 pb-8 pt-2" role="region" aria-label="Gallery">
      {result.groups && g ? (
        result.groups.map((gr) => {
          const closed = g.collapsed.includes(gr.key);
          const rows = gr.rows.map((id) => rowById.get(id)).filter((r): r is DbRow => !!r);
          return (
            <section key={gr.key} className="mb-5" data-group={gr.key}>
              <button
                type="button"
                aria-expanded={!closed}
                onClick={() => {
                  const p = setCollapsed(view, gr.key, !closed);
                  if (p) updateView(p);
                }}
                className="mb-2 flex h-8 items-center gap-1.5 rounded-lg px-1.5 hover:bg-surface"
              >
                {closed ? <ChevronRight size={14} className="rtl:rotate-180" /> : <ChevronDown size={14} />}
                <GroupChip g={gr} />
                <span className="text-xs tabular-nums text-faint" data-count>{gr.count}</span>
              </button>
              {closed ? null : grid(rows, "g:" + gr.key, gr)}
            </section>
          );
        })
      ) : (
        grid(result.rows, "all", null)
      )}
    </div>
  );
}

const Card = memo(function Card({ row, cover, wantsCover, fit, coverH, props, titles, onOpen }: { row: DbRow; cover: string; wantsCover: boolean; fit: boolean; coverH: number; props: Property[]; titles: Map<string, string>; onOpen: (id: string) => void }) {
  const img = cover ? firstImage(row, cover) : null;
  const [broken, setBroken] = useState(false);
  return (
    <div
      role="button"
      tabIndex={0}
      data-card={row.id}
      aria-label={row.title || "Untitled"}
      onClick={() => onOpen(row.id)}
      onKeyDown={(e) => {
        if (e.key === "Enter" && e.target === e.currentTarget) {
          e.preventDefault();
          onOpen(row.id);
        }
      }}
      className="group/card flex min-w-0 cursor-pointer flex-col overflow-hidden rounded-xl border border-line bg-elevated outline-none transition-shadow hover:shadow-float focus-visible:ring-2 focus-visible:ring-accent"
    >
      {wantsCover ? (
        img && !broken ? (
          <img src={imageUrl(img.id)} alt="" draggable={false} loading="lazy" onError={() => setBroken(true)} className={cn("w-full border-b border-line", fit ? "bg-surface object-contain" : "object-cover")} style={{ height: coverH }} data-cover />
        ) : (
          <div className="grid w-full place-items-center border-b border-line bg-surface text-faint" style={{ height: coverH }} data-cover-empty>
            {row.icon ? <span className="text-3xl leading-none">{row.icon}</span> : <ImageIcon size={22} strokeWidth={1.5} />}
          </div>
        )
      ) : null}
      <div className="flex min-w-0 flex-col gap-1.5 px-3 py-2.5">
        <RowTitle row={row} className="text-[0.9rem] font-medium" />
        <PropLines row={row} props={props} titles={titles} />
      </div>
    </div>
  );
});
