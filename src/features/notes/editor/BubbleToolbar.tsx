import { createContext, useContext, useRef, useState } from "react";
import { BubbleMenu } from "@tiptap/react/menus";
import { isTextSelection, useEditorState, type Editor } from "@tiptap/react";
import { Popover } from "radix-ui";
import {
  AlignCenter,
  AlignLeft,
  AlignRight,
  Bold,
  Check,
  ChevronDown,
  Code,
  Eraser,
  Italic,
  Link2,
  Minus,
  Plus,
  Radical,
  Strikethrough,
  Trash2,
  Underline,
} from "lucide-react";
import { HIGHLIGHTS, TEXT_COLORS } from "@/lib/palette";
import { Button, Input } from "@/components/ui";
import { cn } from "@/lib/utils";
import { applyTurnInto, BLOCKS, currentBlock, type BlockKey } from "./turnInto";

export const FONT_SIZES = [10, 12, 14, 16, 18, 20, 24, 28, 32, 40, 48, 64];
export const FONT_FAMILIES = [
  { label: "Default", value: "" },
  { label: "Sans", value: '"Inter Variable", "Segoe UI", sans-serif' },
  { label: "Serif", value: "Georgia, Cambria, serif" },
  { label: "Mono", value: '"JetBrains Mono Variable", Consolas, monospace' },
  { label: "Handwriting", value: '"Segoe Script", "Bradley Hand", cursive' },
  { label: "Rounded", value: '"Segoe UI Variable Display", "Trebuchet MS", sans-serif' },
];

/**
 * The pop-ups below are drawn inside the toolbar itself, not at the end of the page. Otherwise
 * clicking into one of them (to type a link) makes the editor lose focus and the toolbar closes
 * under the pop-up.
 */
const HostContext = createContext<HTMLElement | null>(null);

const panel = "z-[90] rounded-xl border border-line bg-elevated p-1.5 shadow-float outline-none";

/** The floating bar over selected text, and the one over a selected picture. */
export default function BubbleToolbar({ editor }: { editor: Editor }) {
  // The bar must stay open while one of its own pop-ups has focus.
  const popups = useRef(0);
  const track = (open: boolean) => {
    popups.current = Math.max(0, popups.current + (open ? 1 : -1));
  };

  return (
    <>
      <BubbleMenu
        editor={editor}
        pluginKey="textBubble"
        className="z-[85]"
        options={{ placement: "top", offset: 8 }}
        shouldShow={({ editor: ed, state, view, element }) => {
          const { selection } = state;
          if (!ed.isEditable || selection.empty || !isTextSelection(selection)) return false;
          if (ed.isActive("codeBlock")) return false;
          return view.hasFocus() || popups.current > 0 || element.contains(document.activeElement);
        }}
      >
        <TextTools editor={editor} track={track} />
      </BubbleMenu>
      <BubbleMenu
        editor={editor}
        pluginKey="imageBubble"
        className="z-[85]"
        options={{ placement: "top", offset: 8 }}
        shouldShow={({ editor: ed }) => ed.isEditable && ed.isActive("image")}
      >
        <ImageTools editor={editor} />
      </BubbleMenu>
    </>
  );
}

