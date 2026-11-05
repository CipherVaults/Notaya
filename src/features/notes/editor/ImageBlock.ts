// Pictures that can be moved, wrapped by text, sized and placed anywhere on the page, and given a caption.
// Only images stored in the vault (clatt:// URLs) can be shown; the Content Security
// Policy blocks everything else.
//
// How a picture sits on the page (attributes of the "image" node; pages saved before V1.5.1 have none of the
// new ones and look exactly as before):
//   in the text     align left | center | right, width (% of the text column, null = natural size, at most 100%)
//   text wraps it   float: true, with align left or right (the text runs around it)
//   free            free: true, x (left edge, % of the page width), y (top edge, px from the top of the page),
//                   layer "front" | "back" (in front of the text or behind it). It takes no room in the text.
//   shape           ratio "" (as it is) | "1:1" | "4:3" | "3:2" | "16:9" | "9:16" (cropped to fill, object-fit: cover)
// A free picture stays in the document at the block nearest to its height, so reading order (and every export)
// follows where it was put.

import Image from "@tiptap/extension-image";
import { NodeSelection, Plugin } from "@tiptap/pm/state";
import { dropPoint } from "@tiptap/pm/transform";
import type { EditorView } from "@tiptap/pm/view";
import "./image.css";

export type ImageAlign = "left" | "center" | "right";

const MIN_PCT = 5;
const MAX_PCT = 100;
/** The size a floating or free picture gets when it has none yet. */
const DEFAULT_BOXED_PCT = 40;
const RATIOS: [string, string][] = [
  ["", "Original"],
  ["1:1", "1:1 square"],
  ["4:3", "4:3"],
  ["3:2", "3:2"],
  ["16:9", "16:9 wide"],
  ["9:16", "9:16 tall"],
];
const RATIO_VALUES = new Set(RATIOS.map(([v]) => v));

const clamp = (n: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, n));
const round1 = (n: number) => Math.round(n * 10) / 10;
const ratioCss = (r: string) => r.replace(":", " / ");

// ---- the page body as the place free pictures sit in ----

/** Makes the page body tall enough for its lowest free picture, so nothing is clipped or unreachable. */
const growQueued = new WeakSet<HTMLElement>();
export function growPage(ed: HTMLElement | null) {
  if (!ed || growQueued.has(ed)) return;
  growQueued.add(ed);
  requestAnimationFrame(() => {
    growQueued.delete(ed);
    if (!ed.isConnected) return;
    ed.style.minHeight = "";
    const natural = ed.offsetHeight;
    ed.dataset.naturalH = String(natural);
    const top = ed.getBoundingClientRect().top;
    let need = 0;
    ed.querySelectorAll<HTMLElement>('.image-block[data-free="true"]').forEach((el) => {
      need = Math.max(need, el.getBoundingClientRect().bottom - top);
    });
    if (need + 24 > natural) ed.style.minHeight = `${Math.ceil(need + 24)}px`;
  });
}

/** Keeps a free picture inside the page area: its left edge in 0..(100 - its width) %, its top in 0..page height. */
function clampFree(view: EditorView, x: number, y: number, widthPct: number) {
  const natural = Number(view.dom.dataset.naturalH) || 600;
  return { x: round1(clamp(x, 0, Math.max(0, MAX_PCT - widthPct))), y: Math.round(clamp(y, 0, Math.max(400, natural))) };
}

/**
 * The place in the document (a boundary between top-level blocks) that matches a height on the page: the new
 * neighbour of a free picture put at `y`, so that reading order follows the page.
 */
function flowSlot(view: EditorView, pos: number, y: number): number {
  const top = view.dom.getBoundingClientRect().top;
  let target = view.state.doc.content.size;
  let found = false;
  view.state.doc.forEach((_child, offset) => {
    if (found || offset === pos) return;
    const el = view.nodeDOM(offset) as HTMLElement | null;
    if (!el || !el.getBoundingClientRect || el.dataset?.free === "true") return;
    const r = el.getBoundingClientRect();
    if (r.top - top + r.height / 2 > y) {
      target = offset;
      found = true;
    }
  });
  return target;
}

