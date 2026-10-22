import { useRef, useState } from "react";
import { useEditorState, type Editor } from "@tiptap/react";
import { Popover } from "radix-ui";
import {
  AlignCenter,
  AlignLeft,
  AlignRight,
  Bold,
  Code2,
  Heading1,
  Heading2,
  Heading3,
  Highlighter,
  ImagePlus,
  Italic,
  Link2,
  List,
  ListChecks,
  ListOrdered,
  Minus,
  Pilcrow,
  Quote,
  Redo2,
  Strikethrough,
  Table2,
  Type,
  Underline,
  Undo2,
} from "lucide-react";
import { HIGHLIGHTS, TEXT_COLORS } from "@/lib/palette";
import { IconButton, Input, Button } from "@/components/ui";
import { cn } from "@/lib/utils";
import { insertImages } from "./extensions";

export default function Toolbar({ editor }: { editor: Editor }) {
  const s = useEditorState({
    editor,
    selector: ({ editor: e }) => ({
      bold: e.isActive("bold"),
      italic: e.isActive("italic"),
      underline: e.isActive("underline"),
      strike: e.isActive("strike"),
      h1: e.isActive("heading", { level: 1 }),
      h2: e.isActive("heading", { level: 2 }),
      h3: e.isActive("heading", { level: 3 }),
      bullet: e.isActive("bulletList"),
      ordered: e.isActive("orderedList"),
      task: e.isActive("taskList"),
      quote: e.isActive("blockquote"),
      code: e.isActive("codeBlock"),
      link: e.isActive("link"),
      highlight: e.isActive("highlight"),
      left: e.isActive({ textAlign: "left" }),
      center: e.isActive({ textAlign: "center" }),
      right: e.isActive({ textAlign: "right" }),
      table: e.isActive("table"),
      canUndo: e.can().undo(),
      canRedo: e.can().redo(),
    }),
  });
  const fileRef = useRef<HTMLInputElement>(null);
  const c = () => editor.chain().focus();

  return (
    <div className="flex flex-wrap items-center gap-0.5 rounded-2xl border border-line bg-elevated/90 px-1.5 py-1 shadow-sm backdrop-blur-xl">
      <IconButton size="sm" label="Undo (Ctrl+Z)" disabled={!s.canUndo} onClick={() => c().undo().run()}>
        <Undo2 size={15} />
      </IconButton>
      <IconButton size="sm" label="Redo (Ctrl+Y)" disabled={!s.canRedo} onClick={() => c().redo().run()}>
        <Redo2 size={15} />
      </IconButton>
      <Sep />
      <IconButton size="sm" label="Normal text" active={!s.h1 && !s.h2 && !s.h3} onClick={() => c().setParagraph().run()}>
        <Pilcrow size={15} />
      </IconButton>
      <IconButton size="sm" label="Heading 1" active={s.h1} onClick={() => c().toggleHeading({ level: 1 }).run()}>
        <Heading1 size={16} />
      </IconButton>
      <IconButton size="sm" label="Heading 2" active={s.h2} onClick={() => c().toggleHeading({ level: 2 }).run()}>
        <Heading2 size={16} />
      </IconButton>
      <IconButton size="sm" label="Heading 3" active={s.h3} onClick={() => c().toggleHeading({ level: 3 }).run()}>
        <Heading3 size={16} />
      </IconButton>
      <Sep />
      <IconButton size="sm" label="Bold (Ctrl+B)" active={s.bold} onClick={() => c().toggleBold().run()}>
        <Bold size={15} />
      </IconButton>
      <IconButton size="sm" label="Italic (Ctrl+I)" active={s.italic} onClick={() => c().toggleItalic().run()}>
        <Italic size={15} />
      </IconButton>
      <IconButton size="sm" label="Underline (Ctrl+U)" active={s.underline} onClick={() => c().toggleUnderline().run()}>
        <Underline size={15} />
      </IconButton>
      <IconButton size="sm" label="Strikethrough" active={s.strike} onClick={() => c().toggleStrike().run()}>
        <Strikethrough size={15} />
      </IconButton>
      <ColorPopover editor={editor} highlightActive={s.highlight} />
      <LinkPopover editor={editor} active={s.link} />
      <Sep />
      <IconButton size="sm" label="Bullet list" active={s.bullet} onClick={() => c().toggleBulletList().run()}>
        <List size={15} />
      </IconButton>
      <IconButton size="sm" label="Numbered list" active={s.ordered} onClick={() => c().toggleOrderedList().run()}>
        <ListOrdered size={15} />
      </IconButton>
      <IconButton size="sm" label="Checklist" active={s.task} onClick={() => c().toggleTaskList().run()}>
        <ListChecks size={15} />
      </IconButton>
      <IconButton size="sm" label="Quote" active={s.quote} onClick={() => c().toggleBlockquote().run()}>
        <Quote size={15} />
      </IconButton>
      <IconButton size="sm" label="Code block" active={s.code} onClick={() => c().toggleCodeBlock().run()}>
        <Code2 size={15} />
      </IconButton>
      <Sep />
      <IconButton size="sm" label="Align left" active={s.left} onClick={() => c().setTextAlign("left").run()}>
        <AlignLeft size={15} />
      </IconButton>
      <IconButton size="sm" label="Align center" active={s.center} onClick={() => c().setTextAlign("center").run()}>
        <AlignCenter size={15} />
      </IconButton>
      <IconButton size="sm" label="Align right" active={s.right} onClick={() => c().setTextAlign("right").run()}>
        <AlignRight size={15} />
      </IconButton>
      <Sep />
      <IconButton
        size="sm"
        label={s.table ? "Add row below" : "Insert table"}
        active={s.table}
        onClick={() =>
          s.table ? c().addRowAfter().run() : c().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()
        }
      >
        <Table2 size={15} />
      </IconButton>
      <IconButton size="sm" label="Insert image" onClick={() => fileRef.current?.click()}>
        <ImagePlus size={15} />
      </IconButton>
      <IconButton size="sm" label="Divider" onClick={() => c().setHorizontalRule().run()}>
        <Minus size={15} />
      </IconButton>
      <input
        ref={fileRef}
        type="file"
        accept="image/png,image/jpeg,image/gif,image/webp"
        multiple
        hidden
        onChange={(e) => {
          const files = Array.from(e.target.files ?? []);
          e.target.value = "";
          insertImages(editor, files);
        }}
      />
    </div>
  );
}

