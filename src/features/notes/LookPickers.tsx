import { useMemo, useRef, useState } from "react";
import { ImagePlus, Search, Shuffle, Trash2, Upload } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import { cn } from "@/lib/utils";
import { EMOJI_GROUPS, randomEmoji, searchEmoji } from "@/lib/emoji-data";
import { COVER_GRADIENTS, COVER_SOLIDS, PageIcon, gradientCover, parseCover, pictureCover, pictureIcon, solidCover } from "@/lib/pageLook";

const IMAGE_TYPES = ["image/png", "image/jpeg", "image/gif", "image/webp"];

/** Stores a chosen picture in the vault (encrypted like any other) and returns its id. Null (with a message) when it can't be used. */
async function storePicture(file: File, maxEdge?: number): Promise<string | null> {
  if (!IMAGE_TYPES.includes(file.type)) {
    toast.error("Only PNG, JPEG, GIF and WebP pictures can be used.");
    return null;
  }
  try {
    let bytes: Uint8Array;
    if (maxEdge) {
      bytes = await shrink(file, maxEdge);
    } else {
      bytes = new Uint8Array(await file.arrayBuffer());
    }
    return await api.addAttachment(bytes);
  } catch (e) {
    toast.error(errorText(e));
    return null;
  }
}

/** A small, square-cropped PNG of the picture, so an icon is light to keep and to draw. Falls back to the file itself when it is already small. */
async function shrink(file: File, edge: number): Promise<Uint8Array> {
  try {
    const bmp = await createImageBitmap(file);
    const side = Math.min(bmp.width, bmp.height);
    const canvas = document.createElement("canvas");
    canvas.width = canvas.height = Math.min(edge, side);
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("no canvas");
    ctx.drawImage(bmp, (bmp.width - side) / 2, (bmp.height - side) / 2, side, side, 0, 0, canvas.width, canvas.height);
    bmp.close?.();
    const blob = await new Promise<Blob | null>((r) => canvas.toBlob(r, "image/png"));
    if (!blob) throw new Error("no blob");
    return new Uint8Array(await blob.arrayBuffer());
  } catch {
    if (file.size <= 1024 * 1024) return new Uint8Array(await file.arrayBuffer());
    throw new Error("This picture can't be shrunk to an icon. Try a smaller one.");
  }
}

function pickFile(onFile: (f: File) => void) {
  const input = document.createElement("input");
  input.type = "file";
  input.accept = IMAGE_TYPES.join(",");
  input.onchange = () => {
    const f = input.files?.[0];
    if (f) onFile(f);
  };
  input.click();
}

/** The body of the icon popover: a searchable emoji grid, Random, Remove, and (for pages) a small picture. */
export function IconPicker({
  icon,
  allowPicture,
  onPick,
  onClose,
}: {
  icon: string;
  allowPicture: boolean;
  onPick: (icon: string) => void;
  onClose: () => void;
}) {
  const [query, setQuery] = useState("");
  const [tab, setTab] = useState<"emoji" | "picture">("emoji");
  const [busy, setBusy] = useState(false);
  const results = useMemo(() => (query.trim() ? searchEmoji(query) : null), [query]);
  const gridRef = useRef<HTMLDivElement>(null);
  const choose = (e: string) => {
    onPick(e);
    onClose();
  };

  const upload = () =>
    pickFile(async (f) => {
      setBusy(true);
      const id = await storePicture(f, 128);
      setBusy(false);
      if (id) choose(pictureIcon(id));
    });

  return (
    <div className="flex w-[352px] max-w-full flex-col" data-testid="icon-picker">
      <div className="flex items-center gap-1 border-b border-line px-2 pt-2">
        <TabButton active={tab === "emoji"} onClick={() => setTab("emoji")}>Emoji</TabButton>
        {allowPicture && <TabButton active={tab === "picture"} onClick={() => setTab("picture")}>Picture</TabButton>}
        <span className="flex-1" />
        <button type="button" onClick={() => choose(randomEmoji())} className="mb-1 inline-flex h-7 items-center gap-1.5 rounded-md px-2 text-xs text-muted hover:bg-surface-2 hover:text-fg" title="Pick one at random">
          <Shuffle size={13} /> Random
        </button>
        {icon && (
          <button type="button" onClick={() => choose("")} className="mb-1 inline-flex h-7 items-center gap-1.5 rounded-md px-2 text-xs text-muted hover:bg-surface-2 hover:text-danger">
            <Trash2 size={13} /> Remove
          </button>
        )}
      </div>

      {tab === "emoji" ? (
        <>
          <div className="relative m-2 mb-1">
            <Search size={14} className="pointer-events-none absolute start-2.5 top-1/2 -translate-y-1/2 text-faint" />
            <input
              autoFocus
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search emoji"
              aria-label="Search emoji"
              dir="auto"
              className="h-8 w-full rounded-lg border border-line bg-surface ps-8 pe-2 text-sm text-fg outline-none placeholder:text-faint focus:border-accent/60"
            />
          </div>
          {!results && (
            <div className="flex items-center gap-0.5 px-2 pb-1">
              {EMOJI_GROUPS.map((g) => (
                <button
                  key={g.id}
                  type="button"
                  title={g.label}
                  aria-label={g.label}
                  onClick={() => gridRef.current?.querySelector(`[data-group="${g.id}"]`)?.scrollIntoView({ block: "start" })}
                  className="grid h-7 flex-1 place-items-center rounded-md text-[0.95rem] opacity-70 hover:bg-surface-2 hover:opacity-100"
                >
                  {g.glyph}
                </button>
              ))}
            </div>
          )}
          <div ref={gridRef} className="h-[264px] overflow-y-auto px-2 pb-2">
            {results ? (
              results.length ? (
                <EmojiGrid items={results.map((x) => x.e)} current={icon} onPick={choose} />
              ) : (
                <p className="py-10 text-center text-sm text-faint">No emoji match.</p>
              )
            ) : (
              EMOJI_GROUPS.map((g) => (
                <section key={g.id} data-group={g.id}>
                  <h3 className="sticky top-0 z-[1] bg-elevated py-1 text-[0.7rem] font-semibold uppercase tracking-wider text-faint">{g.label}</h3>
                  <EmojiGrid items={g.items.map((x) => x.e)} current={icon} onPick={choose} />
                </section>
              ))
            )}
          </div>
        </>
      ) : (
        <div className="flex h-[330px] flex-col items-center justify-center gap-3 px-6 text-center">
          <div className="grid h-16 w-16 place-items-center rounded-xl border border-dashed border-line text-faint">
            {icon.startsWith("pic:") ? <PageIcon icon={icon} size={56} /> : <ImagePlus size={24} />}
          </div>
          <button
            type="button"
            disabled={busy}
            onClick={upload}
            className="inline-flex h-9 items-center gap-2 rounded-lg border border-line bg-surface px-3 text-sm text-fg hover:bg-surface-2 disabled:opacity-60"
          >
            <Upload size={15} /> {busy ? "Adding…" : "Upload a picture"}
          </button>
          <p className="text-xs leading-relaxed text-faint">PNG, JPEG, GIF or WebP. It is cropped to a square, shrunk, and kept encrypted in your notes.</p>
        </div>
      )}
    </div>
  );
}