/** Changes a picture's attributes in one undo step; a free picture also moves to the block nearest its height. */
function applyAttrs(view: EditorView, pos: number | undefined, patch: Record<string, unknown>) {
  if (pos === undefined) return;
  const node = view.state.doc.nodeAt(pos);
  if (!node || node.type.name !== "image") return;
  const attrs = { ...node.attrs, ...patch };
  let tr = view.state.tr;
  let at = pos;
  const topLevel = view.state.doc.resolve(pos).depth === 0;
  if (attrs.free && topLevel && ("y" in patch || "free" in patch)) {
    const target = flowSlot(view, pos, Number(attrs.y) || 0);
    if (target !== pos && target !== pos + node.nodeSize) {
      tr = tr.delete(pos, pos + node.nodeSize);
      at = tr.mapping.map(target);
      tr = tr.insert(at, node.type.create(attrs, null, node.marks));
    } else tr = tr.setNodeMarkup(pos, undefined, attrs);
  } else tr = tr.setNodeMarkup(pos, undefined, attrs);
  view.dispatch(tr.setSelection(NodeSelection.create(tr.doc, at)));
}

// ---- the little toolbar of the selected picture ----

interface PopContext {
  attrs: () => Record<string, any>;
  apply: (patch: Record<string, unknown>) => void;
  /** The picture's current size and place, as a percentage of the page width and px from its top. */
  measure: () => { widthPct: number; x: number; y: number };
  canFree: () => boolean;
}

const mk = <K extends keyof HTMLElementTagNameMap>(tag: K, cls = "", text = "") => {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text) e.textContent = text;
  return e;
};

