// The database shell: title, icon and description, view tabs, toolbar (filter, sort, group, properties, search, new, menu)
// and the active view. Used as a full page and, with `compact`, inside a page (inline database or linked view).

import { Suspense, lazy, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Dialog, Popover } from "radix-ui";
import {
  AlertTriangle, ArrowUpDown, ChevronDown, Copy, Download, ExternalLink, FileUp, Filter as FilterIcon, GitBranch, Group, ListTree, Loader2, Lock, LockOpen,
  MoreHorizontal, Plus, Search, SlidersHorizontal, Table as TableIcon, Trash2, X, Settings2, LayoutTemplate, Zap,
} from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import type { Database, DbRow, Filter, GroupOut, Property, RowRange, View, ViewKind, ViewResult, ViewWindow } from "@/lib/database-types";
import { PAGE_EMOJIS } from "@/lib/palette";
import { cn } from "@/lib/utils";
import { useApp } from "@/store/app";
import { Menu, MenuContent, MenuItem, MenuSeparator, MenuTrigger, useConfirm } from "@/components/ui";
import { DbShellContext, type DbShell } from "./context";
import { editDatabase, loadDatabase, setDatabase, useDatabase, useSaveCount } from "./dbStore";
import { ViewIcon } from "./icons";
import { VIEW_KINDS, blankView, countConds, emptyFilter, filterIsEmpty, reorderList, rowMatchesFind } from "./model";
import { saveCells } from "./ops";
import { afterRowChange } from "./notices";
import { dropUndo, handleUndoKey, pushUndo, retainUndo } from "./undo";
import { FilterGroup } from "./panels/FilterPanel";
import { GroupPanel, SortPanel } from "./panels/SortGroupPanels";
import { PropertiesPanel, ViewSettingsPanel } from "./panels/ViewPanels";
import { PropertyMenu } from "./panels/PropertyMenu";
import { PanelPopover } from "./panels/ui";
import { viewComponent } from "./views/registry";
import type { ViewProps } from "./viewTypes";
import TemplatesDialog from "./TemplatesDialog";
import AutomationsDialog from "./AutomationsDialog";

const RowPeek = lazy(() => import("./RowPeek"));

export interface DatabaseViewProps {
  databaseId: string;
  /** The view to show first (a linked view block remembers its own). */
  viewId?: string;
  onViewChange?: (viewId: string) => void;
  compact?: boolean;
}

function filterGroups(gs: GroupOut[] | null, keep: Set<string>): GroupOut[] | null {
  if (!gs) return null;
  return gs.map((g) => {
    const rows = g.rows.filter((id) => keep.has(id));
    return { ...g, rows, count: rows.length, subs: g.subs.map((s) => ({ ...s, rows: s.rows.filter((id) => keep.has(id)), count: s.rows.filter((id) => keep.has(id)).length })) };
  });
}

type ShellResult = ViewResult & {
  window?: ViewWindow;
  windowQuery?: { viewKey: string; find: string };
  rowsGeneration?: string;
  pendingRowIds?: string[];
};

const ghostBtn = "inline-flex h-7 shrink-0 items-center gap-1.5 rounded-md px-2 text-[0.8rem] text-muted transition-colors hover:bg-surface-2 hover:text-fg data-[state=open]:bg-surface-2 data-[state=open]:text-fg";

