import { useEffect, useRef, useState } from "react";
import { Popover } from "radix-ui";
import { Check, Image as ImageIcon, Move, SmilePlus, X } from "lucide-react";
import { cn } from "@/lib/utils";
import { PageIcon, coverFill, parseCover, pictureCover, pictureSrc } from "@/lib/pageLook";
import { CoverPicker, IconPicker } from "./LookPickers";

// A page's icon (an emoji or a small picture) and its cover banner. Both are chosen here: hover the title
// to add them, click the icon or "Change cover" to change them. `PageEditor` saves the result with `set_page_look`.

const POPOVER = "z-[80] rounded-xl border border-line bg-elevated shadow-float outline-none";
const GHOST_BTN =
  "inline-flex h-7 items-center gap-1.5 rounded-lg px-2 text-xs text-faint transition-colors hover:bg-surface-2 hover:text-fg";

function CoverPopover({ cover, onPick, children, align = "end" }: { cover: string; onPick: (c: string) => void; children: React.ReactNode; align?: "start" | "end" }) {
  const [open, setOpen] = useState(false);
  return (
    <Popover.Root open={open} onOpenChange={setOpen}>
      <Popover.Trigger asChild>{children}</Popover.Trigger>
      <Popover.Portal>
        <Popover.Content align={align} sideOffset={6} collisionPadding={12} className={POPOVER}>
          <CoverPicker cover={cover} onPick={onPick} onClose={() => setOpen(false)} />
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}

function IconPopover({ icon, allowPicture, onPick, children }: { icon: string; allowPicture: boolean; onPick: (i: string) => void; children: React.ReactNode }) {
  const [open, setOpen] = useState(false);
  return (
    <Popover.Root open={open} onOpenChange={setOpen}>
      <Popover.Trigger asChild>{children}</Popover.Trigger>
      <Popover.Portal>
        <Popover.Content align="start" sideOffset={6} collisionPadding={12} className={POPOVER}>
          <IconPicker icon={icon} allowPicture={allowPicture} onPick={onPick} onClose={() => setOpen(false)} />
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}

/** The cover banner above the page (about 220px tall, the full width of the page). */
export function PageCover({ cover, readOnly, onChange }: { cover: string; readOnly: boolean; onChange: (cover: string) => void }) {
  const parsed = parseCover(cover);
  const boxRef = useRef<HTMLDivElement>(null);
  const imgRef = useRef<HTMLImageElement>(null);
  const [moving, setMoving] = useState(false);
  const [y, setY] = useState(50);
  const drag = useRef<{ startY: number; startPct: number; overflow: number } | null>(null);
  const [fits, setFits] = useState(false);

  // Leave the repositioning mode when the cover is replaced or removed.
  useEffect(() => {
    setMoving(false);
  }, [parsed.kind === "picture" ? parsed.id : cover]);

  if (!cover) return null;

  const savedY = parsed.kind === "picture" ? parsed.y : 50;
  const shownY = moving ? y : savedY;

  const startMove = () => {
    setY(savedY);
    setMoving(true);
    setFits(false);
    setTimeout(() => boxRef.current?.focus(), 0);
  };
  /** How many pixels of the picture are outside the banner (up and down): what dragging can show. */
  const overflowPx = () => {
    const box = boxRef.current;
    const img = imgRef.current;
    if (!box || !img || !img.naturalWidth) return 0;
    const shownH = (box.clientWidth * img.naturalHeight) / img.naturalWidth;
    return Math.max(0, shownH - box.clientHeight);
  };
  const onPointerDown = (e: React.PointerEvent) => {
    if (!moving || (e.target as HTMLElement).closest("button")) return;
    const overflow = overflowPx();
    if (overflow < 1) {
      setFits(true);
      return;
    }
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    drag.current = { startY: e.clientY, startPct: y, overflow };
  };
  const onPointerMove = (e: React.PointerEvent) => {
    const d = drag.current;
    if (!d) return;
    // Dragging the picture down shows more of its top, so the position (from the top) goes down.
    const next = d.startPct - ((e.clientY - d.startY) / d.overflow) * 100;
    setY(Math.round(Math.min(100, Math.max(0, next))));
  };
  const onPointerUp = () => {
    drag.current = null;
  };
  const nudge = (e: React.KeyboardEvent) => {
    if (!moving) return;
    const step = e.shiftKey ? 10 : 2;
    if (e.key === "ArrowUp") setY((v) => Math.max(0, v - step));
    else if (e.key === "ArrowDown") setY((v) => Math.min(100, v + step));
    else if (e.key === "Enter" && parsed.kind === "picture") {
      onChange(pictureCover(parsed.id, y));
      setMoving(false);
    } else if (e.key === "Escape") setMoving(false);
    else return;
    e.preventDefault();
  };

  const fill = coverFill(cover);
  return (
    <div
      ref={boxRef}
      data-testid="page-cover"
      className={cn("group/cover relative h-[220px] w-full shrink-0 select-none overflow-hidden bg-surface-2", fill?.className, moving && "cursor-grab active:cursor-grabbing")}
      style={fill?.style}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerUp}
      onKeyDown={nudge}
      tabIndex={moving ? 0 : -1}
    >
      {parsed.kind === "picture" && (
        <img
          ref={imgRef}
          src={pictureSrc(parsed.id)}
          alt=""
          draggable={false}
          onLoad={() => moving && setFits(overflowPx() < 1)}
          className="pointer-events-none h-full w-full object-cover"
          style={{ objectPosition: `50% ${shownY}%` }}
        />
      )}

      {moving && (
        <div className="absolute inset-x-0 top-3 flex justify-center">
          <span className="rounded-lg bg-bg/85 px-3 py-1 text-xs text-fg shadow-sm backdrop-blur">
            {fits ? "This picture already fits the banner." : "Drag the picture up or down"}
          </span>
        </div>
      )}

      {!readOnly && (
        <div
          className={cn(
            "absolute bottom-3 end-4 flex items-center gap-1.5 transition-opacity",
            moving ? "opacity-100" : "opacity-0 focus-within:opacity-100 group-hover/cover:opacity-100",
          )}
        >
          {moving ? (
            <>
              <CoverBtn onClick={() => setMoving(false)}>
                <X size={13} /> Cancel
              </CoverBtn>
              <CoverBtn
                strong
                onClick={() => {
                  if (parsed.kind === "picture") onChange(pictureCover(parsed.id, y));
                  setMoving(false);
                }}
              >
                <Check size={13} /> Save position
              </CoverBtn>
            </>
          ) : (
            <>
              <CoverPopover cover={cover} onPick={onChange}>
                <CoverBtn>
                  <ImageIcon size={13} /> Change cover
                </CoverBtn>
              </CoverPopover>
              {parsed.kind === "picture" && (
                <CoverBtn onClick={startMove}>
                  <Move size={13} /> Reposition
                </CoverBtn>
              )}
              <CoverBtn onClick={() => onChange("")} label="Remove cover">
                <X size={13} />
              </CoverBtn>
            </>
          )}
        </div>
      )}
    </div>
  );
}

const CoverBtn = ({ children, onClick, strong, label, ...rest }: React.ButtonHTMLAttributes<HTMLButtonElement> & { strong?: boolean; label?: string }) => (
  <button
    type="button"
    onClick={onClick}
    aria-label={label}
    title={label}
    {...rest}
    className={cn(
      "inline-flex h-7 items-center gap-1.5 rounded-lg px-2.5 text-xs shadow-sm backdrop-blur transition-colors",
      strong ? "bg-accent text-accent-fg hover:opacity-90" : "bg-bg/85 text-fg hover:bg-bg",
    )}
  >
    {children}
  </button>
);

interface Props {
  icon: string;
  cover: string;
  readOnly: boolean;
  /** Pictures are allowed as icons on pages, not on database rows. */
  allowPicture?: boolean;
  onChange: (icon: string, cover: string) => void;
}

/**
 * The icon above the title, and the "Add icon" / "Add cover" buttons that appear when the pointer is over the title
 * area (the parent has the `group/head` class). Click the icon to change it.
 */
export function PageIconAndActions({ icon, cover, readOnly, allowPicture = true, onChange }: Props) {
  const hasActions = !readOnly && (!icon || !cover);
  if (!icon && !hasActions) return null;
  return (
    <div className={cn("pointer-events-none relative z-10", cover && icon && "-mt-10")} data-testid="page-icon-row">
      {icon &&
        (readOnly ? (
          <span className="mb-1 inline-grid h-[76px] w-[76px] place-items-center rounded-xl">
            <PageIcon icon={icon} size={58} />
          </span>
        ) : (
          <IconPopover icon={icon} allowPicture={allowPicture} onPick={(i) => onChange(i, cover)}>
            <button
              type="button"
              aria-label="Change icon"
              title="Change icon"
              data-testid="page-icon"
              className="pointer-events-auto mb-1 inline-grid h-[76px] w-[76px] place-items-center rounded-xl transition-colors hover:bg-surface-2"
            >
              <PageIcon icon={icon} size={58} />
            </button>
          </IconPopover>
        ))}
      {hasActions && (
        <div className={cn("pointer-events-auto flex h-7 w-fit items-center gap-1 opacity-0 transition-opacity focus-within:opacity-100 group-hover/head:opacity-100", icon && "mb-1")} data-testid="page-look-actions">
          {!icon && (
            <IconPopover icon="" allowPicture={allowPicture} onPick={(i) => onChange(i, cover)}>
              <button type="button" className={GHOST_BTN} data-testid="add-icon">
                <SmilePlus size={14} /> Add icon
              </button>
            </IconPopover>
          )}
          {!cover && (
            <CoverPopover cover="" align="start" onPick={(c) => onChange(icon, c)}>
              <button type="button" className={GHOST_BTN} data-testid="add-cover">
                <ImageIcon size={14} /> Add cover
              </button>
            </CoverPopover>
          )}
        </div>
      )}
    </div>
  );
}
