// A row opened in a side panel or a centered dialog, with the same editor as the full page.

import { Dialog } from "radix-ui";
import { Maximize2, X } from "lucide-react";
import PageEditor from "@/features/notes/PageEditor";
import { useApp } from "@/store/app";

interface Props {
  rowId: string;
  mode: "side" | "center";
  onClose: () => void;
  onOpenPage: () => void;
}

export default function RowPeek({ rowId, mode, onClose, onOpenPage }: Props) {
  const openChild = (id: string) => {
    onClose();
    useApp.getState().openPage(id);
  };
  const bar = (
    <div className="flex h-9 shrink-0 items-center justify-between border-b border-line px-2">
      <button type="button" onClick={onOpenPage} className="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-muted hover:bg-surface-2 hover:text-fg">
        <Maximize2 size={13} /> Open as page
      </button>
      <button type="button" onClick={onClose} aria-label="Close" className="rounded-md p-1.5 text-muted hover:bg-surface-2 hover:text-fg">
        <X size={15} />
      </button>
    </div>
  );
  if (mode === "side") {
    return (
      <aside
        role="complementary"
        aria-label="Row"
        className="absolute inset-y-0 end-0 z-30 flex w-[min(580px,100%)] flex-col border-s border-line bg-bg shadow-float"
        onKeyDownCapture={(e) => {
          if (e.key !== "Escape" || e.defaultPrevented || e.nativeEvent.isComposing) return;
          if (!(e.target instanceof Element) || e.target.closest('[role="complementary"][aria-label="Row"], [role="dialog"]') !== e.currentTarget) return;
          // Find handles Escape at window capture. ProseMirror cancels it at the editor.
          e.preventDefault();
          e.stopPropagation();
          onClose();
        }}
      >
        {bar}
        <div className="min-h-0 flex-1">
          <PageEditor pageId={rowId} embedded onOpenChild={openChild} />
        </div>
      </aside>
    );
  }
  return (
    <Dialog.Root open onOpenChange={(o) => !o && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-black/45 backdrop-blur-sm" />
        <Dialog.Content
          aria-describedby={undefined}
          className="fixed left-1/2 top-1/2 z-50 flex h-[min(86vh,820px)] w-[min(880px,94vw)] -translate-x-1/2 -translate-y-1/2 flex-col overflow-hidden rounded-xl border border-line bg-bg shadow-float outline-none"
        >
          <Dialog.Title className="sr-only">Row</Dialog.Title>
          {bar}
          <div className="min-h-0 flex-1">
            <PageEditor pageId={rowId} embedded onOpenChild={openChild} />
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
