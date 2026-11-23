"""Compare the synthetic A57 fixtures with the production adapters."""

import argparse
import ast
from decimal import Decimal, localcontext
from fractions import Fraction
import hashlib
import json
import math
from pathlib import Path
import re
import struct
import subprocess
import sys
from tempfile import TemporaryDirectory


HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
MAX_FINITE = Fraction.from_float(sys.float_info.max)
PI = Fraction("3.141592653589793238462643383279502884197169399375105820974944")
NON_FINITE = {"NaN", "Infinity", "-Infinity"}
TARGETS = {
    "rust.formula", "rust.computed", "rust.rollup", "rust.view-footer", "rust.group-footer",
    "rust.subgroup-footer", "rust.chart-total", "sticky.evaluate", "sticky.format",
    "rust.donut-slice", "rust.formula-budget", "rust.view-sort", "ui.editor", "ui.axis",
    "rust.exact-meter", "sticky.exact-meter",
}
DECIMAL = r"[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?"
USD = r"(-?)\$([0-9]{1,3}(?:,[0-9]{3})+|[0-9]+)(\.[0-9]+)?"
ERROR_KIND = {
    "domain": r"not a real number|\bdomain\b",
    "overflow": r"\boverflow\b|too large",
    "non-finite": r"non[- ]?finite|not a real number|not finite|finite decimal number|\bNaN\b|\binfinity\b|\boverflow\b|too large",
    "division-by-zero": r"divi(?:de|sion) by zero",
    "work-budget": r"arithmetic work budget",
    "boundary-budget": r"arithmetic work budget|numeric precision is out of range",
    "precision": r"^(?:\u26a0 )?Numeric precision is out of range$",
}


class NumericError(Exception):
    pass


def finite(value):
    if abs(value) > MAX_FINITE:
        raise NumericError("overflow")
    return value


def decimal_string(text):
    if text in NON_FINITE:
        raise NumericError("non-finite")
    if not re.fullmatch(DECIMAL, text):
        raise ValueError(f"Not a decimal fixture input: {text!r}")
    return finite(Fraction(text))


