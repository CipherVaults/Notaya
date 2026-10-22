import { useEffect, useState } from "react";
import { NOTEBOOK_ICONS, TAG_COLORS } from "@/lib/palette";
import { cn } from "@/lib/utils";
import { Button, Field, Input, Modal } from "@/components/ui";

export interface NotebookDraft {
  name: string;
  icon: string;
  color: string;
}

export default function NotebookDialog({
  open,
  onOpenChange,
  initial,
  title,
  onSubmit,
}: {
  open: boolean;
  onOpenChange: (v: boolean) => void;
  initial?: NotebookDraft;
  title: string;
  onSubmit: (d: NotebookDraft) => void;
}) {
  const [draft, setDraft] = useState<NotebookDraft>({ name: "", icon: "📓", color: TAG_COLORS[0] });

  useEffect(() => {
    if (open) setDraft(initial ?? { name: "", icon: "📓", color: TAG_COLORS[0] });
  }, [open, initial]);

  return (
    <Modal open={open} onOpenChange={onOpenChange} title={title}>
      <form
        className="space-y-5"
        onSubmit={(e) => {
          e.preventDefault();
          if (draft.name.trim()) onSubmit({ ...draft, name: draft.name.trim() });
        }}
      >
        <Field label="Name">
          <Input autoFocus value={draft.name} onChange={(e) => setDraft({ ...draft, name: e.target.value })} placeholder="e.g. Personal" />
        </Field>
        <Field label="Icon">
          <div className="grid grid-cols-8 gap-1.5">
            {NOTEBOOK_ICONS.map((i) => (
              <button
                type="button"
                key={i}
                onClick={() => setDraft({ ...draft, icon: i })}
                className={cn(
                  "grid h-10 place-items-center rounded-xl text-lg transition-all",
                  draft.icon === i ? "bg-accent-soft ring-2 ring-accent" : "bg-surface hover:bg-surface-2",
                )}
              >
                {i}
              </button>
            ))}
          </div>
        </Field>
        <Field label="Color">
          <div className="flex flex-wrap gap-2">
            {TAG_COLORS.map((c) => (
              <button
                type="button"
                key={c}
                onClick={() => setDraft({ ...draft, color: c })}
                className={cn("h-7 w-7 rounded-full transition-transform hover:scale-110", draft.color === c && "ring-2 ring-offset-2 ring-offset-[var(--elevated)]")}
                style={{ background: c, ["--tw-ring-color" as string]: c }}
                aria-label={c}
              />
            ))}
          </div>
        </Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button type="submit" variant="primary" disabled={!draft.name.trim()}>
            Save
          </Button>
        </div>
      </form>
    </Modal>
  );
}
