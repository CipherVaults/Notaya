// Applies appearance settings to the page. A copy of the (non-secret) appearance
// is cached in local storage so the lock screen can look right before unlocking.

import { api } from "./ipc";
import { getKey, setKey } from "./storage";
import type { Settings } from "./types";

export type Appearance = Pick<
  Settings,
  "theme" | "accent" | "backdrop" | "font" | "fontSize" | "editorWidth" | "density" | "unlockBackground"
>;

const KEY = "appearance";

export const DEFAULT_APPEARANCE: Appearance = {
  theme: "sunrise",
  accent: "mono",
  backdrop: "none",
  font: "inter",
  fontSize: 15,
  editorWidth: "normal",
  density: "comfortable",
  unlockBackground: "midnight",
};

const FONTS: Record<Appearance["font"], string> = {
  inter: `"Inter Variable", "Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif`,
  system: `"Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif`,
  serif: `Georgia, Cambria, "Times New Roman", serif`,
  mono: `"JetBrains Mono Variable", Consolas, monospace`,
};

const WIDTHS: Record<Appearance["editorWidth"], string> = {
  narrow: "760px",
  normal: "1040px",
  wide: "1320px",
  full: "1680px",
};

export function cachedAppearance(): Appearance {
  try {
    const raw = getKey(KEY);
    if (raw) {
      const a = { ...DEFAULT_APPEARANCE, ...JSON.parse(raw) };
      // Earlier versions called the first two looks "dark" and "light".
      if ((a.theme as string) === "dark") a.theme = "black";
      if ((a.theme as string) === "light") a.theme = "white";
      return a;
    }
  } catch {
    // Storage unavailable: fall back to defaults.
  }
  return DEFAULT_APPEARANCE;
}

export type ResolvedTheme = Exclude<Appearance["theme"], "system">;

const LOOKS: readonly ResolvedTheme[] = ["sunrise", "sunrise-dark", "black", "graphite", "white", "paper"];

/** The look that is actually shown ("system" follows Windows). */
export function resolveTheme(theme: Appearance["theme"]): ResolvedTheme {
  if (theme === "system") return window.matchMedia("(prefers-color-scheme: dark)").matches ? "black" : "white";
  return LOOKS.includes(theme) ? theme : "black";
}

/** The coral accent of the Sunrise looks. */
export const SUNRISE_ACCENT = "#e8553f";

export function isDark(theme: Appearance["theme"]): boolean {
  const t = resolveTheme(theme);
  return t === "black" || t === "graphite" || t === "sunrise-dark";
}

/**
 * "mono" is the theme's own accent: coral in the two Sunrise looks, and otherwise
 * black and white (white accents on dark, black on light).
 * Any other value is a color, and the text on it is chosen for contrast.
 */
export function resolveAccent(value: string, dark: boolean, look?: ResolvedTheme): { accent: string; onAccent: string } {
  if (value === "mono" || !/^#[0-9a-f]{6}$/i.test(value)) {
    if (look === "sunrise" || look === "sunrise-dark") return { accent: SUNRISE_ACCENT, onAccent: "#ffffff" };
    return dark ? { accent: "#ffffff", onAccent: "#000000" } : { accent: "#0a0a0a", onAccent: "#ffffff" };
  }
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(value.slice(i, i + 2), 16) / 255);
  const lin = (c: number) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
  const lum = 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
  return { accent: value, onAccent: lum > 0.4 ? "#000000" : "#ffffff" };
}

/** Sets CSS variables and, for the main window, the Windows backdrop material. */
let appearanceRevision = 0;
export async function applyAppearance(a: Appearance, applyBackdrop: boolean) {
  const revision = ++appearanceRevision;
  const root = document.documentElement;
  const dark = isDark(a.theme);
  const look = resolveTheme(a.theme);
  root.dataset.theme = look;
  root.dataset.mode = dark ? "dark" : "light";
  root.dataset.density = a.density;
  root.dataset.accent = a.accent === "mono" ? "mono" : "color";
  const { accent, onAccent } = resolveAccent(a.accent, dark, look);
  root.style.setProperty("--accent", accent);
  root.style.setProperty("--accent-fg", onAccent);
  root.style.setProperty("--app-font", FONTS[a.font] ?? FONTS.inter);
  root.style.setProperty("--app-font-size", `${a.fontSize}px`);
  root.style.setProperty("--editor-width", WIDTHS[a.editorWidth] ?? WIDTHS.normal);

  if (applyBackdrop) {
    try {
      const on = await api.setBackdrop(a.backdrop, dark);
      if (revision === appearanceRevision) root.dataset.backdrop = on ? "on" : "off";
    } catch {
      if (revision === appearanceRevision) root.dataset.backdrop = "off";
    }
  }

  // Startup no longer waits for cached backdrop IPC before loading settings.
  // A late cached response must not overwrite the newer settings/cache.
  if (revision !== appearanceRevision) return;
  try {
    const cached: Appearance = {
      theme: a.theme,
      accent: a.accent,
      backdrop: a.backdrop,
      font: a.font,
      fontSize: a.fontSize,
      editorWidth: a.editorWidth,
      density: a.density,
      unlockBackground: a.unlockBackground,
    };
    setKey(KEY, JSON.stringify(cached));
  } catch {
    // Not critical.
  }
}