def reference_root(base, power, approximation, case, subnormal_cutoff=False):
    p, q = power.numerator, power.denominator
    if base:
        raised = base ** p
        if "boundary_budget" in case:
            name = case["boundary_budget"]
            boundary = MAX_FINITE if name == "finite-max" else decimal_string(name)
            # Prove this is an ambiguous boundary case with exact arithmetic.
            # Its mathematical result is within 90 decimal places relative to
            # the threshold, and the larger product exceeds the comparison cap.
            a, c = (base.numerator, base.denominator) if p > 0 else (base.denominator, base.numerator)
            width = max(a.bit_length() * abs(p) + boundary.denominator.bit_length() * q,
                        c.bit_length() * abs(p) + boundary.numerator.bit_length() * q)
            gap = boundary / 10 ** 90
            if (width <= 90_496 or raised == boundary ** q
                    or not (boundary - gap) ** q < raised < (boundary + gap) ** q):
                raise ValueError("Invalid boundary-budget fixture")
            raise NumericError("boundary-budget")
        if raised > MAX_FINITE ** q:
            raise NumericError("overflow")
        if subnormal_cutoff and raised <= Fraction(1, 2 ** 1075) ** q:
            return Fraction()
        approximation = min(approximation, MAX_FINITE)
        quantum = significant_quantum(approximation)
        midpoint = (approximation // quantum + Fraction(1, 2)) * quantum
        # The Decimal approximation can hide which side of the midpoint the root lies on.
        rounds_up = raised >= midpoint ** q
        if rounds_up != (approximation >= midpoint):
            return midpoint if rounds_up else midpoint - quantum / 10 ** 60
    return finite(approximation)


def reference_expression(expression, case):
    def evaluate(node):
        if isinstance(node, ast.Constant) and type(node.value) in (int, float):
            return decimal_string(ast.get_source_segment(expression, node))
        if isinstance(node, ast.Name) and node.id == "ans":
            return decimal_string(case["input"])
        if isinstance(node, ast.List):
            return [evaluate(element) for element in node.elts]
        if isinstance(node, ast.UnaryOp) and isinstance(node.op, (ast.USub, ast.UAdd)):
            value = evaluate(node.operand)
            return -value if isinstance(node.op, ast.USub) else value
        if isinstance(node, ast.BinOp):
            left, right = evaluate(node.left), evaluate(node.right)
            if isinstance(node.op, ast.Add):
                return finite(left + right)
            if isinstance(node.op, ast.Sub):
                return finite(left - right)
            if isinstance(node.op, ast.Mult):
                return finite(left * right)
            if isinstance(node.op, ast.Div):
                if right == 0:
                    raise NumericError("division-by-zero")
                return finite(left / right)
            if isinstance(node.op, ast.Pow):
                if not left and right < 0:
                    raise NumericError("division-by-zero")
                if right.denominator == 1:
                    return finite(left ** right.numerator)
                if left < 0:
                    raise NumericError("domain")
                with localcontext() as ctx:
                    ctx.prec = case.get('reference_precision', 70)
                    base = Decimal(left.numerator) / Decimal(left.denominator)
                    power = Decimal(right.numerator) / Decimal(right.denominator)
                    return reference_root(left, right, Fraction(base ** power), case, right != Fraction(1, 2))
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and len(node.args) == 1:
            name, arg = node.func.id, node.args[0]
            if name == "toNumber" and isinstance(arg, ast.Constant) and isinstance(arg.value, str):
                return decimal_string(arg.value)
            value = evaluate(arg)
            if (name == "sqrt" and value < 0) or (name in {"ln", "log", "log10", "log2"} and value <= 0):
                raise NumericError("domain")
            if name == "sin" and case.get("degrees"):
                radians = value * PI / 180
                if abs(radians) >= Fraction("1e-12"):
                    raise ValueError("Sine reference is limited to tiny degree angles")
                # |sin(x) - x| <= |x|^3/6, sufficient for the seeded 12-digit comparison.
                return radians
            if name in {"sqrt", "cbrt", "ln", "log", "log10", "log2", "exp"}:
                if name == "exp" and value >= 1000:
                    raise NumericError("overflow")
                with localcontext() as ctx:
                    ctx.prec = case.get('reference_precision', 70)
                    argument = Decimal(value.numerator) / Decimal(value.denominator)
                    if name == "exp" and -argument / Decimal(2).ln() >= 16_384 - 224:
                        # The dyadic denominator needs floor(-x / ln(2)) + 224 + 1 bits.
                        raise NumericError("precision")
                    if name == "cbrt":
                        approximation = Fraction(abs(argument) ** (Decimal(1) / 3))
                        root = reference_root(abs(value), Fraction(1, 3), approximation, case)
                        return -root if value < 0 else root
                    if name == "log2":
                        approximation = Fraction(argument.ln() / Decimal(2).ln())
                    else:
                        approximation = Fraction(getattr(argument, "log10" if name == "log" else name)())
                    if name == "sqrt":
                        return reference_root(value, Fraction(1, 2), approximation, case)
                    return finite(approximation)
        raise ValueError(f"Unsupported reference expression: {ast.dump(node)}")

    return evaluate(ast.parse(expression, mode="eval").body)


def reference_aggregate(case):
    # Each nested input is the sum of a single leaf, so its exact value is the leaf value.
    values = [decimal_string(text) for text in case["rows"]]
    operation = case["operation"]
    if operation == "median":
        values.sort()
        middle = len(values) // 2
        return values[middle] if len(values) % 2 else finite(values[middle - 1] + values[middle]) / 2
    if operation not in {"sum", "avg"}:
        raise ValueError(f"Unsupported reference operation: {operation}")
    total = Fraction()
    for value in values:
        total = finite(total + value)
    return total / len(values) if operation == "avg" else total


def half_away(value, quantum):
    units = abs(value) / quantum
    whole, rest = divmod(units.numerator, units.denominator)
    rounded = (whole + (2 * rest >= units.denominator)) * quantum
    return -rounded if value < 0 else rounded


def significant_quantum(value):
    magnitude = abs(value)
    exponent = len(str(magnitude.numerator)) - len(str(magnitude.denominator))
    if magnitude < Fraction(10) ** exponent:
        exponent -= 1
    return Fraction(10) ** (exponent - 11)


def significant(value):
    return half_away(value, significant_quantum(value)) if value else value


def exact_value(case):
    if "rows" in case:
        return reference_aggregate(case)
    if "expression" in case:
        return reference_expression(case.get("oracle_expression", case["expression"]), case)
    return decimal_string(case["input"])


def oracle(case):
    try:
        if "sort_expressions" in case:
            values = [reference_expression(expression.replace("^", "**"), case)
                      for expression in case["sort_expressions"]]
            if len(set(values)) != len(values):
                raise ValueError("View sort fixture must have distinct values")
            return sorted(range(len(values)), key=values.__getitem__)
        if "charges" in case["expected"] or "format_projections" in case:
            value = Fraction(case.get("input", "3.14e-1000"))
            width = max(value.numerator.bit_length(), value.denominator.bit_length())
            limbs = (width + 31) // 32
            format_cost = 32 * (2 * limbs + 2) ** 2
            if "charges" in case["expected"]:
                return [format_cost, 8 * (limbs + 2) ** 2, 32 * (limbs + 1) ** 2]
            if case["format_projections"] * format_cost <= 32_000_000:
                raise ValueError("Fixture does not exceed the formatting budget")
            raise NumericError("work-budget")
        if "work_repeat" in case:
            product = Fraction(2, 7) ** 2400 * Fraction(3, 5) ** 2400
            if case["work_repeat"] < 1 or not 0 < product < 1:
                raise ValueError("Invalid finite work-budget fixture")
            raise NumericError("work-budget")
        exact = exact_value(case)
        if "certification_boundary" in case:
            # These inputs are exactly on a boundary the bounded product
            # intentionally cannot resolve using independent interval bounds.
            if exact != decimal_string(case["certification_boundary"]):
                raise ValueError("Invalid refinement-budget fixture")
            raise NumericError("boundary-budget")
        if "binary64_bits" in case["expected"]:
            if struct.pack(">d", float(exact)).hex() != case["expected"]["binary64_bits"]:
                raise ValueError("Reference projection disagrees with the fixture's IEEE bits")
        if "axis_min" in case:
            if not decimal_string(case["axis_min"]) <= 0 < exact:
                raise ValueError("Expected an axis spanning zero")
            return Fraction(1)
        scale = case["currency_scale"]
        if scale is not None and abs(exact) >= 10 ** 12:
            raise ValueError("Currency fixture outside A57's range")
        return half_away(exact, Fraction(10) ** -scale) if scale is not None else significant(exact)
    except NumericError as error:
        return str(error)


def explicit_error(observed):
    target = observed["target"]
    if target == "sticky.evaluate":
        return observed.get("evaluate_error")
    if target == "sticky.format":
        return observed.get("format_error")
    if isinstance(observed.get("error"), str):
        return observed["error"]
    computed = observed.get("computed", {})
    text = computed.get("text", observed.get("text"))
    if not isinstance(text, str):
        return None
    if computed.get("t") == "error" and computed.get("v") is None:
        return text
    if text.startswith(("⚠", "Error:")):
        return text
    return None


def grade(case, expected, observed):
    target = observed["target"]
    if target == "sticky.evaluate" and observed.get("raw") in NON_FINITE:
        return [f"evaluate() returned {observed['raw']}"]
    error = explicit_error(observed)
    if isinstance(expected, str):
        return [] if error and re.search(ERROR_KIND[expected], error, re.I) else [
            f"Expected explicit numeric error ({expected}), observed {error!r}"
        ]
    if error:
        return [f"Unexpected error: {error}"]
    if observed.get("format_error"):
        return [f"Unexpected formatting error: {observed['format_error']}"]
    if target == "rust.view-sort":
        return [] if observed["order"] == expected else [f"Observed row order {observed['order']}, expected {expected}"]
    if target in {"rust.exact-meter", "sticky.exact-meter"}:
        if observed["charges"] == expected and observed["rejected"] == [True, True, True]:
            return []
        return [f"Expected all three operations rejected with charges {expected}, observed {observed}"]
    if target == "ui.editor":
        return [] if observed.get("token") == case["expected"]["token"] else ["The editor changed the decimal token"]
    if target == "ui.axis":
        ticks = [float(t) for t in observed["ticks"]]
        positions = [float(t) for t in observed["positions"]]
        if (2 <= len(ticks) <= 8 and len(ticks) == len(positions)
                and all(map(math.isfinite, ticks + positions))
                and all(a < b for a, b in zip(ticks, ticks[1:]))
                and all(a < b for a, b in zip(positions, positions[1:]))
                and ticks[0] <= float(case["axis_min"]) < float(case["input"]) <= ticks[-1]
                and positions[0] == 0 and positions[-1] == 1):
            return []
        return ["Expected a bounded, increasing finite scale that spans both endpoints"]

    failures = []
    scale = case["currency_scale"]
    if "binary64_bits" in case["expected"]:
        raw = observed["computed"]["v"] if target == "rust.computed" else observed.get("raw")
        actual_bits = struct.pack(">d", float(raw)).hex() if raw is not None else None
        if actual_bits != case["expected"]["binary64_bits"]:
            failures.append(f"binary64 bits: observed {actual_bits}, expected {case['expected']['binary64_bits']}")

    def compare(label, text, display=False):
        actual = None
        if display and scale is not None and isinstance(text, str):
            money = re.fullmatch(USD, text)
            if money:
                actual = Fraction("".join(part or "" for part in money.groups()).replace(",", ""))
        elif isinstance(text, str):
            if re.fullmatch(DECIMAL, text) or (not display and re.fullmatch(r"-?[0-9]+/[1-9][0-9]*", text)):
                actual = Fraction(text)
        if actual is not None:
            if scale is None and not display:
                actual = significant(actual)
            elif not display:
                actual = half_away(actual, Fraction(10) ** -scale)
        if actual != expected and actual is not None and not display and scale is None and "/" not in text:
            # The numeric wire/geometry value is binary64. At a decimal midpoint
            # its nearest representable value can lie on the opposite side.
            if float(text) == float(exact_value(case)):
                return
        if actual != expected:
            failures.append(f"{label}: observed {text!r}, expected {case['expected']['value']}")

    if target in {"rust.computed", "rust.rollup"}:
        computed = observed["computed"]
        if computed["t"] != "number":
            failures.append(f"computed type: observed {computed['t']!r}, expected 'number'")
        raw_value = computed["v"]
        compare("computed JSON value", str(raw_value) if type(raw_value) is Decimal else None)
        compare("computed display", computed["text"], display=True)
    else:
        if target in {"rust.formula", "rust.chart-total", "rust.donut-slice", "sticky.evaluate"}:
            compare("raw value", observed.get("exact", observed.get("raw")))
        compare("display", observed.get("text"), display=True)
        if target == "rust.donut-slice":
            compare("donut total", observed.get("total"), display=True)
            if observed["points"] != ["$0", "$0"]:
                failures.append("Expected the two individual points to display as $0")
    return failures


def command(*args, input=None, timeout=None):
    result = subprocess.run(args, cwd=REPO, input=input, capture_output=True, encoding="utf-8", timeout=timeout)
    if result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}: {result.stderr.strip()}")
    return result.stdout.strip()


