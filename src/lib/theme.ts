// Applies appearance settings to the page. A copy of the (non-secret) appearance
// is cached in local storage so the lock screen can look right before unlocking.

import { api } from "./ipc";
import type { Settings } from "./types";

export type Appearance = Pick<
  Settings,
  "theme" | "accent" | "backdrop" | "font" | "fontSize" | "editorWidth" | "density" | "unlockBackground"
>;

const KEY = "cipherleaf.appearance";

export const DEFAULT_APPEARANCE: Appearance = {
  theme: "dark",
  accent: "#8b5cf6",
  backdrop: "mica",
  font: "inter",
  fontSize: 15,
  editorWidth: "normal",
  density: "comfortable",
  unlockBackground: "aurora",
};

const FONTS: Record<Appearance["font"], string> = {
  inter: `"Inter Variable", "Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif`,
  system: `"Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif`,
  serif: `Georgia, Cambria, "Times New Roman", serif`,
  mono: `"JetBrains Mono Variable", Consolas, monospace`,
};

const WIDTHS: Record<Appearance["editorWidth"], string> = {
  narrow: "640px",
  normal: "780px",
  wide: "1040px",
};

export function cachedAppearance(): Appearance {
  try {
    const raw = localStorage.getItem(KEY);
    if (raw) return { ...DEFAULT_APPEARANCE, ...JSON.parse(raw) };
  } catch {
    // Storage unavailable: fall back to defaults.
  }
  return DEFAULT_APPEARANCE;
}

export function isDark(theme: Appearance["theme"]): boolean {
  if (theme === "system") return window.matchMedia("(prefers-color-scheme: dark)").matches;
  return theme === "dark";
}

/** Sets CSS variables and, for the main window, the Windows backdrop material. */
export async function applyAppearance(a: Appearance, applyBackdrop: boolean) {
  const root = document.documentElement;
  const dark = isDark(a.theme);
  root.dataset.theme = dark ? "dark" : "light";
  root.dataset.density = a.density;
  root.style.setProperty("--accent", a.accent);
  root.style.setProperty("--app-font", FONTS[a.font] ?? FONTS.inter);
  root.style.setProperty("--app-font-size", `${a.fontSize}px`);
  root.style.setProperty("--editor-width", WIDTHS[a.editorWidth] ?? WIDTHS.normal);

  if (applyBackdrop) {
    try {
      const on = await api.setBackdrop(a.backdrop, dark);
      root.dataset.backdrop = on ? "on" : "off";
    } catch {
      root.dataset.backdrop = "off";
    }
  }

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
    localStorage.setItem(KEY, JSON.stringify(cached));
  } catch {
    // Not critical.
  }
}
