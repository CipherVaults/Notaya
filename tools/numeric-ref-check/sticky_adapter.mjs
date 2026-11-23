import { readFileSync } from "node:fs";
import { CalcError, evaluate, formatNumber } from "../../src/features/sticky/calc.ts";
import { decimalInput } from "../../src/features/database/cells/decimalInput.ts";
import { axisFraction, niceTicks } from "../../src/features/database/views/chartScale.ts";

const cases = JSON.parse(readFileSync(0, "utf8"));
const results = cases.map(({ id, target, expression, degrees, input, axis_min, currency_scale }) => {
  const result = { id, target };
  if (target === "sticky.exact-meter") {
    const options = { degrees: false, ans: 0, exact: true };
    const value = evaluate(input, options);
    const divisor = evaluate("1", options);
    const error = new Error("meter rejected work");
    const charges = [];
    const work = { charge(cost) { charges.push(cost); throw error; } };
    const rejected = [
      () => value.significantText(work),
      () => value.number(work),
      () => value.rem(divisor, work),
    ].map((operation) => {
      try {
        operation();
        return false;
      } catch (thrown) {
        if (thrown !== error) throw thrown;
        return true;
      }
    });
    return { ...result, charges, rejected };
  }
  if (target === "ui.editor") {
    try {
      return { ...result, token: decimalInput(input) };
    } catch (error) {
      return { ...result, error: error.message };
    }
  }
  if (target === "ui.axis") {
    const min = Number(axis_min);
    const max = Number(input);
    const ticks = niceTicks(min, max, 5, true);
    const positions = ticks.map((tick) => axisFraction(tick, ticks[0], ticks.at(-1)));
    return { ...result, ticks: ticks.map(String), positions: positions.map(String) };
  }
  let value;
  if (target === "sticky.evaluate") {
    try {
      value = evaluate(expression, { degrees: degrees ?? false, ans: Number(input ?? "0"), exact: true });
    } catch (error) {
      if (!(error instanceof CalcError)) throw error;
      return { ...result, evaluate_error: error.message };
    }
    try {
      if (currency_scale !== null) {
        result.raw = value.fixed(currency_scale);
        const negative = result.raw.startsWith("-");
        result.text = negative ? "-$" + result.raw.slice(1) : "$" + result.raw;
        return result;
      }
      result.raw = String(value.number());
      if (!value.interval) result.exact = `${value.n}/${value.d}`;
    } catch (error) {
      if (!(error instanceof CalcError)) throw error;
      return { ...result, evaluate_error: error.message };
    }
  } else if (target === "sticky.format") {
    value = Number(input);
  } else {
    throw new Error(`Unknown target ${target}`);
  }
  try {
    result.text = formatNumber(value);
  } catch (error) {
    if (!(error instanceof CalcError)) throw error;
    result.format_error = error.message;
  }
  return result;
});
process.stdout.write(JSON.stringify(results));
