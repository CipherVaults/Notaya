// Color choices offered in the UI.

export const ACCENTS = [
  { name: "Violet", value: "#8b5cf6" },
  { name: "Indigo", value: "#6366f1" },
  { name: "Sky", value: "#0ea5e9" },
  { name: "Teal", value: "#14b8a6" },
  { name: "Emerald", value: "#10b981" },
  { name: "Amber", value: "#f59e0b" },
  { name: "Rose", value: "#f43f5e" },
  { name: "Fuchsia", value: "#d946ef" },
];

export const TAG_COLORS = [
  "#8b5cf6",
  "#6366f1",
  "#0ea5e9",
  "#14b8a6",
  "#22c55e",
  "#eab308",
  "#f97316",
  "#f43f5e",
  "#ec4899",
  "#64748b",
];

/** Sticky-note paper and ink colors. */
export const STICKY_COLORS: Record<string, { paper: string; ink: string; bar: string; label: string }> = {
  yellow: { paper: "#fff3a6", ink: "#3d3510", bar: "#f8e374", label: "Lemon" },
  pink: { paper: "#ffd3e6", ink: "#4a1630", bar: "#ffb3d4", label: "Blossom" },
  green: { paper: "#c9f5d4", ink: "#123b22", bar: "#a6ebba", label: "Mint" },
  blue: { paper: "#cfe6ff", ink: "#12304d", bar: "#aed3ff", label: "Sky" },
  purple: { paper: "#e3d6ff", ink: "#2b1a52", bar: "#cdb8ff", label: "Lavender" },
  orange: { paper: "#ffdcbf", ink: "#4a2510", bar: "#ffc497", label: "Peach" },
  gray: { paper: "#e9e9ef", ink: "#25252e", bar: "#d6d6de", label: "Cloud" },
  charcoal: { paper: "#2a2a33", ink: "#ececf3", bar: "#36363f", label: "Charcoal" },
};

export function stickyColor(name: string) {
  return STICKY_COLORS[name] ?? STICKY_COLORS.yellow;
}

export const NOTEBOOK_ICONS = ["📓", "📘", "📗", "📕", "📙", "🗂️", "💼", "🏠", "✈️", "💡", "🎓", "🧪", "🎨", "💰", "❤️", "🔒"];

export const HIGHLIGHTS = [
  { name: "Yellow", value: "#fde68a" },
  { name: "Green", value: "#bbf7d0" },
  { name: "Blue", value: "#bfdbfe" },
  { name: "Pink", value: "#fbcfe8" },
  { name: "Purple", value: "#ddd6fe" },
];

export const TEXT_COLORS = [
  { name: "Default", value: "" },
  { name: "Violet", value: "#a78bfa" },
  { name: "Blue", value: "#60a5fa" },
  { name: "Green", value: "#34d399" },
  { name: "Amber", value: "#fbbf24" },
  { name: "Red", value: "#f87171" },
  { name: "Gray", value: "#9ca3af" },
];