function TabButton({ active, onClick, children }: { active: boolean; onClick: () => void; children: React.ReactNode }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn("mb-[-1px] h-8 border-b-2 px-2.5 text-sm", active ? "border-accent text-fg" : "border-transparent text-muted hover:text-fg")}
    >
      {children}
    </button>
  );
}

function EmojiGrid({ items, current, onPick }: { items: string[]; current: string; onPick: (e: string) => void }) {
  return (
    <div className="grid grid-cols-8 gap-0.5" role="grid">
      {items.map((e, i) => (
        <button
          key={`${e}${i}`}
          type="button"
          data-emoji={e}
          onClick={() => onPick(e)}
          className={cn("grid h-9 place-items-center rounded-md text-[1.3rem] leading-none hover:bg-surface-2", current === e && "bg-accent-soft ring-1 ring-accent/50")}
        >
          {e}
        </button>
      ))}
    </div>
  );
}

/** The body of the cover popover: gradients, colours, a picture from the computer, and Remove. */
export function CoverPicker({ cover, onPick, onClose }: { cover: string; onPick: (cover: string) => void; onClose: () => void }) {
  const [busy, setBusy] = useState(false);
  const current = parseCover(cover);
  const choose = (c: string) => {
    onPick(c);
    onClose();
  };
  const upload = () =>
    pickFile(async (f) => {
      setBusy(true);
      const id = await storePicture(f);
      setBusy(false);
      if (id) choose(pictureCover(id, 50));
    });

  return (
    <div className="flex w-[360px] max-w-full flex-col gap-3 p-3" data-testid="cover-picker">
      <div className="flex items-center justify-between">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-faint">Cover</h3>
        {cover && (
          <button type="button" onClick={() => choose("")} className="inline-flex h-7 items-center gap-1.5 rounded-md px-2 text-xs text-muted hover:bg-surface-2 hover:text-danger">
            <Trash2 size={13} /> Remove
          </button>
        )}
      </div>

      <button
        type="button"
        disabled={busy}
        onClick={upload}
        className="inline-flex h-9 items-center justify-center gap-2 rounded-lg border border-line bg-surface px-3 text-sm text-fg hover:bg-surface-2 disabled:opacity-60"
      >
        <Upload size={15} /> {busy ? "Adding…" : "Upload picture"}
      </button>

      <div>
        <h4 className="mb-1.5 text-[0.7rem] font-medium text-faint">Gradients</h4>
        <div className="grid grid-cols-6 gap-1.5">
          {COVER_GRADIENTS.map((g) => (
            <button
              key={g.name}
              type="button"
              title={g.label}
              aria-label={`${g.label} gradient`}
              data-cover={gradientCover(g.name)}
              onClick={() => choose(gradientCover(g.name))}
              className={cn("h-9 rounded-md border border-line transition-transform hover:scale-105", `cover-gradient-${g.name}`, current.kind === "gradient" && current.name === g.name && "ring-2 ring-accent ring-offset-1 ring-offset-[var(--elevated)]")}
            />
          ))}
        </div>
      </div>

      <div>
        <h4 className="mb-1.5 text-[0.7rem] font-medium text-faint">Colours</h4>
        <div className="grid grid-cols-8 gap-1.5">
          {COVER_SOLIDS.map((c) => (
            <button
              key={c.color}
              type="button"
              title={c.label}
              aria-label={`${c.label} colour`}
              data-cover={solidCover(c.color)}
              onClick={() => choose(solidCover(c.color))}
              className={cn("h-7 rounded-md border border-line transition-transform hover:scale-110", current.kind === "solid" && current.color.toLowerCase() === c.color && "ring-2 ring-accent ring-offset-1 ring-offset-[var(--elevated)]")}
              style={{ background: c.color }}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
