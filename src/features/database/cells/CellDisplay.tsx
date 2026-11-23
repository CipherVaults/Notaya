// How a cell looks (read-only). Editing is in CellEditor.tsx. Every text is dir="auto" so Arabic and Hebrew start on the right.

import { useState } from "react";
import { Check, File as FileIcon, Image as ImageIcon, MapPin, MousePointerClick } from "lucide-react";
import { api, errorText } from "@/lib/ipc";
import { toast } from "sonner";
import type { DateCell, DbRow, FileCell, PlaceCell, Property, SelectOption } from "@/lib/database-types";
import { cn, initials } from "@/lib/utils";
import { useApp } from "@/store/app";
import { formatBytes, formatDateCell, formatNumber, isComputedValue, optClass, optDot } from "../model";
import { useRowTitles } from "../rowTitles";
import { pollNotices } from "../notices";
import { rowsChanged } from "../dbStore";

export function OptionChip({ option, className }: { option: Pick<SelectOption, "name" | "color"> & { group?: string }; className?: string }) {
  return (
    <span className={cn(optClass(option.color), className)} dir="auto" title={option.name}>
      <span>{option.name || "Untitled"}</span>
    </span>
  );
}

export function StatusChip({ option }: { option: SelectOption }) {
  return (
    <span className={cn(optClass(option.color), "gap-1.5")} dir="auto" title={option.name}>
      <span className="h-2 w-2 shrink-0 rounded-full" style={{ background: optDot(option.color) }} aria-hidden />
      <span>{option.name || "Untitled"}</span>
    </span>
  );
}

// Only web addresses open (the backend allows nothing else). Email and phone values are shown as text.
export function openLink(value: string) {
  const v = value.trim();
  const target = /^[a-z][a-z0-9+.-]*:/i.test(v) ? v : `https://${v}`;
  api.openUrl(target).catch((e) => toast.error(errorText(e)));
}

function Ring({ value, max }: { value: number; max: number }) {
  const f = max > 0 ? Math.max(0, Math.min(1, value / max)) : 0;
  const r = 7;
  const c = 2 * Math.PI * r;
  return (
    <svg width="18" height="18" viewBox="0 0 18 18" className="shrink-0 -rotate-90" aria-hidden>
      <circle cx="9" cy="9" r={r} fill="none" stroke="currentColor" strokeOpacity="0.18" strokeWidth="2.5" />
      <circle cx="9" cy="9" r={r} fill="none" stroke="var(--accent-strong)" strokeWidth="2.5" strokeLinecap="round" strokeDasharray={`${c * f} ${c}`} />
    </svg>
  );
}

export function NumberValue({ prop, value, text = formatNumber(value, prop.numberFormat) }: {
  prop: Property; value: number; text?: string;
}) {
  if (prop.numberDisplay === "bar" || prop.numberDisplay === "ring") {
    const max = prop.numberMax > 0 ? prop.numberMax : 100;
    return (
      <span className="flex min-w-0 items-center gap-2">
        {prop.numberDisplay === "ring" ? (
          <Ring value={value} max={max} />
        ) : (
          <span className="h-1.5 w-14 shrink-0 overflow-hidden rounded-full bg-surface-2" aria-hidden>
            <span className="block h-full rounded-full bg-accent-strong" style={{ width: `${Math.max(0, Math.min(100, (value / max) * 100))}%` }} />
          </span>
        )}
        <span className="truncate tabular-nums">{text}</span>
      </span>
    );
  }
  return <span className="truncate tabular-nums" dir="auto">{text}</span>;
}

function Chips({ children }: { children: React.ReactNode }) {
  return <span className="flex min-w-0 flex-wrap items-center gap-1">{children}</span>;
}

