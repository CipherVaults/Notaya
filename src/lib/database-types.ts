// Mirrors src-tauri/src/database/types.rs and the results of src-tauri/src/commands/database.rs (A35).
// Values are computed in Rust; the UI only shows them and sends edits back.

export type PropKind =
  | "title"
  | "text"
  | "number"
  | "select"
  | "multiSelect"
  | "status"
  | "date"
  | "person"
  | "files"
  | "checkbox"
  | "url"
  | "email"
  | "phone"
  | "formula"
  | "relation"
  | "rollup"
  | "createdTime"
  | "createdBy"
  | "lastEditedTime"
  | "lastEditedBy"
  | "id"
  | "place";

export type OptionColor = "" | "gray" | "brown" | "orange" | "yellow" | "green" | "blue" | "purple" | "pink" | "red";
export type StatusGroup = "todo" | "inProgress" | "complete";

export interface SelectOption {
  id: string;
  name: string;
  color: OptionColor;
  /** Status only. */
  group: StatusGroup | "";
}

export interface Property {
  id: string;
  name: string;
  kind: PropKind;
  options: SelectOption[];
  /** "number" | "numberWithCommas" | "percent" | "dollar" | "euro" | "pound" | "yen" | "rupee" | "won" | "yuan" | "rial" */
  numberFormat: string;
  /** "" | "bar" | "ring" */
  numberDisplay: string;
  numberMax: number;
  /** "full" | "short" | "dmy" | "mdy" | "ymd" | "relative" */
  dateFormat: string;
  formula: string;
  relationDb: string;
  /** Id of the property on the other side, "" for one-way, "+" (when saving) to create the other side. */
  relationBack: string;
  relationLimitOne: boolean;
  rollupRelation: string;
  rollupProp: string;
  rollupCalc: string;
  idPrefix: string;
  description: string;
}

export interface Cond {
  prop: string;
  op: string;
  value: string;
  value2: string;
  values: string[];
  /** Date conditions: "exact" | "today" | "tomorrow" | "yesterday" | "oneWeekAgo" | "oneWeekFromNow" | "oneMonthAgo" | "oneMonthFromNow" | "thisWeek" | "pastWeek" | "pastMonth" | "nextWeek" | "nextMonth" | "pastYear" | "nextYear" | "thisMonth" | "thisYear" */
  rel: string;
}

export interface Filter {
  op: "and" | "or";
  conds: Cond[];
  groups: Filter[];
}

export interface Sort {
  prop: string;
  descending: boolean;
}

export interface GroupBy {
  prop: string;
  /** "manual" | "asc" | "desc" */
  sort: string;
  /** Dates: "day" | "week" | "month" | "year" | "relative"; status: "group" groups by To-do / In progress / Complete. */
  dateBy: string;
  /** Numbers: bucket size (0 = every value). */
  numberStep: number;
  hideEmpty: boolean;
  hidden: string[];
  collapsed: string[];
}

export interface Column {
  prop: string;
  visible: boolean;
  /** Pixels; 0 = default. */
  width: number;
}

export interface ColCalc {
  prop: string;
  calc: string;
}

export type ViewKind = "table" | "board" | "calendar" | "timeline" | "list" | "gallery" | "chart";

export interface ChartConfig {
  /** "bar" | "line" | "donut" | "number" */
  kind: string;
  xProp: string;
  xBy: string;
  xStep: number;
  xSort: string;
  groupProp: string;
  yCalc: string;
  yProp: string;
  stacked: boolean;
  horizontal: boolean;
  showLabels: boolean;
  hideEmpty: boolean;
  color: string;
  title: string;
}

export interface View {
  id: string;
  name: string;
  kind: ViewKind;
  filter: Filter;
  sorts: Sort[];
  group: GroupBy | null;
  subGroup: GroupBy | null;
  columns: Column[];
  calcs: ColCalc[];
  freeze: number;
  wrap: boolean;
  rowNumbers: boolean;
  /** "full" | "side" | "center" */
  openIn: string;
  cardSize: string;
  /** "" | "page" | "none" | a files property id */
  cardCover: string;
  cardFit: boolean;
  dateProp: string;
  endProp: string;
  calendarMode: string;
  timelineZoom: string;
  showDeps: boolean;
  showSubItems: boolean;
  chart: ChartConfig;
}

