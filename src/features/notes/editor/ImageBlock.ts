// Images that can be resized by dragging, aligned, and given a caption.
// Only images stored in the vault (clatt:// URLs) can be shown; the Content Security
// Policy blocks everything else.

import Image from "@tiptap/extension-image";

export type ImageAlign = "left" | "center" | "right";

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
    };
  },

  addNodeView() {
    return ({ node, getPos, editor }) => {
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
      const caption = document.createElement("input");
      caption.className = "image-caption";
      caption.placeholder = "Write a caption…";
      caption.spellcheck = true;
      caption.readOnly = !editor.isEditable;
      frame.append(img, left, right);
      dom.append(frame, caption);

      const paint = () => {
        dom.dataset.align = current.attrs.align ?? "center";
        img.src = current.attrs.src ?? "";
        frame.style.width = current.attrs.width ? `${current.attrs.width}%` : "";
        if (document.activeElement !== caption) caption.value = current.attrs.caption ?? "";
        caption.style.display = !editor.isEditable && !current.attrs.caption ? "none" : "";
        left.style.display = right.style.display = editor.isEditable ? "" : "none";
      };
      paint();

      const setAttrs = (patch: Record<string, unknown>) => {
        const pos = getPos();
        if (pos === undefined) return;
        editor.view.dispatch(editor.state.tr.setNodeMarkup(pos, undefined, { ...current.attrs, ...patch }));
      };

      // Drag a side handle to resize. Both sides work, the image keeps its aspect ratio.
      const startResize = (side: 1 | -1) => (e: PointerEvent) => {
        if (!editor.isEditable) return;
        e.preventDefault();
        e.stopPropagation();
        const startX = e.clientX;
        const startWidth = frame.getBoundingClientRect().width;
        const column = dom.getBoundingClientRect().width || 1;
        const center = current.attrs.align === "center";
        let pct = (startWidth / column) * 100;
        const move = (ev: PointerEvent) => {
          const delta = (ev.clientX - startX) * side * (center ? 2 : 1);
          pct = Math.min(100, Math.max(8, ((startWidth + delta) / column) * 100));
          frame.style.width = `${pct}%`;
        };
        const up = () => {
          window.removeEventListener("pointermove", move);
          window.removeEventListener("pointerup", up);
          setAttrs({ width: Math.round(pct) });
        };
        window.addEventListener("pointermove", move);
        window.addEventListener("pointerup", up);
      };
      left.addEventListener("pointerdown", startResize(-1));
      right.addEventListener("pointerdown", startResize(1));

      caption.addEventListener("change", () => setAttrs({ caption: caption.value }));
      caption.addEventListener("keydown", (e) => {
        e.stopPropagation();
        if (e.key === "Enter" || e.key === "Escape") {
          caption.blur();
          editor.commands.focus();
        }
      });

      return {
        dom,
        update(updated) {
          if (updated.type !== current.type) return false;
          current = updated;
          paint();
          return true;
        },
        selectNode() {
          dom.dataset.selected = "true";
        },
        deselectNode() {
          delete dom.dataset.selected;
        },
        // Let the caption box and resize handles handle their own events.
        stopEvent(event) {
          const t = event.target as HTMLElement | null;
          return !!t && (t === caption || t.classList.contains("image-handle"));
        },
        ignoreMutation: () => true,
      };
    };
  },
});
