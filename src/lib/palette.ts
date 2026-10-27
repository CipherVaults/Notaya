// Color choices offered in the UI.

export const ACCENTS = [
  { name: "Theme color", value: "mono" },
  { name: "Blue", value: "#3b82f6" },
  { name: "Sky", value: "#0ea5e9" },
  { name: "Teal", value: "#14b8a6" },
  { name: "Green", value: "#22c55e" },
  { name: "Amber", value: "#f59e0b" },
  { name: "Orange", value: "#f97316" },
  { name: "Rose", value: "#f43f5e" },
  { name: "Violet", value: "#8b5cf6" },
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

/** Highlight colors are translucent, so they read well on every theme. */
export const HIGHLIGHTS = [
  { name: "Gray", value: "rgba(155, 154, 151, 0.35)" },
  { name: "Brown", value: "rgba(160, 115, 90, 0.38)" },
  { name: "Orange", value: "rgba(224, 123, 57, 0.36)" },
  { name: "Yellow", value: "rgba(236, 189, 40, 0.38)" },
  { name: "Green", value: "rgba(79, 157, 105, 0.36)" },
  { name: "Blue", value: "rgba(59, 130, 196, 0.36)" },
  { name: "Purple", value: "rgba(144, 101, 176, 0.38)" },
  { name: "Pink", value: "rgba(193, 76, 138, 0.36)" },
  { name: "Red", value: "rgba(229, 72, 77, 0.36)" },
];

export const TEXT_COLORS = [
  { name: "Default", value: "" },
  { name: "Gray", value: "#9b9a97" },
  { name: "Brown", value: "#b0805f" },
  { name: "Orange", value: "#e07b39" },
  { name: "Yellow", value: "#d9a21b" },
  { name: "Green", value: "#4f9d69" },
  { name: "Blue", value: "#3b82c4" },
  { name: "Purple", value: "#9065b0" },
  { name: "Pink", value: "#c14c8a" },
  { name: "Red", value: "#e5484d" },
];

export const COVER_GRADIENTS = [
  { name: "Ink", value: "ink" },
  { name: "Snow", value: "snow" },
  { name: "Dusk", value: "dusk" },
  { name: "Ocean", value: "ocean" },
  { name: "Forest", value: "forest" },
  { name: "Ember", value: "ember" },
  { name: "Rose", value: "rose" },
  { name: "Sand", value: "sand" },
];

export const PAGE_EMOJIS = [
  "📄", "📝", "📓", "📚", "💡", "✅", "📌", "⭐", "🎯", "🚀", "🧠", "🔒",
  "🗂️", "📅", "🏠", "✈️", "🎓", "🧪", "🎨", "🎵", "💼", "💰", "🛒", "🍽️",
  "❤️", "🌱", "☀️", "🌙", "🔥", "⚙️", "🔧", "📷", "🎮", "📖", "🧾", "🗒️",
];
