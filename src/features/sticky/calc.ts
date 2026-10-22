// A small, safe calculator engine. It parses text itself; nothing is ever passed to eval.
//
// Supports + - × ÷ ^ ( ) ! %, unary minus, implicit multiplication (2π, 3(4+1)), the constants
// pi / π, e and ans, and the functions sqrt (or √), abs, ln, log (base 10), exp, sin, cos, tan,
// asin, acos, atan. Percent follows the usual calculator habit: "200 + 10%" is 220, "50%" is 0.5.

export class CalcError extends Error {}

export interface CalcOptions {
  /** Angles for sin/cos/tan and the results of asin/acos/atan, in degrees instead of radians. */
  degrees: boolean;
  /** The last result, available as "ans". */
  ans: number;
}

type Token =
  | { t: "num"; v: number }
  | { t: "id"; v: string }
  | { t: "op"; v: string };

const FUNCTIONS = new Set(["sqrt", "abs", "ln", "log", "exp", "sin", "cos", "tan", "asin", "acos", "atan"]);
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
      out.push({ t: "num", v: Number(num[0]) });
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
      const lower = id[0].toLowerCase();
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

function factorial(n: number): number {
  if (!Number.isInteger(n) || n < 0) throw new CalcError("Factorial needs a whole number, 0 or more");
  if (n > 170) throw new CalcError("Too large");
  let r = 1;
  for (let k = 2; k <= n; k++) r *= k;
  return r;
}

/** Tiny floating-point noise from trigonometry (cos 90° = 6e-17) is shown as zero. */
const clean = (v: number) => (Math.abs(v) < 1e-14 ? 0 : v);

export function evaluate(src: string, opts: CalcOptions): number {
  const toks = tokenize(src);
  if (toks.length === 0) throw new CalcError("Type something to calculate");
  let p = 0;
  const peek = () => toks[p];
  const next = () => toks[p++];
  const isOp = (t: Token | undefined, v: string) => !!t && t.t === "op" && t.v === v;

  // value plus whether the number was written with a percent sign
  interface Val {
    v: number;
    pct: boolean;
  }

  const expr = (): number => {
    let left = term();
    while (isOp(peek(), "+") || isOp(peek(), "-")) {
      const op = (next() as { v: string }).v;
      const right = term();
      // "200 + 10%" means 200 plus 10% of 200.
      const r = right.pct ? left.v * right.v : right.v;
      left = { v: op === "+" ? left.v + r : left.v - r, pct: false };
    }
    return left.v;
  };

  const startsFactor = (t: Token | undefined) => !!t && (t.t === "num" || t.t === "id" || isOp(t, "("));

  const term = (): Val => {
    let left = unary();
    for (;;) {
      const t = peek();
      if (isOp(t, "*") || isOp(t, "/")) {
        next();
        const right = unary();
        if ((t as { v: string }).v === "*") left = { v: left.v * right.v, pct: false };
        else {
          if (right.v === 0) throw new CalcError("Can't divide by zero");
          left = { v: left.v / right.v, pct: false };
        }
      } else if (startsFactor(t)) {
        // implicit multiplication: 2π, 3(4+1), 2 sin(30)
        const right = unary();
        left = { v: left.v * right.v, pct: false };
      } else break;
    }
    return left;
  };

  const unary = (): Val => {
    if (isOp(peek(), "-")) {
      next();
      const v = unary();
      return { v: -v.v, pct: v.pct };
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
      const r = Math.pow(base.v, exp.v);
      if (!Number.isFinite(r)) throw new CalcError(Number.isNaN(r) ? "Not a real number" : "Too large");
      return { v: r, pct: false };
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
        v = { v: v.v / 100, pct: true };
      } else break;
    }
    return v;
  };

  const angleIn = (x: number) => (opts.degrees ? (x * Math.PI) / 180 : x);
  const angleOut = (x: number) => (opts.degrees ? (x * 180) / Math.PI : x);

  const call = (name: string, x: number): number => {
    switch (name) {
      case "sqrt":
        if (x < 0) throw new CalcError("Not a real number");
        return Math.sqrt(x);
      case "abs":
        return Math.abs(x);
      case "ln":
        if (x <= 0) throw new CalcError("Not a real number");
        return Math.log(x);
      case "log":
        if (x <= 0) throw new CalcError("Not a real number");
        return Math.log10(x);
      case "exp":
        return Math.exp(x);
      case "sin":
        return clean(Math.sin(angleIn(x)));
      case "cos":
        return clean(Math.cos(angleIn(x)));
      case "tan": {
        const r = Math.tan(angleIn(x));
        if (Math.abs(r) > 1e14) throw new CalcError("Undefined");
        return clean(r);
      }
      case "asin":
        if (x < -1 || x > 1) throw new CalcError("Not a real number");
        return clean(angleOut(Math.asin(x)));
      case "acos":
        if (x < -1 || x > 1) throw new CalcError("Not a real number");
        return clean(angleOut(Math.acos(x)));
      case "atan":
        return clean(angleOut(Math.atan(x)));
    }
    throw new CalcError(`Unknown function "${name}"`);
  };

  const primary = (): Val => {
    const t = next();
    if (!t) throw new CalcError("The expression is incomplete");
    if (t.t === "num") return { v: t.v, pct: false };
    if (t.t === "op" && t.v === "(") {
      const v = expr();
      if (!isOp(next(), ")")) throw new CalcError("Missing )");
      return { v, pct: false };
    }
    if (t.t === "id") {
      if (t.v === "pi") return { v: Math.PI, pct: false };
      if (t.v === "e") return { v: Math.E, pct: false };
      if (t.v === "ans") return { v: opts.ans, pct: false };
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
  if (!Number.isFinite(result)) throw new CalcError(Number.isNaN(result) ? "Not a number" : "Too large");
  return result;
}

/** Up to 12 significant digits, no floating-point tails (0.1 + 0.2 shows 0.3). */
export function formatNumber(v: number): string {
  if (v === 0) return "0";
  const abs = Math.abs(v);
  if (Number.isInteger(v) && abs < 1e15) return String(v);
  if (abs >= 1e15 || abs < 1e-9) {
    return v.toExponential(8).replace(/\.?0+e/, "e").replace("e+", "e");
  }
  return String(Number(v.toPrecision(12)));
}
