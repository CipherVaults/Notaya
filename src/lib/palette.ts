// Color choices offered in the UI.

export const ACCENTS = [
  { name: "Theme color", value: "mono" },
  { name: "Blue", value: "#3b82f6" },
  { name: "Sky", value: "#0ea5e9" },
  { name: "Teal", value: "#14b8a6" },
  { name: "Green", value: "#22c55e" },
  { name: "Amber", value: "#f59e0b" },
  { name: "Orange", value: "#f97316" },
  { name: "Pink", value: "#f43f5e" },
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
  orange: { paper: "#ffdcbf", ink: "#4a2510", bar: "#ffc497", label: "Orange" },
  gray: { paper: "#e9e9ef", ink: "#25252e", bar: "#d6d6de", label: "Cloud" },
  charcoal: { paper: "#2a2a33", ink: "#ececf3", bar: "#36363f", label: "Charcoal" },
};

export function stickyColor(name: string) {
  return STICKY_COLORS[name] ?? STICKY_COLORS.yellow;
}

export const NOTEBOOK_ICONS = ["📓", "📘", "📗", "📕", "📙", "🗂️", "💼", "🏠", "✈️", "💡", "🎓", "🧪", "🎨", "💰", "❤️", "🔒"];

/**
 * Highlight colors for text. They are matte: soft, low-saturation tints, and translucent, so the
 * same saved value looks right on every light and dark theme and the text stays readable on top.
 * The names and the order are Notion's (Gray, Brown, Orange, Yellow, Green, Blue, Purple, Pink, Red), so a page
 * imported from Notion maps color for color. (Older pages may hold other values; they still show, only these are offered.)
 */
export const HIGHLIGHTS = [
  { name: "Gray", value: "rgba(140, 142, 148, 0.30)" },
  { name: "Brown", value: "rgba(176, 130, 105, 0.32)" },
  { name: "Peach", value: "rgba(214, 140, 90, 0.32)" },
  { name: "Yellow", value: "rgba(206, 178, 80, 0.34)" },
  { name: "Green", value: "rgba(110, 160, 125, 0.32)" },
  { name: "Blue", value: "rgba(100, 150, 190, 0.32)" },
  { name: "Purple", value: "rgba(150, 125, 190, 0.32)" },
  { name: "Pink", value: "rgba(200, 120, 150, 0.32)" },
  { name: "Red", value: "rgba(200, 105, 100, 0.32)" },
];

/**
 * Text colors: mid-tones with at least about 3.5:1 contrast on the light themes and 4.4:1 on the dark
 * ones, so a color chosen on one theme stays legible on the others. "Default" removes the color.
 */
export const TEXT_COLORS = [
  { name: "Default", value: "" },
  { name: "Gray", value: "#78838f" },
  { name: "Brown", value: "#a8765a" },
  { name: "Orange", value: "#c8683a" },
  { name: "Yellow", value: "#a67c1c" },
  { name: "Green", value: "#43905f" },
  { name: "Blue", value: "#4a8bc2" },
  { name: "Purple", value: "#8d6bc2" },
  { name: "Pink", value: "#c9557f" },
  { name: "Red", value: "#d04f4f" },
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
