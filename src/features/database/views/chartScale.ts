/** The position within a finite axis, including axes wider than binary64 can represent. */
export function axisFraction(value: number, min: number, max: number): number {
  const width = max - min;
  if (Number.isFinite(width)) return (value - min) / (width || 1);
  return (value / 2 - min / 2) / (max / 2 - min / 2);
}

/** Tick values that end on round numbers where the rounded bounds are representable. */
export function niceTicks(min: number, max: number, count = 5, integers = false): number[] {
  count = Math.max(1, Math.min(20, Math.floor(count) || 1));
  if (!Number.isFinite(min) || !Number.isFinite(max)) return [0, 1];
  if (max === min && min === 0) return [0, 1];
  if (max <= min) return [min];
  const raw = max / count - min / count;
  const mag = 10 ** Math.floor(Math.log10(raw));
  const norm = raw / mag;
  let step = (norm <= 1 ? 1 : norm <= 2 ? 2 : norm <= 2.5 && !integers ? 2.5 : norm <= 5 ? 5 : 10) * mag;
  if (integers) step = Math.max(1, step);
  const first = Math.floor(min / step);
  const last = Math.ceil(max / step);
  const length = last - first + 1;
  if (length >= 2 && length <= count + 3 && Number.isFinite(first * step) && Number.isFinite(last * step)) {
    const ticks = Array.from({ length }, (_, i) => (first + i) * step);
    if (ticks[0] <= min && ticks[ticks.length - 1] >= max && ticks.every((v, i) => i === 0 || v > ticks[i - 1])) return ticks;
  }
  // Convex interpolation avoids overflowing the span or rounding past a finite endpoint.
  const ticks = [min];
  for (let i = 1; i < count; i++) {
    const portion = i / count;
    const next = min * (1 - portion) + max * portion;
    if (next > ticks[ticks.length - 1] && next < max) ticks.push(next);
  }
  return [...ticks, max];
}
