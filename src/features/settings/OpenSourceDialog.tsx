import { useEffect, useState } from "react";
import { Modal } from "@/components/ui";

/**
 * Credits and licenses for the open source software Cipherleaf includes or runs.
 * The full text is THIRD-PARTY-NOTICES.md (also in the Licenses folder of the portable copy).
 * It is loaded only when this dialog opens, so it does not slow down the app's start.
 */
export default function OpenSourceDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (v: boolean) => void }) {
  const [text, setText] = useState<string | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    if (!open || text !== null) return;
    let alive = true;
    import("../../../THIRD-PARTY-NOTICES.md?raw")
      .then((m) => alive && setText(m.default))
      .catch(() => alive && setFailed(true));
    return () => {
      alive = false;
    };
  }, [open, text]);

  return (
    <Modal
      open={open}
      onOpenChange={onOpenChange}
      title="Open source licenses"
      description="Cipherleaf is built on open source software. Thank you to everyone who made it."
      width={760}
    >
      <div className="space-y-3 text-sm leading-relaxed">
        <ul className="list-disc space-y-1 pl-5 text-muted">
          <li>
            Passwords are kept by <span className="text-fg">KeePassXC 2.8.0</span> (GPL-2.0 or GPL-3.0), bundled unmodified
            as a separate program. Its license text and source are in the <span className="font-mono text-xs">Licenses</span>{" "}
            folder next to Cipherleaf.exe.
          </li>
          <li>
            The app is built with Tauri, Rust crates and npm packages (React, TipTap, ProseMirror, KaTeX and others),
            fonts under the SIL Open Font License, and the EFF large word list (CC BY 3.0 US, Electronic Frontier Foundation).
          </li>
          <li>The license for Cipherleaf itself has not been chosen yet. This list is attribution, not legal advice.</li>
        </ul>
        <pre
          tabIndex={0}
          className="selectable max-h-[48vh] overflow-auto whitespace-pre-wrap break-words rounded-xl border border-line bg-surface p-3 font-mono text-[11px] leading-relaxed text-muted"
        >
          {failed ? "The notices could not be loaded. They are in THIRD-PARTY-NOTICES.md next to the app." : (text ?? "Loading...")}
        </pre>
      </div>
    </Modal>
  );
}
