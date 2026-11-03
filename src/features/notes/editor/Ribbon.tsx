// The OneNote-style ribbon over the page: tabs of grouped commands. It sits next to the "/" menu
// and the selection toolbar; all three call the same editor commands.

import { useEffect, useState } from "react";
import { useEditorState, type Editor } from "@tiptap/react";
import {
  AlignCenter,
  AlignLeft,
  AlignRight,
  BetweenHorizontalEnd,
  BetweenHorizontalStart,
  BetweenVerticalEnd,
  BetweenVerticalStart,
  Bold,
  CalendarDays,
  ChevronDown,
  ChevronUp,
  Code,
  Columns3,
  Eraser,
  ImagePlus,
  Italic,
  ListIndentDecrease,
  ListIndentIncrease,
  Minus,
  PanelTop,
  PilcrowLeft,
  PilcrowRight,
  Plus,
  Radical,
  Redo2,
  Rows3,
  Strikethrough,
  Table2,
  TableCellsMerge,
  Trash2,
  Underline,
  Undo2,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { ColorPop, FONT_FAMILIES, FONT_SIZES, LinkPop, Pop, Row, Sep } from "./BubbleToolbar";
import { pickImages } from "./extensions";
import { applyTurnInto, BLOCKS, currentBlock, type BlockKey } from "./turnInto";
import { storedDirection } from "./direction";
import { getKey, setKey } from "@/lib/storage";

type Tab = "home" | "insert" | "table";

const COLLAPSED_KEY = "ribbonCollapsed";
const s = 16;
const blockIcon = (key: BlockKey) => BLOCKS.find((b) => b.key === key)!.icon;
// The ribbon's pop-ups are not inside a floating bar, so they don't need to keep it open.
const noTrack = () => {};

export default function Ribbon({ editor }: { editor: Editor }) {
  const [tab, setTab] = useState<Tab>("home");
  const [collapsed, setCollapsed] = useState(() => {
    try {
      return getKey(COLLAPSED_KEY) === "1";
    } catch {
      return false;
    }
  });
  const inTable = useEditorState({ editor, selector: ({ editor: e }) => e.isActive("table") });
  // The Table tab only shows while the cursor is in a table, like OneNote's contextual tab.
  const shown: Tab = tab === "table" && !inTable ? "home" : tab;
  // Leaving the table goes back to Home, so the next table doesn't jump straight to its tab.
  useEffect(() => {
    if (!inTable) setTab((t) => (t === "table" ? "home" : t));
  }, [inTable]);

  const setCollapsedSaved = (v: boolean) => {
    setCollapsed(v);
    try {
      setKey(COLLAPSED_KEY, v ? "1" : "0");
    } catch {
      /* the ribbon just opens expanded next time */
    }
  };
  const pick = (t: Tab) => {
    setTab(t);
    if (collapsed) setCollapsedSaved(false);
  };

  return (
    <div className="shrink-0 border-b border-line bg-surface/60" role="region" aria-label="Ribbon">
      <div className="flex h-8 items-end gap-0.5 px-3">
        <div role="tablist" aria-label="Ribbon tabs" className="flex items-end gap-0.5">
        <TabButton active={shown === "home"} onClick={() => pick("home")}>
          Home
        </TabButton>
        <TabButton active={shown === "insert"} onClick={() => pick("insert")}>
          Insert
        </TabButton>
        {inTable && (
          <TabButton active={shown === "table"} onClick={() => pick("table")}>
            Table
          </TabButton>
        )}
        </div>
        <div className="flex-1" />
        <button
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => setCollapsedSaved(!collapsed)}
          title={collapsed ? "Show the ribbon" : "Hide the ribbon (the / menu still works)"}
          aria-label={collapsed ? "Show the ribbon" : "Hide the ribbon"}
          aria-expanded={!collapsed}
          className="mb-1 grid h-6 w-6 place-items-center rounded-md text-faint hover:bg-surface-2 hover:text-fg"
        >
          {collapsed ? <ChevronDown size={14} /> : <ChevronUp size={14} />}
        </button>
      </div>
      {!collapsed && (
        <div role="tabpanel" aria-label={shown} className="flex min-h-[4.25rem] flex-wrap items-stretch gap-y-1 overflow-x-auto px-2 pb-1">
          {shown === "home" && <HomeTab editor={editor} />}
          {shown === "insert" && <InsertTab editor={editor} />}
          {shown === "table" && <TableTab editor={editor} />}
        </div>
      )}
    </div>
  );
}

