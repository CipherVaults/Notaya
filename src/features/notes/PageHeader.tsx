import { X } from "lucide-react";
import { cn } from "@/lib/utils";

// Pages made earlier may carry a cover picture and an icon. They are still shown, and can be removed,
// but new ones can't be added any more.

interface Props {
  icon: string;
  cover: string;
  readOnly: boolean;
  onChange: (icon: string, cover: string) => void;
}

/** A cover picture shown above the page, if the page already has one. */
export function PageCover({ cover, readOnly, onChange }: { cover: string; readOnly: boolean; onChange: (cover: string) => void }) {
  if (!cover) return null;
  const gradient = cover.startsWith("gradient:") ? cover.slice(9) : null;
  return (
    <div className="group relative h-[220px] w-full shrink-0 overflow-hidden">
      {gradient ? (
        <div className={cn("h-full w-full", `cover-gradient-${gradient}`)} />
      ) : (
        <img src={cover} alt="" className="h-full w-full object-cover" draggable={false} />
      )}
      {!readOnly && (
        <button
          onClick={() => onChange("")}
          className="absolute bottom-3 right-4 inline-flex h-7 items-center gap-1.5 rounded-lg bg-bg/85 px-2.5 text-xs text-fg opacity-0 shadow-sm backdrop-blur transition-opacity hover:bg-bg group-hover:opacity-100"
        >
          <X size={13} /> Remove cover
        </button>
      )}
    </div>
  );
}

/** The icon beside the title, if the page already has one. */
export function PageIconAndActions({ icon, cover, readOnly, onChange }: Props) {
  if (!icon) return null;
  return (
    <div className={cn("group/head relative z-10 mb-1", cover && "-mt-10")}>
      <span className="inline-grid h-[76px] w-[76px] place-items-center rounded-xl text-[3.6rem] leading-none">{icon}</span>
      {!readOnly && (
        <button
          onClick={() => onChange("", cover)}
          className="ml-2 inline-flex h-7 items-center gap-1.5 rounded-lg px-2 text-xs text-faint opacity-0 transition-opacity hover:bg-surface-2 hover:text-fg group-hover/head:opacity-100"
        >
          <X size={13} /> Remove icon
        </button>
      )}
    </div>
  );
}
