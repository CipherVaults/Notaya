// The popover editors of the cells that are not plain text: options, people, files, relations and places.

import { useEffect, useMemo, useRef, useState } from "react";
import { Check, Download, FolderOpen, Plus, Search, Trash2, Upload, X } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import type { FileCell, PlaceCell, Property, RowHit, SelectOption } from "@/lib/database-types";
import { cn, initials } from "@/lib/utils";
import { MAX_FILE_BYTES, STATUS_GROUPS, formatBytes } from "../model";
import { createOption, uploadFile } from "../ops";
import { rememberRow, useRowTitles } from "../rowTitles";
import { FileChip, OptionChip, StatusChip } from "./CellDisplay";

const searchCls =
  "h-8 w-full rounded-md border border-line bg-surface px-2.5 text-sm text-fg outline-none placeholder:text-faint focus:border-accent focus:ring-2 focus:ring-accent-soft";

/* ---------- select, multi-select, status ---------- */

interface OptionPickerProps {
  dbId: string;
  prop: Property;
  value: string | string[] | undefined;
  multi: boolean;
  readOnlyOptions?: boolean;
  onChange: (v: string | string[] | null) => void;
  onClose: () => void;
}

export function OptionPicker({ dbId, prop, value, multi, readOnlyOptions, onChange, onClose }: OptionPickerProps) {
  const [q, setQ] = useState("");
  const [hi, setHi] = useState(0);
  const selected = multi ? ((value as string[] | undefined) ?? []) : value ? [value as string] : [];
  const ql = q.trim().toLowerCase();
  const shown = prop.options.filter((o) => !ql || o.name.toLowerCase().includes(ql));
  const canCreate = !readOnlyOptions && !!ql && !prop.options.some((o) => o.name.toLowerCase() === ql);
  const rows: (SelectOption | "create")[] = [...shown, ...(canCreate ? (["create"] as const) : [])];

  const pick = async (o: SelectOption | "create") => {
    let opt: SelectOption | null;
    if (o === "create") {
      opt = await createOption(dbId, prop.id, q.trim());
      if (!opt) return;
    } else opt = o;
    if (multi) {
      onChange(selected.includes(opt.id) ? selected.filter((x) => x !== opt!.id) : [...selected, opt.id]);
      setQ("");
    } else {
      onChange(selected[0] === opt.id ? null : opt.id);
      onClose();
    }
  };

  const onKey = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setHi((h) => Math.min(rows.length - 1, h + 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setHi((h) => Math.max(0, h - 1));
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (rows[hi]) void pick(rows[hi]);
    } else if (e.key === "Backspace" && !q && multi && selected.length) {
      onChange(selected.slice(0, -1));
    }
  };

  const groups = prop.kind === "status" ? STATUS_GROUPS.map((g) => ({ ...g, items: shown.filter((o) => o.group === g.id) })).filter((g) => g.items.length) : [{ id: "", label: "", items: shown }];

  return (
    <div className="w-[240px]" onKeyDown={onKey}>
      <div className="flex flex-wrap items-center gap-1 border-b border-line p-2">
        {multi && selected.map((id) => {
          const o = prop.options.find((x) => x.id === id);
          return o ? (
            <span key={id} className="inline-flex items-center gap-0.5">
              <OptionChip option={o} />
              <button type="button" aria-label={`Remove ${o.name}`} onClick={() => onChange(selected.filter((x) => x !== id))} className="rounded p-0.5 text-faint hover:text-fg">
                <X size={11} />
              </button>
            </span>
          ) : null;
        })}
        <input
          autoFocus
          value={q}
          dir="auto"
          onChange={(e) => {
            setQ(e.target.value);
            setHi(0);
          }}
          placeholder={readOnlyOptions ? "Search…" : "Search or create…"}
          aria-label="Search options"
          className="h-6 min-w-[5rem] flex-1 bg-transparent text-sm outline-none placeholder:text-faint"
        />
      </div>
      <div className="max-h-[260px] overflow-y-auto p-1" role="listbox" aria-label={prop.name}>
        {groups.map((g) => (
          <div key={g.id}>
            {g.label && <div className="px-2 pb-0.5 pt-1.5 text-[0.7rem] font-medium uppercase tracking-wide text-faint">{g.label}</div>}
            {g.items.map((o) => {
              const idx = rows.indexOf(o);
              return (
                <button
                  key={o.id}
                  type="button"
                  role="option"
                  aria-selected={selected.includes(o.id)}
                  onMouseEnter={() => setHi(idx)}
                  onClick={() => void pick(o)}
                  className={cn("flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-start", idx === hi && "bg-surface-2")}
                >
                  {prop.kind === "status" ? <StatusChip option={o} /> : <OptionChip option={o} />}
                  <span className="flex-1" />
                  {selected.includes(o.id) && <Check size={14} className="text-muted" />}
                </button>
              );
            })}
          </div>
        ))}
        {canCreate && (
          <button
            type="button"
            onMouseEnter={() => setHi(rows.length - 1)}
            onClick={() => void pick("create")}
            className={cn("flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-start text-sm", hi === rows.length - 1 && "bg-surface-2")}
          >
            <Plus size={14} className="text-muted" /> Create <OptionChip option={{ name: q.trim(), color: "gray" }} />
          </button>
        )}
        {!rows.length && <div className="px-2 py-3 text-center text-sm text-faint">No options</div>}
      </div>
    </div>
  );
}