export default function DatabaseView({ databaseId, viewId: viewIdProp, onViewChange, compact = false }: DatabaseViewProps) {
  const db = useDatabase(databaseId);
  const saveCount = useSaveCount(databaseId);
  const tree = useApp((s) => s.tree);
  const setTree = useApp((s) => s.setTree);
  const openRowPage = useApp((s) => s.openRow);
  const openDatabase = useApp((s) => s.openDatabase);
  const confirm = useConfirm();
  const [loadError, setLoadError] = useState<string | null>(null);
  const [localViewId, setViewId] = useState<string | undefined>(viewIdProp);
  // Full-page navigation is controlled by the vault-backed store; linked blocks own their view.
  const viewId = onViewChange ? viewIdProp : localViewId;
  const [result, setResult] = useState<ShellResult | null>(null);
  const [queryError, setQueryError] = useState<string | null>(null);
  const [tick, setTick] = useState(0);
  const [search, setSearch] = useState("");
  const [searchOpen, setSearchOpen] = useState(false);
  const [peek, setPeek] = useState<{ id: string; mode: "side" | "center" } | null>(null);
  const [focusRowId, setFocusRowId] = useState<string | null>(null);
  const [propMenu, setPropMenu] = useState<string | null>(null);
  const [renaming, setRenaming] = useState<string | null>(null);
  const [templates, setTemplates] = useState(false);
  const [automations, setAutomations] = useState(false);
  const reqRef = useRef(0);
  const requeryTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  // Load the definition.
  useEffect(() => {
    let alive = true;
    setLoadError(null);
    setResult(null);
    loadDatabase(databaseId).catch((e) => alive && setLoadError(errorText(e)));
    return () => {
      alive = false;
    };
  }, [databaseId]);

  const view: View | undefined = useMemo(() => db?.views.find((v) => v.id === viewId) ?? db?.views[0], [db, viewId]);
  const viewKey = useMemo(() => JSON.stringify(view), [view]);
  const propsKey = useMemo(() => JSON.stringify(db?.props ?? []), [db?.props]);
  const locked = !!db?.locked;
  const automationsOn = !!db?.automations?.some((a) => a.enabled);
  const tableFind = view?.kind === "table" ? search.trim() : "";
  const windowQuery = useMemo(() => ({ viewKey, find: tableFind }), [viewKey, tableFind]);

  const requery = useCallback(() => {
    clearTimeout(requeryTimer.current);
    requeryTimer.current = setTimeout(() => setTick((n) => n + 1), 120);
  }, []);
  useEffect(() => () => clearTimeout(requeryTimer.current), []);

  // Run the view whenever what it shows could have changed. The unsaved local view goes along, so edits show at once.
  useEffect(() => {
    if (!db || !view) return;
    const my = ++reqRef.current;
    const t = setTimeout(() => {
      const request = { databaseId: db.id, viewId: view.id, view };
      const run = view.kind === "table"
        ? api.queryViewWindow(request, windowQuery.find)
        : api.queryView(request);
      run
        .then((r) => {
          if (reqRef.current !== my) return;
          if ("rowIds" in r) {
            setResult((prior) => {
              const rows = prior?.window ? prior.rows : [];
              return { ...r, window: r, windowQuery, rows, rowsGeneration: prior?.rowsGeneration };
            });
          } else setResult(r);
          setQueryError(null);
        })
        .catch((e) => {
          if (reqRef.current !== my) return;
          setQueryError(errorText(e));
        });
    }, 60);
    return () => {
      ++reqRef.current;
      clearTimeout(t);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [db?.id, windowQuery, propsKey, tick, saveCount]);

  const tableWindow = result?.window;
  const positions = useMemo(() => new Map(tableWindow?.rowIds.map((id, i) => [id, i])), [tableWindow]);
  const pendingRowIds = result?.pendingRowIds;
  const orderedTableIds = useMemo(() => {
    if (!tableWindow) return undefined;
    return pendingRowIds ? [...tableWindow.rowIds, ...pendingRowIds] : tableWindow.rowIds;
  }, [tableWindow, pendingRowIds]);
  const resultRef = useRef(result);
  resultRef.current = result;
  const loadTableRows = useCallback((ids: readonly string[], isCurrent: () => boolean) => {
    if (!tableWindow || !view) return;
    const current = resultRef.current;
    const loaded = new Set(current?.rowsGeneration === tableWindow.generation ? current.rows.map((row) => row.id) : []);
    const indices = ids.flatMap((id) => {
      const position = positions.get(id);
      return position === undefined || loaded.has(id) ? [] : [position];
    }).sort((a, b) => a - b);
    const batches: RowRange[][] = [];
    for (let offset = 0; offset < indices.length; offset += 256) {
      const ranges: RowRange[] = [];
      for (const start of indices.slice(offset, offset + 256)) {
        const last = ranges[ranges.length - 1];
        if (last && last.start + last.count === start) last.count++;
        else ranges.push({ start, count: 1 });
      }
      batches.push(ranges);
    }
    void Promise.all(batches.map((ranges) => api.viewRows(databaseId, view.id, tableWindow.generation, ranges))).then((windows) => {
      if (!isCurrent()) return;
      if (windows.some((rows) => rows === null)) {
        requery();
        return;
      }
      setResult((prior) => {
        if (prior?.window !== tableWindow) return prior;
        const received = new Map(windows.flatMap((rows) => rows ?? []).map((row) => [row.id, row]));
        const keep = new Set(ids);
        // Keep newly created rows until metadata catches up, even if they
        // haven't scrolled into the range where their editor will take focus.
        for (const id of prior.pendingRowIds ?? []) keep.add(id);
        for (const row of prior.rows) {
          if (keep.has(row.id) && !received.has(row.id)) received.set(row.id, row);
        }
        if (windows.length === 0 && received.size === prior.rows.length) return prior;
        return { ...prior, rows: [...received.values()], rowsGeneration: tableWindow.generation };
      });
    }).catch((e) => {
      if (isCurrent()) toast.error(errorText(e));
    });
  }, [databaseId, view?.id, positions, tableWindow, requery]);

  const rowById = useMemo(() => new Map((result?.rows ?? []).map((r) => [r.id, r])), [result]);
  const dbRef = useRef(db);
  dbRef.current = db;

  // Undo and redo of this database live as long as one of its views is open.
  useEffect(() => retainUndo(databaseId), [databaseId]);
  // A full-page database also takes Ctrl+Z when nothing has the focus (the keys inside it are handled on its root below).
  useEffect(() => {
    if (compact) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.defaultPrevented || (document.activeElement && document.activeElement !== document.body)) return;
      handleUndoKey(databaseId, e);
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [databaseId, compact]);

  // ----- editing

  const editDb = useCallback((fn: (d: Database) => Database, now = false) => editDatabase(databaseId, fn, now), [databaseId]);
  const editView = useCallback(
    (fn: (v: View) => View) => {
      void editDatabase(databaseId, (d) => {
        const cur = d.views.find((v) => v.id === (view?.id ?? "")) ?? d.views[0];
        const next = fn(cur);
        return next === cur ? d : { ...d, views: d.views.map((v) => (v === cur ? next : v)) };
      });
    },
    [databaseId, view?.id],
  );

  const selectView = (id: string) => {
    setViewId(id);
    setResult(null);
    onViewChange?.(id);
  };

  const patchRow = useCallback((row: DbRow) => setResult((r) => (r ? { ...r, rows: r.rows.map((x) => (x.id === row.id ? row : x)) } : r)), []);

  const applyCells = useCallback(
    async (rowId: string, patch: Record<string, unknown>) => {
      setResult((r) =>
        r
          ? {
              ...r,
              rows: r.rows.map((x) => {
                if (x.id !== rowId) return x;
                const values = { ...x.values };
                for (const [k, v] of Object.entries(patch)) {
                  if (v === null) delete values[k];
                  else values[k] = v;
                }
                return { ...x, values };
              }),
            }
          : r,
      );
      const row = await saveCells(rowId, patch);
      if (row) patchRow(row);
      requery();
      afterRowChange(automationsOn);
    },
    [patchRow, requery, automationsOn],
  );
  const applyTitle = useCallback(
    async (rowId: string, title: string) => {
      setResult((r) => (r ? { ...r, rows: r.rows.map((x) => (x.id === rowId ? { ...x, title } : x)) } : r));
      const row = await saveCells(rowId, {}, title);
      if (row) patchRow(row);
      requery();
    },
    [patchRow, requery],
  );
  // The same two, remembering how to take the change back (and do it again).
  const same = (a: unknown, b: unknown) => JSON.stringify(a ?? null) === JSON.stringify(b ?? null);
  const setCells = useCallback(
    async (rowId: string, patch: Record<string, unknown>) => {
      const row = resultRef.current?.rows.find((x) => x.id === rowId);
      if (row) {
        const prev: Record<string, unknown> = {};
        for (const k of Object.keys(patch)) prev[k] = row.values[k] ?? null;
        if (Object.keys(patch).some((k) => !same(patch[k], prev[k]))) {
          const names = Object.keys(patch).map((k) => dbRef.current?.props.find((p) => p.id === k)?.name ?? "a value");
          const clear = Object.values(patch).every((v) => v == null);
          pushUndo(databaseId, { label: `${clear ? "cleared" : "edit of"} ${names.join(", ")}`, undo: () => applyCells(rowId, prev), redo: () => applyCells(rowId, patch) });
        }
      }
      await applyCells(rowId, patch);
    },
    [applyCells, databaseId],
  );
  const setTitle = useCallback(
    async (rowId: string, title: string) => {
      const prev = resultRef.current?.rows.find((x) => x.id === rowId)?.title;
      if (prev !== undefined && prev !== title) pushUndo(databaseId, { label: "edit of the name", undo: () => applyTitle(rowId, prev), redo: () => applyTitle(rowId, title) });
      await applyTitle(rowId, title);
    },
    [applyTitle, databaseId],
  );
  // Rows to the trash and back without a message of their own (the undo and redo say what happened).
  const trashQuiet = useCallback(
    async (ids: string[]) => {
      let t = useApp.getState().tree;
      for (const id of ids) t = await api.trashPage(id);
      setTree(t);
      requery();
    },
    [requery, setTree],
  );
  const restoreQuiet = useCallback(
    async (ids: string[]) => {
      let t = useApp.getState().tree;
      for (const id of ids) t = await api.restorePage(id);
      setTree(t);
      requery();
    },
    [requery, setTree],
  );
  const createRow = useCallback(
    async (cells: Record<string, unknown> = {}, title = "", templateId = ""): Promise<DbRow | null> => {
      try {
        const { row } = await api.createRow(databaseId, title, cells, templateId);
        ++reqRef.current;
        setResult((r) => {
          if (!r || r.window?.rowIds.includes(row.id) || r.rows.some((x) => x.id === row.id)) return r;
          return {
            ...r,
            rows: [...r.rows, row],
            total: r.total + 1,
            filtered: r.filtered + 1,
            pendingRowIds: r.window ? [...(r.pendingRowIds ?? []), row.id] : undefined,
          };
        });
        requery();
        afterRowChange(automationsOn);
        void api.listNotes().then(setTree).catch(() => undefined);
        pushUndo(databaseId, { label: "new row", undo: () => trashQuiet([row.id]), redo: () => restoreQuiet([row.id]) });
        return row;
      } catch (e) {
        toast.error(errorText(e));
        return null;
      }
    },
    [databaseId, requery, setTree, automationsOn, trashQuiet, restoreQuiet],
  );
  const reorderRows = useCallback(
    async (ids: string[]) => {
      const current = resultRef.current;
      const before = current?.window
        ? [...current.window.rowIds, ...(current.pendingRowIds ?? [])]
        : current?.rows.map((r) => r.id);
      const send = async (list: string[]) => {
        await api.reorderRows(list);
        requery();
      };
      try {
        await send(ids);
        if (before && before.length === ids.length && before.some((id, i) => id !== ids[i])) pushUndo(databaseId, { label: "row order", undo: () => send(before), redo: () => send(ids) });
      } catch (e) {
        toast.error(errorText(e));
        requery();
      }
    },
    [requery, databaseId],
  );
  const duplicateRow = useCallback(
    async (id: string) => {
      try {
        await api.duplicateRow(id);
        requery();
      } catch (e) {
        toast.error(errorText(e));
      }
    },
    [requery],
  );
  const trashRows = useCallback(
    async (ids: string[]) => {
      try {
        await trashQuiet(ids);
        // The toast's Undo and Ctrl+Z do the same; each one takes the step off the stack so it cannot be undone twice.
        const entry = { label: ids.length === 1 ? "row moved to trash" : `${ids.length} rows moved to trash`, undo: () => restoreQuiet(ids), redo: () => trashQuiet(ids) };
        pushUndo(databaseId, entry);
        toast(ids.length === 1 ? "Row moved to trash" : `${ids.length} rows moved to trash`, {
          action: {
            label: "Undo",
            onClick: async () => {
              try {
                await restoreQuiet(ids);
                dropUndo(databaseId, entry);
              } catch (e) {
                toast.error(errorText(e));
              }
            },
          },
        });
      } catch (e) {
        toast.error(errorText(e));
      }
    },
    [databaseId, trashQuiet, restoreQuiet],
  );
  const patchRowLocal = useCallback(
    (rowId: string, propId: string, value: unknown) => {
      setResult((r) =>
        r ? { ...r, rows: r.rows.map((x) => (x.id === rowId ? { ...x, values: value == null ? Object.fromEntries(Object.entries(x.values).filter(([k]) => k !== propId)) : { ...x.values, [propId]: value } } : x)) } : r,
      );
      requery();
    },
    [requery],
  );

  const openRow = useCallback(
    (rowId: string) => {
      const mode = view?.openIn ?? "full";
      if (mode === "full" || (compact && mode === "side")) {
        if (mode === "full") openRowPage(rowId);
        else setPeek({ id: rowId, mode: "center" });
      } else setPeek({ id: rowId, mode: mode as "side" | "center" });
    },
    [view?.openIn, compact, openRowPage],
  );

  const sampleOf = useCallback(
    (propId: string) => {
      const c = result?.rows[0]?.values[propId] as { t?: string; text?: string } | undefined;
      return c && typeof c === "object" && "t" in c ? { t: String(c.t), text: String(c.text ?? "") } : null;
    },
    [result],
  );

  // ----- search (on the rows we have)

  const shown: ShellResult | null = useMemo(() => {
    if (!result || !db) return result;
    if (result.window) return result;
    const q = search.trim().toLowerCase();
    if (!q) return result;
    const keep = new Set(
      result.rows
        .filter((row) => rowMatchesFind(db.props, row, q))
        .map((r) => r.id),
    );
    return { ...result, rows: result.rows.filter((r) => keep.has(r.id)), filtered: keep.size, groups: filterGroups(result.groups, keep) };
  }, [result, search, db]);
  const matchingIds = tableWindow?.matchingIds;
  const tableGroups = useMemo(() => matchingIds && tableWindow
    ? filterGroups(tableWindow.groups, new Set(matchingIds)) : tableWindow?.groups, [matchingIds, tableWindow]);

  // ----- title, icon, lock

  const setLook = async (patch: Partial<{ title: string; icon: string; favorite: boolean; locked: boolean }>) => {
    if (!db) return;
    const next = { title: db.title, icon: db.icon, favorite: db.favorite, locked: db.locked, ...patch };
    setDatabase({ ...db, ...next });
    try {
      const meta = await api.setDatabaseLook(db.id, next.title, next.icon, next.favorite, next.locked);
      const t = useApp.getState().tree;
      setTree({ ...t, databases: t.databases.map((d) => (d.id === meta.id ? meta : d)) });
    } catch (e) {
      toast.error(errorText(e));
    }
  };
  const titleTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const [titleDraft, setTitleDraft] = useState<string | null>(null);
  const commitTitle = (v: string) => {
    clearTimeout(titleTimer.current);
    setTitleDraft(null);
    if (db && v !== db.title) void setLook({ title: v });
  };

  // ----- views

  const addView = (kind: ViewKind) => {
    if (!db) return;
    const label = VIEW_KINDS.find((k) => k.kind === kind)!.label;
    const same = db.views.filter((v) => v.kind === kind).length;
    const nv = blankView(kind, same ? `${label} ${same + 1}` : label, db);
    void editDb((d) => ({ ...d, views: [...d.views, nv] }), true);
    selectView(nv.id);
  };
  const duplicateView = (v: View) => {
    const copy: View = { ...structuredClone(v), id: crypto.randomUUID().replace(/-/g, ""), name: `${v.name} copy` };
    void editDb((d) => ({ ...d, views: [...d.views, copy] }), true);
    selectView(copy.id);
  };
  const deleteView = async (v: View) => {
    if (!db || db.views.length <= 1) return;
    const ok = await confirm({ title: `Delete the view "${v.name}"?`, body: "The rows stay; only this way of looking at them is removed.", confirm: "Delete view", danger: true });
    if (!ok) return;
    const rest = db.views.filter((x) => x.id !== v.id);
    void editDb((d) => ({ ...d, views: rest }), true);
    if (view?.id === v.id) selectView(rest[0].id);
  };
  const moveView = (v: View, delta: number) => {
    if (!db) return;
    const i = db.views.findIndex((x) => x.id === v.id);
    const j = i + delta;
    if (j < 0 || j >= db.views.length) return;
    void editDb((d) => ({ ...d, views: reorderList(d.views, i, j) }));
  };

  // ----- menu actions

  const importCsv = async () => {
    if (!db) return;
    try {
      const r = await api.importCsv(db.id);
      if (!r) return;
      toast.success(`Imported ${r.rows} ${r.rows === 1 ? "row" : "rows"}.`);
      if (r.notice) toast.info(r.notice, { duration: 20000 });
      if (r.skipped.length) toast.warning(`Skipped: ${r.skipped.slice(0, 3).join("; ")}${r.skipped.length > 3 ? ` and ${r.skipped.length - 3} more` : ""}`);
      await loadDatabase(db.id);
      requery();
      void api.listNotes().then(setTree).catch(() => undefined);
    } catch (e) {
      toast.error(errorText(e));
    }
  };
  const exportCsv = async () => {
    if (!db || !view) return;
    try {
      const r = await api.exportCsv(db.id, view.id);
      if (r) toast.success(`Exported ${r.rows} ${r.rows === 1 ? "row" : "rows"} to ${r.path}`);
    } catch (e) {
      toast.error(errorText(e));
    }
  };
  const exportAs = async (format: "html" | "markdown" | "pdf" | "docx") => {
    if (!db || !view) return;
    try {
      const r = await api.exportDatabase(db.id, view.id, format);
      if (r) {
        toast.success(`Exported to ${r.path}`);
        if (r.charsNoFont > 0) toast.warning(`${r.charsNoFont} characters have no font in the PDF and were left out.`);
      }
    } catch (e) {
      toast.error(errorText(e));
    }
  };
  const enable = async (what: "sub" | "deps") => {
    if (!db) return;
    try {
      setDatabase(await (what === "sub" ? api.enableSubItems(db.id) : api.enableDependencies(db.id)));
      requery();
    } catch (e) {
      toast.error(errorText(e));
    }
  };
  const trashDatabase = async () => {
    if (!db) return;
    const ok = await confirm({ title: `Move "${db.title || "Untitled"}" to the trash?`, body: "Its rows go with it. You can restore it from the trash.", confirm: "Move to trash", danger: true });
    if (!ok) return;
    try {
      setTree(await api.trashPage(db.id));
      if (useApp.getState().activePageId === db.id) useApp.getState().openPage(null);
      toast("Database moved to trash", { action: { label: "Undo", onClick: async () => setTree(await api.restorePage(db.id)) } });
    } catch (e) {
      toast.error(errorText(e));
    }
  };

  // ----- shell context

  const shell: DbShell | null = useMemo(
    () =>
      db && view
        ? {
            db, view, databases: tree.databases, locked, compact, editDb, editView, trashRows, duplicateRow, requery, rowById, focusRowId,
            clearFocusRow: () => setFocusRowId(null), focusRow: (id: string) => setFocusRowId(id), patchRowLocal, sampleOf, openProp: (id: string) => setPropMenu(id),
          }
        : null,
    [db, view, tree.databases, locked, compact, editDb, editView, trashRows, duplicateRow, requery, rowById, focusRowId, patchRowLocal, sampleOf],
  );

  if (loadError) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-2 p-8 text-center">
        <AlertTriangle className="text-danger" />
        <div className="font-medium">This database could not be opened</div>
        <div className="max-w-md text-sm text-muted">{loadError}</div>
      </div>
    );
  }
  if (!db || !view || !shell) {
    return (
      <div className={cn("grid place-items-center text-muted", compact ? "py-8" : "h-full")}>
        <Loader2 className="animate-spin" size={20} />
      </div>
    );
  }

  const viewProps: ViewProps | null = shown
    ? {
        db, view, result: tableWindow && matchingIds ? { ...shown, filtered: matchingIds.length, groups: tableGroups ?? null } : shown,
        rowById, tableRowIds: matchingIds ?? orderedTableIds, loadTableRows: tableWindow ? loadTableRows : undefined,
        locked, openRow, setCells, setTitle, createRow: (c, t) => createRow(c, t),
        updateView: (patch) => editView((v) => ({ ...v, ...patch })), reorderRows,
      }
    : null;
  const ViewBody = viewComponent(view.kind);
  const hasFilter = !filterIsEmpty(view.filter);
  const nFilters = countConds(view.filter);
  const groupProp = view.group ? db.props.find((p) => p.id === view.group!.prop) : undefined;
  const editingProp = propMenu ? db.props.find((p) => p.id === propMenu) : undefined;
  const sortable = view.kind !== "chart";
  const isEmpty = result && result.total === 0;
  const hasCurrentMatchCount = view.kind !== "table" || result?.windowQuery === windowQuery;

  const toolbar = (
    <div className="flex min-w-0 items-center gap-0.5">
      <PanelPopover
        width={560}
        align="end"
        trigger={
          <button type="button" className={cn(ghostBtn, hasFilter && "text-accent-strong")} aria-label="Filter">
            <FilterIcon size={14} /> {!compact && "Filter"}{hasFilter && <span className="rounded-full bg-accent-soft px-1.5 text-[0.7rem]">{nFilters}</span>}
          </button>
        }
      >
        <FilterGroup filter={view.filter} props={db.props.filter((p) => p.kind !== "button")} depth={1} onChange={(f: Filter) => editView((v) => ({ ...v, filter: f }))} />
        {hasFilter && (
          <button type="button" className="mt-2 text-xs text-muted hover:text-danger" onClick={() => editView((v) => ({ ...v, filter: emptyFilter() }))}>
            Delete filter
          </button>
        )}
      </PanelPopover>
      {sortable && (
        <PanelPopover
          width={420}
          align="end"
          trigger={
            <button type="button" className={cn(ghostBtn, view.sorts.length > 0 && "text-accent-strong")} aria-label="Sort">
              <ArrowUpDown size={14} /> {!compact && "Sort"}{view.sorts.length > 0 && <span className="rounded-full bg-accent-soft px-1.5 text-[0.7rem]">{view.sorts.length}</span>}
            </button>
          }
        >
          <SortPanel sorts={view.sorts} props={db.props} onChange={(sorts) => editView((v) => ({ ...v, sorts }))} />
        </PanelPopover>
      )}
      {view.kind !== "calendar" && view.kind !== "chart" && (
        <PanelPopover
          width={380}
          align="end"
          trigger={
            <button type="button" className={cn(ghostBtn, view.group && "text-accent-strong")} aria-label="Group">
              <Group size={14} /> {!compact && (groupProp ? `Group: ${groupProp.name}` : "Group")}
            </button>
          }
        >
          <GroupPanel
            group={view.group}
            subGroup={view.subGroup}
            props={db.props}
            allowSub={view.kind === "table" || view.kind === "list" || view.kind === "board"}
            onChange={(group, subGroup) => editView((v) => ({ ...v, group, subGroup }))}
          />
        </PanelPopover>
      )}
      {(view.kind === "table" || view.kind === "list" || view.kind === "gallery" || view.kind === "board") && (
        <PanelPopover
          width={340}
          align="end"
          trigger={
            <button type="button" className={ghostBtn} aria-label="Properties">
              <SlidersHorizontal size={14} /> {!compact && "Properties"}
            </button>
          }
        >
          <PropertiesPanel db={db} view={view} onChange={(nv) => editView(() => nv)} onOpenProp={(id) => setPropMenu(id)} />
        </PanelPopover>
      )}
      {searchOpen ? (
        <span className="relative">
          <Search size={13} className="pointer-events-none absolute start-2 top-1/2 -translate-y-1/2 text-faint" />
          <input
            autoFocus
            value={search}
            dir="auto"
            aria-label="Search rows"
            onChange={(e) => setSearch(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Escape") {
                setSearch("");
                setSearchOpen(false);
              }
            }}
            onBlur={() => !search && setSearchOpen(false)}
            placeholder="Search…"
            className="h-7 w-40 rounded-md border border-line bg-surface ps-7 pe-2 text-[0.8rem] outline-none focus:border-accent"
          />
        </span>
      ) : (
        <button type="button" className={ghostBtn} aria-label="Search rows" onClick={() => setSearchOpen(true)}>
          <Search size={14} />
        </button>
      )}
      <PanelPopover
        width={360}
        align="end"
        trigger={
          <button type="button" className={ghostBtn} aria-label="View settings">
            <Settings2 size={14} />
          </button>
        }
      >
        <ViewSettingsPanel db={db} view={view} locked={locked} onChange={(nv) => editView(() => nv)} />
      </PanelPopover>
      <Menu>
        <MenuTrigger asChild>
          <button type="button" className={ghostBtn} aria-label="Database menu">
            <MoreHorizontal size={15} />
          </button>
        </MenuTrigger>
        <MenuContent>
          <MenuItem icon={locked ? <LockOpen size={15} /> : <Lock size={15} />} onSelect={() => void setLook({ locked: !locked })}>
            {locked ? "Unlock database" : "Lock database"}
          </MenuItem>
          <MenuItem icon={<LayoutTemplate size={15} />} onSelect={() => setTemplates(true)}>Templates…</MenuItem>
          <MenuItem icon={<Zap size={15} />} onSelect={() => setAutomations(true)}>Automations…</MenuItem>
          <MenuSeparator />
          <MenuItem icon={<FileUp size={15} />} onSelect={importCsv}>Import CSV…</MenuItem>
          <MenuItem icon={<Download size={15} />} onSelect={exportCsv}>Export CSV (this view)…</MenuItem>
          <MenuItem icon={<Download size={15} />} onSelect={() => exportAs("markdown")}>Export Markdown (this view)…</MenuItem>
          <MenuItem icon={<Download size={15} />} onSelect={() => exportAs("html")}>Export HTML (this view)…</MenuItem>
          <MenuItem icon={<Download size={15} />} onSelect={() => exportAs("docx")}>Export Word (this view)…</MenuItem>
          <MenuItem icon={<Download size={15} />} onSelect={() => exportAs("pdf")}>Export PDF (this view)…</MenuItem>
          {!locked && !db.subParentProp && <MenuItem icon={<ListTree size={15} />} onSelect={() => enable("sub")}>Turn on sub-items</MenuItem>}
          {!locked && !db.blockedByProp && <MenuItem icon={<GitBranch size={15} />} onSelect={() => enable("deps")}>Turn on dependencies</MenuItem>}
          {compact && <MenuItem icon={<ExternalLink size={15} />} onSelect={() => openDatabase(db.id)}>Open as page</MenuItem>}
          <MenuSeparator />
          <MenuItem danger icon={<Trash2 size={15} />} onSelect={trashDatabase}>Move to trash</MenuItem>
        </MenuContent>
      </Menu>
      <NewButton
        db={db}
        onNew={async (templateId) => {
          const row = await createRow({}, "", templateId);
          if (row) {
            setFocusRowId(row.id);
            if (view.kind !== "table") openRow(row.id);
          }
        }}
        onTemplates={() => setTemplates(true)}
      />
    </div>
  );

  return (
    <DbShellContext.Provider value={shell}>
      <div className={cn("relative flex min-h-0 flex-col", compact ? "rounded-lg border border-line" : "h-full")} data-database={db.id} onKeyDown={(e) => handleUndoKey(databaseId, e)}>
        {!compact && (
          <div className="shrink-0 px-6 pb-1 pt-8 sm:px-10">
            <div className="flex items-start gap-3">
              <Popover.Root>
                <Popover.Trigger asChild>
                  <button type="button" aria-label="Change icon" className="grid h-12 w-12 shrink-0 place-items-center rounded-xl text-4xl hover:bg-surface-2">
                    {db.icon ? <span className="emoji-mono">{db.icon}</span> : <TableIcon size={30} className="text-faint" />}
                  </button>
                </Popover.Trigger>
                <Popover.Portal>
                  <Popover.Content align="start" sideOffset={6} className="z-[80] w-[300px] rounded-xl border border-line bg-elevated p-2 shadow-float">
                    <div className="grid grid-cols-9 gap-0.5">
                      {PAGE_EMOJIS.map((e) => (
                        <button key={e} type="button" onClick={() => void setLook({ icon: e })} className="grid h-8 w-8 place-items-center rounded-md text-lg hover:bg-surface-2">
                          {e}
                        </button>
                      ))}
                    </div>
                    {db.icon && <button type="button" onClick={() => void setLook({ icon: "" })} className="mt-1 w-full rounded-md py-1 text-sm text-muted hover:bg-surface-2">Remove icon</button>}
                  </Popover.Content>
                </Popover.Portal>
              </Popover.Root>
              <div className="min-w-0 flex-1">
                <input
                  value={titleDraft ?? db.title}
                  dir="auto"
                  aria-label="Database name"
                  placeholder="Untitled database"
                  onChange={(e) => {
                    setTitleDraft(e.target.value);
                    clearTimeout(titleTimer.current);
                    const v = e.target.value;
                    titleTimer.current = setTimeout(() => commitTitle(v), 600);
                  }}
                  onBlur={(e) => titleDraft !== null && commitTitle(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && (e.target as HTMLInputElement).blur()}
                  className="w-full min-w-0 bg-transparent text-[2rem] font-bold leading-tight tracking-tight outline-none placeholder:text-faint/70"
                />
                <input
                  value={db.description}
                  dir="auto"
                  disabled={locked}
                  aria-label="Description"
                  placeholder="Add a description"
                  onChange={(e) => void editDb((d) => ({ ...d, description: e.target.value }))}
                  className="mt-0.5 w-full min-w-0 bg-transparent text-sm text-muted outline-none placeholder:text-faint/70"
                />
              </div>
              {locked && (
                <span className="mt-2 inline-flex items-center gap-1 rounded-md bg-surface-2 px-2 py-1 text-xs text-muted" title="Properties and views are locked. Rows can still change.">
                  <Lock size={12} /> Locked
                </span>
              )}
            </div>
          </div>
        )}

        <div className={cn("flex shrink-0 items-center gap-2 border-b border-line", compact ? "px-2 py-1" : "px-6 sm:px-10")}>
          {compact && (
            <button type="button" onClick={() => openDatabase(db.id)} className="inline-flex max-w-[12rem] shrink-0 items-center gap-1.5 rounded-md px-1.5 py-1 text-sm font-medium hover:bg-surface-2" title="Open as page">
              {db.icon ? <span className="emoji-mono">{db.icon}</span> : <TableIcon size={14} className="text-muted" />}
              <span dir="auto" className="truncate">{db.title || "Untitled"}</span>
            </button>
          )}
          <div role="tablist" aria-label="Views" className="no-scrollbar flex min-w-0 flex-1 items-center gap-0.5 overflow-x-auto">
            {db.views.map((v) => {
              const active = v.id === view.id;
              return (
                <span key={v.id} className="flex shrink-0 items-center">
                  {renaming === v.id ? (
                    <input
                      autoFocus
                      aria-label="View name"
                      defaultValue={v.name}
                      dir="auto"
                      onFocus={(e) => e.target.select()}
                      onBlur={(e) => {
                        const n = e.target.value.trim();
                        if (n && n !== v.name) void editDb((d) => ({ ...d, views: d.views.map((x) => (x.id === v.id ? { ...x, name: n } : x)) }));
                        setRenaming(null);
                      }}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") (e.target as HTMLInputElement).blur();
                        if (e.key === "Escape") setRenaming(null);
                      }}
                      className="h-7 w-28 rounded-md border border-accent bg-surface px-2 text-sm outline-none"
                    />
                  ) : (
                    <button
                      type="button"
                      role="tab"
                      aria-selected={active}
                      onClick={() => (active ? undefined : selectView(v.id))}
                      onDoubleClick={() => !locked && setRenaming(v.id)}
                      className={cn(
                        "inline-flex h-9 items-center gap-1.5 border-b-2 px-2 text-[0.82rem] transition-colors",
                        active ? "border-fg font-medium text-fg" : "border-transparent text-muted hover:text-fg",
                      )}
                    >
                      <ViewIcon kind={v.kind} size={13} />
                      <span dir="auto" className="max-w-[10rem] truncate">{v.name || "Untitled view"}</span>
                    </button>
                  )}
                  {active && !locked && renaming !== v.id && (
                    <Menu>
                      <MenuTrigger asChild>
                        <button type="button" aria-label={`Options for ${v.name}`} className="-ms-1 me-0.5 rounded p-0.5 text-faint hover:bg-surface-2 hover:text-fg">
                          <ChevronDown size={13} />
                        </button>
                      </MenuTrigger>
                      <MenuContent align="start">
                        <MenuItem onSelect={() => setRenaming(v.id)}>Rename</MenuItem>
                        <MenuItem icon={<Copy size={14} />} onSelect={() => duplicateView(v)}>Duplicate view</MenuItem>
                        <MenuItem onSelect={() => moveView(v, -1)}>Move left</MenuItem>
                        <MenuItem onSelect={() => moveView(v, 1)}>Move right</MenuItem>
                        {db.views.length > 1 && (
                          <MenuItem danger icon={<Trash2 size={14} />} onSelect={() => void deleteView(v)}>Delete view</MenuItem>
                        )}
                      </MenuContent>
                    </Menu>
                  )}
                </span>
              );
            })}
            {!locked && (
              <Menu>
                <MenuTrigger asChild>
                  <button type="button" aria-label="Add a view" title="Add a view" className="ms-1 grid h-7 w-7 shrink-0 place-items-center rounded-md text-muted hover:bg-surface-2 hover:text-fg">
                    <Plus size={15} />
                  </button>
                </MenuTrigger>
                <MenuContent align="start">
                  <div className="px-2.5 py-1 text-xs text-faint">Add a view</div>
                  {VIEW_KINDS.map((k) => (
                    <MenuItem key={k.kind} icon={<ViewIcon kind={k.kind} size={15} />} onSelect={() => addView(k.kind)}>{k.label}</MenuItem>
                  ))}
                </MenuContent>
              </Menu>
            )}
          </div>
          {toolbar}
        </div>

        {result?.truncated && (
          <div className="flex items-center gap-2 border-b border-line bg-warn/10 px-6 py-1.5 text-xs" role="status">
            <AlertTriangle size={13} className="text-warn-icon" /> View limit reached: only the first {(orderedTableIds?.length ?? result.filtered ?? result.rows.length).toLocaleString()} matching rows are included. This database has {result.total.toLocaleString()} rows in total. Add a filter to narrow it down.
          </div>
        )}
        {queryError && (
          <div className="flex items-center gap-2 border-b border-line bg-danger/10 px-6 py-1.5 text-xs text-danger" role="alert">
            <AlertTriangle size={13} /> {queryError}
            <button type="button" className="ms-auto underline" onClick={requery}>Try again</button>
          </div>
        )}

        {/* A table scrolls inside itself (header stays, the bars sit at the edge of what is visible); the other views scroll here. */}
        <div className={cn("min-h-0 flex-1", compact ? "max-h-[520px] overflow-auto" : view.kind === "table" ? "flex flex-col overflow-hidden" : "overflow-auto")}>
          {!viewProps ? (
            <div className="grid place-items-center py-16 text-muted"><Loader2 className="animate-spin" size={20} /></div>
          ) : (
            <Suspense fallback={<div className="grid place-items-center py-16 text-muted"><Loader2 className="animate-spin" size={20} /></div>}>
              {isEmpty && view.kind !== "table" && view.kind !== "chart" ? (
                <div className="px-6 py-14 text-center text-sm text-muted">
                  No rows yet. Use <span className="font-medium text-fg">New</span> to add the first one.
                </div>
              ) : (
                <ViewBody {...viewProps} />
              )}
              {!isEmpty && hasCurrentMatchCount && viewProps.result.filtered === 0 && (hasFilter || search) && (
                <div className="shrink-0 px-6 py-10 text-center text-sm text-muted">
                  No rows match.{" "}
                  <button type="button" className="underline" onClick={() => (setSearch(""), editView((v) => ({ ...v, filter: emptyFilter() })))}>Clear the filter and search</button>
                </div>
              )}
            </Suspense>
          )}
        </div>

        {editingProp && (
          <PropertyDialog
            key={editingProp.id}
            db={db}
            prop={editingProp}
            databases={tree.databases}
            locked={locked}
            view={view}
            sample={sampleOf(editingProp.id)}
            onClose={() => setPropMenu(null)}
            editDb={editDb}
            editView={editView}
            confirm={confirm}
          />
        )}
        {templates && <TemplatesDialog db={db} onClose={() => setTemplates(false)} onSaved={requery} />}
        {automations && <AutomationsDialog db={db} onClose={() => setAutomations(false)} onSaved={requery} />}
        {peek && (
          <Suspense fallback={null}>
            <RowPeek rowId={peek.id} mode={peek.mode} onClose={() => setPeek(null)} onOpenPage={() => (setPeek(null), openRowPage(peek.id))} />
          </Suspense>
        )}
      </div>
    </DbShellContext.Provider>
  );
}

