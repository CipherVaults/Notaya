import StarterKit from "@tiptap/starter-kit";
import { Color, FontFamily, FontSize, TextStyle } from "@tiptap/extension-text-style";
import Highlight from "@tiptap/extension-highlight";
import { TaskItem, TaskList } from "@tiptap/extension-list";
import { TableKit } from "@tiptap/extension-table";
import Image from "@tiptap/extension-image";
import { Callout, Toggle } from "./blocks";
import { Column, Columns, Comment, PageLink } from "./more-blocks";
import { MathBlock, MathInline } from "./math";
import { ImageBlock } from "./ImageBlock";
import { SlashCommand, type SlashOptions } from "./slash";
import { Placeholder } from "@tiptap/extensions";
import CodeBlockLowlight from "@tiptap/extension-code-block-lowlight";
import TextAlign from "@tiptap/extension-text-align";
import { common, createLowlight } from "lowlight";
import type { Editor } from "@tiptap/react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";

const lowlight = createLowlight(common);

const link = {
  openOnClick: false,
  autolink: true,
  defaultProtocol: "https",
  HTMLAttributes: { rel: "noopener noreferrer nofollow", target: null },
} as const;

/** Full editor for notebook pages: block types, the "/" menu and pictures. */
export function pageExtensions(slash: SlashOptions) {
  return [
    StarterKit.configure({ codeBlock: false, heading: { levels: [1, 2, 3, 4] }, link }),
    TextStyle,
    Color,
    FontSize,
    FontFamily,
    Highlight.configure({ multicolor: true }),
    TaskList,
    TaskItem.configure({ nested: true }),
    TableKit.configure({ table: { resizable: true } }),
    // Only images stored in the vault (or pasted files) are allowed; remote
    // images are also blocked by the Content Security Policy.
    ImageBlock.configure({ allowBase64: false }),
    Callout,
    Toggle,
    Columns,
    Column,
    PageLink,
    Comment,
    MathBlock,
    MathInline,
    Placeholder.configure({
      showOnlyCurrent: true,
      includeChildren: true,
      placeholder: ({ node }) => {
        if (node.type.name === "heading") return `Heading ${node.attrs.level}`;
        return "Press / to insert something, or just start typing";
      },
    }),
    CodeBlockLowlight.configure({ lowlight }),
    TextAlign.configure({ types: ["heading", "paragraph"] }),
    SlashCommand.configure(slash),
  ];
}

/** Lighter editor for sticky notes. */
export function stickyExtensions() {
  return [
    StarterKit.configure({ codeBlock: false, heading: { levels: [2, 3] }, link }),
    Highlight,
    TaskList,
    TaskItem.configure({ nested: true }),
    Image.configure({ allowBase64: false }),
    Placeholder.configure({ placeholder: "Write something…" }),
  ];
}

const IMAGE_TYPES = ["image/png", "image/jpeg", "image/gif", "image/webp"];

/** Encrypts an image into the vault and returns the URL the editor can show. */
export async function uploadImage(file: File): Promise<string | null> {
  if (!IMAGE_TYPES.includes(file.type)) {
    toast.error("Only PNG, JPEG, GIF and WebP images can be added.");
    return null;
  }
  try {
    const bytes = new Uint8Array(await file.arrayBuffer());
    const id = await api.addAttachment(bytes);
    return convertFileSrc(id, "clatt");
  } catch (e) {
    toast.error(errorText(e));
    return null;
  }
}

/** Opens the file chooser and inserts the chosen pictures at the cursor. */
export function pickImages(editor: Editor) {
  const input = document.createElement("input");
  input.type = "file";
  input.accept = IMAGE_TYPES.join(",");
  input.multiple = true;
  input.onchange = () => {
    const files = Array.from(input.files ?? []);
    if (files.length) insertImages(editor, files);
  };
  input.click();
}

export async function insertImages(editor: Editor, files: File[], pos?: number) {
  for (const file of files) {
    const src = await uploadImage(file);
    if (!src) continue;
    const chain = editor.chain().focus();
    if (pos !== undefined) chain.insertContentAt(pos, { type: "image", attrs: { src } }).run();
    else chain.setImage({ src }).run();
  }
}

/** Shared paste/drop/click handling: images become encrypted attachments, Ctrl+click opens links. */
export function editorProps(getEditor: () => Editor | null) {
  return {
    handlePaste: (_view: unknown, event: ClipboardEvent) => {
      const files = Array.from(event.clipboardData?.files ?? []).filter((f) => f.type.startsWith("image/"));
      const editor = getEditor();
      if (files.length && editor) {
        event.preventDefault();
        insertImages(editor, files);
        return true;
      }
      return false;
    },
    handleDrop: (view: { posAtCoords: (c: { left: number; top: number }) => { pos: number } | null }, event: DragEvent) => {
      const files = Array.from(event.dataTransfer?.files ?? []).filter((f) => f.type.startsWith("image/"));
      const editor = getEditor();
      if (files.length && editor) {
        event.preventDefault();
        const pos = view.posAtCoords({ left: event.clientX, top: event.clientY })?.pos;
        insertImages(editor, files, pos);
        return true;
      }
      return false;
    },
    handleClick: (_view: unknown, _pos: number, event: MouseEvent) => {
      const a = (event.target as HTMLElement).closest("a");
      if (a && event.ctrlKey) {
        const href = a.getAttribute("href");
        if (href) api.openUrl(href).catch((e) => toast.error(errorText(e)));
        return true;
      }
      return false;
    },
  };
}

export function parseContent(json: string) {
  if (!json) return "";
  try {
    return JSON.parse(json);
  } catch {
    return "";
  }
}