/* ---------- person ---------- */

export function PersonPicker({ value, onChange }: { value: string[] | undefined; onChange: (v: string[] | null) => void }) {
  const [q, setQ] = useState("");
  const names = value ?? [];
  const add = () => {
    const n = q.trim();
    if (!n || names.includes(n)) return;
    onChange([...names, n]);
    setQ("");
  };
  return (
    <div className="w-[240px] p-2">
      <div className="flex flex-wrap gap-1.5 pb-2">
        {names.map((n) => (
          <span key={n} className="inline-flex items-center gap-1.5 rounded-full bg-surface-2 py-0.5 pe-1.5 ps-0.5 text-sm">
            <span className="grid h-5 w-5 place-items-center rounded-full bg-accent-soft text-[0.62rem] font-semibold text-accent-strong">{initials(n)}</span>
            <span dir="auto">{n}</span>
            <button type="button" aria-label={`Remove ${n}`} onClick={() => onChange(names.filter((x) => x !== n).length ? names.filter((x) => x !== n) : null)} className="text-faint hover:text-fg">
              <X size={12} />
            </button>
          </span>
        ))}
      </div>
      <input
        autoFocus
        value={q}
        dir="auto"
        onChange={(e) => setQ(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            add();
          }
        }}
        placeholder="Type a name and press Enter"
        aria-label="Add a person"
        className={searchCls}
      />
    </div>
  );
}

/* ---------- files ---------- */

interface FilesProps {
  rowId: string;
  prop: Property;
  value: FileCell[] | undefined;
  onFiles: (files: FileCell[]) => void;
}

export function FilesPicker({ rowId, prop, value, onFiles }: FilesProps) {
  const [busy, setBusy] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  const files = value ?? [];
  const upload = async (list: FileList | null) => {
    if (!list || !list.length) return;
    setBusy(true);
    let latest: FileCell[] | null = null;
    for (const f of Array.from(list)) {
      try {
        latest = await uploadFile(rowId, prop.id, f, MAX_FILE_BYTES);
      } catch (e) {
        toast.error(errorText(e));
      }
    }
    if (latest) onFiles(latest);
    setBusy(false);
    if (input.current) input.current.value = "";
  };
  return (
    <div className="w-[300px] p-2">
      {files.length === 0 && <div className="py-2 text-center text-sm text-faint">No files yet</div>}
      <ul className="space-y-1">
        {files.map((f) => (
          <li key={f.id} className="flex items-center gap-1.5 rounded-md px-1.5 py-1 hover:bg-surface-2">
            <span className="min-w-0 flex-1"><FileChip file={f} /></span>
            <span className="shrink-0 text-xs text-faint">{formatBytes(f.size)}</span>
            <button type="button" aria-label={`Open ${f.name}`} title="Open" onClick={() => api.openFile(rowId, prop.id, f.id).catch((e) => toast.error(errorText(e)))} className="rounded p-1 text-muted hover:text-fg">
              <FolderOpen size={14} />
            </button>
            <button type="button" aria-label={`Save ${f.name} as`} title="Save as…" onClick={() => api.saveFileAs(rowId, prop.id, f.id).then((p) => p && toast.success(`Saved to ${p}`)).catch((e) => toast.error(errorText(e)))} className="rounded p-1 text-muted hover:text-fg">
              <Download size={14} />
            </button>
            <button
              type="button"
              aria-label={`Remove ${f.name}`}
              title="Remove"
              onClick={() => api.removeFile(rowId, prop.id, f.id).then(onFiles).catch((e) => toast.error(errorText(e)))}
              className="rounded p-1 text-muted hover:text-danger"
            >
              <Trash2 size={14} />
            </button>
          </li>
        ))}
      </ul>
      <input ref={input} type="file" multiple hidden aria-label="Choose files" onChange={(e) => void upload(e.target.files)} />
      <button
        type="button"
        disabled={busy}
        onClick={() => input.current?.click()}
        className="mt-2 flex w-full items-center justify-center gap-2 rounded-md border border-dashed border-line py-2 text-sm text-muted hover:bg-surface-2 hover:text-fg disabled:opacity-50"
      >
        <Upload size={14} /> {busy ? "Uploading…" : "Upload a file (up to 900 MB)"}
      </button>
    </div>
  );
}

/* ---------- relation ---------- */