export function FileChip({ file }: { file: FileCell }) {
  const image = file.mime.startsWith("image/");
  return (
    <span className="inline-flex max-w-full items-center gap-1 rounded-md bg-surface-2 px-1.5 py-0.5 text-[0.8em]" title={`${file.name} (${formatBytes(file.size)})`}>
      {image ? <ImageIcon size={12} className="shrink-0 text-muted" /> : <FileIcon size={12} className="shrink-0 text-muted" />}
      <span dir="auto" className="truncate">{file.name}</span>
    </span>
  );
}

export function RelationChips({ ids, wrap }: { ids: string[]; wrap?: boolean }) {
  const titles = useRowTitles(ids);
  const openRow = useApp((s) => s.openRow);
  return (
    <span className={cn("flex min-w-0 items-center gap-1", wrap ? "flex-wrap" : "overflow-hidden")}>
      {ids.map((id) => {
        const t = titles.get(id)!;
        return (
          <button
            key={id}
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              openRow(id);
            }}
            className="inline-flex max-w-full items-center gap-1 rounded-md bg-surface-2 px-1.5 py-0.5 text-[0.86em] hover:bg-line"
            title={t.title || "Untitled"}
          >
            {t.icon && <span className="emoji-mono">{t.icon}</span>}
            <span dir="auto" className="truncate">{t.title || "Untitled"}</span>
          </button>
        );
      })}
    </span>
  );
}

export function CheckBox({ checked, className }: { checked: boolean; className?: string }) {
  return (
    <span
      className={cn("inline-grid h-4 w-4 shrink-0 place-items-center rounded border", checked ? "border-accent bg-accent text-accent-fg" : "border-faint bg-transparent", className)}
      aria-hidden
    >
      {checked && <Check size={12} strokeWidth={3} />}
    </span>
  );
}

/** A button property: pressing it runs its actions on the row (Rust: `press_button`). */
export function ButtonCell({ prop, row }: { prop: Property; row?: DbRow }) {
  const [busy, setBusy] = useState(false);
  const label = prop.buttonLabel || prop.name || "Button";
  return (
    <button
      type="button"
      disabled={!row || busy || prop.actions.length === 0}
      title={prop.actions.length === 0 ? "This button has no actions yet (edit the property)" : label}
      onClick={async (e) => {
        e.stopPropagation();
        if (!row) return;
        setBusy(true);
        try {
          await api.pressButton(row.id, prop.id);
        } catch (err) {
          toast.error(errorText(err));
        } finally {
          setBusy(false);
        }
        await pollNotices();
        rowsChanged();
      }}
      className="inline-flex max-w-full items-center gap-1.5 rounded-md border border-line bg-surface px-2 py-0.5 text-[0.86em] font-medium text-fg hover:bg-surface-2 disabled:opacity-50"
    >
      <MousePointerClick size={12} className="shrink-0 text-muted" aria-hidden />
      <span dir="auto" className="truncate">{label}</span>
    </button>
  );
}

interface Props {
  prop: Property;
  value: unknown;
  row?: DbRow;
  /** Let long text and chips wrap onto more lines. */
  wrap?: boolean;
}

