// A small, safe calculator engine. It parses text itself; nothing is ever passed to eval.
//
// Supports + - × ÷ ^ ( ) ! %, unary minus, implicit multiplication (2π, 3(4+1)), the constants
// pi / π, e and ans, and the functions sqrt (or √), cbrt, abs, ln, log (base 10), log2, exp, sin, cos, tan,
// asin, acos, atan. Percent follows the usual calculator habit: "200 + 10%" is 220, "50%" is 0.5.

export class CalcError extends Error {}

export interface CalcOptions {
  /** Angles for sin/cos/tan and the results of asin/acos/atan, in degrees instead of radians. */
  degrees: boolean;
  /** The last result, available as "ans". */
  ans: number | Rational;
}

type Token =
  | { t: "num"; v: string }
  | { t: "id"; v: string }
  | { t: "op"; v: string };

const FUNCTIONS = new Set(["sqrt", "cbrt", "abs", "ln", "log", "log10", "log2", "exp", "sin", "cos", "tan", "asin", "acos", "atan"]);
const CONSTANTS = new Set(["pi", "e", "ans"]);

function tokenize(src: string): Token[] {
  const out: Token[] = [];
  let i = 0;
  const s = src.replace(/\s+/g, "");
  while (i < s.length) {
    const rest = s.slice(i);
    const num = /^(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?/.exec(rest);
    if (num) {
      // "2e" is 2 times e, and "2e3" is 2000: the exponent form needs digits.
      if (rest[num[0].length] === ".") throw new CalcError("Check the number");
      out.push({ t: "num", v: num[0] });
      i += num[0].length;
      continue;
    }
    if (rest[0] === "π" || rest[0] === "√") {
      out.push({ t: "id", v: rest[0] === "π" ? "pi" : "sqrt" });
      i += 1;
      continue;
    }
    const id = /^[a-zA-Z]+/.exec(rest);
    if (id) {
      // Take the longest known name at this point, so "2pi" and "sinpi" split correctly.
      // Digit-bearing function names require parentheses to preserve bare log arguments.
      const namedLog = /^log(?:2|10)(?=\()/i.exec(rest);
      const lower = (namedLog?.[0] ?? id[0]).toLowerCase();
      let matched = "";
      for (const n of [...FUNCTIONS, ...CONSTANTS]) if (lower.startsWith(n) && n.length > matched.length) matched = n;
      if (!matched) throw new CalcError(`Unknown name "${id[0]}"`);
      out.push({ t: "id", v: matched });
      i += matched.length;
      continue;
    }
    const ch = rest[0];
    if ("+-*/^()!%,".includes(ch)) out.push({ t: "op", v: ch });
    else if (ch === "×" || ch === "·") out.push({ t: "op", v: "*" });
    else if (ch === "÷") out.push({ t: "op", v: "/" });
    else if (ch === "−" || ch === "–") out.push({ t: "op", v: "-" });
    else throw new CalcError(`Unexpected "${ch}"`);
    i += 1;
  }
  return out;
}

const MAX_FINITE = ((1n << 53n) - 1n) << 971n;
const MAX_BITS = 16_384;
const POWER_BITS = 224;
const MAX_REFINE_BITS = 4096;
const MAX_EXPR_NODES = 128;
const MAX_ARITHMETIC_WORK = 32_000_000;
// Match Rust's maximum width: 4 * ceil(bits / 32)^2 <= 32,000,000.
const MAX_BOUNDARY_PRODUCT_BITS = 90_496n;
const PRECISION_ERROR = "Numeric precision is out of range";
const DOMAIN_ERROR = "Not a real number";
const OVERFLOW = "Too large";
const DIVISION_BY_ZERO = "Can't divide by zero";

class Work {
  private remaining = MAX_ARITHMETIC_WORK;
  charge(cost: number): void {
    if (cost > this.remaining) throw new CalcError(PRECISION_ERROR);
    this.remaining -= cost;
  }
}

type Bounds = readonly [Rational, Rational];
type LogBase = "ln" | "log10" | "log2";
type Unary = "sqrt" | "cbrt" | LogBase | "exp" | "neg" | "abs" | "intPow";
type Binary = "add" | "sub" | "mul" | "div" | "rem" | "fracPow" | "pow";
type ApproxExpr =
  | { op: "exact"; value: Rational }
  | { op: Unary; input: ApproxExpr; power?: bigint }
  | { op: Binary; left: ApproxExpr; right: ApproxExpr };
interface Interval {
  bounds: Bounds;
  bits: number;
  expr?: ApproxExpr;
  significant?: string;
}

function nextPrecision(precision: number): number {
  if (precision >= MAX_REFINE_BITS) throw new CalcError(PRECISION_ERROR);
  return Math.min(precision * 2, MAX_REFINE_BITS);
}

function boundedExpr(root: ApproxExpr): ApproxExpr | undefined {
  const seen = new Set<ApproxExpr>();
  const pending = [root];
  while (pending.length) {
    const node = pending.pop()!;
    if (seen.has(node)) continue;
    seen.add(node);
    if (seen.size > MAX_EXPR_NODES) return;
    if ("input" in node) pending.push(node.input);
    if ("left" in node) pending.push(node.left, node.right);
  }
  return root;
}

const limbCount = (width: number) => Math.ceil(width / 32);
const powerWork = (width: number) => 4 * limbCount(width) ** 2;
const pairWork = (a: number, b: number) => 32 * (limbCount(a) + limbCount(b)) ** 2;
const sqrtWork = (width: number) => 128 * limbCount(width) ** 2;
const seriesWork = (precision: number) => (Math.floor(precision / 3) + 4) * (limbCount(precision) + 1) ** 2;

const magnitude = (n: bigint) => n < 0n ? -n : n;
const bits = (n: bigint) => n.toString(2).length;
const divCeil = (n: bigint, d: bigint) => (n + d - 1n) / d;

function binaryExponent(n: bigint, d: bigint): number {
  const shift = bits(n) - bits(d);
  const below = shift >= 0 ? n < (d << BigInt(shift)) : (n << BigInt(-shift)) < d;
  return shift - Number(below);
}

function finite(n: number): number {
  if (!Number.isFinite(n)) throw new CalcError(Number.isNaN(n) ? "Not a real number" : "Too large");
  return n;
}

// ln((1 + z)/(1 - z)) = 2*z*(1 + z²/3 + z⁴/5 + ...), for 0 <= z <= 1/3.
// Keep the leading z exact so a base extremely close to 1 survives multiplication
// by a large rational exponent.
function logSeries(n: bigint, d: bigint, precision: number): [bigint, bigint] {
  const shift = BigInt(precision);
  const terms = Math.floor(precision / 3) + 4;
  const z = (n << shift) / d;
  const square = (z * z) >> shift;
  let term = 1n << shift;
  let sum = term;
  for (let k = 1; k < terms && term !== 0n; k++) {
    term = (term * square) >> shift;
    sum += term / BigInt(2 * k + 1);
  }
  // Each power loses <4 ulps. After division and doubling each summand
  // loses <2*(4/3 + 1) ulps, and the scaled tail contributes <1 ulp.
  const lo = sum * 2n;
  return [lo, lo + BigInt(5 * terms + 8)];
}

function ln2Bounds(precision: number): [bigint, bigint] {
  const [lo, hi] = logSeries(1n, 3n, precision);
  return [lo / 3n, divCeil(hi, 3n)];
}

const LN_2 = ln2Bounds(POWER_BITS);

// For x in [0, 1], each term loses <2 ulps after the exact linear term.
// When the truncated term vanishes, the remaining scaled tail is <4 ulps.
function expSeries(x: bigint, precision: number, upward: boolean): bigint {
  const shift = BigInt(precision);
  let term = 1n << shift;
  let sum = term;
  for (let k = 1n; ; k++) {
    term = ((term * x) >> shift) / k;
    if (term === 0n) return upward ? sum + 2n * k + 6n : sum;
    sum += term;
  }
}

const floatBuffer = new DataView(new ArrayBuffer(8));
function floatBits(value: number): bigint {
  floatBuffer.setFloat64(0, value);
  return floatBuffer.getBigUint64(0);
}

function integerRoot(n: bigint, degree: bigint): bigint {
  if (n <= 1n) return n;
  const width = divCeil(BigInt(bits(n)), degree);
  // At high degrees, bit trials avoid Newton's slow initial descent.
  if (degree > width) {
    let root = 0n;
    for (let bit = 1n << (width - 1n); bit !== 0n; bit >>= 1n) {
      const trial = root | bit;
      if (trial ** degree <= n) root = trial;
    }
    return root;
  }
  let root = 1n << width;
  for (;;) {
    const next = ((degree - 1n) * root + n / root ** (degree - 1n)) / degree;
    if (next >= root) return root;
    root = next;
  }
}

/** Exact rationals and certified enclosures for irrational-derived results. */
class Rational {
  readonly n: bigint;
  readonly d: bigint;
  readonly interval?: Interval;

  private constructor(n: bigint, d: bigint, interval?: Interval) {
    this.n = n;
    this.d = d;
    if (interval) this.interval = interval;
  }

  private static reduced(n: bigint, d: bigint): Rational {
    if (d === 0n) throw new CalcError(DIVISION_BY_ZERO);
    if (n === 0n) return new Rational(0n, 1n);
    if (d < 0n) {
      n = -n;
      d = -d;
    }
    let a = magnitude(n);
    let b = d;
    while (b !== 0n) [a, b] = [b, a % b];
    return new Rational(n / a, d / a);
  }

  static of(n: bigint, d = 1n): Rational {
    const result = Rational.reduced(n, d);
    if (magnitude(result.n) > MAX_FINITE * result.d) throw new CalcError(OVERFLOW);
    if (result.width() > MAX_BITS) throw new CalcError(PRECISION_ERROR);
    return result;
  }

  static decimal(s: string): Rational {
    if (!/^[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?$/i.test(s)) {
      throw new CalcError("Not a real number");
    }
    const [mantissa, exponent = "0"] = s.toLowerCase().split("e");
    const [whole, fraction = ""] = mantissa.split(".");
    const digits = (whole + fraction).replace(/^[-+]?0*/, "");
    if (!digits) return Rational.of(0n);
    const trimmed = digits.replace(/0+$/, "");
    const shift = Number(exponent) - fraction.length + digits.length - trimmed.length;
    if (shift + trimmed.length > 309) throw new CalcError(OVERFLOW);
    if (Math.abs(shift) > MAX_BITS || trimmed.length > MAX_BITS) throw new CalcError(PRECISION_ERROR);
    const n = BigInt(trimmed) * (s.startsWith("-") ? -1n : 1n);
    const scale = 10n ** BigInt(Math.abs(shift));
    return shift < 0 ? Rational.of(n, scale) : Rational.of(n * scale);
  }

  add(other: Rational): Rational {
    if (this.interval || other.interval) return this.binary("add", other);
    return Rational.of(this.n * other.d + other.n * this.d, this.d * other.d);
  }

  sub(other: Rational): Rational {
    if (this.interval || other.interval) return this.binary("sub", other);
    return Rational.of(this.n * other.d - other.n * this.d, this.d * other.d);
  }

  neg(): Rational {
    if (this.interval) return this.unary("neg");
    return new Rational(-this.n, this.d);
  }

  abs(): Rational {
    if (this.interval) return this.unary("abs");
    return new Rational(magnitude(this.n), this.d);
  }

  mul(other: Rational): Rational {
    if (this.interval || other.interval) return this.binary("mul", other);
    return Rational.of(this.n * other.n, this.d * other.d);
  }

  div(other: Rational): Rational {
    if (this.interval || other.interval) return this.binary("div", other);
    return Rational.of(this.n * other.d, this.d * other.n);
  }

  rem(other: Rational, work = new Work()): Rational {
    if (!this.interval && !other.interval) {
      work.charge(pairWork(this.width(), other.width()));
      if (other.n === 0n) throw new CalcError(DIVISION_BY_ZERO);
      return Rational.of(this.n * other.d % (other.n * this.d), this.d * other.d);
    }
    return this.binary("rem", other, work);
  }

  private sameRatio(other: Rational): boolean {
    return this.n === other.n && this.d === other.d;
  }

  private width(): number {
    return Math.max(bits(magnitude(this.n)), bits(this.d));
  }

  private static dyadic(mantissa: bigint, shift: number): Rational {
    return shift < 0
      ? Rational.reduced(mantissa, 1n << BigInt(-shift))
      : new Rational(mantissa << BigInt(shift), 1n);
  }

  private static binaryFloat(encoded: bigint): Rational {
    const biased = Number((encoded >> 52n) & 0x7ffn);
    const fraction = encoded & ((1n << 52n) - 1n);
    const significand = biased === 0 ? fraction : fraction | (1n << 52n);
    return Rational.dyadic(encoded >> 63n ? -significand : significand, biased === 0 ? -1074 : biased - 1075);
  }

  private exactRoot(degree: bigint): Rational | undefined {
    const absolute = magnitude(this.n);
    const numerator = integerRoot(absolute, degree);
    if (numerator ** degree !== absolute) return;
    const denominator = integerRoot(this.d, degree);
    if (denominator ** degree === this.d) {
      return new Rational(this.n < 0n ? -numerator : numerator, denominator);
    }
  }

  private compare(other: Rational): bigint {
    if (this.d === other.d) return this.n - other.n;
    return this.n * other.d - other.n * this.d;
  }

  compareChecked(other: Rational): bigint {
    if (!this.interval && !other.interval) return this.compare(other);
    const work = new Work();
    const aRoot = this.directRoot();
    const bRoot = other.directRoot();
    if (aRoot && bRoot) {
      if (aRoot.negative !== bRoot.negative) return aRoot.negative ? -1n : 1n;
      if (aRoot.exponent.sameRatio(bRoot.exponent)) {
        work.charge(pairWork(aRoot.base.width(), bRoot.base.width()));
        const side = aRoot.base.compare(bRoot.base);
        return aRoot.negative !== (aRoot.exponent.n < 0n) ? -side : side;
      }
    }
    for (let precision = Math.min(this.precision(), other.precision()); ; precision = nextPrecision(precision)) {
      const a = this.boundsAt(precision, work);
      const b = other.boundsAt(precision, work);
      if (!a || !b) continue;
      work.charge(pairWork(Rational.boundWidth(a), Rational.boundWidth(b)));
      if (a[1].compare(b[0]) < 0n) return -1n;
      if (a[0].compare(b[1]) > 0n) return 1n;
      const aPoint = a[0].sameRatio(a[1]);
      const bPoint = b[0].sameRatio(b[1]);
      if (aPoint && bPoint) return a[0].compare(b[0]);
      const side = bPoint ? this.compareRoot(a, b[0], work) : aPoint ? other.compareRoot(b, a[0], work) : undefined;
      if (side !== undefined) return bPoint ? side : -side;
    }
  }

  private certifyRootBoundary(exponent: Rational, [lo, hi]: Bounds, boundary: Rational, work: Work): bigint {
    if (boundary.compare(lo) < 0n) return 1n;
    if (boundary.compare(hi) > 0n) return -1n;
    const [a, c] = exponent.n < 0n ? [this.d, this.n] : [this.n, this.d];
    const p = magnitude(exponent.n);
    const q = exponent.d;
    const leftWidth = BigInt(bits(a)) * p + BigInt(bits(boundary.d)) * q;
    const rightWidth = BigInt(bits(c)) * p + BigInt(bits(boundary.n)) * q;
    if (leftWidth > MAX_BOUNDARY_PRODUCT_BITS || rightWidth > MAX_BOUNDARY_PRODUCT_BITS) {
      throw new CalcError(PRECISION_ERROR);
    }
    work.charge(powerWork(Number(leftWidth > rightWidth ? leftWidth : rightWidth)));
    return a ** p * boundary.d ** q - c ** p * boundary.n ** q;
  }

  private displayMidpoint(strict = false): Rational {
    const [scaled, denominator, scaleExponent] = this.significantParts();
    let exponent = scaleExponent;
    let whole = scaled / denominator;
    const twiceRemainder = 2n * (scaled % denominator);
    if (twiceRemainder > denominator || (strict && twiceRemainder === denominator)) {
      whole++;
      if (whole === 1_000_000_000_000n) {
        whole /= 10n;
        exponent++;
      }
    }
    const digits = whole * 10n + 5n;
    const shift = exponent - 12;
    return shift < 0
      ? Rational.of(digits, 10n ** BigInt(-shift))
      : new Rational(digits * 10n ** BigInt(shift), 1n);
  }

  private rootSignificant(bounds: Bounds, exponent: Rational, work: Work): string | undefined {
    const [lo, hi] = bounds;
    if (lo.n === 0n) return;
    const midpoint = lo.displayMidpoint();
    if (midpoint.compare(hi) > 0n || midpoint.displayMidpoint(true).compare(hi) <= 0n) return;
    const side = this.certifyRootBoundary(exponent, bounds, midpoint, work);
    return (side < 0n ? lo : side === 0n ? midpoint : hi).significantText(work);
  }

  private rootValue(bounds: Bounds, exponent: Rational, sqrt: boolean, work: Work): Rational {
    const [lo, hi] = bounds;
    const maxSide = this.certifyRootBoundary(exponent, bounds, MAX, work);
    if (maxSide > 0n) throw new CalcError(OVERFLOW);
    if (maxSide === 0n) return MAX;
    if (hi.compare(MAX) > 0n) bounds = [lo, MAX];
    if (!sqrt && this.certifyRootBoundary(exponent, bounds, CUTOFF, work) <= 0n) return ZERO;
    const significant = this.rootSignificant(bounds, exponent, work);
    const expr: ApproxExpr = sqrt
      ? { op: "sqrt", input: this.expression()! }
      : { op: "fracPow", left: this.expression()!, right: exponent.expression()! };
    return new Rational(0n, 1n, { bounds, bits: POWER_BITS, expr, significant });
  }

  // An absent result certifies that the magnitude exceeds 2^1024.
  private fractionalPowerAt(other: Rational, precision: number): Bounds | undefined {
    if (this.n === this.d || other.n === 0n) return [ONE, ONE];
    if (this.n === 0n) {
      if (other.n < 0n) throw new CalcError(DIVISION_BY_ZERO);
      return [ZERO, ZERO];
    }
    const reciprocal = this.n < this.d;
    const top = reciprocal ? this.d : this.n;
    const bottom = reciprocal ? this.n : this.d;
    const exponent = binaryExponent(top, bottom);
    const scaled = bottom << BigInt(exponent);
    const zTop = top - scaled;
    const zBottom = top + scaled;
    const [seriesLo, seriesHi] = logSeries(zTop, zBottom, precision);
    const ln2 = precision === POWER_BITS ? LN_2 : ln2Bounds(precision);
    const [ln2Lo, ln2Hi] = ln2;
    const scaledExponent = BigInt(exponent) * zBottom;
    const p = magnitude(other.n);
    const denominator = zBottom * other.d;
    const poweredLo = (scaledExponent * ln2Lo + zTop * seriesLo) * p / denominator;
    const poweredHi = divCeil((scaledExponent * ln2Hi + zTop * seriesHi) * p, denominator);
    return Rational.expBounds(poweredLo, poweredHi, reciprocal !== (other.n < 0n), precision, ln2, "fracPow");
  }

  private logBounds(precision: number, [ln2Lo, ln2Hi]: [bigint, bigint]): Bounds {
    const reciprocal = this.n < this.d;
    const top = reciprocal ? this.d : this.n;
    const bottom = reciprocal ? this.n : this.d;
    const exponent = binaryExponent(top, bottom);
    const scaled = bottom << BigInt(exponent);
    const zTop = top - scaled;
    const zBottom = top + scaled;
    const [seriesLo, seriesHi] = logSeries(zTop, zBottom, precision);
    const scale = zBottom << BigInt(precision);
    const scaledExponent = BigInt(exponent) * zBottom;
    // Keep z exact, and use a reciprocal below one to avoid cancellation.
    const bound = (ln2: bigint, series: bigint) =>
      Rational.reduced(ln2 * scaledExponent + zTop * series, scale);
    const lo = bound(ln2Lo, seriesLo);
    const hi = bound(ln2Hi, seriesHi);
    return reciprocal ? [hi.neg(), lo.neg()] : [lo, hi];
  }

  private expAt(precision: number, ln2: [bigint, bigint]): Bounds | undefined {
    if (this.n === 0n) return [ONE, ONE];
    const scaled = magnitude(this.n) << BigInt(precision);
    const lo = scaled / this.d;
    if (lo === 0n) {
      // For -1 < x < 1, 1 + x <= exp(x) <= 1/(1 - x).
      // Keep the exact linear term below the fixed-point grid.
      return [Rational.reduced(this.d + this.n, this.d), Rational.reduced(this.d, this.d - this.n)];
    }
    return Rational.expBounds(lo, divCeil(scaled, this.d), this.n < 0n, precision, ln2, "exp");
  }

  private static expBounds(
    scaledLo: bigint, scaledHi: bigint, negative: boolean, precision: number, [ln2Lo, ln2Hi]: [bigint, bigint],
    mode: "exp" | "fracPow",
  ): Bounds | undefined {
    // Reduce both bounds by the same multiple of ln(2) with outward rounding.
    const whole = scaledLo / ln2Hi;
    if (negative) {
      if (mode === "fracPow" && whole >= 1075n) return [ZERO, CUTOFF];
      // A denominator of 2^(whole + precision) needs whole + precision + 1 bits.
      if (mode === "exp" && whole >= BigInt(MAX_BITS - precision)) throw new CalcError(PRECISION_ERROR);
    } else if (whole >= 1024n) return;
    const remainderHi = scaledHi - whole * ln2Lo;
    const unit = 1n << BigInt(precision);
    if (remainderHi > unit) throw new CalcError(PRECISION_ERROR);
    const expLo = expSeries(scaledLo % ln2Hi, precision, false);
    const expHi = expSeries(remainderHi, precision, true);
    const shift = Number(negative ? -whole : whole) - precision;
    const unitSquared = unit * unit;
    return [
      Rational.dyadic(negative ? unitSquared / expHi : expLo, shift),
      Rational.dyadic(negative ? divCeil(unitSquared, expLo) : expHi, shift),
    ];
  }

  private static fractionalWork(base: number, exponent: number, precision: number): number {
    return 4 * seriesWork(precision) + 4 * pairWork(base + precision, exponent);
  }

  pow(other: Rational): Rational {
    if (other.interval) return this.binary("pow", other);
    if (this.interval) {
      if (other.d === 1n) return this.unary("intPow", other.n);
      if (other.n === 1n && other.d === 2n) return this.sqrt();
      return this.binary("fracPow", other);
    }
    if (this.n === 0n && other.n < 0n) throw new CalcError(DIVISION_BY_ZERO);
    if (other.d !== 1n) {
      if (this.n < 0n) throw new CalcError(DOMAIN_ERROR);
      if (this.n === 0n || this.n === this.d) return this;
      if (other.n === 1n && other.d === 2n) return this.sqrt();
      const work = new Work();
      const left = limbCount(this.width());
      const right = limbCount(other.width());
      work.charge(32_768 + 32 * (left * (left + right) + right * right));
      const bounds = this.fractionalPowerAt(other, POWER_BITS);
      if (!bounds) throw new CalcError(OVERFLOW);
      if (bounds[1].compare(CUTOFF) <= 0n) return ZERO;

      const degree = other.d;
      const p = magnitude(other.n);
      const width = this.width();
      const rootWidth = divCeil(BigInt(width), degree);
      if ((this.n === 1n || degree < BigInt(bits(this.n))) &&
          (this.d === 1n || degree < BigInt(bits(this.d))) &&
          rootWidth * p <= BigInt(MAX_BITS)) {
        const trialWork = powerWork(width + Number(degree));
        const rootWork = Math.max(sqrtWork(width), 2 * Number(rootWidth < degree ? rootWidth : degree) * trialWork);
        const workNeeded = rootWork + powerWork(Number(rootWidth * p));
        if (workNeeded <= MAX_ARITHMETIC_WORK) {
          work.charge(workNeeded);
          const exact = this.exactRoot(degree);
          if (exact) {
            const result = exact.pow(new Rational(other.n, 1n));
            return result.compare(CUTOFF) <= 0n ? ZERO : result;
          }
        }
      }
      return this.rootValue(bounds, other, false, work);
    }
    const exponent = magnitude(other.n);
    if (exponent === 0n) return ONE;
    if (this.n === this.d) return this;
    if (this.n === -this.d) return new Rational(exponent % 2n ? -1n : 1n, 1n);
    if (this.n === 0n) return this;
    if (exponent * BigInt(this.width()) > BigInt(MAX_BITS)) throw new CalcError(OVERFLOW);
    const top = this.n ** exponent;
    const bottom = this.d ** exponent;
    return other.n < 0n ? Rational.of(bottom, top) : Rational.of(top, bottom);
  }

  private rootAt(degree: number, precision: number): Bounds {
    if (this.n === 0n) return [this, this];
    const n = magnitude(this.n);
    const shift = Math.floor(binaryExponent(n, this.d) / degree) - precision;
    const top = n << BigInt(Math.max(0, -degree * shift));
    const bottom = this.d << BigInt(Math.max(0, degree * shift));
    // Flooring before the integer root preserves the floor on this dyadic grid.
    const mantissa = integerRoot(top / bottom, BigInt(degree));
    const lo = Rational.dyadic(mantissa, shift);
    const hi = mantissa ** BigInt(degree) * bottom === top ? lo : Rational.dyadic(mantissa + 1n, shift);
    return this.n < 0n ? [hi.neg(), lo.neg()] : [lo, hi];
  }

  sqrt(): Rational {
    if (this.interval) return this.unary("sqrt");
    if (this.n < 0n) throw new CalcError(DOMAIN_ERROR);
    if (this.n === 0n) return this;
    const work = new Work();
    work.charge(sqrtWork(this.width()));
    const exact = this.exactRoot(2n);
    if (exact) return exact;
    work.charge(Math.max(0, sqrtWork(2 * POWER_BITS) - sqrtWork(this.width())));
    return this.rootValue(this.rootAt(2, POWER_BITS), HALF, true, work);
  }

  cbrt(): Rational {
    const work = new Work();
    if (!this.interval) {
      work.charge(sqrtWork(this.width()));
      const exact = this.exactRoot(3n);
      if (exact) return exact;
    }
    return this.unary("cbrt", undefined, work);
  }

  logarithm(base: LogBase): Rational {
    const work = new Work();
    if (!this.interval) {
      if (this.n <= 0n) throw new CalcError(DOMAIN_ERROR);
      if (this.n === this.d) return ZERO;
      if (base !== "ln" && (this.n === 1n || this.d === 1n)) {
        const reciprocal = this.n === 1n;
        const value = reciprocal ? this.d : this.n;
        work.charge(powerWork(bits(value)));
        if (base === "log2" && (value & (value - 1n)) === 0n) {
          return Rational.of(BigInt(bits(value) - 1) * (reciprocal ? -1n : 1n));
        }
        if (base === "log10") {
          const digits = value.toString();
          if (/^10*$/.test(digits)) {
            return Rational.of(BigInt(digits.length - 1) * (reciprocal ? -1n : 1n));
          }
        }
      }
    }
    return this.unary(base, undefined, work);
  }

  exp(): Rational {
    return this.unary("exp");
  }

  private precision(): number {
    return this.interval?.bits ?? POWER_BITS;
  }

  private expression(): ApproxExpr | undefined {
    return this.interval ? this.interval.expr : { op: "exact", value: this };
  }

  private boundsAt(precision: number, work: Work): Bounds | undefined {
    if (!this.interval) return [this, this];
    if (precision <= this.interval.bits) return this.interval.bounds;
    if (!this.interval.expr) throw new CalcError(PRECISION_ERROR);
    return Rational.replay(this.interval.expr, precision, work);
  }

  private unary(op: Unary, power?: bigint, work = new Work()): Rational {
    const precision = this.precision();
    const initial = Rational.applyUnary(op, this.boundsAt(precision, work)!, precision, work, power);
    const input = this.expression();
    const expr = input && boundedExpr({ op, input, power });
    return Rational.finish(initial, precision, expr, work);
  }

  private binary(op: Binary, other: Rational, work = new Work()): Rational {
    const precision = Math.min(this.precision(), other.precision());
    const left = this.boundsAt(precision, work)!;
    const right = op === "div" || op === "rem" ? other.nonzeroBounds(precision, work) : other.boundsAt(precision, work)!;
    const initial = Rational.applyBinary(op, left, right, precision, work);
    const leftExpr = this.expression();
    const rightExpr = other.expression();
    const expr = leftExpr && rightExpr && boundedExpr({ op, left: leftExpr, right: rightExpr });
    return Rational.finish(initial, precision, expr, work);
  }

  private nonzeroBounds(precision: number, work: Work): Bounds {
    for (;; precision = nextPrecision(precision)) {
      const bounds = this.boundsAt(precision, work);
      if (!bounds) continue;
      const [lo, hi] = bounds;
      if (lo.n > 0n || hi.n < 0n) return bounds;
      if (lo.n === 0n && hi.n === 0n) throw new CalcError(DIVISION_BY_ZERO);
    }
  }

  private static finish(bounds: Bounds | undefined, precision: number, expr: ApproxExpr | undefined, work: Work): Rational {
    for (;;) {
      if (bounds && Rational.inRange(bounds, work)) {
        if (bounds[0].sameRatio(bounds[1])) return Rational.of(bounds[0].n, bounds[0].d);
        return new Rational(0n, 1n, { bounds, bits: precision, expr });
      }
      if (!expr) throw new CalcError(PRECISION_ERROR);
      precision = nextPrecision(precision);
      bounds = Rational.replay(expr, precision, work);
    }
  }

  private static replay(
    expr: ApproxExpr, precision: number, work: Work, memo = new Map<ApproxExpr, Bounds | undefined>(),
  ): Bounds | undefined {
    if (memo.has(expr)) return memo.get(expr);
    const bounds = Rational.replayNode(expr, precision, work, memo);
    memo.set(expr, bounds);
    return bounds;
  }

  private static replayNode(
    expr: ApproxExpr, precision: number, work: Work, memo: Map<ApproxExpr, Bounds | undefined>,
  ): Bounds | undefined {
    if (expr.op === "exact") return [expr.value, expr.value];
    if (expr.op === "fracPow" && expr.left.op === "exact" && expr.right.op === "exact") {
      const base = expr.left.value;
      const power = expr.right.value;
      work.charge(Rational.fractionalWork(base.width(), power.width(), precision));
      const bounds = base.fractionalPowerAt(power, precision);
      if (!bounds) throw new CalcError(OVERFLOW);
      if (base.certifyRootBoundary(power, bounds, CUTOFF, work) <= 0n) return [ZERO, ZERO];
      const side = base.certifyRootBoundary(power, bounds, MAX, work);
      if (side > 0n) throw new CalcError(OVERFLOW);
      if (side === 0n) return [MAX, MAX];
      return bounds[1].compare(MAX) > 0n ? [bounds[0], MAX] : bounds;
    }
    let bounds: Bounds | undefined;
    if ("input" in expr) {
      const input = Rational.replay(expr.input, precision, work, memo);
      if (input) bounds = Rational.applyUnary(expr.op, input, precision, work, expr.power);
    } else {
      const left = Rational.replay(expr.left, precision, work, memo);
      const right = Rational.replay(expr.right, precision, work, memo);
      if (left && right) bounds = Rational.applyBinary(expr.op, left, right, precision, work);
    }
    return bounds && Rational.inRange(bounds, work) ? bounds : undefined;
  }

  private static boundWidth([lo, hi]: Bounds): number {
    return Math.max(lo.width(), hi.width());
  }

  private static inRange(bounds: Bounds, work: Work): boolean {
    const [lo, hi] = bounds;
    if ([lo, hi].every((x) => bits(magnitude(x.n)) - bits(x.d) < 1023)) return true;
    work.charge(pairWork(Rational.boundWidth(bounds), MAX.width()));
    if (lo.compare(MAX) > 0n || hi.compare(MIN) < 0n) throw new CalcError(OVERFLOW);
    return hi.compare(MAX) <= 0n && lo.compare(MIN) >= 0n;
  }

  private static compact(bounds: Bounds, precision: number, work: Work): Bounds {
    const [lo, hi] = bounds;
    const width = Rational.boundWidth(bounds);
    if (width <= precision + 64 || lo.sameRatio(hi)) return bounds;
    work.charge(pairWork(width, Math.max(width, precision + 32)));
    // Use a common grid relative to the largest magnitude.
    const exponent = Math.max(...bounds.filter((x) => x.n !== 0n).map((x) => binaryExponent(magnitude(x.n), x.d)));
    const shift = exponent - precision - 32;
    const round = (value: Rational, down: boolean) => {
      const n = value.n << BigInt(Math.max(0, -shift));
      const d = value.d << BigInt(Math.max(0, shift));
      const direction = down ? -1n : 1n;
      const adjust = n * direction > 0n && n % d !== 0n ? direction : 0n;
      return Rational.dyadic(n / d + adjust, shift);
    };
    return [round(lo, true), round(hi, false)];
  }

  private static applyUnary(op: Unary, bounds: Bounds, precision: number, work: Work, power?: bigint): Bounds | undefined {
    const [lo, hi] = bounds;
    work.charge(4 * limbCount(Rational.boundWidth(bounds)));
    switch (op) {
      case "neg":
        return [hi.neg(), lo.neg()];
      case "abs":
        if (lo.n >= 0n) return bounds;
        if (hi.n <= 0n) return [hi.neg(), lo.neg()];
        return [ZERO, lo.neg().compare(hi) > 0n ? lo.neg() : hi];
      case "sqrt":
        if (hi.n < 0n) throw new CalcError(DOMAIN_ERROR);
        if (lo.n < 0n) return;
        work.charge(2 * sqrtWork(Math.max(Rational.boundWidth(bounds), 2 * precision)));
        return [lo.rootAt(2, precision)[0], hi.rootAt(2, precision)[1]];
      case "cbrt":
        work.charge(2 * sqrtWork(Math.max(Rational.boundWidth(bounds), 3 * precision)));
        return [lo.rootAt(3, precision)[0], hi.rootAt(3, precision)[1]];
      case "ln":
      case "log2":
      case "log10":
        return Rational.logarithmBounds(op, bounds, precision, work);
      case "exp":
        return Rational.exponentialBounds(bounds, precision, work);
      case "intPow":
        return Rational.intPow(bounds, power!, precision, work);
    }
  }

  private static logarithmBounds(base: LogBase, bounds: Bounds, precision: number, work: Work): Bounds | undefined {
    const [lo, hi] = bounds;
    if (hi.n <= 0n) throw new CalcError(DOMAIN_ERROR);
    if (lo.n <= 0n) return;
    work.charge((base === "log10" ? 4 : 3) * seriesWork(precision) + 2 * pairWork(Rational.boundWidth(bounds), precision));
    const ln2 = precision === POWER_BITS ? LN_2 : ln2Bounds(precision);
    const lower = lo.logBounds(precision, ln2);
    const result: Bounds = [lower[0], lo.sameRatio(hi) ? lower[1] : hi.logBounds(precision, ln2)[1]];
    if (base === "ln") return Rational.compact(result, precision, work);
    const divisor: Bounds = base === "log10"
      ? new Rational(10n, 1n).logBounds(precision, ln2)
      : [Rational.dyadic(ln2[0], -precision), Rational.dyadic(ln2[1], -precision)];
    return Rational.applyBinary("div", result, divisor, precision, work);
  }

  private static exponentialBounds(bounds: Bounds, precision: number, work: Work): Bounds | undefined {
    const [lo, hi] = bounds;
    if (lo.n === 0n && hi.n === 0n) return [ONE, ONE];
    work.charge(5 * seriesWork(precision) + 2 * pairWork(Rational.boundWidth(bounds), precision));
    const ln2 = precision === POWER_BITS ? LN_2 : ln2Bounds(precision);
    const lower = lo.expAt(precision, ln2);
    if (!lower) throw new CalcError(OVERFLOW);
    const upper = lo.sameRatio(hi) ? lower : hi.expAt(precision, ln2);
    if (!upper) return;
    const result: Bounds = [lower[0], upper[1]];
    if (Rational.boundWidth(result) > MAX_BITS) throw new CalcError(PRECISION_ERROR);
    return result;
  }

  private static intPow(bounds: Bounds, power: bigint, precision: number, work: Work): Bounds | undefined {
    if (power === 0n) return [ONE, ONE];
    let [lo, hi] = bounds;
    if (power < 0n) {
      if (lo.n <= 0n && hi.n >= 0n) {
        if (lo.n === 0n && hi.n === 0n) throw new CalcError(DIVISION_BY_ZERO);
        return;
      }
      work.charge(pairWork(Rational.boundWidth(bounds), 1));
      [lo, hi] = [Rational.reduced(hi.d, hi.n), Rational.reduced(lo.d, lo.n)];
    }
    const exponent = magnitude(power);
    if (exponent % 2n === 0n) {
      [lo, hi] = Rational.applyUnary("abs", [lo, hi], precision, work)!;
    }
    const projected = BigInt(Rational.boundWidth([lo, hi])) * exponent;
    work.charge(2 * powerWork(Number(projected)));
    const powerEndpoint = (n: Rational): Bounds => {
      let count = exponent;
      let base: Bounds = [n, n];
      let result: Bounds = [ONE, ONE];
      while (count !== 0n) {
        if (count & 1n) result = Rational.compact(Rational.products(result, base, false, work), precision, work);
        count >>= 1n;
        if (count !== 0n) base = Rational.compact(Rational.products(base, base, false, work), precision, work);
      }
      return result;
    };
    return Rational.compact([powerEndpoint(lo)[0], powerEndpoint(hi)[1]], precision, work);
  }

  private static products(left: Bounds, right: Bounds, divide: boolean, work: Work): Bounds {
    work.charge(pairWork(Rational.boundWidth(left), Rational.boundWidth(right)));
    const values = left.flatMap((a) => right.map((b) => divide
      ? Rational.reduced(a.n * b.d, a.d * b.n)
      : Rational.reduced(a.n * b.n, a.d * b.d)));
    values.sort((a, b) => {
      const side = a.compare(b);
      return side < 0n ? -1 : side > 0n ? 1 : 0;
    });
    return [values[0], values[3]];
  }

  private static applyBinary(op: Binary, left: Bounds, right: Bounds, precision: number, work: Work): Bounds | undefined {
    const [a, b] = left;
    const [c, d] = right;
    let bounds: Bounds;
    switch (op) {
      case "add":
      case "sub": {
        work.charge(pairWork(Rational.boundWidth(left), Rational.boundWidth(right)));
        const combine = (x: Rational, y: Rational, sign: bigint) =>
          Rational.reduced(x.n * y.d + sign * y.n * x.d, x.d * y.d);
        bounds = op === "add" ? [combine(a, c, 1n), combine(b, d, 1n)] : [combine(a, d, -1n), combine(b, c, -1n)];
        break;
      }
      case "div":
      case "rem":
        if (c.n <= 0n && d.n >= 0n) {
          if (c.n === 0n && d.n === 0n) throw new CalcError(DIVISION_BY_ZERO);
          return;
        }
        bounds = Rational.products(left, right, true, work);
        if (op === "rem") {
          work.charge(pairWork(Rational.boundWidth(bounds), 1));
          const units = bounds[0].n / bounds[0].d;
          if (units !== bounds[1].n / bounds[1].d) return;
          const multiplier = new Rational(units, 1n);
          return Rational.applyBinary("sub", left, Rational.products(right, [multiplier, multiplier], false, work), precision, work);
        }
        break;
      case "mul":
        bounds = Rational.products(left, right, false, work);
        break;
      case "fracPow":
      case "pow":
        return Rational.realPow(left, right, precision, work);
    }
    return Rational.compact(bounds, precision, work);
  }

  private static realPow(left: Bounds, right: Bounds, precision: number, work: Work): Bounds | undefined {
    const [a, b] = left;
    const [c, d] = right;
    if (c.sameRatio(d) && c.d === 1n) return Rational.intPow(left, c.n, precision, work);
    if (b.n < 0n) {
      work.charge(pairWork(c.width(), d.width()));
      if (c.ceil().n > d.floor().n) throw new CalcError(DOMAIN_ERROR);
      return;
    }
    if (a.n < 0n) return;
    if (a.n === 0n && c.n <= 0n) {
      if (b.n === 0n && d.n < 0n) throw new CalcError(DIVISION_BY_ZERO);
      return;
    }
    // log(x)*y attains its extrema at corners of the rectangle.
    const exponents = c.sameRatio(d) ? [c] : [c, d];
    const count = 2 * exponents.length;
    work.charge(count * Rational.fractionalWork(Rational.boundWidth(left), Rational.boundWidth(right), precision));
    let result: Bounds | undefined;
    let overflow = 0;
    for (const base of left) {
      for (const exponent of exponents) {
        const bounds = base.fractionalPowerAt(exponent, precision);
        if (!bounds) overflow++;
        else if (!result) result = bounds;
        else result = [
          bounds[0].compare(result[0]) < 0n ? bounds[0] : result[0],
          bounds[1].compare(result[1]) > 0n ? bounds[1] : result[1],
        ];
      }
    }
    if (overflow === count) throw new CalcError(OVERFLOW);
    if (overflow || !result) return;
    if (result[1].compare(CUTOFF) <= 0n) return [ZERO, ZERO];
    return result[0].compare(CUTOFF) > 0n ? result : undefined;
  }

  private directRoot(): { base: Rational; exponent: Rational; negative: boolean } | undefined {
    let expr = this.interval?.expr;
    let negative = false;
    let absolute = false;
    while (expr?.op === "neg" || expr?.op === "abs") {
      if (expr.op === "abs") absolute = true;
      else if (!absolute) negative = !negative;
      expr = expr.input;
    }
    if (expr?.op === "sqrt" && expr.input.op === "exact") {
      return { base: expr.input.value, exponent: HALF, negative };
    }
    if (expr?.op === "fracPow" && expr.left.op === "exact" && expr.right.op === "exact") {
      return { base: expr.left.value, exponent: expr.right.value, negative };
    }
  }

  private compareRoot(bounds: Bounds, boundary: Rational, work: Work): bigint | undefined {
    const root = this.directRoot();
    if (!root) return;
    const { base, exponent, negative } = root;
    if (negative) {
      if (boundary.n >= 0n) return -1n;
      return -base.certifyRootBoundary(exponent, [bounds[1].neg(), bounds[0].neg()], boundary.neg(), work);
    }
    if (boundary.n <= 0n) return 1n;
    return base.certifyRootBoundary(exponent, bounds, boundary, work);
  }

  private observe<T>(project: (lo: Rational, hi: Rational) => T | undefined, work: Work): T {
    for (let precision = this.precision(); ; precision = nextPrecision(precision)) {
      const bounds = this.boundsAt(precision, work);
      if (!bounds) continue;
      if (this.interval) work.charge(8 * (limbCount(Rational.boundWidth(bounds)) + 2) ** 2);
      const value = project(...bounds);
      if (value !== undefined) return value;
    }
  }

  number(work = new Work()): number {
    if (!this.interval) {
      work.charge(8 * (limbCount(this.width()) + 2) ** 2);
      return this.exactNumber();
    }
    return this.observe((lo, hi) => {
      const low = lo.exactNumber();
      const high = hi.exactNumber();
      if (Object.is(low, high)) return low;
      const lowBits = floatBits(low);
      const highBits = floatBits(high);
      if (magnitude(highBits - lowBits) !== 1n || !this.directRoot()) return;
      const a = Rational.binaryFloat(lowBits);
      const b = Rational.binaryFloat(highBits);
      const midpoint = Rational.reduced(a.n * b.d + b.n * a.d, 2n * a.d * b.d);
      const side = this.compareRoot([lo, hi], midpoint, work)!;
      return side < 0n || (side === 0n && lowBits % 2n === 0n) ? low : high;
    }, work);
  }

  private exactNumber(): number {
    const n = magnitude(this.n);
    if (n === 0n) return 0;
    // Round the quotient to binary64 precision in one step, including subnormal values.
    const exponent = binaryExponent(n, this.d);
    const shift = Math.max(-1074, exponent - 52);
    const top = shift < 0 ? n << BigInt(-shift) : n;
    const bottom = shift > 0 ? this.d << BigInt(shift) : this.d;
    const quotient = top / bottom;
    const remainder = (top % bottom) * 2n;
    const rounded = quotient + (remainder > bottom || (remainder === bottom && quotient % 2n !== 0n) ? 1n : 0n);
    const result = Number(rounded) * 2 ** shift;
    return this.n < 0n ? -result : result;
  }

  floor(): Rational {
    return this.directedInteger(true);
  }

  ceil(): Rational {
    return this.directedInteger(false);
  }

  private directedInteger(down: boolean): Rational {
    const work = new Work();
    const value = this.observe((lo, hi) => {
      const round = (x: Rational) => {
        const quotient = x.n / x.d;
        if (x.n % x.d === 0n) return quotient;
        return down && x.n < 0n ? quotient - 1n : !down && x.n > 0n ? quotient + 1n : quotient;
      };
      const low = round(lo);
      const high = round(hi);
      if (low === high) return low;
      if (high !== low + 1n || !this.directRoot()) return;
      const side = this.compareRoot([lo, hi], new Rational(down ? high : low, 1n), work)!;
      return (down ? side < 0n : side <= 0n) ? low : high;
    }, work);
    return Rational.of(value);
  }

  sign(): Rational {
    const side = this.compareChecked(ZERO);
    return side === 0n ? ZERO : side < 0n ? NEGATIVE_ONE : ONE;
  }

  round(places: number): Rational {
    if (!Number.isInteger(places) || Math.abs(places) > 308) throw new CalcError(PRECISION_ERROR);
    const factor = 10n ** BigInt(Math.abs(places));
    const units = this.roundedUnits(places, factor);
    return places < 0 ? Rational.of(units * factor) : Rational.of(units, factor);
  }

  private roundedUnits(places: number, factor: bigint, work = new Work()): bigint {
    return this.observe((lo, hi) => {
      const round = (x: Rational) => {
        const scaled = magnitude(x.n) * (places < 0 ? 1n : factor);
        const divisor = x.d * (places < 0 ? factor : 1n);
        const rounded = scaled / divisor + (2n * (scaled % divisor) >= divisor ? 1n : 0n);
        return x.n < 0n ? -rounded : rounded;
      };
      const low = round(lo);
      const high = round(hi);
      if (low === high) return low;
      if (high !== low + 1n || !this.directRoot()) return;
      const midpoint = places < 0
        ? Rational.reduced((2n * low + 1n) * factor, 2n)
        : Rational.reduced(2n * low + 1n, 2n * factor);
      const side = this.compareRoot([lo, hi], midpoint, work)!;
      return side < 0n || (side === 0n && midpoint.n < 0n) ? low : high;
    }, work);
  }

  fixed(places: number): string {
    if (!Number.isInteger(places) || places < 0 || places > 308) throw new CalcError(PRECISION_ERROR);
    const work = new Work();
    work.charge(32 * limbCount((this.interval ? Rational.boundWidth(this.interval.bounds) : this.width()) + 4 * places) ** 2);
    const factor = 10n ** BigInt(places);
    const units = this.roundedUnits(places, factor, work);
    const absolute = magnitude(units);
    const whole = absolute / factor;
    const fraction = absolute % factor;
    const sign = units < 0n ? "-" : "";
    return fraction === 0n ? sign + whole : sign + whole + "." + String(fraction).padStart(places, "0").replace(/0+$/, "");
  }

  private significantParts(): [bigint, bigint, number] {
    const n = magnitude(this.n);
    let exponent = n.toString().length - this.d.toString().length;
    const power = 10n ** BigInt(Math.abs(exponent));
    if (exponent < 0 ? n * power < this.d : n < this.d * power) exponent--;
    const shift = 11 - exponent;
    const scale = 10n ** BigInt(Math.abs(shift));
    const numerator = shift < 0 ? n : n * scale;
    const denominator = shift < 0 ? this.d * scale : this.d;
    return [numerator, denominator, exponent];
  }

  significantText(work = new Work()): string {
    if (this.interval) {
      if (this.interval.significant !== undefined) {
        work.charge(32 * (2 * limbCount(Rational.boundWidth(this.interval.bounds)) + 2) ** 2);
        return this.interval.significant;
      }
      return this.observe((lo, hi) => {
        if ((lo.n < 0n) !== (hi.n < 0n) || (lo.n === 0n) !== (hi.n === 0n)) return;
        const low = lo.significantText(work);
        if (low === hi.significantText(work)) return low;
        const root = this.directRoot();
        if (root) {
          const bounds: Bounds = root.negative ? [hi.neg(), lo.neg()] : [lo, hi];
          const text = root.base.rootSignificant(bounds, root.exponent, work);
          if (text !== undefined) return root.negative ? "-" + text : text;
        }
      }, work);
    }
    work.charge(32 * (2 * limbCount(this.width()) + 2) ** 2);
    if (this.n === 0n) return "0";
    const [numerator, denominator, scaleExponent] = this.significantParts();
    const digits = String(numerator / denominator + (numerator % denominator * 2n >= denominator ? 1n : 0n));
    let exponent = scaleExponent;
    exponent += digits.length - 12;
    const trimmed = digits.replace(/0+$/, "");
    const sign = this.n < 0n ? "-" : "";
    if (exponent >= 15 || exponent < -9) {
      return sign + trimmed[0] + (trimmed.length > 1 ? "." + trimmed.slice(1) : "") + "e" + exponent;
    }
    const point = exponent + 1;
    if (point <= 0) return sign + "0." + "0".repeat(-point) + trimmed;
    if (point >= trimmed.length) return sign + trimmed.padEnd(point, "0");
    return sign + trimmed.slice(0, point) + "." + trimmed.slice(point);
  }
}

const ZERO = Rational.of(0n);
const ONE = Rational.of(1n);
const NEGATIVE_ONE = ONE.neg();
const HALF = Rational.of(1n, 2n);
const MAX = Rational.of(MAX_FINITE);
const MIN = MAX.neg();
const CUTOFF = Rational.of(1n, 1n << 1075n);
const HUNDRED = Rational.of(100n);
const FULL_TURN = Rational.of(360n);

function factorial(n: Rational): Rational {
  if (n.interval || n.d !== 1n || n.n < 0n) throw new CalcError("Factorial needs a whole number, 0 or more");
  if (n.n > 170n) throw new CalcError("Too large");
  let result = 1n;
  for (let k = 2n; k <= n.n; k++) result *= k;
  return Rational.of(result);
}

/**
 * True if switching between degrees and radians can change the answer: the expression has sin, cos or
 * tan (or an inverse of them). An expression that uses "ans" is left out, because "ans" is the answer
 * it would be recalled from.
 */
export function dependsOnAngleUnit(expr: string): boolean {
  return /sin|cos|tan/i.test(expr) && !/ans/i.test(expr);
}

export function evaluate(src: string, opts: CalcOptions & { exact: true }): Rational;
export function evaluate(src: string, opts: CalcOptions): number;
export function evaluate(src: string, opts: CalcOptions & { exact?: true }): number | Rational {
  const toks = tokenize(src);
  if (toks.length === 0) throw new CalcError("Type something to calculate");
  let p = 0;
  const peek = () => toks[p];
  const next = () => toks[p++];
  const isOp = (t: Token | undefined, v: string) => !!t && t.t === "op" && t.v === v;

  // value plus whether the number was written with a percent sign
  interface Val {
    v: Rational;
    pct: boolean;
  }

  const expr = (): Rational => {
    let left = term();
    while (isOp(peek(), "+") || isOp(peek(), "-")) {
      const op = (next() as { v: string }).v;
      const right = term();
      // "200 + 10%" means 200 plus 10% of 200.
      const r = right.pct ? left.v.mul(right.v) : right.v;
      left = { v: op === "+" ? left.v.add(r) : left.v.sub(r), pct: false };
    }
    return left.v;
  };

  const startsFactor = (t: Token | undefined) => !!t && (t.t === "num" || t.t === "id" || isOp(t, "("));

  const term = (): Val => {
    let left = unary();
    for (;;) {
      const t = peek();
      if (t?.t === "op" && (t.v === "*" || t.v === "/")) {
        next();
        const right = unary();
        left = { v: t.v === "*" ? left.v.mul(right.v) : left.v.div(right.v), pct: false };
      } else if (startsFactor(t)) {
        // implicit multiplication: 2π, 3(4+1), 2 sin(30)
        const right = unary();
        left = { v: left.v.mul(right.v), pct: false };
      } else break;
    }
    return left;
  };

  const unary = (): Val => {
    if (isOp(peek(), "-")) {
      next();
      const v = unary();
      return { v: v.v.neg(), pct: v.pct };
    }
    if (isOp(peek(), "+")) {
      next();
      return unary();
    }
    return power();
  };

  const power = (): Val => {
    const base = postfix();
    if (isOp(peek(), "^")) {
      next();
      const exp = unary(); // right-associative, and 2^-1 works
      return { v: base.v.pow(exp.v), pct: false };
    }
    return base;
  };

  const postfix = (): Val => {
    let v = primary();
    for (;;) {
      if (isOp(peek(), "!")) {
        next();
        v = { v: factorial(v.v), pct: false };
      } else if (isOp(peek(), "%")) {
        next();
        v = { v: v.v.div(HUNDRED), pct: true };
      } else break;
    }
    return v;
  };

  const angleIn = (x: number) => opts.degrees ? x * (Math.PI / 180) : x;
  const angleOut = (x: number) => opts.degrees ? x * (180 / Math.PI) : x;
  const halfTurn = opts.degrees ? 180 : Math.PI;

  const math = (name: string, x: number): number => {
    switch (name) {
      case "sin":
        return x % halfTurn === 0 ? 0 : Math.sin(angleIn(x));
      case "cos":
        return Math.abs(x % halfTurn) === halfTurn / 2 ? 0 : Math.cos(angleIn(x));
      case "tan": {
        const r = Math.tan(angleIn(x));
        if (Math.abs(r) > 1e14) throw new CalcError("Undefined");
        return x % halfTurn === 0 ? 0 : r;
      }
      case "asin":
        return angleOut(Math.asin(x));
      case "acos":
        return angleOut(Math.acos(x));
      case "atan":
        return angleOut(Math.atan(x));
    }
    throw new CalcError(`Unknown function "${name}"`);
  };

  const call = (name: string, x: Rational): Rational => {
    if (name === "abs") return x.abs();
    if (name === "sqrt") return x.sqrt();
    if (name === "cbrt") return x.cbrt();
    if (name === "ln" || name === "log2" || name === "log10") return x.logarithm(name);
    if (name === "log") return x.logarithm("log10");
    if (name === "exp") return x.exp();
    if ((name === "asin" || name === "acos") && x.abs().compareChecked(ONE) > 0n) {
      throw new CalcError("Not a real number");
    }
    // Reduce degrees before binary64 conversion, where a large argument can lose small offsets.
    if (opts.degrees && (name === "sin" || name === "cos" || name === "tan")) {
      x = x.rem(FULL_TURN);
    }
    return Rational.decimal(String(finite(math(name, x.number()))));
  };

  const primary = (): Val => {
    const t = next();
    if (!t) throw new CalcError("The expression is incomplete");
    if (t.t === "num") return { v: Rational.decimal(t.v), pct: false };
    if (t.t === "op" && t.v === "(") {
      const v = expr();
      if (!isOp(next(), ")")) throw new CalcError("Missing )");
      return { v, pct: false };
    }
    if (t.t === "id") {
      if (t.v === "pi") return { v: Rational.decimal(String(finite(Math.PI))), pct: false };
      if (t.v === "e") return { v: Rational.decimal(String(finite(Math.E))), pct: false };
      if (t.v === "ans") {
        const v = typeof opts.ans === "number" ? Rational.decimal(String(finite(opts.ans))) : opts.ans;
        return { v, pct: false };
      }
      // a function: sqrt(9), or sqrt 9 / √9 with a single factor
      if (isOp(peek(), "(")) {
        next();
        const x = expr();
        if (!isOp(next(), ")")) throw new CalcError("Missing )");
        return { v: call(t.v, x), pct: false };
      }
      const arg = power();
      return { v: call(t.v, arg.v), pct: false };
    }
    throw new CalcError("The expression is incomplete");
  };

  const result = expr();
  if (p < toks.length) {
    const t = toks[p];
    throw new CalcError(t.t === "op" && t.v === ")" ? "Extra )" : "Check the expression");
  }
  return opts.exact ? result : finite(result.number());
}

/** Up to 12 significant digits, no floating-point tails (0.1 + 0.2 shows 0.3). */
export function formatNumber(v: number | Rational): string {
  if (typeof v !== "number") return v.significantText();
  finite(v);
  // Keep the legacy direct-number API for callers that already hold a binary64 integer.
  if (Number.isInteger(v) && Math.abs(v) < 1e15) return String(v);
  return Rational.decimal(String(v)).significantText();
}

/**
 * The +/- key: flips the sign of the whole expression, not just its first number. "-2" becomes "2",
 * "-(2+3)" becomes "2+3", and "-2+3" becomes "-(-2+3)".
 */
export function negateExpr(x: string): string {
  if (x === "") return "-";
  if (x === "-") return "";
  if (/^-(\d+\.?\d*|\.\d+)$/.test(x)) return x.slice(1);
  if (x.startsWith("-(") && x.endsWith(")")) {
    // The "(" after the minus must close at the very end, or "-(2)+(3)" would lose its meaning.
    let depth = 0;
    for (let i = 1; i < x.length; i++) {
      if (x[i] === "(") depth++;
      else if (x[i] === ")" && --depth === 0) {
        if (i === x.length - 1) return x.slice(2, -1);
        break;
      }
    }
  }
  return "-(" + x + ")";
}
