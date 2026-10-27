import { useEffect } from "react";
import { motion } from "motion/react";
import { Calculator, ExternalLink, MonitorUp, Plus, StickyNote, Trash2 } from "lucide-react";
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

  const refresh = () => api.listStickies().then(setStickies);

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
    <div className="h-full overflow-y-auto">
      <div className="px-8 py-8">
        <div className="mb-6 flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-semibold tracking-tight">Sticky notes</h2>
            <p className="mt-1 text-sm text-muted">They hide automatically whenever the vault locks.</p>
          </div>
          <div className="flex gap-2">
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
            return (
              <motion.div
                key={s.id}
                initial={{ opacity: 0, y: 12, rotate: 0 }}
                animate={{ opacity: 1, y: 0, rotate: (i % 3) - 1 }}
                whileHover={{ rotate: 0, y: -4, scale: 1.02 }}
                transition={{ type: "spring", stiffness: 300, damping: 22, delay: i * 0.03 }}
                className="group relative flex h-56 cursor-pointer flex-col overflow-hidden rounded-2xl shadow-[0_10px_30px_-10px_rgba(0,0,0,0.5)]"
                style={{ background: c.paper, color: c.ink }}
                onClick={() => api.openSticky(s.id).then(refresh)}
              >
                <div className="h-2 shrink-0" style={{ background: c.bar }} />
                <p className="selectable flex-1 overflow-hidden whitespace-pre-wrap px-4 py-3 text-sm leading-relaxed">
                  {s.text || <span className="opacity-50">Empty note</span>}
                </p>
                <div className="flex items-center justify-between px-4 pb-3 text-xs opacity-60">
                  <span>{timeAgo(s.updated)}</span>
                  <span className="flex gap-1 opacity-0 transition-opacity group-hover:opacity-100">
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
