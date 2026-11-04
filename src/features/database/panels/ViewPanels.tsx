// The Properties panel (show, hide and reorder columns) and the view settings.

import { Eye, EyeOff } from "lucide-react";
import type { ChartConfig, Database, Property, View } from "@/lib/database-types";
import { cn } from "@/lib/utils";
import { CALC_LABELS, calcsFor, reorderList } from "../model";
import { PSelect, PanelTitle, PropLabel, PropSelect, Row, SortableList, SortableRow, SwitchRow, fieldCls } from "./ui";

export function PropertiesPanel({ db, view, onChange, onOpenProp }: { db: Database; view: View; onChange: (v: View) => void; onOpenProp: (id: string) => void }) {
  const cols = view.columns.filter((c) => db.props.some((p) => p.id === c.prop));
  const setVisible = (id: string, visible: boolean) => onChange({ ...view, columns: view.columns.map((c) => (c.prop === id ? { ...c, visible } : c)) });
  const anyHidden = cols.some((c) => !c.visible && c.prop !== "title");
  return (
    <div>
      <div className="flex items-center justify-between pb-1.5">
        <PanelTitle>Properties</PanelTitle>
        <button
          type="button"
          className="rounded-md px-2 py-0.5 text-xs text-muted hover:bg-surface-2 hover:text-fg"
          onClick={() => onChange({ ...view, columns: view.columns.map((c) => (c.prop === "title" ? c : { ...c, visible: anyHidden })) })}
        >
          {anyHidden ? "Show all" : "Hide all"}
        </button>
      </div>
      <SortableList ids={cols.map((c) => c.prop)} onReorder={(from, to) => onChange({ ...view, columns: reorderList(cols, from, to) })}>
        {cols.map((c) => {
          const p = db.props.find((x) => x.id === c.prop)!;
          const isTitle = p.kind === "title";
          return (
            <SortableRow key={c.prop} id={c.prop}>
              {(handle) => (
                <div className="flex items-center gap-1 rounded-md py-0.5 hover:bg-surface-2">
                  {handle}
                  <button type="button" onClick={() => onOpenProp(p.id)} className="min-w-0 flex-1 text-start text-sm" title="Edit property">
                    <PropLabel prop={p} />
                  </button>
                  <button
                    type="button"
                    disabled={isTitle}
                    aria-label={c.visible ? `Hide ${p.name}` : `Show ${p.name}`}
                    aria-pressed={c.visible}
                    onClick={() => setVisible(p.id, !c.visible)}
                    className={cn("rounded p-1", isTitle ? "opacity-30" : "text-muted hover:text-fg")}
                  >
                    {c.visible ? <Eye size={15} /> : <EyeOff size={15} />}
                  </button>
                </div>
              )}
            </SortableRow>
          );
        })}
      </SortableList>
    </div>
  );
}

