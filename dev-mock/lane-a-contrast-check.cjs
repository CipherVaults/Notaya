// Static WCAG non-text contrast checks of the icon tokens against theme surfaces.
// This is not a substitute for the required mock screenshots and rendered-icon checks.
// Run: node dev-mock/lane-a-contrast-check.cjs
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const css = fs.readFileSync(path.join(__dirname, '../src/styles/app.css'), 'utf8');
const themes = ['sunrise', 'sunrise-dark', 'black', 'graphite', 'white', 'paper'];
const rgb = (hex) => [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
const lum = (color) => color.map((c) => c / 255).map((v) => v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4)
  .reduce((sum, v, i) => sum + v * [0.2126, 0.7152, 0.0722][i], 0);
const ratio = (a, b) => (Math.max(lum(a), lum(b)) + 0.05) / (Math.min(lum(a), lum(b)) + 0.05);
const composite = (rgba, background) => {
  const [r, g, b, a] = rgba.match(/[\d.]+/g).map(Number);
  return [r, g, b].map((v, i) => v * a + background[i] * (1 - a));
};
let count = 0;
for (const theme of themes) {
  const block = css.match(new RegExp(`\\[data-theme="${theme}"\\]\\s*\\{([^}]+)\\}`))[1];
  const token = (name) => block.match(new RegExp(`--${name}:\\s*([^;]+)`))[1].trim();
  const mode = ['sunrise-dark', 'black', 'graphite'].includes(theme) ? 'dark' : 'light';
  const gold = css.match(new RegExp(`\\[data-mode="${mode}"\\]\\s*\\{ --favorite: (#[a-f0-9]+);`))[1];
  let lowest = Infinity;
  for (const base of ['bg', 'sidebar', 'elevated']) {
    for (const overlay of [null, 'surface', 'surface-2']) {
      const background = overlay ? composite(token(overlay), rgb(token(base))) : rgb(token(base));
      for (const color of [token('faint'), token('muted'), gold]) {
        const contrast = ratio(rgb(color), background);
        assert(contrast >= 3, `${theme} ${color} on ${base}/${overlay}: ${contrast.toFixed(2)}:1`);
        lowest = Math.min(lowest, contrast);
        count++;
      }
    }
  }
  console.log(`ok ${theme}: lowest tested icon-token contrast ${lowest.toFixed(2)}:1`);
}
console.log(`ALL OK (${count} static color pairs; system uses black/white; rendered UI not checked)`);
