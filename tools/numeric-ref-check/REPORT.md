# A57 Fix 8 numeric reference

All **336/336** fixture and target checks passed (246 Rust and 90 Sticky or UI-helper checks). The good control exited 0. The planted display defect exited 1 after it reported 6 for 2 + 3. The run used baseline cbf0193f and working-tree fingerprint bf323284c20b3ed8b12784a307070a8460350afd780702e8be77bddce8fa092b. [Fix 8 observations](observations-post.json)

| Review 7 case | Observed result |
| --- | --- |
| Exact formatting, binary64 conversion and remainder | The rejecting meter observed the expected debit for each operation in Rust and Sticky. Both Rust list formatting paths reported an arithmetic work-budget error with 64 projections. |
| ln(sqrt(1+1e-100)) | 5e-101 across Sticky, direct Rust formula and computed JSON. Its below-one sibling displays -5e-101. |
| exp(sqrt(1e-200+1e-250))-1 | 1e-100 across all three targets. The negative input sibling displays -1e-100. |
| exp(1e-1000)-1 | The display preserves 1e-1000. Its binary64 encoding is zero. |
| Ascending table view sort | The row with sqrt(2) precedes the row with sqrt(2)*sqrt(2)*sqrt(2)/2 + 1e-100. The latter row is first in manual order. |

The checker also covers logarithm domains and integer powers, signed rational and irrational cube roots, exponential overflow, and the subnormal cutoff. It checks binary64 bits 4005bf0a8b145769 and display 2.71828182846 for exp(1). Python uses high-precision Decimal calculations and exact rational arithmetic for the reference cases.

# A57 Fix 7 numeric reference (historical)

The independent checker passed **259/259** fixture and target checks on baseline `cbf0193f` with working-tree fingerprint `aa199cfb4d186881cf996ad1390662df53161050feaba80ddbc3b8fbdfbea0bc`. The good control passed 1/1 and exited 0. The planted display defect returned 6 for `2 + 3`, failed 1/1 and exited 1.

| Projection | Expected and verified |
| --- | --- |
| General power just above `2^-1075` | Display `2.47032822921e-324`, binary64 bits `0000000000000001`. The tie and strict lower sibling produce zero. The negative sibling retains the sign bit. |
| Square root near a normal binary64 midpoint | The strict lower input and the exact tie round to the even float. The strict upper input rounds to its adjacent float. |
| Subtraction near an exact rational | `sqrt(1.234567890124^2 + 1e-100) - 1.234567890124` displays `4.05000003645e-101` in both engines. The lower sibling produces the negative result to the same precision. |
| Currency | The strict upper half-cent input and the exact tie produce `$0.01`. The strict lower input produces `$0`. Rust computed JSON uses the certified rounded amount. Compounding the root with subtraction preserves the rounding. |
| Unresolved boundaries | An ambiguous cancellation or half-cent decision returns a numeric precision error in Sticky and an arithmetic work-budget error in Rust. Computed cells carry an error type and a null value. |

The new cases exercise Sticky, direct Rust formulas and Rust computed JSON. The checker retains 194 Rust observations and 65 Sticky and UI-helper observations. Python evaluates the new boundary fixtures at increased decimal precision and uses exact rational power comparisons at the general-power cutoff. Selected cases assert the complete binary64 encoding as well as the display. Both adapters surface errors from the projection they exercise.

The original baseline remains available for comparison. These results describe the Fix 7 source.

# A57 numeric reference baseline

**35 passed, 87 failed across 122 fixture–target checks.** The harness ran on Ahmed’s side (`AE0101`) on `GAMING-PC`, completing at `2026-11-22T21:15:59Z`. The requested source was commit `cbf0193fabe77d04cdd84f40e2560f8dd643c333`, tree `c67a85e52d1bcd71e66e6b20b3699595085445e2`. The remote `main` read returned `0e08e5489d36fbd2db1fc3a537cdebfa66bb3ad9`. This report applies to the requested commit. The owner’s chat message authorized the test-only work, and its UTC timestamp was unavailable. All changes are under `tools/numeric-ref-check/` in the named worktree.

