// A cell that can be edited in place: text-like kinds turn into an input, the others open a popover.

import { useEffect, useRef, useState } from "react";
import { Popover } from "radix-ui";
import { toast } from "sonner";
import type { DateCell, DbRow, FileCell, PlaceCell, Property } from "@/lib/database-types";
import { cn } from "@/lib/utils";
import { isComputedKind } from "../model";
import CellDisplay, { CheckBox } from "./CellDisplay";
import { DateEditor } from "./DatePicker";
import { decimalInput } from "./decimalInput";
import { FilesPicker, OptionPicker, PersonPicker, PlacePicker, RelationPicker } from "./pickers";

export interface EditableCellProps {
  dbId: string;
  prop: Property;
  row: DbRow;
  value: unknown;
  readOnly?: boolean;
  editing: boolean;
  setEditing: (b: boolean) => void;
  /** Saves a new value (null clears). For the title it is the new title text. */
  commit: (value: unknown) => void;
  /** The backend already changed the cell (files); only the screen needs the new value. */
  onLocalChange?: (value: unknown) => void;
  wrap?: boolean;
  /** The character that started editing by typing (replaces the old text). */
  seed?: string;
  /** After a text edit ended: how, so a grid can move the selection. */
  onEnd?: (how: "enter" | "tab" | "blur" | "escape") => void;
}

const TEXT_KINDS = new Set(["title", "text", "url", "email", "phone", "number"]);

export default function EditableCell(p: EditableCellProps) {
  const { prop, row, value, readOnly, editing, setEditing, commit } = p;
  const computed = isComputedKind(prop.kind);
  const display = <CellDisplay prop={prop} value={prop.kind === "title" ? row.title : value} row={row} wrap={p.wrap} />;

  if (computed || (readOnly && prop.kind !== "checkbox")) return display;

  if (prop.kind === "checkbox") {
    return (
      <button
        type="button"
        role="checkbox"
        aria-checked={value === true}
        aria-label={prop.name}
        disabled={readOnly}
        onClick={(e) => {
          e.stopPropagation();
          commit(value !== true);
        }}
        className="inline-flex rounded"
      >
        <CheckBox checked={value === true} />
      </button>
    );
  }

  if (TEXT_KINDS.has(prop.kind)) {
    if (!editing) return display;
    return <TextInput {...p} />;
  }

  // Popover kinds
  const body = (() => {
    const close = () => setEditing(false);
    switch (prop.kind) {
      case "select":
      case "status":
        return <OptionPicker dbId={p.dbId} prop={prop} value={value as string | undefined} multi={false} onChange={(v) => commit(v)} onClose={close} />;
      case "multiSelect":
        return <OptionPicker dbId={p.dbId} prop={prop} value={value as string[] | undefined} multi onChange={(v) => commit(v && (v as string[]).length ? v : null)} onClose={close} />;
      case "date":
        return <DateEditor value={value as DateCell | undefined} onChange={(v) => commit(v)} />;
      case "person":
        return <PersonPicker value={value as string[] | undefined} onChange={(v) => commit(v)} />;
      case "files":
        return <FilesPicker rowId={row.id} prop={prop} value={value as FileCell[] | undefined} onFiles={(f) => p.onLocalChange?.(f.length ? f : null)} />;
      case "relation":
        return <RelationPicker prop={prop} value={value as string[] | undefined} onChange={(v) => commit(v)} />;
      case "place":
        return <PlacePicker value={value as PlaceCell | undefined} onChange={(v) => commit(v)} onClose={close} />;
      default:
        return null;
    }
  })();
  return (
    <Popover.Root open={editing} onOpenChange={setEditing}>
      <Popover.Anchor asChild>
        <div className="min-w-0">{display}</div>
      </Popover.Anchor>
      <Popover.Portal>
        <Popover.Content
          align="start"
          sideOffset={4}
          collisionPadding={12}
          onClick={(e) => e.stopPropagation()}
          onCloseAutoFocus={(e) => {
            e.preventDefault();
            p.onEnd?.("blur"); // a grid takes the focus back, so the keys keep working
          }}
          className="z-[80] max-h-[var(--radix-popover-content-available-height)] overflow-y-auto rounded-lg border border-line bg-elevated shadow-float outline-none"
        >
          {body}
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}

function TextInput({ prop, row, value, setEditing, commit, seed, onEnd }: EditableCellProps) {
  const start = (() => {
    if (seed !== undefined) return seed;
    if (prop.kind === "title") return row.title;
    if (prop.kind === "number") return row.numberInput?.[prop.id] ?? (typeof value === "number" ? String(value) : "");
    return typeof value === "string" ? value : "";
  })();
  const [text, setText] = useState(start);
  const [error, setError] = useState(false);
  const done = useRef(false);
  const ref = useRef<HTMLInputElement>(null);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    el.focus();
    if (seed === undefined) el.select();
    else el.setSelectionRange(seed.length, seed.length);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const finish = (save: boolean, how: "enter" | "tab" | "blur" | "escape") => {
    if (done.current) return;
    let token: string | null = null;
    if (save && prop.kind === "number") {
      try {
        token = decimalInput(text);
      } catch (e) {
        if (!error && e instanceof Error) toast.error(e.message);
        setError(true);
        return;
      }
    }
    done.current = true;
    if (save) {
      if (prop.kind === "number") {
        if (seed !== undefined || token !== (start || null)) commit(token);
      } else if (prop.kind === "title") {
        if (text !== row.title) commit(text);
      } else {
        const next = text.trim();
        if (next !== (typeof value === "string" ? value : "")) commit(next === "" ? null : text);
      }
    }
    setEditing(false);
    onEnd?.(how);
  };

  return (
    <input
      ref={ref}
      data-cell-editor
      value={text}
      dir="auto"
      aria-label={prop.name}
      aria-invalid={error || undefined}
      inputMode={prop.kind === "number" ? "decimal" : undefined}
      onChange={(e) => {
        setText(e.target.value);
        setError(false);
      }}
      onClick={(e) => e.stopPropagation()}
      onBlur={() => finish(true, "blur")}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          e.stopPropagation();
          finish(true, "enter");
        } else if (e.key === "Escape") {
          e.preventDefault();
          e.stopPropagation();
          finish(false, "escape");
        } else if (e.key === "Tab") {
          e.preventDefault();
          e.stopPropagation();
          finish(true, "tab");
        } else e.stopPropagation();
      }}
      className={cn("w-full min-w-0 bg-transparent outline-none", prop.kind === "number" && "tabular-nums", prop.kind === "title" && "font-medium")}
    />
  );
}
