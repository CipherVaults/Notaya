// "New database": a name and a starter (blank, tasks, reading list, projects), or a new database from a CSV file.

import { useState } from "react";
import { BookOpen, CheckSquare, FileUp, FolderKanban, Table as TableIcon } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import { cn } from "@/lib/utils";
import { useApp } from "@/store/app";
import { Button, Modal } from "@/components/ui";
import { fieldCls } from "./panels/ui";

type Starter = "blank" | "tasks" | "reading" | "projects";
const STARTERS: { id: Starter; label: string; hint: string; icon: React.ReactNode; title: string }[] = [
  { id: "blank", label: "Blank", hint: "A table with a name column", icon: <TableIcon size={20} />, title: "Untitled database" },
  { id: "tasks", label: "Tasks", hint: "Status, due date, priority; board and calendar views", icon: <CheckSquare size={20} />, title: "Tasks" },
  { id: "reading", label: "Reading list", hint: "Author, status, rating, tags; gallery view", icon: <BookOpen size={20} />, title: "Reading list" },
  { id: "projects", label: "Projects", hint: "Status and dates; timeline view", icon: <FolderKanban size={20} />, title: "Projects" },
];

export interface NewDatabaseTarget {
  sectionId: string;
  /** A page to put the database under ("" = directly in the section). */
  parent: string;
}

export default function NewDatabaseDialog({ target, onClose }: { target: NewDatabaseTarget | null; onClose: () => void }) {
  const [starter, setStarter] = useState<Starter>("blank");
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);
  const setTree = useApp((s) => s.setTree);

  const create = async () => {
    if (!target) return;
    setBusy(true);
    try {
      const title = name.trim() || STARTERS.find((s) => s.id === starter)!.title;
      const { database, notice } = await api.createDatabase(target.sectionId, target.parent, false, title, starter);
      setTree(await api.listNotes());
      useApp.getState().openDatabase(database.id);
      if (notice) toast.info(notice, { duration: 20000 });
      setName("");
      setStarter("blank");
      onClose();
    } catch (e) {
      toast.error(errorText(e));
    } finally {
      setBusy(false);
    }
  };
  const fromCsv = async () => {
    setBusy(true);
    try {
      const r = await api.importCsv(null);
      if (!r) return;
      setTree(await api.listNotes());
      useApp.getState().openDatabase(r.databaseId);
      toast.success(`Imported ${r.rows} ${r.rows === 1 ? "row" : "rows"} into a new database.`);
      if (r.skipped.length) toast.warning(`Skipped: ${r.skipped.slice(0, 3).join("; ")}`);
      onClose();
    } catch (e) {
      toast.error(errorText(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Modal open={!!target} onOpenChange={(o) => !o && onClose()} title="New database" description="Rows are pages. Pick a starting point; you can change everything later." width={520}>
      <div className="space-y-4">
        <div className="grid grid-cols-2 gap-2" role="radiogroup" aria-label="Starter">
          {STARTERS.map((s) => (
            <button
              key={s.id}
              type="button"
              role="radio"
              aria-checked={starter === s.id}
              onClick={() => setStarter(s.id)}
              className={cn("flex items-start gap-3 rounded-lg border p-3 text-start transition-colors", starter === s.id ? "border-accent bg-accent-soft" : "border-line hover:bg-surface-2")}
            >
              <span className="mt-0.5 text-muted">{s.icon}</span>
              <span>
                <span className="block text-sm font-medium">{s.label}</span>
                <span className="block text-xs text-muted">{s.hint}</span>
              </span>
            </button>
          ))}
        </div>
        <input
          autoFocus
          aria-label="Database name"
          dir="auto"
          value={name}
          onChange={(e) => setName(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && void create()}
          placeholder={STARTERS.find((s) => s.id === starter)!.title}
          className={cn(fieldCls, "h-9 w-full")}
        />
        <div className="flex items-center justify-between">
          <Button variant="ghost" size="sm" onClick={fromCsv} disabled={busy}>
            <FileUp size={14} /> From a CSV file…
          </Button>
          <div className="flex gap-2">
            <Button variant="ghost" onClick={onClose}>Cancel</Button>
            <Button variant="primary" onClick={() => void create()} disabled={busy}>Create</Button>
          </div>
        </div>
      </div>
    </Modal>
  );
}