function buildPopover(ctx: PopContext, anchor: () => HTMLElement) {
  const root = mk("div", "image-pop");
  root.setAttribute("role", "toolbar");
  root.setAttribute("aria-label", "Picture settings");
  root.addEventListener("mousedown", (e) => {
    // Buttons keep the editor's focus and selection; the number box and the list take theirs.
    if (!(e.target as HTMLElement).closest("input, select")) e.preventDefault();
  });

  const button = (label: string, title: string, onClick: () => void) => {
    const b = mk("button", "", label);
    b.type = "button";
    b.title = title;
    b.addEventListener("click", onClick);
    return b;
  };
  const row = (label: string) => {
    const r = mk("div", "row");
    r.append(mk("span", "lbl", label));
    return r;
  };

  // Size
  const sizeRow = row("Width");
  const num = mk("input");
  num.type = "number";
  num.min = String(MIN_PCT);
  num.max = String(MAX_PCT);
  num.step = "1";
  num.setAttribute("aria-label", "Width in percent of the page");
  const widthPatch = (v: number) => {
    const a = ctx.attrs();
    const patch: Record<string, unknown> = { width: round1(clamp(v, MIN_PCT, MAX_PCT)) };
    if (a.free) patch.x = round1(clamp(Number(a.x) || 0, 0, Math.max(0, MAX_PCT - v)));
    return patch;
  };
  // True while the number box itself is the cause of a change, so the box is not rewritten under the typing hand.
  let typing = false;
  num.addEventListener("input", () => {
    const v = parseFloat(num.value);
    if (!Number.isFinite(v) || v < MIN_PCT || v > MAX_PCT) return;
    typing = true;
    try {
      ctx.apply(widthPatch(v));
    } finally {
      typing = false;
    }
  });
  num.addEventListener("blur", () => refresh());
  num.addEventListener("keydown", (e) => {
    e.stopPropagation();
    if (e.key === "Enter" || e.key === "Escape") num.blur();
  });
  sizeRow.append(num, mk("span", "", "%"));
  const presets = [25, 50, 75, 100].map((p) => {
    const b = button(`${p}`, `${p}% of the page width`, () => ctx.apply(widthPatch(p)));
    return [p, b] as const;
  });
  presets.forEach(([, b]) => sizeRow.append(b));
  sizeRow.append(button("Reset size", "Back to the natural size and shape", () => ctx.apply({ width: null, ratio: "" })));

  // Shape
  const shapeRow = row("Shape");
  const ratio = mk("select");
  ratio.setAttribute("aria-label", "Aspect ratio");
  for (const [v, label] of RATIOS) {
    const o = mk("option", "", label);
    o.value = v;
    ratio.append(o);
  }
  ratio.addEventListener("change", () => {
    const patch: Record<string, unknown> = { ratio: ratio.value };
    if (ratio.value && ctx.attrs().width == null) patch.width = clamp(Math.round(ctx.measure().widthPct), MIN_PCT, MAX_PCT);
    ctx.apply(patch);
  });
  shapeRow.append(ratio);

  // Place
  const placeRow = row("Place");
  const inText = (align: ImageAlign) => () => ctx.apply({ free: false, float: false, align });
  const wrap = (align: ImageAlign) => () => {
    const w = ctx.attrs().width;
    ctx.apply({ free: false, float: true, align, ...(w == null || w > 60 ? { width: DEFAULT_BOXED_PCT } : {}) });
  };
  const bLeft = button("Left", "In the text, at the left", inText("left"));
  const bCenter = button("Center", "In the text, centered", inText("center"));
  const bRight = button("Right", "In the text, at the right", inText("right"));
  const bWrapL = button("Wrap left", "At the left, text wraps around it", wrap("left"));
  const bWrapR = button("Wrap right", "At the right, text wraps around it", wrap("right"));
  const bFull = button("Full", "The whole width of the text", () => ctx.apply({ free: false, float: false, align: "center", width: 100 }));
  const bFree = button("Free", "Place it anywhere on the page: drag it, or use the arrow keys", () => {
    const m = ctx.measure();
    const a = ctx.attrs();
    ctx.apply({ free: true, float: false, layer: a.layer === "back" ? "back" : "front", width: clamp(Math.round(m.widthPct), MIN_PCT, MAX_PCT), x: m.x, y: m.y });
  });
  placeRow.append(bLeft, bCenter, bRight, bFull, bWrapL, bWrapR, bFree);

  // Free: layer and way back
  const freeRow = row("Layer");
  const bFront = button("In front of text", "Above the text (the text does not wrap around it)", () => ctx.apply({ layer: "front" }));
  const bBack = button("Behind text", "Behind the text: typing and clicks reach the text", () => ctx.apply({ layer: "back" }));
  const bTextBack = button("Back to text", "Put it back into the text, at the nearest block", () => ctx.apply({ free: false }));
  freeRow.append(bFront, bBack, bTextBack);

  root.append(sizeRow, shapeRow, placeRow, freeRow);

  const pressed = (b: HTMLButtonElement, on: boolean) => b.setAttribute("aria-pressed", String(on));
  function refresh() {
    const a = ctx.attrs();
    const free = !!a.free;
    const float = !free && !!a.float;
    if (!(typing && document.activeElement === num)) {
      const pct = a.width ?? (free || float ? DEFAULT_BOXED_PCT : round1(ctx.measure().widthPct));
      num.value = String(Math.round(pct * 10) / 10);
    }
    for (const [p, b] of presets) pressed(b, a.width === p);
    ratio.value = RATIO_VALUES.has(a.ratio ?? "") ? (a.ratio ?? "") : "";
    pressed(bLeft, !free && !float && a.align === "left");
    pressed(bCenter, !free && !float && (a.align ?? "center") === "center");
    pressed(bRight, !free && !float && a.align === "right");
    pressed(bFull, !free && !float && a.width === 100);
    pressed(bWrapL, float && a.align !== "right");
    pressed(bWrapR, float && a.align === "right");
    pressed(bFree, free);
    bFree.disabled = !free && !ctx.canFree();
    if (bFree.disabled) bFree.title = "Take the picture out of the block it is in (a column, a list, a table) first";
    freeRow.style.display = free ? "" : "none";
    pressed(bFront, a.layer !== "back");
    pressed(bBack, a.layer === "back");
    place();
  }

  /** Above the picture, or below it when there is no room above; kept on screen. */
  function place() {
    const r = anchor().getBoundingClientRect();
    const off = r.bottom < 0 || r.top > window.innerHeight;
    root.style.visibility = off ? "hidden" : "visible";
    const pw = root.offsetWidth;
    const ph = root.offsetHeight;
    let top = r.top - ph - 8;
    if (top < 8) top = r.bottom + 8 + ph < window.innerHeight ? r.bottom + 8 : 8;
    root.style.top = `${Math.round(top)}px`;
    root.style.left = `${Math.round(clamp(r.left + r.width / 2 - pw / 2, 8, Math.max(8, window.innerWidth - pw - 8)))}px`;
  }

  const onScroll = () => place();
  const onOutside = (e: MouseEvent) => {
    const t = e.target as Node;
    if (!root.contains(t) && !anchor().closest(".image-block")?.contains(t)) hide();
  };
  let shown = false;
  function show() {
    if (shown) return;
    shown = true;
    document.body.append(root);
    window.addEventListener("scroll", onScroll, true);
    window.addEventListener("resize", onScroll);
    document.addEventListener("mousedown", onOutside, true);
    refresh();
  }
  function hide() {
    if (!shown) return;
    shown = false;
    root.remove();
    window.removeEventListener("scroll", onScroll, true);
    window.removeEventListener("resize", onScroll);
    document.removeEventListener("mousedown", onOutside, true);
  }
  return { show, hide, refresh: () => shown && refresh(), isShown: () => shown };
}