## Qualification and counts

The same grader checked the real Sticky output of `2 + 3` as the known-good control. It passed 1/1 and exited `0`. The planted control kept the raw `5` and changed only the observed display to `6`. The grader printed `FAIL known-good / sticky.evaluate: display: observed '6', expected 5` and exited `1`. The baseline also exited `1`, with the following counts from 44 synthetic fixtures. Each fixture–target pair counts once, even if multiple output fields differ. Controls are excluded from the baseline totals.

| Target | Pass | Fail |
| --- | ---: | ---: |
| Rust direct formula | 12 | 12 |
| Rust `Ctx` computed JSON | 4 | 21 |
| Rust rollup | 2 | 9 |
| Rust view footer | 2 | 9 |
| Rust group footer | 2 | 9 |
| Rust subgroup footer | 2 | 9 |
| Rust number-chart total | 2 | 9 |
| Sticky `evaluate` and its formatted result | 9 | 6 |
| Sticky `formatNumber` with direct non-finite input | 0 | 3 |
| **Total** | **35** | **87** |

## Numeric mismatches

| Fixture or expression | Failing targets | Expected | Observed |
| --- | --- | --- | --- |
| Below half-cent, `±0.004999` | Computed JSON display | `0` for both signs | `$0.004999`, `-$0.004999` |
| At half-cent, `±0.005` | Computed JSON display | `±0.01` | `$0.005`, `-$0.005` |
| Above half-cent, `±0.005001` | Computed JSON display | `±0.01` | `$0.005001`, `-$0.005001` |
| `0.004 + 0.004` | Computed JSON display | `0.01` | `$0.008` |
| `(9 / 7 / 3) * (17 / 100)` | Computed JSON display | `0.07` | `$0.07285714285712` |
| `(1 / 3) * 100000000000 * 3 - 100000000000 + 0.005` | Direct formula and computed JSON | `0.01` | Raw and JSON `-0.004994506835938`, display `-$0.004994506835938` |
| `1e11 + 0.005 - 1e11` | Direct formula and computed JSON | `0.01` | Raw and JSON zero, display `$0` |
| `999999999999.9848 + 0.0003` | Direct formula and computed JSON | `999999999999.99` | Raw and JSON `1000000000000`, display `$1,000,000,000,000` |
| `1e16 + 1 - 1e16` | Direct formula, computed JSON, Sticky evaluate | `1` | All returned zero |
| `1.23456789123e-10` | Sticky evaluate, display only | `1.23456789123e-10` | `1.23456789e-10` |
| `1.23456789123e16` | Sticky evaluate, display only | `1.23456789123e16` | `1.23456789e16` |
| `sin(1e-13)` in degrees | Sticky evaluate | `1.74532925199e-15` | Raw and display `0` |
| Sum `0.004, 0.004` | Rollup, all three footers, chart | `0.01` | `$0.008` at all five targets |
| Median `0.004, 0.006` | Rollup, all three footers, chart | `0.01` | `$0.005` at all five targets |
| Average `-0.004, -0.006` | Rollup, all three footers, chart | `-0.01` | `-$0.005` at all five targets |
| Sum `1e11, 0.005, -1e11` | Rollup, all three footers, chart | `0.01` | `$0.0050048828125` at all five targets |
| Sum `1e11, 0.004999, -1e11` | Rollup, all three footers, chart | `0` | `$0.0050048828125` at all five targets. Rollup JSON and chart raw were `0.0050048828125`, already above the half-cent boundary. |

## Missing explicit errors