function ChartSettings({ chart, props, onChange }: { chart: ChartConfig; props: Property[]; onChange: (c: ChartConfig) => void }) {
  const xp = props.find((p) => p.id === chart.xProp);
  const set = (patch: Partial<ChartConfig>) => onChange({ ...chart, ...patch });
  const isDate = xp && (xp.kind === "date" || xp.kind === "createdTime" || xp.kind === "lastEditedTime");
  return (
    <>
      <PanelTitle>Chart</PanelTitle>
      <Row label="Chart type">
        <PSelect value={chart.kind} label="Chart type" className="w-[11rem]" onChange={(v) => set({ kind: v })}>
          <option value="bar">Bar</option>
          <option value="line">Line</option>
          <option value="donut">Donut</option>
          <option value="number">Number</option>
        </PSelect>
      </Row>
      {chart.kind !== "number" && (
        <Row label="X axis">
          <PropSelect props={props} value={chart.xProp} label="X axis property" allowNone noneLabel="None" className="w-[11rem]" onChange={(v) => set({ xProp: v })} />
        </Row>
      )}
      {isDate && chart.kind !== "number" && (
        <Row label="Group dates by">
          <PSelect value={chart.xBy || "month"} label="Date grouping" className="w-[11rem]" onChange={(v) => set({ xBy: v })}>
            <option value="day">Day</option>
            <option value="week">Week</option>
            <option value="month">Month</option>
            <option value="year">Year</option>
          </PSelect>
        </Row>
      )}
      {xp?.kind === "number" && (
        <Row label="Bucket size">
          <input type="number" min={0} aria-label="Bucket size" value={chart.xStep || ""} placeholder="Each value" onChange={(e) => set({ xStep: Math.max(0, Number(e.target.value) || 0) })} className={cn(fieldCls, "w-[11rem]")} />
        </Row>
      )}
      {(chart.kind === "bar" || chart.kind === "line") && (
        <Row label="Group by">
          <PropSelect props={props} value={chart.groupProp} label="Second grouping" allowNone noneLabel="None" className="w-[11rem]" onChange={(v) => set({ groupProp: v })} />
        </Row>
      )}
      <Row label="Y axis">
        <PSelect value={chart.yCalc || "count"} label="Y axis calculation" className="w-[11rem]" onChange={(v) => set({ yCalc: v })}>
          <option value="count">Count of rows</option>
          {["sum", "avg", "median", "min", "max", "range", "countUnique", "percentNotEmpty"].map((c) => (
            <option key={c} value={c}>{c === "countUnique" ? "Unique values" : CALC_LABELS[c]} of…</option>
          ))}
        </PSelect>
      </Row>
      {chart.yCalc && chart.yCalc !== "count" && (
        <Row label="Property">
          <PropSelect props={props} value={chart.yProp} label="Y axis property" className="w-[11rem]" filter={(p) => calcsFor(p.kind).includes(chart.yCalc === "countUnique" ? "countUnique" : chart.yCalc)} onChange={(v) => set({ yProp: v })} />
        </Row>
      )}
      <Row label="Title">
        <input aria-label="Chart title" dir="auto" value={chart.title} onChange={(e) => set({ title: e.target.value })} className={cn(fieldCls, "w-[11rem]")} />
      </Row>
      <PSelect value={chart.color || ""} label="Chart color" className="mt-1 w-full" onChange={(v) => set({ color: v })}>
        <option value="">Automatic colors</option>
        {["gray", "brown", "orange", "yellow", "green", "blue", "purple", "pink", "red"].map((c) => (
          <option key={c} value={c}>{c[0].toUpperCase() + c.slice(1)}</option>
        ))}
      </PSelect>
      {(chart.kind === "bar") && <SwitchRow label="Stacked" checked={chart.stacked} onChange={(v) => set({ stacked: v })} />}
      {chart.kind === "bar" && <SwitchRow label="Horizontal bars" checked={chart.horizontal} onChange={(v) => set({ horizontal: v })} />}
      <SwitchRow label="Show data labels" checked={chart.showLabels} onChange={(v) => set({ showLabels: v })} />
      <SwitchRow label="Hide empty groups" checked={chart.hideEmpty} onChange={(v) => set({ hideEmpty: v })} />
    </>
  );
}