export function RelationPicker({ prop, value, onChange }: { prop: Property; value: string[] | undefined; onChange: (v: string[] | null) => void }) {
  const [q, setQ] = useState("");
  const [hits, setHits] = useState<RowHit[]>([]);
  const [err, setErr] = useState("");
  const ids = value ?? [];
  const titles = useRowTitles(ids);
  useEffect(() => {
    let alive = true;
    const t = setTimeout(() => {
      api.searchRows(prop.relationDb, q, 30).then((h) => alive && (setHits(h), setErr(""))).catch((e) => alive && setErr(errorText(e)));
    }, 120);
    return () => {
      alive = false;
      clearTimeout(t);
    };
  }, [q, prop.relationDb]);
  const toggle = (h: RowHit) => {
    rememberRow(h);
    if (ids.includes(h.id)) {
      const next = ids.filter((x) => x !== h.id);
      onChange(next.length ? next : null);
    } else onChange(prop.relationLimitOne ? [h.id] : [...ids, h.id]);
  };
  return (
    <div className="w-[270px]">
      <div className="flex flex-wrap items-center gap-1 border-b border-line p-2">
        {ids.map((id) => (
          <span key={id} className="inline-flex items-center gap-1 rounded-md bg-surface-2 px-1.5 py-0.5 text-[0.86em]">
            <span dir="auto" className="max-w-[9rem] truncate">{titles.get(id)?.title || "Untitled"}</span>
            <button type="button" aria-label="Remove link" onClick={() => onChange(ids.filter((x) => x !== id).length ? ids.filter((x) => x !== id) : null)} className="text-faint hover:text-fg">
              <X size={11} />
            </button>
          </span>
        ))}
        <span className="relative min-w-[6rem] flex-1">
          <Search size={13} className="pointer-events-none absolute start-1 top-1/2 -translate-y-1/2 text-faint" />
          <input autoFocus value={q} dir="auto" onChange={(e) => setQ(e.target.value)} placeholder="Search rows…" aria-label="Search rows" className="h-6 w-full bg-transparent ps-6 text-sm outline-none placeholder:text-faint" />
        </span>
      </div>
      <div className="max-h-[240px] overflow-y-auto p-1">
        {err && <div className="px-2 py-2 text-sm text-danger">{err}</div>}
        {hits.map((h) => (
          <button key={h.id} type="button" onClick={() => toggle(h)} className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-start text-sm hover:bg-surface-2">
            {h.icon && <span className="emoji-mono">{h.icon}</span>}
            <span dir="auto" className="min-w-0 flex-1 truncate">{h.title || "Untitled"}</span>
            {ids.includes(h.id) && <Check size={14} className="text-muted" />}
          </button>
        ))}
        {!err && !hits.length && <div className="px-2 py-3 text-center text-sm text-faint">No rows found</div>}
      </div>
    </div>
  );
}

/* ---------- place ---------- */

export function PlacePicker({ value, onChange, onClose }: { value: PlaceCell | undefined; onChange: (v: PlaceCell | null) => void; onClose: () => void }) {
  const [address, setAddress] = useState(value?.address ?? "");
  const [lat, setLat] = useState(value?.lat != null ? String(value.lat) : "");
  const [lng, setLng] = useState(value?.lng != null ? String(value.lng) : "");
  const num = (s: string) => (s.trim() === "" || Number.isNaN(Number(s)) ? null : Number(s));
  const save = () => {
    const a = address.trim();
    const la = num(lat);
    const ln = num(lng);
    onChange(!a && la == null && ln == null ? null : { address: a, lat: la, lng: ln });
    onClose();
  };
  return (
    <div className="w-[270px] space-y-2 p-2.5" onKeyDown={(e) => e.key === "Enter" && (e.preventDefault(), save())}>
      <input autoFocus value={address} dir="auto" onChange={(e) => setAddress(e.target.value)} placeholder="Address or place name" aria-label="Address" className={searchCls} />
      <div className="flex gap-1.5">
        <input value={lat} inputMode="decimal" onChange={(e) => setLat(e.target.value)} placeholder="Latitude" aria-label="Latitude" className={searchCls} />
        <input value={lng} inputMode="decimal" onChange={(e) => setLng(e.target.value)} placeholder="Longitude" aria-label="Longitude" className={searchCls} />
      </div>
      <div className="flex justify-between">
        <button type="button" onClick={() => (onChange(null), onClose())} className="rounded-md px-2 py-1 text-sm text-muted hover:bg-surface-2">Clear</button>
        <button type="button" onClick={save} className="rounded-md bg-accent px-3 py-1 text-sm font-medium text-accent-fg hover:opacity-90">Done</button>
      </div>
    </div>
  );
}

/** Helper for panels that list options without editing a cell. */
export function useOptionIndex(prop: Property | undefined) {
  return useMemo(() => new Map((prop?.options ?? []).map((o) => [o.id, o])), [prop]);
}
