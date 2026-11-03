// Templates of a database: a name, an icon, default values for the properties, an optional repeat and a default flag.
// The page body of a template comes from "Save as template" on a row page.

import { useMemo, useState } from "react";
import { Plus, Star, Trash2 } from "lucide-react";
import type { Database, DbRow, Property, Template } from "@/lib/database-types";
import { cn } from "@/lib/utils";
import { Button, Modal } from "@/components/ui";
import { useDatabase, editDatabase } from "./dbStore";
import { newId } from "./model";
import PropertyLines from "./PropertyLines";
import { PSelect, fieldCls } from "./panels/ui";

function parseCells(t: Template): Record<string, unknown> {
  try {
    const v = JSON.parse(t.cells || "{}");
    return v && typeof v === "object" && !Array.isArray(v) ? v : {};
  } catch {
    return {};
  }
}

export default function TemplatesDialog({ db: initial, onClose, onSaved }: { db: Database; onClose: () => void; onSaved: () => void }) {
  const live = useDatabase(initial.id) ?? initial;
  const [sel, setSel] = useState<string | null>(live.templates[0]?.id ?? null);
  const t = live.templates.find((x) => x.id === sel) ?? null;
  const edit = (patch: Partial<Template>) => {
    void editDatabase(live.id, (d) => ({ ...d, templates: d.templates.map((x) => (x.id === sel ? { ...x, ...patch } : x)) }));
    onSaved();
  };
  const add = () => {
    const nt: Template = { id: newId(), name: "New template", icon: "", cells: "{}", content: "", text: "", repeat: "", repeatLast: 0 };
    void editDatabase(live.id, (d) => ({ ...d, templates: [...d.templates, nt] }), true);
    setSel(nt.id);
  };
  const remove = () => {
    if (!t) return;
    const rest = live.templates.filter((x) => x.id !== t.id);
    void editDatabase(live.id, (d) => ({ ...d, templates: rest, defaultTemplate: d.defaultTemplate === t.id ? "" : d.defaultTemplate }), true);
    setSel(rest[0]?.id ?? null);
  };
  const cells = useMemo(() => (t ? parseCells(t) : {}), [t]);
  const fake: DbRow | null = t ? { id: "", title: "", icon: t.icon, number: 0, created: 0, updated: 0, favorite: false, values: cells } : null;
  const setCell = (p: Property, v: unknown) => {
    const next = { ...cells };
    if (v == null) delete next[p.id];
    else next[p.id] = v;
    edit({ cells: JSON.stringify(next) });
  };

  return (
    <Modal open onOpenChange={(o) => !o && onClose()} title="Templates" description="Rows made from a template start with these values." width={720}>
      <div className="flex min-h-[320px] gap-4">
        <div className="w-48 shrink-0 space-y-1">
          {live.templates.map((x) => (
            <button
              key={x.id}
              type="button"
              onClick={() => setSel(x.id)}
              className={cn("flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-start text-sm", x.id === sel ? "bg-surface-2 font-medium" : "hover:bg-surface")}
            >
              <span className="emoji-mono">{x.icon || "📄"}</span>
              <span dir="auto" className="min-w-0 flex-1 truncate">{x.name || "Untitled"}</span>
              {live.defaultTemplate === x.id && <Star size={12} className="shrink-0" fill="currentColor" aria-label="Default" />}
            </button>
          ))}
          <Button variant="outline" size="sm" className="w-full" onClick={add}>
            <Plus size={14} /> New template
          </Button>
        </div>
        <div className="min-w-0 flex-1">
          {!t || !fake ? (
            <p className="py-10 text-center text-sm text-muted">No templates yet. Add one, or choose "Save as template" in the menu of a row page.</p>
          ) : (
            <div className="space-y-3">
              <div className="flex gap-2">
                <input aria-label="Template icon" value={t.icon} maxLength={4} onChange={(e) => edit({ icon: e.target.value })} placeholder="📄" className={cn(fieldCls, "w-14 text-center")} />
                <input aria-label="Template name" dir="auto" value={t.name} onChange={(e) => edit({ name: e.target.value })} className={cn(fieldCls, "min-w-0 flex-1")} />
              </div>
              <div className="flex flex-wrap items-center gap-x-4 gap-y-2 text-sm">
                <label className="flex cursor-pointer items-center gap-2">
                  <input
                    type="checkbox"
                    checked={live.defaultTemplate === t.id}
                    onChange={(e) => void editDatabase(live.id, (d) => ({ ...d, defaultTemplate: e.target.checked ? t.id : "" }), true)}
                    className="h-4 w-4 accent-[var(--accent)]"
                  />
                  Use for every new row
                </label>
                <label className="flex items-center gap-2">
                  Repeat
                  <PSelect value={t.repeat} label="Repeat" className="w-28" onChange={(v) => edit({ repeat: v })}>
                    <option value="">Never</option>
                    <option value="daily">Daily</option>
                    <option value="weekly">Weekly</option>
                    <option value="monthly">Monthly</option>
                  </PSelect>
                </label>
              </div>
              {t.repeat && <p className="text-xs text-muted">A row is added from this template when the database is opened and one is due.</p>}
              <div>
                <div className="pb-1 text-xs font-semibold uppercase tracking-wider text-faint">Default values</div>
                <PropertyLines db={live} row={fake} noFiles onCommit={setCell} />
              </div>
              {t.content && <p className="text-xs text-muted">This template also has a page body.</p>}
              <Button variant="ghost" size="sm" className="text-danger hover:text-danger" onClick={remove}>
                <Trash2 size={14} /> Delete template
              </Button>
            </div>
          )}
        </div>
      </div>
    </Modal>
  );
}
