// The "Page or section" choice of the link menu: pick a page, then optionally one of its headings, and turn the
// selected text into an internal link. Also carries out a jump to a heading that was asked for before this page loaded.

import { useEffect, useRef, useState } from "react";
import type { Editor } from "@tiptap/react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import { useApp } from "@/store/app";
import PagePicker, { HeadingList, type HeadingEntry, type PickerPlacement } from "./PagePicker";
import { consumePendingJump, makeInternalHref, newAnchor, onLinkPickerRequest } from "./internalLinks";
import { parseContent } from "./extensions";

type Step =
  | { kind: "page"; placement: PickerPlacement }
  | { kind: "heading"; placement: PickerPlacement; pageId: string; title: string; headings: HeadingEntry[] };

/** Headings in saved page content (JSON), in reading order. */
function headingsOfJson(node: unknown, out: HeadingEntry[] = []): HeadingEntry[] {
  if (!node || typeof node !== "object") return out;
  const n = node as { type?: string; attrs?: { level?: number; anchor?: string | null }; content?: unknown[]; text?: string };
  if (n.type === "heading") {
    const text = (function flat(x: unknown): string {
      const t = x as { text?: string; content?: unknown[] };
      return (t.text ?? "") + (t.content ?? []).map(flat).join("");
    })(n);
    out.push({ level: Math.min(4, Math.max(1, Number(n.attrs?.level) || 1)), text: text.trim(), anchor: n.attrs?.anchor ?? null });
    return out;
  }
  for (const c of n.content ?? []) headingsOfJson(c, out);
  return out;
}

function headingsOfEditor(editor: Editor): HeadingEntry[] {
  const out: HeadingEntry[] = [];
  editor.state.doc.descendants((node, pos) => {
    if (node.type.name !== "heading") return;
    out.push({ level: Number(node.attrs.level) || 1, text: node.textContent.trim(), anchor: (node.attrs.anchor as string | null) ?? null, pos });
  });
  return out;
}

export default function InternalLinkHost({ editor, pageId }: { editor: Editor; pageId: string }) {
  const [step, setStep] = useState<Step | null>(null);
  const saved = useRef<{ from: number; to: number }>({ from: 0, to: 0 });

  // A link followed from another page asked to land on a heading here.
  useEffect(() => {
    consumePendingJump(editor, pageId);
  }, [editor, pageId]);

  useEffect(
    () =>
      onLinkPickerRequest((ed) => {
        if (ed !== editor) return;
        const { from, to } = editor.state.selection;
        saved.current = { from, to };
        const c = editor.view.coordsAtPos(from);
        setStep({ kind: "page", placement: { left: c.left, top: c.top, bottom: c.bottom } });
      }),
    [editor],
  );

  const close = () => {
    setStep(null);
    editor.commands.focus();
  };

  const apply = (href: string, label: string) => {
    setStep(null);
    const { from, to } = saved.current;
    const max = editor.state.doc.content.size;
    const a = Math.min(from, max);
    const b = Math.min(to, max);
    const chain = editor.chain().focus();
    if (a !== b) {
      chain.setTextSelection({ from: a, to: b }).setLink({ href }).run();
    } else if (editor.state.doc.resolve(a).marks().some((m) => m.type.name === "link")) {
      chain.setTextSelection(a).extendMarkRange("link").setLink({ href }).run();
    } else {
      chain
        .setTextSelection(a)
        .insertContent({ type: "text", text: label })
        .setTextSelection({ from: a, to: a + label.length })
        .setLink({ href })
        .setTextSelection(a + label.length)
        .run();
    }
  };

  const choosePage = async (id: string) => {
    if (step?.kind !== "page") return;
    const placement = step.placement;
    const page = useApp.getState().tree.pages.find((p) => p.id === id);
    const title = page?.title || "Untitled";
    try {
      const headings = id === pageId ? headingsOfEditor(editor) : headingsOfJson(parseContent((await api.getPage(id)).content));
      // A page without headings can only be linked as a whole.
      if (headings.length === 0) apply(makeInternalHref(id), selectedText() || title);
      else setStep({ kind: "heading", placement, pageId: id, title, headings });
    } catch (e) {
      toast.error(errorText(e));
      setStep(null);
    }
  };

  const selectedText = () => {
    const { from, to } = saved.current;
    return from === to ? "" : editor.state.doc.textBetween(from, to, " ");
  };

  const chooseHeading = (h: HeadingEntry | null) => {
    if (step?.kind !== "heading") return;
    if (!h) return apply(makeInternalHref(step.pageId), selectedText() || step.title);
    let anchor = h.anchor;
    // A heading in this page that has no anchor yet gets one now, so the link keeps working if the heading is renamed.
    if (!anchor && h.pos !== undefined && step.pageId === pageId) {
      const node = editor.state.doc.nodeAt(h.pos);
      if (node?.type.name === "heading") {
        anchor = newAnchor();
        editor.view.dispatch(editor.state.tr.setNodeMarkup(h.pos, undefined, { ...node.attrs, anchor }).setMeta("addToHistory", false));
        // Positions of the saved selection do not move: only an attribute changed.
      }
    }
    apply(makeInternalHref(step.pageId, { anchor, text: h.text }), selectedText() || h.text || step.title);
  };

  if (!step) return null;
  if (step.kind === "page") return <PagePicker includeCurrent placement={step.placement} onPick={(id) => void choosePage(id)} onClose={close} />;
  return (
    <HeadingList
      placement={step.placement}
      pageTitle={step.title}
      headings={step.headings}
      onPick={chooseHeading}
      onBack={() => setStep({ kind: "page", placement: step.placement })}
      onClose={close}
    />
  );
}