/* ---------- Home ---------- */

function HomeTab({ editor }: { editor: Editor }) {
  const st = useEditorState({
    editor,
    selector: ({ editor: e }) => ({
      bold: e.isActive("bold"),
      italic: e.isActive("italic"),
      underline: e.isActive("underline"),
      strike: e.isActive("strike"),
      code: e.isActive("code"),
      link: e.isActive("link"),
      block: currentBlock(e),
      listItem: innermostListItem(e),
      canUndo: e.can().undo(),
      canRedo: e.can().redo(),
      fontSize: (e.getAttributes("textStyle").fontSize as string | undefined) ?? "",
      fontFamily: (e.getAttributes("textStyle").fontFamily as string | undefined) ?? "",
      dir: storedDirection(e.state),
      align: e.isActive({ textAlign: "center" }) ? "center" : e.isActive({ textAlign: "right" }) ? "right" : "left",
    }),
  });
  const c = () => editor.chain().focus();
  // A choice made in a drop-down puts the cursor back in the page, so typing carries on there.
  const refocus = () => editor.commands.focus();
  const sizeNow = st.fontSize ? parseInt(st.fontSize, 10) : null;
  const familyNow = FONT_FAMILIES.find((f) => f.value === st.fontFamily)?.label ?? (st.fontFamily ? "Custom" : "Default");
  const step = (delta: number) => c().setFontSize(`${Math.min(96, Math.max(8, (sizeNow ?? 16) + delta))}px`).run();
  const turn = (key: BlockKey) => void applyTurnInto(editor, key);

  return (
    <>
      <Group label="Undo">
        <Btn label="Undo (Ctrl+Z)" disabled={!st.canUndo} onClick={() => c().undo().run()}>
          <Undo2 size={s} />
        </Btn>
        <Btn label="Redo (Ctrl+Y)" disabled={!st.canRedo} onClick={() => c().redo().run()}>
          <Redo2 size={s} />
        </Btn>
      </Group>

      <Group label="Basic text">
        <div className="flex flex-col gap-0.5">
          <div className="flex items-center gap-0.5">
            <Pop
              track={noTrack}
              width={190}
              onClose={refocus}
              trigger={
                <button
                  onMouseDown={(e) => e.preventDefault()}
                  title="Font"
                  aria-label="Font"
                  className="inline-flex h-7 w-28 items-center justify-between gap-1 rounded-md border border-line bg-bg px-2 text-xs hover:bg-surface-2"
                  style={{ fontFamily: st.fontFamily || undefined }}
                >
                  <span className="truncate">{familyNow}</span>
                  <ChevronDown size={12} className="shrink-0 text-faint" />
                </button>
              }
            >
              {FONT_FAMILIES.map((f) => (
                <Row key={f.label} active={f.value === st.fontFamily} onClick={() => (f.value ? c().setFontFamily(f.value).run() : c().unsetFontFamily().run())}>
                  <span style={{ fontFamily: f.value || undefined }}>{f.label}</span>
                </Row>
              ))}
            </Pop>
            <Pop
              track={noTrack}
              width={96}
              onClose={refocus}
              trigger={
                <button
                  onMouseDown={(e) => e.preventDefault()}
                  title="Text size"
                  aria-label="Text size"
                  className="inline-flex h-7 w-14 items-center justify-between rounded-md border border-line bg-bg px-1.5 text-xs tabular-nums hover:bg-surface-2"
                >
                  {sizeNow ?? "Auto"}
                  <ChevronDown size={12} className="shrink-0 text-faint" />
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
            <Btn small label="Smaller text" onClick={() => step(-2)}>
              <Minus size={14} />
            </Btn>
            <Btn small label="Larger text" onClick={() => step(2)}>
              <Plus size={14} />
            </Btn>
          </div>
          <div className="flex items-center gap-0.5">
            <Btn small label="Bold (Ctrl+B)" active={st.bold} onClick={() => c().toggleBold().run()}>
              <Bold size={15} />
            </Btn>
            <Btn small label="Italic (Ctrl+I)" active={st.italic} onClick={() => c().toggleItalic().run()}>
              <Italic size={15} />
            </Btn>
            <Btn small label="Underline (Ctrl+U)" active={st.underline} onClick={() => c().toggleUnderline().run()}>
              <Underline size={15} />
            </Btn>
            <Btn small label="Strikethrough" active={st.strike} onClick={() => c().toggleStrike().run()}>
              <Strikethrough size={15} />
            </Btn>
            <Btn small label="Code" active={st.code} onClick={() => c().toggleCode().run()}>
              <Code size={15} />
            </Btn>
            <ColorPop editor={editor} track={noTrack} onClose={refocus} />
            <LinkPop editor={editor} active={st.link} track={noTrack} onClose={refocus} />
            <Btn small label="Remove formatting" onClick={() => c().unsetAllMarks().run()}>
              <Eraser size={15} />
            </Btn>
          </div>
        </div>
      </Group>

      <Group label="Paragraph">
        <div className="flex flex-col gap-0.5">
          <div className="flex items-center gap-0.5">
            <Btn small label="Bullet list" active={st.block === "bullet"} onClick={() => turn(st.block === "bullet" ? "p" : "bullet")}>
              {blockIcon("bullet")}
            </Btn>
            <Btn small label="Numbered list" active={st.block === "ordered"} onClick={() => turn(st.block === "ordered" ? "p" : "ordered")}>
              {blockIcon("ordered")}
            </Btn>
            <Btn small label="Checklist" active={st.block === "task"} onClick={() => turn(st.block === "task" ? "p" : "task")}>
              {blockIcon("task")}
            </Btn>
          </div>
          <div className="flex items-center gap-0.5">
            <Btn small label="Decrease indent (Shift+Tab)" disabled={!st.listItem} onClick={() => st.listItem && c().liftListItem(st.listItem).run()}>
              <ListIndentDecrease size={15} />
            </Btn>
            <Btn small label="Increase indent (Tab)" disabled={!st.listItem} onClick={() => st.listItem && c().sinkListItem(st.listItem).run()}>
              <ListIndentIncrease size={15} />
            </Btn>
            <Btn small label="Align left" active={st.align === "left"} onClick={() => c().setTextAlign("left").run()}>
              <AlignLeft size={15} />
            </Btn>
            <Btn small label="Center" active={st.align === "center"} onClick={() => c().setTextAlign("center").run()}>
              <AlignCenter size={15} />
            </Btn>
            <Btn small label="Align right" active={st.align === "right"} onClick={() => c().setTextAlign("right").run()}>
              <AlignRight size={15} />
            </Btn>
          </div>
        </div>
      </Group>

      <Group label="Direction">
        <Btn label="Right-to-left (Arabic, Hebrew). Click again for automatic." active={st.dir === "rtl"} onClick={() => c().setBlockDirection(st.dir === "rtl" ? null : "rtl").run()}>
          <PilcrowLeft size={s} />
        </Btn>
        <Btn label="Left-to-right (English). Click again for automatic." active={st.dir === "ltr"} onClick={() => c().setBlockDirection(st.dir === "ltr" ? null : "ltr").run()}>
          <PilcrowRight size={s} />
        </Btn>
      </Group>

      <Group label="Styles" last>
        {(["p", "h1", "h2", "h3", "quote", "code"] as const).map((key) => (
          <StyleTile key={key} active={st.block === key} label={BLOCKS.find((b) => b.key === key)!.label} onClick={() => turn(key)}>
            {key === "p" && <span className="text-sm">Normal</span>}
            {key === "h1" && <span className="text-lg font-bold leading-none">Aa</span>}
            {key === "h2" && <span className="text-base font-bold leading-none">Aa</span>}
            {key === "h3" && <span className="text-sm font-semibold leading-none">Aa</span>}
            {key === "quote" && <span className="border-l-2 border-current pl-1 text-sm italic leading-none">Aa</span>}
            {key === "code" && <span className="font-mono text-xs leading-none">{"{ }"}</span>}
          </StyleTile>
        ))}
      </Group>
    </>
  );
}

/* ---------- Insert ---------- */

function InsertTab({ editor }: { editor: Editor }) {
  const c = () => editor.chain().focus();
  const turn = (key: BlockKey) => () => void applyTurnInto(editor, key);
  const today = () => new Date().toLocaleDateString(undefined, { dateStyle: "long" });
  return (
    <>
      <Group label="Tables">
        <Big label="Table" title="Insert a 3 by 3 table" onClick={() => c().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()}>
          <Table2 size={20} />
        </Big>
      </Group>
      <Group label="Pictures">
        <Big label="Picture" title="Add an image from your PC" onClick={() => pickImages(editor)}>
          <ImagePlus size={20} />
        </Big>
      </Group>
      <Group label="Blocks">
        <Big label="Highlight box" onClick={turn("callout")}>
          {blockIcon("callout")}
        </Big>
        <Big label="Collapsible" title="Collapsible section" onClick={turn("toggle")}>
          {blockIcon("toggle")}
        </Big>
        <Big label="Code block" onClick={turn("code")}>
          {blockIcon("code")}
        </Big>
        <Big label="Divider" title="A thin line across the page" onClick={() => c().setHorizontalRule().run()}>
          <Minus size={20} />
        </Big>
        <Big label="Sub-page" title="A new page inside this one" onClick={turn("page")}>
          {blockIcon("page")}
        </Big>
      </Group>
      <Group label="Symbols">
        <Big label="Equation" title="Maths written in TeX, on its own line" onClick={turn("equation")}>
          {blockIcon("equation")}
        </Big>
        <Big label="Inline" title="Maths inside a line (or type $x^2$)" onClick={() => c().insertMathInline().run()}>
          <Radical size={20} />
        </Big>
      </Group>
      <Group label="Time stamp" last>
        <Big label="Date" title={today()} onClick={() => c().insertContent(today()).run()}>
          <CalendarDays size={20} />
        </Big>
      </Group>
    </>
  );
}

/* ---------- Table (only while the cursor is in a table) ---------- */

function TableTab({ editor }: { editor: Editor }) {
  const st = useEditorState({
    editor,
    selector: ({ editor: e }) => ({
      canMerge: e.can().mergeOrSplit(),
      header: hasHeaderRow(e),
    }),
  });
  const c = () => editor.chain().focus();
  return (
    <>
      <Group label="Rows">
        <Big label="Above" title="Insert a row above" onClick={() => c().addRowBefore().run()}>
          <BetweenHorizontalStart size={20} />
        </Big>
        <Big label="Below" title="Insert a row below" onClick={() => c().addRowAfter().run()}>
          <BetweenHorizontalEnd size={20} />
        </Big>
        <Big label="Delete" title="Delete this row" onClick={() => c().deleteRow().run()}>
          <Rows3 size={20} />
        </Big>
      </Group>
      <Group label="Columns">
        <Big label="Left" title="Insert a column to the left" onClick={() => c().addColumnBefore().run()}>
          <BetweenVerticalStart size={20} />
        </Big>
        <Big label="Right" title="Insert a column to the right" onClick={() => c().addColumnAfter().run()}>
          <BetweenVerticalEnd size={20} />
        </Big>
        <Big label="Delete" title="Delete this column" onClick={() => c().deleteColumn().run()}>
          <Columns3 size={20} />
        </Big>
      </Group>
      <Group label="Cells">
        <Big label="Merge / split" title="Merge the selected cells, or split a merged cell" disabled={!st.canMerge} onClick={() => c().mergeOrSplit().run()}>
          <TableCellsMerge size={20} />
        </Big>
        <Big label="Header row" title="Turn the top row's header style on or off" active={st.header} onClick={() => c().toggleHeaderRow().run()}>
          <PanelTop size={20} />
        </Big>
      </Group>
      <Group label="Table" last>
        <Big label="Delete table" danger onClick={() => c().deleteTable().run()}>
          <Trash2 size={20} />
        </Big>
      </Group>
    </>
  );
}

/* ---------- helpers ---------- */

/** The list item nearest the cursor (a bullet inside a checklist item is a "listItem"). */
function innermostListItem(editor: Editor): "listItem" | "taskItem" | null {
  const { $from } = editor.state.selection;
  for (let d = $from.depth; d >= 1; d--) {
    const name = $from.node(d).type.name;
    if (name === "listItem" || name === "taskItem") return name;
  }
  return null;
}

/** Whether the table the cursor is in starts with a header row, wherever the cursor is in it. */
function hasHeaderRow(editor: Editor): boolean {
  const { $from } = editor.state.selection;
  for (let d = $from.depth; d >= 1; d--) {
    const node = $from.node(d);
    if (node.type.name === "table") {
      const row = node.firstChild;
      return !!row && row.childCount > 0 && row.firstChild?.type.name === "tableHeader" && row.lastChild?.type.name === "tableHeader";
    }
  }
  return false;
}

/* ---------- small pieces ---------- */

function TabButton({ active, onClick, children }: { active: boolean; onClick: () => void; children: React.ReactNode }) {
  return (
    <button
      role="tab"
      aria-selected={active}
      onMouseDown={(e) => e.preventDefault()}
      onClick={onClick}
      className={cn(
        "h-7 rounded-t-md border-b-2 px-3 text-xs font-medium transition-colors",
        active ? "border-accent text-fg" : "border-transparent text-muted hover:bg-surface-2 hover:text-fg",
      )}
    >
      {children}
    </button>
  );
}

function Group({ label, last, children }: { label: string; last?: boolean; children: React.ReactNode }) {
  return (
    <>
      <div className="flex shrink-0 flex-col px-1.5" role="group" aria-label={label}>
        <div className="flex flex-1 items-center gap-0.5">{children}</div>
        <div className="pt-0.5 text-center text-[0.65rem] text-faint">{label}</div>
      </div>
      {!last && (
        <div className="flex items-center">
          <Sep />
        </div>
      )}
    </>
  );
}

function Btn({
  label,
  active,
  disabled,
  small,
  onClick,
  children,
}: {
  label: string;
  active?: boolean;
  disabled?: boolean;
  small?: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      title={label}
      aria-label={label}
      aria-pressed={active}
      disabled={disabled}
      onMouseDown={(e) => e.preventDefault()}
      onClick={onClick}
      className={cn(
        "grid place-items-center rounded-md transition-colors disabled:pointer-events-none disabled:opacity-40",
        small ? "h-7 w-7" : "h-9 w-9",
        active ? "bg-surface-2 text-fg" : "text-muted hover:bg-surface-2 hover:text-fg",
      )}
    >
      {children}
    </button>
  );
}

/** A tall button with an icon over a caption, like OneNote's Insert tab. */
function Big({
  label,
  title,
  active,
  disabled,
  danger,
  onClick,
  children,
}: {
  label: string;
  title?: string;
  active?: boolean;
  disabled?: boolean;
  danger?: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      title={title ?? label}
      aria-label={title ?? label}
      aria-pressed={active}
      disabled={disabled}
      onMouseDown={(e) => e.preventDefault()}
      onClick={onClick}
      className={cn(
        "flex h-[3.25rem] min-w-[3.25rem] flex-col items-center justify-center gap-1 rounded-md px-1.5 text-[0.7rem] transition-colors disabled:pointer-events-none disabled:opacity-40",
        active ? "bg-surface-2 text-fg" : danger ? "text-muted hover:bg-danger/10 hover:text-danger" : "text-muted hover:bg-surface-2 hover:text-fg",
      )}
    >
      <span className="[&>svg]:h-5 [&>svg]:w-5">{children}</span>
      <span className="whitespace-nowrap leading-none">{label}</span>
    </button>
  );
}

function StyleTile({ label, active, onClick, children }: { label: string; active: boolean; onClick: () => void; children: React.ReactNode }) {
  return (
    <button
      title={label}
      aria-label={label}
      aria-pressed={active}
      onMouseDown={(e) => e.preventDefault()}
      onClick={onClick}
      className={cn(
        "grid h-[3.25rem] w-14 place-items-center rounded-md border text-fg transition-colors",
        active ? "border-fg/40 bg-surface-2" : "border-line bg-bg hover:bg-surface-2",
      )}
    >
      {children}
    </button>
  );
}