function TextTools({ editor, track }: { editor: Editor; track: (open: boolean) => void }) {
  const s = useEditorState({
    editor,
    selector: ({ editor: e }) => ({
      bold: e.isActive("bold"),
      italic: e.isActive("italic"),
      underline: e.isActive("underline"),
      strike: e.isActive("strike"),
      code: e.isActive("code"),
      link: e.isActive("link"),
      block: currentBlock(e),
      fontSize: (e.getAttributes("textStyle").fontSize as string | undefined) ?? "",
      fontFamily: (e.getAttributes("textStyle").fontFamily as string | undefined) ?? "",
      color: (e.getAttributes("textStyle").color as string | undefined) ?? "",
      align: e.isActive({ textAlign: "center" }) ? "center" : e.isActive({ textAlign: "right" }) ? "right" : "left",
    }),
  });
  const [host, setHost] = useState<HTMLDivElement | null>(null);
  const c = () => editor.chain().focus();
  const current = BLOCKS.find((b) => b.key === s.block) ?? BLOCKS[0];
  const sizeNow = s.fontSize ? parseInt(s.fontSize, 10) : null;
  const familyNow = FONT_FAMILIES.find((f) => f.value === s.fontFamily)?.label ?? (s.fontFamily ? "Custom" : "Default");
  const step = (delta: number) => c().setFontSize(`${Math.min(96, Math.max(8, (sizeNow ?? 16) + delta))}px`).run();

  return (
    <HostContext.Provider value={host}>
      <div ref={setHost} className="flex max-w-[calc(100vw-48px)] flex-wrap items-center gap-0.5 rounded-xl border border-line bg-elevated p-1 shadow-float">
        <TypeMenu editor={editor} track={track} current={s.block}>
          <button
            onMouseDown={(e) => e.preventDefault()}
            title="Change what kind of block this is"
            className="inline-flex h-8 items-center gap-1.5 rounded-lg px-2 text-xs text-fg hover:bg-surface-2"
          >
            <span className="text-muted">{current.icon}</span>
            {current.label}
            <ChevronDown size={12} className="text-faint" />
          </button>
        </TypeMenu>

        <Sep />

        <Pop
          track={track}
          width={190}
          trigger={
            <button
              onMouseDown={(e) => e.preventDefault()}
              title="Font"
              className="inline-flex h-8 min-w-[5.5rem] items-center justify-between gap-1 rounded-lg px-2 text-xs hover:bg-surface-2"
              style={{ fontFamily: s.fontFamily || undefined }}
            >
              <span className="truncate">{familyNow}</span>
              <ChevronDown size={12} className="shrink-0 text-faint" />
            </button>
          }
        >
          {FONT_FAMILIES.map((f) => (
            <Row key={f.label} active={f.value === s.fontFamily} onClick={() => (f.value ? c().setFontFamily(f.value).run() : c().unsetFontFamily().run())}>
              <span style={{ fontFamily: f.value || undefined }}>{f.label}</span>
            </Row>
          ))}
        </Pop>
        <Btn label="Smaller text" onClick={() => step(-2)}>
          <Minus size={14} />
        </Btn>
        <Pop
          track={track}
          width={96}
          trigger={
            <button onMouseDown={(e) => e.preventDefault()} title="Text size" className="h-8 min-w-10 rounded-lg px-1.5 text-center text-xs tabular-nums hover:bg-surface-2">
              {sizeNow ?? "Auto"}
            </button>
          }
        >
          <Row onClick={() => c().unsetFontSize().run()} active={sizeNow === null}>
            Auto
          </Row>
          {FONT_SIZES.map((n) => (
            <Row key={n} active={sizeNow === n} onClick={() => c().setFontSize(`${n}px`).run()}>
              <span className="tabular-nums">{n}</span>
            </Row>
          ))}
        </Pop>
        <Btn label="Larger text" onClick={() => step(2)}>
          <Plus size={14} />
        </Btn>

        <Sep />

        <Btn label="Bold (Ctrl+B)" active={s.bold} onClick={() => c().toggleBold().run()}>
          <Bold size={15} />
        </Btn>
        <Btn label="Italic (Ctrl+I)" active={s.italic} onClick={() => c().toggleItalic().run()}>
          <Italic size={15} />
        </Btn>
        <Btn label="Underline (Ctrl+U)" active={s.underline} onClick={() => c().toggleUnderline().run()}>
          <Underline size={15} />
        </Btn>
        <Btn label="Strikethrough" active={s.strike} onClick={() => c().toggleStrike().run()}>
          <Strikethrough size={15} />
        </Btn>
        <Btn label="Code" active={s.code} onClick={() => c().toggleCode().run()}>
          <Code size={15} />
        </Btn>

        <Sep />

        <LinkPop editor={editor} active={s.link} track={track} />
        <ColorPop editor={editor} color={s.color} track={track} />
        <Btn label="Make the selected text an equation (TeX)" onClick={() => c().insertMathInline().run()}>
          <Radical size={15} />
        </Btn>
        <AlignPop editor={editor} align={s.align} track={track} />
        <Btn label="Remove formatting" onClick={() => c().unsetAllMarks().run()}>
          <Eraser size={15} />
        </Btn>
      </div>
    </HostContext.Provider>
  );
}

/* ---------- block type list ---------- */

export function TypeMenu({
  editor,
  track,
  current,
  children,
  side = "bottom",
}: {
  editor: Editor;
  track: (open: boolean) => void;
  current: BlockKey;
  children: React.ReactNode;
  side?: "right" | "bottom";
}) {
  const host = useContext(HostContext);
  return (
    <Popover.Root onOpenChange={track}>
      <Popover.Trigger asChild>{children}</Popover.Trigger>
      <Popover.Portal container={host}>
        <Popover.Content
          side={side}
          align="start"
          sideOffset={6}
          collisionPadding={8}
          style={{ width: 216, maxHeight: "min(70vh, 560px)" }}
          className={cn(panel, "overflow-y-auto")}
          onOpenAutoFocus={(e) => e.preventDefault()}
        >
          {BLOCKS.map((b) => (
            <Popover.Close asChild key={b.key}>
              <button
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => void applyTurnInto(editor, b.key)}
                className="flex h-8 w-full items-center gap-2.5 rounded-lg px-2.5 text-left text-sm text-fg hover:bg-surface-2"
              >
                <span className="text-muted">{b.icon}</span>
                <span className="flex-1">{b.label}</span>
                {b.key === current && <Check size={14} className="text-muted" />}
              </button>
            </Popover.Close>
          ))}
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}