export interface Template {
  id: string;
  name: string;
  icon: string;
  /** JSON object text: property id -> value. */
  cells: string;
  content: string;
  text: string;
  /** "" | "daily" | "weekly" | "monthly" */
  repeat: string;
  repeatLast: number;
}

export interface Database {
  id: string;
  sectionId: string;
  parent: string;
  inline: boolean;
  title: string;
  icon: string;
  description: string;
  order: number;
  favorite: boolean;
  locked: boolean;
  props: Property[];
  views: View[];
  templates: Template[];
  defaultTemplate: string;
  nextId: number;
  subParentProp: string;
  subChildProp: string;
  blockedByProp: string;
  blockingProp: string;
  trashedAt: number | null;
  created: number;
  updated: number;
}

/** The tree entry of a database (in `NotesTree.databases`). */
export interface DatabaseMeta {
  id: string;
  sectionId: string;
  parent: string;
  inline: boolean;
  title: string;
  icon: string;
  favorite: boolean;
  locked: boolean;
  order: number;
  rows: number;
  trashedAt: number | null;
  created: number;
  updated: number;
}

/** A date cell as stored. */
export interface DateCell {
  /** "YYYY-MM-DD" or "YYYY-MM-DDTHH:MM" */
  start: string;
  end?: string;
  /** Minutes before the start; a local notification. */
  reminder?: number;
}

export interface FileCell {
  id: string;
  name: string;
  mime: string;
  size: number;
}

export interface PlaceCell {
  address: string;
  lat: number | null;
  lng: number | null;
}

/** A computed cell (formula, rollup, created/edited time and by, ID). `t` is the value type. */
export interface ComputedCell {
  t: "number" | "text" | "bool" | "date" | "list" | "null" | "error";
  v: unknown;
  /** The text to show. */
  text: string;
}

/**
 * One row of a result. `values[propId]` is the stored JSON for stored kinds:
 * text/url/email/phone: string; number: number; select/status: option id; multiSelect: option ids;
 * date: DateCell; person: string[]; files: FileCell[]; checkbox: boolean; relation: row ids; place: PlaceCell.
 * For computed kinds (formula, rollup, createdTime, createdBy, lastEditedTime, lastEditedBy, id) it is a ComputedCell.
 * The title is `title`.
 */
export interface DbRow {
  id: string;
  title: string;
  icon: string;
  number: number;
  created: number;
  updated: number;
  favorite: boolean;
  values: Record<string, unknown>;
}

export interface GroupOut {
  key: string;
  label: string;
  color: string;
  count: number;
  /** Row ids in this group, in view order. */
  rows: string[];
  subs: GroupOut[];
  /** Footer calculations as text, by property id. */
  calcs: Record<string, string>;
}

export interface ViewResult {
  rows: DbRow[];
  /** Rows in the database, rows that passed the filter, true when the list was cut. */
  total: number;
  filtered: number;
  truncated: boolean;
  groups: GroupOut[] | null;
  calcs: Record<string, string>;
}

export interface QueryRequest {
  databaseId: string;
  viewId: string;
  /** A view being edited and not saved yet; it replaces the saved one for this query. */
  view?: View;
}

export interface RowHit {
  id: string;
  title: string;
  icon: string;
}

export interface DatabaseFull {
  database: Database;
  notice: string | null;
}

export interface ChartPoint {
  label: string;
  value: number;
  /** Text of the value (formatted). */
  text: string;
  color: string;
}

export interface ChartSeries {
  name: string;
  color: string;
  points: ChartPoint[];
}

export interface ChartData {
  /** Labels along the x axis (or the slices of a donut), in order. */
  labels: string[];
  series: ChartSeries[];
  /** "number" charts: the single value as text. */
  total: string | null;
  /** Number of rows that went into the chart. */
  rows: number;
}

/** The calculations a column footer or a rollup can use (see `CALCS` in value.rs). */
export const CALC_NAMES = [
  "none", "countAll", "countValues", "countUnique", "countEmpty", "countNotEmpty", "percentEmpty", "percentNotEmpty",
  "sum", "avg", "median", "min", "max", "range", "earliest", "latest", "dateRange",
  "checked", "unchecked", "percentChecked", "percentUnchecked", "showOriginal", "showUnique",
] as const;