| Fixture or expression | Failing targets | Observed |
| --- | --- | --- |
| `sqrt(-1)` and `toNumber("NaN")` | Direct formula and computed JSON | Raw `NaN`. Computed `t: "number"`, `v: null`, text `NaN`. |
| `exp(1000)` and `toNumber("Infinity")` | Direct formula and computed JSON | Raw `inf`. Computed `t: "number"`, `v: null`, text `∞`. |
| `ln(0)` and `toNumber("-Infinity")` | Direct formula and computed JSON | Raw `-inf`. Computed `t: "number"`, `v: null`, text `-∞`. |
| `1/exp(1000)` and `1/(1e308 * 10)` | Direct formula, computed JSON, Sticky evaluate | Finite zero masked the overflow on all three targets. |
| `[1, [sqrt(-1)]]` | Computed JSON | `t: "list"`, `v: [1.0, [null]]`, text `1, NaN`. |
| Nested sum of `2` and `NaN` | Rollup, all three footers, chart | Rollup numeric tag with `v: null`. All five displayed `NaN`. Chart raw `NaN`. |
| Nested sum of `2` and `Infinity` | Rollup, all three footers, chart | Rollup numeric tag with `v: null`. All five displayed `∞`. Chart raw `inf`. |
| Nested sum of `2` and `-Infinity` | Rollup, all three footers, chart | Rollup numeric tag with `v: null`. All five displayed `-∞`. Chart raw `-inf`. |
| Average `1e308, 1e308` | Rollup, all three footers, chart | Rollup numeric tag with `v: null`. All five displayed `∞`. Chart raw `inf`. |
| Direct Sticky formatter inputs `NaN`, `Infinity`, `-Infinity` | `formatNumber` | Returned the corresponding strings `NaN`, `Infinity`, `-Infinity`. |

All three evaluation targets returned explicit errors for `1/0`. Sticky also returned explicit errors for `sqrt(-1)`, `ln(0)`, `exp(1000)`, and each non-finite `ans` input.

## Files and reproduction

- [fixtures.json](fixtures.json) names the decimal inputs, target paths, currency scales and expected values or errors. [check.py](check.py) independently calculates those expectations with Python `Fraction` and grades the observations.
- [rust_adapter.rs](rust_adapter.rs) reuses the existing `db-bench` stand-ins and source includes, adding the production chart module by path. [Cargo.toml](Cargo.toml) and [Cargo.lock](Cargo.lock) reuse the bench’s dependency set and locked versions. The build used cached crates and a temporary target directory outside the worktree.
- [sticky_adapter.mjs](sticky_adapter.mjs) imports `evaluate` and `formatNumber` directly from `src/features/sticky/calc.ts`. [observations.json](observations.json) contains all 122 results, including the exact serialized decimal tokens captured as strings and each failing comparison.

From the worktree root with Python, Node and Rust on PATH:

```text
python -B tools/numeric-ref-check/check.py --control good
python -B tools/numeric-ref-check/check.py --control wrong
python -B tools/numeric-ref-check/check.py
```

To record a new run, pass `--output` with a path outside the checkout.

Exit `0` is a passing selection, `1` is a contract mismatch, and `2` is a harness or environment error. `git diff` against `HEAD` was clean for tracked files before and after the run.

## Observation limits

- Currency cases use the default two-place USD format. The grader tests the final displayed amount and, where available, whether the raw or JSON value yields the correct amount after one independent rounding. It accepts equivalent zero padding. The below-half-cent cancellation case exposes an arithmetic defect even if display rounding is applied. Passing fixtures do not establish exact internal representation.
- Plain amounts are compared after independently rounding to 12 significant digits. Scientific notation is assessed for numeric precision. The tiny degree-sine oracle uses a rational approximation with truncation error below the tested precision. The overflow cases use the binary64 finite bound at intermediate operations, including the sum in the tested average.
- The synthetic Rust aggregates use one populated group and subgroup, one number-chart configuration and one level of nested input rollups. Other group memberships, chart types, currency formats and unseeded formula operations remain uninspected. Sticky formatter-only calls show its direct API behavior. They do not establish that the UI passes non-finite values to it.
- The app, IPC, persistence, installed build, private vault data, real UI rendering and wider test suites were outside this run. The result is a verified observation of this baseline. Product repair and acceptance are separate decisions.