function NewButton({ db, onNew, onTemplates }: { db: Database; onNew: (templateId: string) => void; onTemplates: () => void }) {
  return (
    <span className="ms-1 inline-flex shrink-0 overflow-hidden rounded-md bg-accent text-accent-fg">
      <button type="button" onClick={() => onNew("")} className="inline-flex h-7 items-center gap-1 ps-2.5 pe-2 text-[0.8rem] font-medium hover:opacity-90">
        New
      </button>
      <Menu>
        <MenuTrigger asChild>
          <button type="button" aria-label="New with a template" className="inline-flex h-7 items-center border-s border-accent-fg/25 px-1 hover:opacity-90">
            <ChevronDown size={13} />
          </button>
        </MenuTrigger>
        <MenuContent>
          <MenuItem icon={<Plus size={14} />} onSelect={() => onNew("-")}>Blank row</MenuItem>
          {db.templates.length > 0 && <MenuSeparator />}
          {db.templates.map((t) => (
            <MenuItem key={t.id} icon={<span className="emoji-mono">{t.icon || "📄"}</span>} onSelect={() => onNew(t.id)}>
              {t.name || "Untitled template"}{db.defaultTemplate === t.id ? " (default)" : ""}
            </MenuItem>
          ))}
          <MenuSeparator />
          <MenuItem icon={<LayoutTemplate size={14} />} onSelect={onTemplates}>Manage templates…</MenuItem>
        </MenuContent>
      </Menu>
    </span>
  );
}

