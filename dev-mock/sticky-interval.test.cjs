const assert = require("node:assert/strict");
const test = require("node:test");
const fixtures = require("../tools/numeric-ref-check/fixtures.json");
const { CalcError, evaluate, formatNumber } = require("../src/features/sticky/calc.ts");
const options = { degrees: false, ans: 0 };
const exact = (expression, extra) => evaluate(expression, { ...options, ...extra, exact: true });
const precisionError = (error) => error instanceof CalcError && /precision is out of range/.test(error.message);

const reviewCases = fixtures.filter(({ id }) => /^(pow-subnormal|negative-pow-subnormal|sqrt-cancellation|sqrt-cent|sqrt-negative-cent|compound-sqrt-cent|unresolved-interval|sqrt-binary64-midpoint)/.test(id));
for (const { id, expression, expected, currency_scale: scale } of reviewCases) {
  test(id, () => {
    const value = exact(expression);
    if (expected.error) {
      assert.throws(() => scale === null ? formatNumber(value) : value.fixed(scale), precisionError);
      if (scale === null) assert.throws(() => value.number(), precisionError);
      return;
    }
    assert.equal(scale === null ? formatNumber(value) : value.fixed(scale), expected.value);
    if (expected.binary64_bits) {
      const binary = Buffer.alloc(8);
      binary.writeDoubleBE(evaluate(expression, options));
      assert.equal(binary.toString("hex"), expected.binary64_bits);
    }
  });
}

test("interval arithmetic and recalled answers retain refinement witnesses", () => {
  const root = exact("sqrt(1.234567890124^2 + 1e-100)");
  assert.equal(formatNumber(exact("ans-1.234567890124", { ans: root })), "4.05000003645e-101");
  assert.equal(formatNumber(exact("1/(sqrt(1+1e-100)-1)")), "2e100");
  assert.equal(formatNumber(exact("(sqrt(2)+1)^3")), "14.0710678119");
  assert.equal(formatNumber(exact("2^sqrt(2)")), "2.66514414269");
  assert.equal(formatNumber(exact("sqrt(sqrt(2))")), "1.189207115");
  assert.throws(() => evaluate("1/(sqrt(2)-sqrt(2))", options), precisionError);
});

test("large interval chains keep their bounds when their witness exceeds the cap", () => {
  const sum = exact(Array(128).fill("sqrt(2)").join("+"));
  assert.equal(formatNumber(sum), "181.019335984");
  assert.throws(() => formatNumber(exact("ans-ans", { ans: sum })), precisionError);
});

test("shared interval witnesses survive the cap and replay once per node", () => {
  let value = exact("sqrt(1+1e-100)-1");
  for (let i = 0; i < 12; i++) value = value.add(value);
  const recovered = value.div(exact("4096"));
  assert.equal(formatNumber(recovered), "5e-101");
});

test("discontinuous projections certify signed rounding boundaries", () => {
  const below = exact("sqrt(1 - 1e-100)");
  const above = exact("sqrt(1 + 1e-100)");
  assert.equal(below.floor().number(), 0);
  assert.equal(above.ceil().number(), 2);
  assert.equal(exact("-sqrt(0.005^2 + 1e-90)").round(2).number(), -0.01);
  assert.equal(exact("sqrt(1+1e-100)-1").sign().number(), 1);
  assert.equal(formatNumber(above.rem(exact("1"))), "5e-101");
  assert.equal(exact("sqrt(2)").compareChecked(exact("sqrt(2)")), 0n);
  assert.throws(() => exact("sqrt(2)-sqrt(2)").sign(), precisionError);
});

test("transcendental intervals preserve compound precision", () => {
  const cases = fixtures.filter(({ id, targets }) => id.startsWith("transcendental-") && targets.includes("sticky.evaluate"));
  assert.ok(cases.length >= 10);
  for (const { expression, expected, currency_scale: scale } of cases) {
    if (expected.error) {
      const errors = { overflow: /Too large/, domain: /Not a real number/, precision: precisionError };
      const error = errors[expected.error];
      assert.ok(error, expected.error);
      assert.throws(() => exact(expression), error, expression);
    } else {
      const value = exact(expression);
      assert.equal(scale === null ? formatNumber(value) : value.fixed(scale), expected.value, expression);
      if (expected.binary64_bits) {
        const binary = Buffer.alloc(8);
        binary.writeDoubleBE(value.number());
        assert.equal(binary.toString("hex"), expected.binary64_bits, expression);
      }
    }
  }
  assert.equal(formatNumber(exact("log2")), "0.301029995664");
});

test("exponential respects reduced endpoint width", () => {
  const maxBits = 16_384;
  const value = exact("exp(-11201)");
  const [lo, hi] = value.interval.bounds;
  assert.ok(lo.n > 0n);
  assert.equal(Math.max(lo.width(), hi.width()), maxBits);
  assert.throws(() => exact("exp(-11201.3)"), precisionError);

  // The tiny input fits, but the reduced numerator of its lower bound 1 + input does not.
  const input = lo.constructor.of(1n, (1n << BigInt(maxBits)) - 1n);
  assert.equal(input.width(), maxBits);
  assert.throws(() => exact("exp(ans)", { ans: input }), precisionError);
});

test("exact observations and remainder debit work before producing a result", () => {
  const value = exact("3.14e-1000");
  const divisor = exact("1");
  const root = exact("sqrt(2)");
  const costs = [];
  const rejected = new Error("meter rejected work");
  const work = { charge(cost) { costs.push(cost); throw rejected; } };
  assert.throws(() => value.significantText(work), rejected);
  assert.throws(() => value.number(work), rejected);
  assert.throws(() => value.rem(divisor, work), rejected);
  assert.throws(() => root.significantText(work), rejected);
  assert.equal(costs.length, 4);
  assert.ok(costs.every((cost) => cost > 0));
});
