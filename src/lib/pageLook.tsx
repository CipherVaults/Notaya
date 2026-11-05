import type { CSSProperties } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { cn } from "@/lib/utils";

// How a page's icon and cover are stored and drawn. The Rust side (`apply_page_look` in commands/notes.rs)
// accepts exactly these forms and nothing else:
//   icon   an emoji (or short text), or "pic:<32 hex>" (a small picture kept in the vault)
//   cover  "" | "gradient:<name>" | "solid:#rrggbb" | "http://clatt.localhost/<32 hex>" with an optional "#y=<0..100>"

export const COVER_GRADIENTS: { name: string; label: string }[] = [
  { name: "dusk", label: "Dusk" },
  { name: "ocean", label: "Ocean" },
  { name: "forest", label: "Forest" },
  { name: "ember", label: "Ember" },
  { name: "rose", label: "Rose" },
  { name: "sand", label: "Sand" },
  { name: "ink", label: "Ink" },
  { name: "snow", label: "Snow" },
  { name: "aurora", label: "Aurora" },
  { name: "peach", label: "Peach" },
  { name: "lagoon", label: "Lagoon" },
  { name: "grape", label: "Grape" },
  { name: "mint", label: "Mint" },
  { name: "citrus", label: "Citrus" },
  { name: "candy", label: "Candy" },
  { name: "midnight", label: "Midnight" },
  { name: "slate", label: "Slate" },
  { name: "sunrise", label: "Sunrise" },
];

export const COVER_SOLIDS: { color: string; label: string }[] = [
  { color: "#f4d8d4", label: "Blush" },
  { color: "#f6e1c3", label: "Apricot" },
  { color: "#f3ecb7", label: "Butter" },
  { color: "#cfe8cf", label: "Sage" },
  { color: "#c4e4e9", label: "Mist" },
  { color: "#cdd9f3", label: "Periwinkle" },
  { color: "#e0d0f1", label: "Lilac" },
  { color: "#d9d4cc", label: "Stone" },
  { color: "#e5484d", label: "Red" },
  { color: "#f2994a", label: "Orange" },
  { color: "#e9b949", label: "Gold" },
  { color: "#30a46c", label: "Green" },
  { color: "#0ea5e9", label: "Sky" },
  { color: "#3b5bdb", label: "Blue" },
  { color: "#7c4dff", label: "Violet" },
  { color: "#2b2b2b", label: "Charcoal" },
];

const PIC_ID = /^[0-9a-fA-F]{32}$/;
const COVER_URL = /^(?:https?:\/\/clatt\.localhost\/|clatt:\/\/(?:localhost\/)?)([0-9a-fA-F]{32})(?:#y=(\d{1,3}))?$/;

export type ParsedCover =
  | { kind: "none" }
  | { kind: "gradient"; name: string }
  | { kind: "solid"; color: string }
  | { kind: "picture"; id: string; y: number }
  | { kind: "unknown" };

export function parseCover(cover: string | undefined | null): ParsedCover {
  if (!cover) return { kind: "none" };
  if (cover.startsWith("gradient:")) return { kind: "gradient", name: cover.slice(9).replace(/[^a-z0-9-]/gi, "") };
  if (cover.startsWith("solid:#") && /^solid:#[0-9a-fA-F]{6}$/.test(cover)) return { kind: "solid", color: cover.slice(6) };
  const m = COVER_URL.exec(cover);
  if (m) return { kind: "picture", id: m[1].toLowerCase(), y: m[2] === undefined ? 50 : Math.min(100, Number(m[2])) };
  return { kind: "unknown" };
}

export const pictureCover = (id: string, y = 50) => `http://clatt.localhost/${id}#y=${Math.round(Math.min(100, Math.max(0, y)))}`;
export const gradientCover = (name: string) => `gradient:${name}`;
export const solidCover = (color: string) => `solid:${color}`;

/** The address the page shows a vault picture from. */
export const pictureSrc = (id: string) => convertFileSrc(id, "clatt");

export const isPictureIcon = (icon: string | undefined | null) => !!icon && icon.startsWith("pic:") && PIC_ID.test(icon.slice(4));
export const pictureIcon = (id: string) => `pic:${id}`;

/** The text to use where only text fits (a window title, a menu line): a picture icon has none. */
export const iconText = (icon: string | undefined | null) => (icon && !icon.startsWith("pic:") ? icon : "");

/** A page icon: the emoji, or the small picture. Nothing (or `fallback`) when the page has none. */
export function PageIcon({ icon, size = 16, fallback = null, className }: { icon: string | undefined | null; size?: number; fallback?: React.ReactNode; className?: string }) {
  if (!icon) return <>{fallback}</>;
  if (icon.startsWith("pic:")) {
    if (!isPictureIcon(icon)) return <>{fallback}</>;
    return (
      <img
        src={pictureSrc(icon.slice(4).toLowerCase())}
        alt=""
        draggable={false}
        className={cn("inline-block shrink-0 rounded-[3px] object-cover align-middle", className)}
        style={{ width: size, height: size }}
      />
    );
  }
  return (
    <span className={cn("inline-block shrink-0 leading-none", className)} style={{ fontSize: size, lineHeight: 1 }} aria-hidden>
      {icon}
    </span>
  );
}

/** What fills a banner or a card strip for this cover (a picture is drawn by the caller as an image). */
export function coverFill(cover: string | undefined | null): { className?: string; style?: CSSProperties } | null {
  const c = parseCover(cover);
  if (c.kind === "gradient") return { className: `cover-gradient-${c.name}` };
  if (c.kind === "solid") return { style: { background: c.color } };
  return null;
}

/** A banner-shaped box showing the cover (for a card). `y` is read from the cover itself. */
export function CoverStrip({ cover, className }: { cover: string | undefined | null; className?: string }) {
  const c = parseCover(cover);
  if (c.kind === "none" || c.kind === "unknown") return null;
  if (c.kind === "picture") {
    return (
      <div className={cn("overflow-hidden", className)}>
        <img src={pictureSrc(c.id)} alt="" loading="lazy" draggable={false} className="h-full w-full object-cover" style={{ objectPosition: `50% ${c.y}%` }} />
      </div>
    );
  }
  const fill = coverFill(cover);
  return <div className={cn(className, fill?.className)} style={fill?.style} />;
}
