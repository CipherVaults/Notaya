// Maps a view kind to its component. Every view is `(props: ViewProps) => JSX` in a file named after the kind
// (TableView.tsx, BoardView.tsx, ...). Files that do not exist yet show a "coming" notice.

import { lazy, type ComponentType, type LazyExoticComponent } from "react";
import { Construction } from "lucide-react";
import type { ViewKind } from "@/lib/database-types";
import type { ViewProps } from "../viewTypes";

const loaders = import.meta.glob<{ default: ComponentType<ViewProps> }>("./*View.tsx");

function Coming({ kind }: { kind: ViewKind }) {
  return (
    <div className="flex flex-col items-center gap-2 px-6 py-16 text-center text-muted">
      <Construction size={22} />
      <div className="text-sm font-medium text-fg">The {kind} view is coming</div>
      <div className="text-xs">It is not part of this build yet. Switch to another view to keep working.</div>
    </div>
  );
}

const cache = new Map<ViewKind, LazyExoticComponent<ComponentType<ViewProps>>>();

export function viewComponent(kind: ViewKind): LazyExoticComponent<ComponentType<ViewProps>> {
  let c = cache.get(kind);
  if (!c) {
    const file = `./${kind[0].toUpperCase()}${kind.slice(1)}View.tsx`;
    const load = loaders[file];
    c = lazy(load ?? (async () => ({ default: () => <Coming kind={kind} /> })));
    cache.set(kind, c);
  }
  return c;
}