/** The value of one cell as it is shown in tables, cards and the property list of a row. */
export default function CellDisplay({ prop, value, row, wrap }: Props) {
  // `block`, so the text lines up with its own direction (Arabic and Hebrew start at the right edge of the cell).
  const trunc = wrap ? "block whitespace-pre-wrap break-words" : "block truncate";
  if (prop.kind === "title") {
    const title = row?.title ?? (typeof value === "string" ? value : "");
    return (
      <span className="flex min-w-0 items-center gap-1.5 font-medium">
        {row?.icon && <span className="emoji-mono shrink-0">{row.icon}</span>}
        <span dir="auto" className={cn("min-w-0 flex-1", trunc, !title && "text-faint")}>{title || "Untitled"}</span>
      </span>
    );
  }
  if (prop.kind === "button") return <ButtonCell prop={prop} row={row} />;
  if (isComputedValue(value)) return <ComputedValue prop={prop} cell={value} wrap={wrap} />;
  if (value == null || value === "") {
    return prop.kind === "checkbox" ? <CheckBox checked={false} /> : null;
  }
  switch (prop.kind) {
    case "text":
      return <span dir="auto" className={trunc}>{String(value)}</span>;
    case "number":
      return typeof value === "number" ? <NumberValue prop={prop} value={value} text={row?.numberText?.[prop.id]} /> : null;
    case "select": {
      const o = prop.options.find((x) => x.id === value);
      return o ? <OptionChip option={o} /> : null;
    }
    case "status": {
      const o = prop.options.find((x) => x.id === value);
      return o ? <StatusChip option={o} /> : null;
    }
    case "multiSelect": {
      const opts = (value as string[]).map((id) => prop.options.find((x) => x.id === id)).filter((o): o is SelectOption => !!o);
      return <span className={cn("flex min-w-0 items-center gap-1", wrap ? "flex-wrap" : "overflow-hidden")}>{opts.map((o) => <OptionChip key={o.id} option={o} />)}</span>;
    }
    case "date":
      return <span dir="auto" className={trunc}>{formatDateCell(value as DateCell, prop.dateFormat)}</span>;
    case "person":
      return (
        <Chips>
          {(value as string[]).map((n, i) => (
            <span key={n + i} className="inline-flex max-w-full items-center gap-1.5">
              <span className="grid h-5 w-5 shrink-0 place-items-center rounded-full bg-accent-soft text-[0.62rem] font-semibold text-accent-strong">{initials(n)}</span>
              <span dir="auto" className="truncate">{n}</span>
            </span>
          ))}
        </Chips>
      );
    case "files":
      return <Chips>{(value as FileCell[]).map((f) => <FileChip key={f.id} file={f} />)}</Chips>;
    case "checkbox":
      return <CheckBox checked={value === true} />;
    case "email":
    case "phone":
      return <span dir="auto" className={cn("text-fg", trunc)} title={String(value)}>{String(value)}</span>;
    case "url":
      return (
        <a
          href="#"
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            openLink(String(value));
          }}
          dir="auto"
          className={cn("text-accent-strong underline decoration-line underline-offset-2 hover:decoration-current", trunc)}
          title={String(value)}
        >
          {String(value)}
        </a>
      );
    case "relation":
      return <RelationChips ids={value as string[]} wrap={wrap} />;
    case "place": {
      const p = value as PlaceCell;
      return (
        <span className="flex min-w-0 items-center gap-1.5">
          <MapPin size={13} className="shrink-0 text-muted" />
          <span dir="auto" className={trunc}>{p.address || (p.lat != null && p.lng != null ? `${p.lat}, ${p.lng}` : "")}</span>
        </span>
      );
    }
    default:
      return <span dir="auto" className={trunc}>{String(value)}</span>;
  }
}

function ComputedValue({ prop, cell, wrap }: { prop: Property; cell: { t: string; v: unknown; text: string }; wrap?: boolean }) {
  // `block`, so the text lines up with its own direction (Arabic and Hebrew start at the right edge of the cell).
  const trunc = wrap ? "block whitespace-pre-wrap break-words" : "block truncate";
  if (cell.t === "error") {
    return <span dir="auto" className={cn("text-danger", trunc)} title={cell.text}>{cell.text}</span>;
  }
  if (cell.t === "null" || (cell.text === "" && cell.t !== "bool")) return null;
  if (cell.t === "bool") return <CheckBox checked={cell.v === true} />;
  if (cell.t === "number" && typeof cell.v === "number" && (prop.numberDisplay === "bar" || prop.numberDisplay === "ring")) {
    return <NumberValue prop={prop} value={cell.v} text={cell.text} />;
  }
  if (cell.t === "list" && Array.isArray(cell.v) && prop.kind !== "formula" && prop.kind !== "rollup") {
    return <span dir="auto" className={trunc}>{cell.text}</span>;
  }
  const num = cell.t === "number";
  return <span dir="auto" className={cn(trunc, num && "tabular-nums")} title={cell.text}>{cell.text}</span>;
}