/* ---------- picture toolbar ---------- */

function ImageTools({ editor }: { editor: Editor }) {
  const s = useEditorState({
    editor,
    selector: ({ editor: e }) => ({
      align: (e.getAttributes("image").align as string | undefined) ?? "center",
      width: (e.getAttributes("image").width as number | null | undefined) ?? null,
    }),
  });
  const set = (attrs: Record<string, unknown>) => editor.chain().focus().updateAttributes("image", attrs).run();
  return (
    <div className="flex items-center gap-0.5 rounded-xl border border-line bg-elevated p-1 shadow-float">
      <Btn label="Align left" active={s.align === "left"} onClick={() => set({ align: "left" })}>
        <AlignLeft size={15} />
      </Btn>
      <Btn label="Center" active={s.align === "center"} onClick={() => set({ align: "center" })}>
        <AlignCenter size={15} />
      </Btn>
      <Btn label="Align right" active={s.align === "right"} onClick={() => set({ align: "right" })}>
        <AlignRight size={15} />
      </Btn>
      <Sep />
      {[25, 50, 75, 100].map((w) => (
        <button
          key={w}
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => set({ width: w })}
          className={cn("h-8 rounded-lg px-2 text-xs tabular-nums transition-colors", s.width === w ? "bg-surface-2 text-fg" : "text-muted hover:bg-surface-2 hover:text-fg")}
        >
          {w}%
        </button>
      ))}
      <Sep />
      <Btn label="Remove picture" onClick={() => editor.chain().focus().deleteSelection().run()}>
        <Trash2 size={15} />
      </Btn>
    </div>
  );
}

/* ---------- small pieces ---------- */

function Btn({ label, active, onClick, children }: { label: string; active?: boolean; onClick: () => void; children: React.ReactNode }) {
  return (
    <button
      title={label}
      aria-label={label}
      onMouseDown={(e) => e.preventDefault()}
      onClick={onClick}
      className={cn("grid h-8 w-8 place-items-center rounded-lg transition-colors", active ? "bg-surface-2 text-fg" : "text-muted hover:bg-surface-2 hover:text-fg")}
    >
      {children}
    </button>
  );
}

export function Sep() {
  return <div className="mx-0.5 h-5 w-px bg-line" />;
}

export function Row({ active, onClick, children }: { active?: boolean; onClick: () => void; children: React.ReactNode }) {
  return (
    <Popover.Close asChild>
      <button
        onMouseDown={(e) => e.preventDefault()}
        onClick={onClick}
        className="flex h-8 w-full items-center gap-2 rounded-lg px-2.5 text-left text-sm text-fg hover:bg-surface-2"
      >
        <span className="flex-1">{children}</span>
        {active && <Check size={14} className="text-muted" />}
      </button>
    </Popover.Close>
  );
}

