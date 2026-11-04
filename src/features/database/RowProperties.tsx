// The properties block shown above the body of a row page.

import { useCallback, useEffect, useMemo, useState } from "react";
import { Loader2 } from "lucide-react";
import type { DbRow, Property } from "@/lib/database-types";
import { loadDatabase, peekDatabase, useDatabase } from "./dbStore";
import { afterRowChange } from "./notices";
import { saveCells } from "./ops";
import { loadRow } from "./rowOps";
import PropertyLines from "./PropertyLines";
import { rememberRow } from "./rowTitles";

interface Props {
  dbId: string;
  rowId: string;
  /** The title as typed in the page header, so relation chips elsewhere show it at once. */
  title?: string;
  readOnly?: boolean;
}

export default function RowProperties({ dbId, rowId, title, readOnly }: Props) {
  const db = useDatabase(dbId);
  const [row, setRow] = useState<DbRow | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    let alive = true;
    setRow(null);
    setFailed(false);
    if (!db) void loadDatabase(dbId).catch(() => alive && setFailed(true));
    loadRow(dbId, rowId).then((r) => alive && (r ? setRow(r) : setFailed(true)));
    return () => {
      alive = false;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dbId, rowId]);

  // A button or an automation changed the row: read it again.
  useEffect(() => {
    const again = () => void loadRow(dbId, rowId).then((r) => r && setRow(r));
    window.addEventListener("notaya:rows-changed", again);
    return () => window.removeEventListener("notaya:rows-changed", again);
  }, [dbId, rowId]);

  useEffect(() => {
    if (row && title !== undefined) rememberRow({ id: rowId, title, icon: row.icon });
  }, [title, row, rowId]);

  const commit = useCallback(
    async (p: Property, value: unknown) => {
      setRow((r) => {
        if (!r) return r;
        const values = { ...r.values };
        if (value == null) delete values[p.id];
        else values[p.id] = value;
        return { ...r, values };
      });
      const saved = await saveCells(rowId, { [p.id]: value ?? null });
      if (saved) setRow(saved);
      else void loadRow(dbId, rowId).then((r) => r && setRow(r));
      afterRowChange(!!peekDatabase(dbId)?.automations?.some((a) => a.enabled));
    },
    [rowId, dbId],
  );
  const local = useCallback((p: Property, value: unknown) => {
    setRow((r) => {
      if (!r) return r;
      const values = { ...r.values };
      if (value == null) delete values[p.id];
      else values[p.id] = value;
      return { ...r, values };
    });
  }, []);

  const hiddenIds = useMemo(() => new Set((db?.views[0]?.columns ?? []).filter((c) => !c.visible).map((c) => c.prop)), [db]);

  if (failed) return null;
  if (!db || !row) {
    return (
      <div className="mb-3 flex h-10 items-center gap-2 text-sm text-faint">
        <Loader2 size={14} className="animate-spin" /> Loading properties…
      </div>
    );
  }
  if (db.props.length <= 1) return null;
  return <PropertyLines db={db} row={row} readOnly={readOnly} hiddenIds={hiddenIds} onCommit={(p, v) => void commit(p, v)} onLocalChange={local} />;
}