/** The property menu as a small dialog-like popover anchored at the top of the database. */
function PropertyDialog({
  db, prop, databases, locked, view, sample, onClose, editDb, editView, confirm,
}: {
  db: Database; prop: Property; databases: DbShell["databases"]; locked: boolean; view: View; sample: { t: string; text: string } | null;
  onClose: () => void; editDb: DbShell["editDb"]; editView: DbShell["editView"]; confirm: ReturnType<typeof useConfirm>;
}) {
  return (
    <Dialog.Root open onOpenChange={(o) => !o && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-[70] bg-black/30" />
        <Dialog.Content
          aria-describedby={undefined}
          className="fixed start-1/2 top-20 z-[70] max-h-[78vh] w-[min(420px,94vw)] -translate-x-1/2 overflow-y-auto rounded-xl border border-line bg-elevated p-4 shadow-float outline-none rtl:translate-x-1/2"
        >
        <Dialog.Title className="sr-only">{`Property ${prop.name}`}</Dialog.Title>
        <Dialog.Close aria-label="Close" className="absolute end-2 top-2 rounded-md p-1 text-faint hover:bg-surface-2 hover:text-fg">
          <X size={15} />
        </Dialog.Close>
        <PropertyMenu
          db={db}
          prop={prop}
          databases={databases}
          locked={locked}
          sample={sample}
          onChange={(p) => void editDb((d) => ({ ...d, props: d.props.map((x) => (x.id === p.id ? p : x)) }))}
          onSort={(descending) => (editView((v) => ({ ...v, sorts: [{ prop: prop.id, descending }] })), onClose())}
          onHide={() => (editView((v) => ({ ...v, columns: v.columns.map((c) => (c.prop === prop.id ? { ...c, visible: false } : c)) })), onClose())}
          onDuplicate={() => {
            const copy: Property = { ...structuredClone(prop), id: crypto.randomUUID().replace(/-/g, ""), name: `${prop.name} (copy)`, relationBack: prop.kind === "relation" ? "" : prop.relationBack };
            void editDb((d) => ({ ...d, props: [...d.props, copy], views: d.views.map((v) => ({ ...v, columns: [...v.columns, { prop: copy.id, visible: v.id === view.id, width: 0 }] })) }), true);
            onClose();
          }}
          onDelete={async () => {
            const ok = await confirm({ title: `Delete the property "${prop.name}"?`, body: "Its values are removed from every row. This cannot be undone.", confirm: "Delete property", danger: true });
            if (!ok) return;
            void editDb(
              (d) => ({
                ...d,
                props: d.props.filter((p) => p.id !== prop.id),
                views: d.views.map((v) => ({
                  ...v,
                  columns: v.columns.filter((c) => c.prop !== prop.id),
                  sorts: v.sorts.filter((s) => s.prop !== prop.id),
                  calcs: v.calcs.filter((c) => c.prop !== prop.id),
                  group: v.group?.prop === prop.id ? null : v.group,
                  subGroup: v.subGroup?.prop === prop.id ? null : v.subGroup,
                })),
              }),
              true,
            );
            onClose();
          }}
        />
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
