import {
  AlignLeft, ArrowUpRight, Calendar, CalendarDays, CircleDot, Clock, Columns3, Fingerprint, GanttChart, Hash, History, Layers, LayoutGrid,
  Link as LinkIcon, List as ListIcon, Loader, Mail, MapPin, Paperclip, Phone, Sigma, SquareCheck, Table as TableIcon, Tags, Type, User, UserPen,
  ChartColumn, type LucideIcon,
} from "lucide-react";
import type { PropKind, ViewKind } from "@/lib/database-types";

const KIND_ICONS: Record<PropKind, LucideIcon> = {
  title: Type, text: AlignLeft, number: Hash, select: CircleDot, multiSelect: Tags, status: Loader, date: Calendar, person: User,
  files: Paperclip, checkbox: SquareCheck, url: LinkIcon, email: Mail, phone: Phone, formula: Sigma, relation: ArrowUpRight, rollup: Layers,
  createdTime: Clock, createdBy: UserPen, lastEditedTime: History, lastEditedBy: UserPen, id: Fingerprint, place: MapPin,
};
const VIEW_ICONS: Record<ViewKind, LucideIcon> = {
  table: TableIcon, board: Columns3, calendar: CalendarDays, timeline: GanttChart, list: ListIcon, gallery: LayoutGrid, chart: ChartColumn,
};

export function KindIcon({ kind, size = 14, className }: { kind: PropKind; size?: number; className?: string }) {
  const I = KIND_ICONS[kind] ?? AlignLeft;
  return <I size={size} className={className} aria-hidden />;
}
export function ViewIcon({ kind, size = 14, className }: { kind: ViewKind; size?: number; className?: string }) {
  const I = VIEW_ICONS[kind] ?? TableIcon;
  return <I size={size} className={className} aria-hidden />;
}
