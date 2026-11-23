import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button, Modal } from "@/components/ui";
import { applyAppearance, DEFAULT_APPEARANCE, type Appearance } from "@/lib/theme";

type Prompt = {
  title: string;
  body: string;
  action: string;
  danger: boolean;
  notice?: boolean;
  appearance?: Pick<Appearance, "theme" | "accent">;
};

/** Render only the prompt supplied to this isolated window by Rust. */
export default function ConfirmationWindow() {
  const [prompt, setPrompt] = useState<Prompt>();
  const [busy, setBusy] = useState(false);
  const answering = useRef(false);

  useEffect(() => {
    let active = true;
    void invoke<Prompt>("confirmation_prompt").then(async (value) => {
      if (!active) return;
      await applyAppearance({ ...DEFAULT_APPEARANCE, ...value.appearance }, false);
      if (active) setPrompt(value);
    }).catch(() => {
      if (active) void invoke("confirmation_answer", { allow: false }).catch(() => {});
    });
    return () => { active = false; };
  }, []);

  useEffect(() => {
    if (!prompt) return;
    // The native window remains hidden until its themed dialog is mounted.
    void invoke("confirmation_ready").catch(() => {});
  }, [prompt]);

  async function answer(allow: boolean) {
    if (answering.current) return;
    answering.current = true;
    setBusy(true);
    try {
      await invoke("confirmation_answer", { allow });
    } catch {
      answering.current = false;
      setBusy(false);
    }
  }

  if (!prompt) return null;
  return (
    <Modal
      open
      title={prompt.title}
      onOpenChange={(open) => { if (!open) void answer(false); }}
      busy={busy}
    >
      <p className="whitespace-pre-line break-words text-sm text-fg">{prompt.body}</p>
      <div className="mt-6 flex justify-end gap-2">
        {!prompt.notice && (
          <Button type="button" variant="secondary" onClick={() => void answer(false)} disabled={busy}>
            Cancel
          </Button>
        )}
        <Button type="button" variant={prompt.danger ? "danger" : "primary"} onClick={() => void answer(true)} disabled={busy}>
          {prompt.action}
        </Button>
      </div>
    </Modal>
  );
}