def source_identity():
    # This checker also qualifies uncommitted repairs. Bind the receipt to the
    # files the adapters can consume, including newly added production modules.
    paths = command(
        "git", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--",
        "src-tauri", "src", "tools/db-bench",
        *((HERE.relative_to(REPO) / name).as_posix() for name in (
            "Cargo.lock", "Cargo.toml", "check.py", "fixtures.json", "rust_adapter.rs", "sticky_adapter.mjs"
        )),
    ).split("\0")
    digest = hashlib.sha256()
    for name in sorted(set(filter(None, paths))):
        path = REPO / name
        digest.update(name.encode("utf-8") + b"\0")
        digest.update(hashlib.sha256(path.read_bytes()).digest() if path.is_file() else b"missing\0")
    return {
        "commit": command("git", "rev-parse", "HEAD"),
        "tree": command("git", "rev-parse", "HEAD^{tree}"),
        "working_tree_sha256": digest.hexdigest(),
    }


def run_adapter(name, cases):
    payload = json.dumps(cases)
    if name == "sticky":
        output = command("node", str(HERE / "sticky_adapter.mjs"), input=payload, timeout=30)
    else:
        with TemporaryDirectory(prefix="notaya-numeric-ref-") as target:
            command("cargo", "build", "--locked", "--offline", "--quiet",
                    "--manifest-path", str(HERE / "Cargo.toml"), "--target-dir", target)
            executable = Path(target) / "debug" / ("numeric-ref-check.exe" if sys.platform == "win32" else "numeric-ref-check")
            output = command(str(executable), input=payload, timeout=60)
    # Preserve numeric JSON tokens as decimals when grading, including full serializer precision.
    observations = json.loads(output, parse_float=Decimal, parse_int=Decimal)
    for observation in observations:
        for key in ("order", "charges"):
            if key in observation:
                observation[key] = list(map(int, observation[key]))
    if [(o["id"], o["target"]) for o in observations] != [(c["id"], c["target"]) for c in cases]:
        raise ValueError(f"{name} adapter returned missing, extra, or misordered observations")
    return observations


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--control", choices=["good", "wrong"])
    parser.add_argument("--output", type=Path, help="Write full observations as JSON")
    args = parser.parse_args()
    source = source_identity()
    fixtures = json.loads((HERE / "fixtures.json").read_text(encoding="utf-8"))
    checks = []
    ids = set()
    for case in fixtures:
        if bool(case.get("control")) != bool(args.control):
            continue
        expected = oracle(case)
        manifest = case["expected"]
        if "order" in manifest:
            declared = manifest["order"]
        elif "charges" in manifest:
            declared = manifest["charges"]
        elif "value" in manifest:
            declared = Fraction(manifest["value"])
        else:
            declared = manifest["error"]
        if expected != declared:
            raise ValueError(f"{case['id']}: manifest {declared} disagrees with oracle {expected}")
        for target in case["targets"]:
            if target not in TARGETS:
                raise ValueError(f"Unknown target: {target}")
            key = (case["id"], target)
            if key in ids:
                raise ValueError(f"Duplicate check: {key}")
            ids.add(key)
            checks.append(({**case, "target": target}, expected))
    if not checks or (args.control and len(checks) != 1):
        raise ValueError("Expected one control or a nonempty suite")

    results = []
    for adapter in ("sticky", "rust"):
        selected = [(case, expected) for case, expected in checks
                    if case["target"].startswith(adapter + ".")
                    or (adapter == "sticky" and case["target"].startswith("ui."))]
        if not selected:
            continue
        for (case, expected), observation in zip(selected, run_adapter(adapter, [c for c, _ in selected])):
            observed = {**observation, "text": "6"} if args.control == "wrong" else observation
            failures = grade(case, expected, observed)
            results.append({"id": case["id"], "target": case["target"], "expected": case["expected"],
                            "observed": observed, "failures": failures})

    if source != source_identity():
        raise ValueError("Source identity changed during the run")
    failed = sum(bool(result["failures"]) for result in results)
    passed = len(results) - failed
    for result in results:
        if result["failures"]:
            print(f"FAIL {result['id']} / {result['target']}: " + "; ".join(result["failures"]))
    print(f"{passed} passed, {failed} failed ({len(results)} checks) on working tree {source['working_tree_sha256']}")
    if args.output:
        report = {**source, "control": args.control, "passed": passed, "failed": failed, "results": results}
        args.output.write_text(json.dumps(report, indent=2, default=str) + "\n", encoding="utf-8")
    return int(failed != 0)


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, RuntimeError, ValueError, KeyError, subprocess.TimeoutExpired) as error:
        print(f"HARNESS ERROR: {error}", file=sys.stderr)
        sys.exit(2)