function Sep() {
  return <div className="mx-1 h-5 w-px bg-line" />;
}

function ColorPopover({ editor, highlightActive }: { editor: Editor; highlightActive: boolean }) {
  return (
    <Popover.Root>
      <Popover.Trigger asChild>
        <button
          aria-label="Colors"
          title="Text color & highlight"
          className={cn(
            "inline-flex h-7 w-7 items-center justify-center rounded-lg transition-colors",
            highlightActive ? "bg-accent-soft text-accent" : "text-muted hover:bg-surface-2 hover:text-fg",
          )}
        >
          <Highlighter size={15} />
        </button>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content sideOffset={8} className="z-[70] w-60 rounded-xl border border-line bg-elevated p-3 shadow-float">
          <div className="mb-2 flex items-center gap-1.5 text-xs font-medium text-faint">
            <Highlighter size={12} /> Highlight
          </div>
          <div className="mb-3 flex gap-1.5">
            {HIGHLIGHTS.map((h) => (
              <button
                key={h.value}
                title={h.name}
                onClick={() => editor.chain().focus().toggleHighlight({ color: h.value }).run()}
                className="h-7 w-7 rounded-lg border border-line transition-transform hover:scale-110"
                style={{ background: h.value }}
              />
            ))}
            <button
              title="Remove highlight"
              onClick={() => editor.chain().focus().unsetHighlight().run()}
              className="h-7 w-7 rounded-lg border border-line text-xs text-muted hover:bg-surface-2"
            >
              ✕
            </button>
          </div>
          <div className="mb-2 flex items-center gap-1.5 text-xs font-medium text-faint">
            <Type size={12} /> Text color
          </div>
          <div className="flex flex-wrap gap-1.5">
            {TEXT_COLORS.map((t) => (
              <button
                key={t.name}
                title={t.name}
                onClick={() =>
                  t.value
                    ? editor.chain().focus().setColor(t.value).run()
                    : editor.chain().focus().unsetColor().run()
                }
                className="grid h-7 w-7 place-items-center rounded-lg border border-line text-sm font-bold hover:bg-surface-2"
                style={{ color: t.value || "var(--text)" }}
              >
                A
              </button>
            ))}
          </div>
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}

function LinkPopover({ editor, active }: { editor: Editor; active: boolean }) {
  const [url, setUrl] = useState("");
  const [open, setOpen] = useState(false);
  return (
    <Popover.Root
      open={open}
      onOpenChange={(o) => {
        setOpen(o);
        if (o) setUrl(editor.getAttributes("link").href ?? "");
      }}
    >
      <Popover.Trigger asChild>
        <button
          aria-label="Link"
          title="Link (Ctrl+click a link to open it)"
          className={cn(
            "inline-flex h-7 w-7 items-center justify-center rounded-lg transition-colors",
            active ? "bg-accent-soft text-accent" : "text-muted hover:bg-surface-2 hover:text-fg",
          )}
        >
          <Link2 size={15} />
        </button>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content sideOffset={8} className="z-[70] w-80 rounded-xl border border-line bg-elevated p-3 shadow-float">
          <form
            className="flex gap-2"
            onSubmit={(e) => {
              e.preventDefault();
              const chain = editor.chain().focus().extendMarkRange("link");
              if (url.trim()) chain.setLink({ href: url.trim() }).run();
              else chain.unsetLink().run();
              setOpen(false);
            }}
          >
            <Input autoFocus value={url} onChange={(e) => setUrl(e.target.value)} placeholder="https://…" className="h-9" />
            <Button type="submit" variant="primary" size="sm" className="h-9">
              Save
            </Button>
          </form>
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}
