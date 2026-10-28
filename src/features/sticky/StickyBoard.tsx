import { useEffect, useRef, useState } from "react";
import { motion } from "motion/react";
import { Calculator, Check, CheckSquare, ExternalLink, MonitorUp, Plus, StickyNote, Trash2, X } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import { stickyColor } from "@/lib/palette";
import { timeAgo } from "@/lib/utils";
import { useApp } from "@/store/app";
import { Button, useConfirm } from "@/components/ui";

export default function StickyBoard() {
  const stickies = useApp((s) => s.stickies);
  const setStickies = useApp((s) => s.setStickies);
  const confirm = useConfirm();
  // Select mode: clicking a note ticks it instead of opening it.
  const [selecting, setSelecting] = useState(false);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  // Keeps keyboard focus (and so Escape) on the board while selecting.
  const boardRef = useRef<HTMLDivElement>(null);

  const refresh = () => api.listStickies().then(setStickies);

  // Drop ticks for notes that were deleted elsewhere (for example in their own window).
  useEffect(() => {
    setSelected((prev) => {
      const ids = new Set(stickies.map((s) => s.id));
      const next = new Set([...prev].filter((id) => ids.has(id)));
      return next.size === prev.size ? prev : next;
    });
  }, [stickies]);

  const startSelecting = () => {
    setSelecting(true);
    boardRef.current?.focus();
  };

  const stopSelecting = () => {
    setSelecting(false);
    setSelected(new Set());
    boardRef.current?.focus();
  };

  // Nothing left to select (for example all deleted in their own windows).
  useEffect(() => {
    if (selecting && stickies.length === 0) stopSelecting();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selecting, stickies.length]);

  const toggle = (id: string) =>
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });

  const allSelected = stickies.length > 0 && selected.size === stickies.length;

  const deleteSelected = async () => {
    const ids = [...selected];
    if (ids.length === 0) return;
    const one = ids.length === 1;
    const ok = await confirm({
      title: one ? "Delete this sticky note?" : `Delete ${ids.length} sticky notes?`,
      body: one ? "It will be permanently removed." : "They will be permanently removed.",
      confirm: "Delete",
      danger: true,
    });
    // The dialog drops focus to the page body as it closes; put it back afterwards so Escape still works.
    setTimeout(() => boardRef.current?.focus(), 0);
    if (!ok) return;
    try {
      await api.deleteStickies(ids);
      stopSelecting();
    } catch (e) {
      toast.error(errorText(e));
    }
    refresh().catch(() => {});
  };

  // Notes are edited in their own windows; refresh when this view gains focus.
  useEffect(() => {
    refresh();
    const onFocus = () => refresh();
    window.addEventListener("focus", onFocus);
    return () => window.removeEventListener("focus", onFocus);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const create = async () => {
    try {
      await api.createSticky(true);
      await refresh();
    } catch (e) {
      toast.error(errorText(e));
    }
  };

  // Each press opens one more calculator window; they all share one saved history.
  const openCalculator = () => api.openCalculator().catch((e) => toast.error(errorText(e)));

  const openAll = async () => {
    for (const s of stickies) await api.openSticky(s.id);
    await refresh();
  };

  return (
    <div
      ref={boardRef}
      tabIndex={-1}
      className="h-full overflow-y-auto outline-none"
      onKeyDown={(e) => {
        // A key handler on the board, not the window, so Escape in the confirm dialog only closes the dialog.
        if (selecting && e.key === "Escape") stopSelecting();
      }}
    >
      <div className="px-8 py-8">
        <div className="mb-6 flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-semibold tracking-tight">Sticky notes</h2>
            <p className="mt-1 text-sm text-muted">They hide automatically whenever the vault locks.</p>
          </div>
          {selecting ? (
            <div className="flex items-center gap-2">
              <span className="mr-1 text-sm text-muted" aria-live="polite">
                {selected.size} selected
              </span>
              <Button
                variant="outline"
                onClick={() => setSelected(allSelected ? new Set() : new Set(stickies.map((s) => s.id)))}
              >
                {allSelected ? "Select none" : "Select all"}
              </Button>
              <Button variant="danger" onClick={deleteSelected} disabled={selected.size === 0}>
                <Trash2 size={16} /> Delete
              </Button>
              <Button variant="ghost" onClick={stopSelecting}>
                <X size={16} /> Cancel
              </Button>
            </div>
          ) : (
            <div className="flex gap-2">
              {stickies.length > 0 && (
                <Button variant="outline" onClick={startSelecting}>
                  <CheckSquare size={16} /> Select
                </Button>
              )}
              <Button variant="outline" onClick={openCalculator} title="Open a calculator in its own window">
                <Calculator size={16} /> Calculator
              </Button>
              <Button variant="outline" onClick={openAll}>
                <MonitorUp size={16} /> Show all on desktop
              </Button>
              <Button variant="primary" onClick={create}>
                <Plus size={16} /> New note
              </Button>
            </div>
          )}
        </div>

        {stickies.length === 0 && (
          <div className="mb-2 flex items-center gap-4 rounded-2xl border border-dashed border-line px-6 py-8">
            <StickyNote size={26} className="shrink-0 text-faint" />
            <p className="text-sm leading-relaxed text-muted">
              Sticky notes float on your desktop while the vault is unlocked, and vanish the instant it locks. Make one with "New note".
            </p>
          </div>
        )}
        <div className="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-5">
          {stickies.map((s, i) => {
            const c = stickyColor(s.color);
            const ticked = selected.has(s.id);
            return (
              <motion.div
                key={s.id}
                initial={{ opacity: 0, y: 12, rotate: 0 }}
                animate={{ opacity: 1, y: 0, rotate: (i % 3) - 1 }}
                whileHover={{ rotate: 0, y: -4, scale: 1.02 }}
                transition={{ type: "spring", stiffness: 300, damping: 22, delay: i * 0.03 }}
                className={
                  "group relative flex h-56 cursor-pointer flex-col overflow-hidden rounded-2xl shadow-[0_10px_30px_-10px_rgba(0,0,0,0.5)]" +
                  (ticked ? " ring-2 ring-accent ring-offset-2 ring-offset-bg" : "")
                }
                style={{ background: c.paper, color: c.ink }}
                role={selecting ? "checkbox" : undefined}
                aria-checked={selecting ? ticked : undefined}
                aria-label={selecting ? (s.text.split("\n")[0].trim() || "Empty note") : undefined}
                tabIndex={selecting ? 0 : undefined}
                onKeyDown={(e) => {
                  if (selecting && (e.key === " " || e.key === "Enter")) {
                    e.preventDefault();
                    toggle(s.id);
                  }
                }}
                onClick={(e) => {
                  // Ctrl+click starts selecting from the normal view too.
                  if (selecting || e.ctrlKey || e.metaKey) {
                    if (!selecting) startSelecting();
                    toggle(s.id);
                  } else {
                    api.openSticky(s.id).then(refresh);
                  }
                }}
              >
                {selecting && (
                  <span
                    className={
                      "absolute right-3 top-4 flex h-5 w-5 items-center justify-center rounded-md border-2 " +
                      (ticked ? "border-accent bg-accent text-accent-fg" : "border-current bg-white/40 opacity-60")
                    }
                    aria-hidden
                  >
                    {ticked && <Check size={13} strokeWidth={3} />}
                  </span>
                )}
                <div className="h-2 shrink-0" style={{ background: c.bar }} />
                <p className={"selectable flex-1 overflow-hidden whitespace-pre-wrap px-4 py-3 text-sm leading-relaxed" + (selecting ? " pr-10" : "")}>
                  {s.text || <span className="opacity-50">Empty note</span>}
                </p>
                <div className="flex items-center justify-between px-4 pb-3 text-xs opacity-60">
                  <span>{timeAgo(s.updated)}</span>
                  <span className={"flex gap-1 opacity-0 transition-opacity group-hover:opacity-100" + (selecting ? " invisible" : "")}>
                    {s.open && <ExternalLink size={13} />}
                    <button
                      onClick={async (e) => {
                        e.stopPropagation();
                        const ok = await confirm({ title: "Delete this sticky note?", body: "It will be permanently removed.", confirm: "Delete", danger: true });
                        if (ok) {
                          await api.deleteSticky(s.id);
                          refresh();
                        }
                      }}
                      className="rounded p-0.5 hover:bg-black/10"
                      aria-label="Delete"
                    >
                      <Trash2 size={13} />
                    </button>
                  </span>
                </div>
              </motion.div>
            );
          })}
        </div>

      </div>
    </div>
  );
}