// ---- the extension ----

const attrNumber = (el: Element, name: string) => {
  const n = Number(el.getAttribute(name));
  return el.hasAttribute(name) && Number.isFinite(n) ? n : null;
};

export const ImageBlock = Image.extend({
  draggable: true,

  addAttributes() {
    return {
      ...this.parent?.(),
      /** Width as a percentage of the page column (null = natural size, at most 100%). */
      width: {
        default: null,
        parseHTML: (el) => {
          const w = Number(el.getAttribute("data-width"));
          return Number.isFinite(w) && w > 0 ? w : null;
        },
        renderHTML: (attrs) => (attrs.width ? { "data-width": String(attrs.width) } : {}),
      },
      align: {
        default: "center",
        parseHTML: (el) => el.getAttribute("data-align") ?? "center",
        renderHTML: (attrs) => ({ "data-align": attrs.align }),
      },
      caption: {
        default: "",
        parseHTML: (el) => el.getAttribute("data-caption") ?? "",
        renderHTML: (attrs) => (attrs.caption ? { "data-caption": attrs.caption } : {}),
      },
      /** "" (as it is) or "w:h". The picture is cropped to fill that shape. */
      ratio: {
        default: "",
        parseHTML: (el) => {
          const r = el.getAttribute("data-ratio") ?? "";
          return RATIO_VALUES.has(r) ? r : "";
        },
        renderHTML: (attrs) => (attrs.ratio ? { "data-ratio": attrs.ratio } : {}),
      },
      /** The text wraps around the picture (it sits at the left or right, as `align` says). */
      float: {
        default: false,
        parseHTML: (el) => el.getAttribute("data-float") === "true",
        renderHTML: (attrs) => (attrs.float ? { "data-float": "true" } : {}),
      },
      /** Placed anywhere on the page: x is the left edge in % of the page width, y the top edge in px. */
      free: {
        default: false,
        parseHTML: (el) => el.getAttribute("data-free") === "true",
        renderHTML: (attrs) => (attrs.free ? { "data-free": "true" } : {}),
      },
      x: {
        default: null,
        parseHTML: (el) => attrNumber(el, "data-x"),
        renderHTML: (attrs) => (attrs.x != null ? { "data-x": String(attrs.x) } : {}),
      },
      y: {
        default: null,
        parseHTML: (el) => attrNumber(el, "data-y"),
        renderHTML: (attrs) => (attrs.y != null ? { "data-y": String(attrs.y) } : {}),
      },
      /** "front" (above the text) or "back" (behind it). */
      layer: {
        default: "front",
        parseHTML: (el) => (el.getAttribute("data-layer") === "back" ? "back" : "front"),
        renderHTML: (attrs) => (attrs.layer === "back" ? { "data-layer": "back" } : {}),
      },
    };
  },

  addKeyboardShortcuts() {
    // A selected free picture moves with the arrow keys (Shift = bigger steps); Escape lets go of it.
    const nudge = (dx: number, dy: number) => () => {
      const view = this.editor.view;
      const sel = view.state.selection;
      if (!(sel instanceof NodeSelection) || sel.node.type.name !== "image" || !sel.node.attrs.free || !this.editor.isEditable) return false;
      const a = sel.node.attrs;
      const w = a.width ?? DEFAULT_BOXED_PCT;
      const p = clampFree(view, (Number(a.x) || 0) + dx, (Number(a.y) || 0) + dy, w);
      applyAttrs(view, sel.from, { x: p.x, y: p.y });
      return true;
    };
    const step = (big: boolean) => ({ x: big ? 5 : 0.5, y: big ? 48 : 6 });
    const keys: Record<string, () => boolean> = {
      Escape: () => {
        const sel = this.editor.state.selection;
        if (!(sel instanceof NodeSelection) || sel.node.type.name !== "image" || !sel.node.attrs.free) return false;
        return this.editor.commands.setTextSelection(sel.to);
      },
    };
    for (const big of [false, true]) {
      const s = step(big);
      const pre = big ? "Shift-" : "";
      keys[`${pre}ArrowLeft`] = nudge(-s.x, 0);
      keys[`${pre}ArrowRight`] = nudge(s.x, 0);
      keys[`${pre}ArrowUp`] = nudge(0, -s.y);
      keys[`${pre}ArrowDown`] = nudge(0, s.y);
    }
    return keys;
  },

  addProseMirrorPlugins() {
    return [
      new Plugin({
        props: {
          // Dropping a picture that was dragged inside the page: to the left or right part of the page it floats there
          // (text wraps around it), to the middle it sits in the text. (Free placement is chosen in the picture's toolbar.)
          handleDrop: (view, event, slice, moved) => {
            if (!moved || slice.content.childCount !== 1 || slice.openStart || slice.openEnd) return false;
            const node = slice.content.firstChild;
            const sel = view.state.selection;
            if (!node || node.type.name !== "image" || !(sel instanceof NodeSelection) || sel.node.type.name !== "image") return false;
            const over = view.posAtCoords({ left: event.clientX, top: event.clientY });
            if (!over) return false;
            const box = view.dom.getBoundingClientRect();
            const rel = (event.clientX - box.left) / Math.max(1, box.width);
            const insert = dropPoint(view.state.doc, over.pos, slice) ?? over.pos;
            const attrs: Record<string, any> = { ...node.attrs };
            if (rel < 0.3 || rel > 0.7) {
              Object.assign(attrs, { free: false, float: true, align: rel < 0.3 ? "left" : "right", width: attrs.width == null || attrs.width > 60 ? DEFAULT_BOXED_PCT : attrs.width });
            } else if (attrs.float || attrs.free) {
              Object.assign(attrs, { free: false, float: false, align: "center" });
            }
            event.preventDefault();
            let tr = view.state.tr.delete(sel.from, sel.to);
            const at = tr.mapping.map(insert);
            tr = tr.insert(at, node.type.create(attrs, null, node.marks));
            view.dispatch(tr.setSelection(NodeSelection.create(tr.doc, at)).setMeta("uiEvent", "drop"));
            view.focus();
            return true;
          },
        },
      }),
    ];
  },

  addNodeView() {
    return ({ node, getPos, editor }) => {
      // The editor's view does not exist yet while its first pictures are made: reach it only when needed.
      const page = () => dom.closest(".ProseMirror") as HTMLElement | null;
      let current = node;
      const dom = document.createElement("div");
      dom.className = "image-block";
      const frame = document.createElement("div");
      frame.className = "image-frame";
      const img = document.createElement("img");
      img.draggable = false;
      img.alt = "";
      const left = document.createElement("div");
      left.className = "image-handle left";
      const right = document.createElement("div");
      right.className = "image-handle right";
      const corners = (["nw", "ne", "sw", "se"] as const).map((c) => {
        const d = document.createElement("div");
        d.className = "image-corner";
        d.dataset.c = c;
        return d;
      });
      const grip = document.createElement("div");
      grip.className = "image-grip";
      grip.title = "Drag to move the picture";
      grip.setAttribute("aria-label", "Move the picture");
      // A fixed drawing: four arrows.
      const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
      for (const [k, v] of Object.entries({ viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round", "aria-hidden": "true" })) svg.setAttribute(k, v);
      const arrows = document.createElementNS("http://www.w3.org/2000/svg", "path");
      arrows.setAttribute("d", "M5 9l-3 3 3 3M9 5l3-3 3 3M15 19l-3 3-3-3M19 9l3 3-3 3M2 12h20M12 2v20");
      svg.append(arrows);
      grip.append(svg);
      const caption = document.createElement("input");
      caption.className = "image-caption";
      caption.placeholder = "Write a caption…";
      caption.dir = "auto";
      caption.spellcheck = true;
      caption.readOnly = !editor.isEditable;
      frame.append(img, left, right, ...corners, grip);
      dom.append(frame, caption);

      const isFree = () => !!current.attrs.free;
      const isBoxed = () => isFree() || !!current.attrs.float;

      const paint = () => {
        const a = current.attrs;
        const free = isFree();
        const float = !free && !!a.float;
        const boxed = free || float;
        dom.dataset.align = a.align ?? "center";
        if (free) {
          dom.dataset.free = "true";
          dom.dataset.layer = a.layer === "back" ? "back" : "front";
          dom.style.left = `${Number(a.x) || 0}%`;
          dom.style.top = `${Number(a.y) || 0}px`;
        } else {
          delete dom.dataset.free;
          delete dom.dataset.layer;
          dom.style.left = dom.style.top = "";
        }
        if (float) dom.dataset.float = a.align === "right" ? "right" : "left";
        else delete dom.dataset.float;
        if (boxed) dom.dataset.boxed = "";
        else delete dom.dataset.boxed;
        if (a.ratio && RATIO_VALUES.has(a.ratio)) {
          dom.dataset.ratio = a.ratio;
          frame.style.aspectRatio = ratioCss(a.ratio);
        } else {
          delete dom.dataset.ratio;
          frame.style.aspectRatio = "";
        }
        const pct: number | null = a.width ?? (boxed ? DEFAULT_BOXED_PCT : null);
        // A floating or free picture is sized by its own box; one in the text by its frame.
        dom.style.width = boxed && pct ? `${pct}%` : "";
        frame.style.width = !boxed && (pct || a.ratio) ? `${pct || 100}%` : "";
        if (img.getAttribute("src") !== (a.src ?? "")) img.src = a.src ?? "";
        if (document.activeElement !== caption) caption.value = a.caption ?? "";
        caption.style.display = !editor.isEditable && !a.caption ? "none" : "";
        const edit = editor.isEditable;
        left.style.display = right.style.display = edit && !free ? "" : "none";
        corners.forEach((c) => (c.style.display = edit ? "" : "none"));
        growPage(page());
      };
      img.addEventListener("load", () => {
        growPage(page());
        pop?.refresh();
      });
      paint();

      const apply = (patch: Record<string, unknown>) => applyAttrs(editor.view, getPos(), patch);
      const select = () => {
        const pos = getPos();
        if (pos === undefined) return;
        if (!(editor.view.state.selection instanceof NodeSelection && editor.view.state.selection.from === pos)) {
          editor.view.dispatch(editor.view.state.tr.setSelection(NodeSelection.create(editor.view.state.doc, pos)));
        }
        editor.view.focus();
      };

      // ---- the toolbar ----
      const pageWidth = () => Math.max(1, editor.view.dom.clientWidth);
      const pop = editor.isEditable
        ? buildPopover(
            {
              attrs: () => current.attrs,
              apply,
              measure: () => {
                const r = img.getBoundingClientRect();
                const box = editor.view.dom.getBoundingClientRect();
                return { widthPct: (r.width / pageWidth()) * 100, x: ((r.left - box.left) / pageWidth()) * 100, y: r.top - box.top };
              },
              canFree: () => {
                const pos = getPos();
                return pos !== undefined && editor.view.state.doc.resolve(pos).depth === 0;
              },
            },
            () => img,
          )
        : null;

      // ---- resizing: the side bars and the corners keep the picture's proportions ----
      const startResize = (side: 1 | -1) => (e: PointerEvent) => {
        if (!editor.isEditable) return;
        e.preventDefault();
        e.stopPropagation();
        select();
        const boxed = isBoxed();
        const free = isFree();
        const startX = e.clientX;
        const startWidth = (boxed ? dom : frame).getBoundingClientRect().width;
        const pct0: number = current.attrs.width ?? DEFAULT_BOXED_PCT;
        // What 100% is: the text column for a picture in the text, the page or text width for the others.
        const base = boxed ? startWidth / (pct0 / 100) : dom.getBoundingClientRect().width || 1;
        const center = !boxed && current.attrs.align === "center";
        const x0 = Number(current.attrs.x) || 0;
        let pct = (startWidth / base) * 100;
        let nx = x0;
        const move = (ev: PointerEvent) => {
          const delta = (ev.clientX - startX) * side * (center ? 2 : 1);
          const max = free && side === 1 ? MAX_PCT - x0 : MAX_PCT;
          pct = clamp(((startWidth + delta) / base) * 100, MIN_PCT, max);
          if (boxed) dom.style.width = `${pct}%`;
          else frame.style.width = `${pct}%`;
          if (free && side === -1) {
            // Dragging the left edge keeps the right edge where it was.
            nx = clamp(x0 + (pct0 - pct), 0, MAX_PCT - pct);
            dom.style.left = `${nx}%`;
          }
          growPage(page());
        };
        const up = () => {
          window.removeEventListener("pointermove", move);
          window.removeEventListener("pointerup", up);
          apply({ width: round1(pct), ...(free ? { x: round1(nx) } : {}) });
        };
        window.addEventListener("pointermove", move);
        window.addEventListener("pointerup", up);
      };
      left.addEventListener("pointerdown", startResize(-1));
      right.addEventListener("pointerdown", startResize(1));
      corners.forEach((c) => c.addEventListener("pointerdown", startResize(c.dataset.c!.endsWith("w") ? -1 : 1)));

      // ---- moving a free picture: grab it (or its grip) and release it anywhere on the page ----
      const startMove = (e: PointerEvent) => {
        e.preventDefault();
        select();
        const startX = e.clientX;
        const startY = e.clientY;
        const x0 = Number(current.attrs.x) || 0;
        const y0 = Number(current.attrs.y) || 0;
        const w: number = current.attrs.width ?? DEFAULT_BOXED_PCT;
        const pw = pageWidth();
        let moved = false;
        let nx = x0;
        let ny = y0;
        const move = (ev: PointerEvent) => {
          const dx = ev.clientX - startX;
          const dy = ev.clientY - startY;
          if (!moved && Math.hypot(dx, dy) < 3) return;
          if (!moved) {
            moved = true;
            dom.dataset.dragging = "true";
          }
          ({ x: nx, y: ny } = clampFree(editor.view, x0 + (dx / pw) * 100, y0 + dy, w));
          dom.style.left = `${nx}%`;
          dom.style.top = `${ny}px`;
          growPage(page());
          pop?.refresh();
        };
        const up = () => {
          window.removeEventListener("pointermove", move);
          window.removeEventListener("pointerup", up);
          delete dom.dataset.dragging;
          if (moved) apply({ x: nx, y: ny });
        };
        window.addEventListener("pointermove", move);
        window.addEventListener("pointerup", up);
      };
      const ownControl = (t: EventTarget | null) =>
        t instanceof HTMLElement && (t === caption || t.classList.contains("image-handle") || t.classList.contains("image-corner"));
      dom.addEventListener("pointerdown", (e) => {
        if (!editor.isEditable || e.button !== 0 || ownControl(e.target)) return;
        pop?.show();
        if (isFree()) startMove(e);
      });
      // A free picture is moved by hand (above); the browser's own drag must not start, nor a text selection.
      dom.addEventListener("dragstart", (e) => {
        if (isFree()) e.preventDefault();
      });
      dom.addEventListener("mousedown", (e) => {
        if (isFree() && editor.isEditable && !ownControl(e.target)) e.preventDefault();
      });

      caption.addEventListener("change", () => apply({ caption: caption.value }));
      caption.addEventListener("keydown", (e) => {
        e.stopPropagation();
        if (e.key === "Enter" || e.key === "Escape") {
          caption.blur();
          editor.commands.focus();
        }
      });

      const ro = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(() => isFree() && growPage(page()));
      ro?.observe(dom);

      return {
        dom,
        update(updated) {
          if (updated.type !== current.type) return false;
          current = updated;
          paint();
          pop?.refresh();
          return true;
        },
        selectNode() {
          dom.dataset.selected = "true";
          pop?.show();
        },
        deselectNode() {
          delete dom.dataset.selected;
          pop?.hide();
        },
        destroy() {
          pop?.hide();
          ro?.disconnect();
          growPage(page());
        },
        // Let the caption box, the handles and (for a free picture, moved by hand) the picture itself handle their own events.
        // The grip of a picture in the text is left to ProseMirror, which drags the whole block from it.
        stopEvent(event) {
          const t = event.target as HTMLElement | null;
          if (!t) return false;
          if (t === caption || t.classList.contains("image-handle") || t.classList.contains("image-corner")) return true;
          return isFree() && editor.isEditable;
        },
        ignoreMutation: () => true,
      };
    };
  },
});