export function Pop({
  trigger,
  children,
  width,
  track,
  onOpen,
  onClose,
  align = "start",
}: {
  trigger: React.ReactNode;
  children: React.ReactNode;
  width: number;
  track: (open: boolean) => void;
  onOpen?: () => void;
  /** Where focus goes when the pop-up closes; without it, focus returns to the trigger. */
  onClose?: () => void;
  align?: "start" | "center" | "end";
}) {
  const host = useContext(HostContext);
  return (
    <Popover.Root
      onOpenChange={(o) => {
        track(o);
        if (o) onOpen?.();
      }}
    >
      <Popover.Trigger asChild>{trigger}</Popover.Trigger>
      <Popover.Portal container={host}>
        <Popover.Content
          sideOffset={6}
          align={align}
          collisionPadding={8}
          style={{ width, maxHeight: "min(60vh, 420px)" }}
          className={cn(panel, "overflow-y-auto")}
          onOpenAutoFocus={(e) => e.preventDefault()}
          onCloseAutoFocus={
            onClose
              ? (e) => {
                  e.preventDefault();
                  onClose();
                }
              : undefined
          }
        >
          {children}
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}

export function ColorPop({ editor, color, track, onClose }: { editor: Editor; color: string; track: (o: boolean) => void; onClose?: () => void }) {
  return (
    <Pop
      track={track}
      onClose={onClose}
      width={236}
      trigger={
        <button
          onMouseDown={(e) => e.preventDefault()}
          title="Text color and highlight"
          aria-label="Text color and highlight"
          className="grid h-8 w-8 place-items-center rounded-lg text-muted transition-colors hover:bg-surface-2 hover:text-fg"
        >
          <span className="text-sm font-bold" style={{ color: color || undefined, borderBottom: `2px solid ${color || "currentColor"}`, lineHeight: 1 }}>
            A
          </span>
        </button>
      }
    >
      <div className="px-1.5 pb-1 pt-0.5 text-xs font-medium text-faint">Text color</div>
      <div className="grid grid-cols-5 gap-1.5 p-1">
        {TEXT_COLORS.map((t) => (
          <button
            key={t.name}
            title={t.name}
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => (t.value ? editor.chain().focus().setColor(t.value).run() : editor.chain().focus().unsetColor().run())}
            className="grid h-9 place-items-center rounded-lg border border-line text-sm font-bold hover:bg-surface-2"
            style={{ color: t.value || "var(--text)" }}
          >
            A
          </button>
        ))}
      </div>
      <div className="px-1.5 pb-1 pt-2 text-xs font-medium text-faint">Highlight</div>
      <div className="grid grid-cols-5 gap-1.5 p-1">
        <button
          title="No highlight"
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => editor.chain().focus().unsetHighlight().run()}
          className="grid h-9 place-items-center rounded-lg border border-line text-xs text-muted hover:bg-surface-2"
        >
          ✕
        </button>
        {HIGHLIGHTS.map((h) => (
          <button
            key={h.name}
            title={h.name}
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => editor.chain().focus().setHighlight({ color: h.value }).run()}
            className="h-9 rounded-lg border border-line"
            style={{ background: h.value }}
          />
        ))}
      </div>
    </Pop>
  );
}

function AlignPop({ editor, align, track }: { editor: Editor; align: string; track: (o: boolean) => void }) {
  const Icon = align === "center" ? AlignCenter : align === "right" ? AlignRight : AlignLeft;
  return (
    <Pop
      track={track}
      width={150}
      align="end"
      trigger={
        <button
          onMouseDown={(e) => e.preventDefault()}
          title="Align text"
          aria-label="Align text"
          className="grid h-8 w-8 place-items-center rounded-lg text-muted transition-colors hover:bg-surface-2 hover:text-fg"
        >
          <Icon size={15} />
        </button>
      }
    >
      <div className="flex gap-1 p-0.5">
        {(
          [
            ["left", <AlignLeft size={15} key="l" />],
            ["center", <AlignCenter size={15} key="c" />],
            ["right", <AlignRight size={15} key="r" />],
          ] as const
        ).map(([a, icon]) => (
          <Popover.Close asChild key={a}>
            <button
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => editor.chain().focus().setTextAlign(a).run()}
              aria-label={`Align ${a}`}
              className={cn("grid h-8 flex-1 place-items-center rounded-lg border hover:bg-surface-2", align === a ? "border-fg/50 bg-surface-2" : "border-line text-muted")}
            >
              {icon}
            </button>
          </Popover.Close>
        ))}
      </div>
    </Pop>
  );
}

export function LinkPop({ editor, active, track, onClose }: { editor: Editor; active: boolean; track: (o: boolean) => void; onClose?: () => void }) {
  const [url, setUrl] = useState("");
  return (
    <Pop
      track={track}
      onClose={onClose}
      width={300}
      onOpen={() => setUrl((editor.getAttributes("link").href as string | undefined) ?? "")}
      trigger={
        <button
          onMouseDown={(e) => e.preventDefault()}
          title="Link (Ctrl+click a link to open it)"
          aria-label="Link"
          className={cn("grid h-8 w-8 place-items-center rounded-lg transition-colors", active ? "bg-surface-2 text-fg" : "text-muted hover:bg-surface-2 hover:text-fg")}
        >
          <Link2 size={15} />
        </button>
      }
    >
      <form
        className="flex gap-2 p-0.5"
        onSubmit={(e) => {
          e.preventDefault();
          const chain = editor.chain().focus().extendMarkRange("link");
          if (url.trim()) chain.setLink({ href: url.trim() }).run();
          else chain.unsetLink().run();
        }}
      >
        <Input autoFocus value={url} onChange={(e) => setUrl(e.target.value)} placeholder="Paste a link…" className="h-8" />
        <Popover.Close asChild>
          <Button type="submit" variant="primary" size="sm">
            Save
          </Button>
        </Popover.Close>
      </form>
    </Pop>
  );
}