export function ViewSettingsPanel({ db, view, onChange, locked }: { db: Database; view: View; onChange: (v: View) => void; locked: boolean }) {
  const set = (patch: Partial<View>) => onChange({ ...view, ...patch });
  const dates = db.props.filter((p) => p.kind === "date" || p.kind === "createdTime" || p.kind === "lastEditedTime");
  const files = db.props.filter((p) => p.kind === "files");
  const k = view.kind;
  return (
    <div>
      <PanelTitle>View settings</PanelTitle>
      <Row label="Name">
        <input aria-label="View name" dir="auto" value={view.name} disabled={locked} onChange={(e) => set({ name: e.target.value })} className={cn(fieldCls, "w-[11rem]")} />
      </Row>
      <Row label="Open rows in">
        <PSelect value={view.openIn} label="Open rows in" className="w-[11rem]" onChange={(v) => set({ openIn: v })}>
          <option value="full">Full page</option>
          <option value="side">Side peek</option>
          <option value="center">Center peek</option>
        </PSelect>
      </Row>
      {k === "table" && (
        <>
          <Row label="Frozen columns">
            <PSelect value={String(view.freeze)} label="Frozen columns" className="w-[11rem]" onChange={(v) => set({ freeze: Number(v) })}>
              {[0, 1, 2, 3, 4].map((n) => (
                <option key={n} value={n}>{n === 0 ? "None" : n === 1 ? "Title only" : `${n} columns`}</option>
              ))}
            </PSelect>
          </Row>
          <SwitchRow label="Wrap all columns" checked={view.wrap} onChange={(v) => set({ wrap: v })} />
          <SwitchRow label="Show row numbers" checked={view.rowNumbers} onChange={(v) => set({ rowNumbers: v })} />
        </>
      )}
      {(k === "calendar" || k === "timeline") && (
        <>
          <Row label={k === "timeline" ? "Start date" : "Date property"}>
            <PropSelect props={dates} value={view.dateProp} label="Date property" allowNone noneLabel="None" className="w-[11rem]" onChange={(v) => set({ dateProp: v })} />
          </Row>
          {k === "timeline" && (
            <Row label="End date">
              <PropSelect props={dates} value={view.endProp} label="End date property" allowNone noneLabel="Same as start" className="w-[11rem]" onChange={(v) => set({ endProp: v })} />
            </Row>
          )}
          {k === "calendar" && (
            <Row label="Show">
              <PSelect value={view.calendarMode || "month"} label="Calendar mode" className="w-[11rem]" onChange={(v) => set({ calendarMode: v })}>
                <option value="month">Month</option>
                <option value="week">Week</option>
              </PSelect>
            </Row>
          )}
          {k === "timeline" && (
            <Row label="Zoom">
              <PSelect value={view.timelineZoom || "month"} label="Timeline zoom" className="w-[11rem]" onChange={(v) => set({ timelineZoom: v })}>
                <option value="week">Weeks</option>
                <option value="month">Months</option>
                <option value="quarter">Quarters</option>
                <option value="year">Years</option>
              </PSelect>
            </Row>
          )}
        </>
      )}
      {k === "timeline" && (
        <>
          <SwitchRow label="Show dependencies" checked={view.showDeps} onChange={(v) => set({ showDeps: v })} />
          <SwitchRow label="Show sub-items" checked={view.showSubItems} onChange={(v) => set({ showSubItems: v })} />
        </>
      )}
      {(k === "gallery" || k === "board") && (
        <>
          <Row label="Card size">
            <PSelect value={view.cardSize || "medium"} label="Card size" className="w-[11rem]" onChange={(v) => set({ cardSize: v })}>
              <option value="small">Small</option>
              <option value="medium">Medium</option>
              <option value="large">Large</option>
            </PSelect>
          </Row>
          <Row label="Card preview">
            <PSelect value={view.cardCover} label="Card preview" className="w-[11rem]" onChange={(v) => set({ cardCover: v })}>
              <option value="">None</option>
              <option value="page">Page content</option>
              {files.map((f) => (
                <option key={f.id} value={f.id}>{f.name}</option>
              ))}
            </PSelect>
          </Row>
          <SwitchRow label="Fit image" checked={view.cardFit} onChange={(v) => set({ cardFit: v })} />
        </>
      )}
      {k === "list" && <SwitchRow label="Show sub-items" checked={view.showSubItems} onChange={(v) => set({ showSubItems: v })} />}
      {k === "chart" && (
        <div className="mt-2 border-t border-line pt-2">
          <ChartSettings chart={view.chart} props={db.props} onChange={(c) => set({ chart: c })} />
        </div>
      )}
    </div>
  );
}
