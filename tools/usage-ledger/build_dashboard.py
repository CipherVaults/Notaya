#!/usr/bin/env python3
"""Build the Notaya usage-ledger dashboard page from docs/ledger/*.csv.

Usage (from the repository root):
    python3 -I tools/usage-ledger/build_dashboard.py OUTPUT.html

It reads claude-sessions.csv, codex-runs.csv and quota-readings.csv, keeps the
newest reading of each session, and writes one self-contained HTML page from
tools/usage-ledger/dashboard-template.html. The page is published as an
Artifact; republish the same file path to the same link when the ledger changes.
The CSV files own the facts; this script only arranges them. Nothing is guessed:
cost estimates for rows with no reported cost are marked as estimates and are
derived only from the ratios of rows that do report a cost.
"""
import csv
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LEDGER = ROOT / "docs" / "ledger"
TEMPLATE = Path(__file__).with_name("dashboard-template.html")

CREDIT_TOTAL_USD = 100
# Fallbacks used only when quota-readings.csv has no measured credit row. Ahmed (project
# chat, 2026-11-07 19:49:26 UTC): "less than 24 hours"; the coordinator assumed 19:00 UTC.
CREDIT_EXPIRES_UTC = "2026-11-08T19:00:00Z"


def read(name):
    with open(LEDGER / name, newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


def num(v):
    v = (v or "").strip()
    try:
        return float(v)
    except ValueError:
        return None


def iso(v):
    """Return an ISO UTC string, or None when the text is not a plain timestamp."""
    v = (v or "").strip()
    m = re.search(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}(:\d{2})?Z", v)
    if not m:
        return None
    s = m.group(0)
    return s if len(s) > 17 else s[:-1] + ":00Z"


def model_label(served, configured):
    t = (served or "").lower()
    if t in ("", "not recorded"):
        t = (configured or "").lower()
    has_o, has_s = "opus" in t, "sonnet" in t
    if has_o and has_s:
        return "Opus + Sonnet 5.5"
    if has_o:
        return "Opus 5.5"
    if has_s:
        return "Sonnet 5.5"
    return "not recorded"


def main(out):
    raw = read("claude-sessions.csv")
    # Newest reading of each (session, start) wins; an older reading is replaced, never added.
    latest = {}
    for r in raw:
        key = (r["session_id"], r["start_utc"])
        if key not in latest or r["snapshot_utc"] >= latest[key]["snapshot_utc"]:
            latest[key] = r
    sessions = []
    for r in latest.values():
        w = num(r["weighted"])
        where = r["where"]
        machine = "Cloud" if where.lower().startswith("cloud") else where
        sessions.append({
            "id": r["session_id"],
            "partner": r["partner"],
            "account": "Ahmed's Claude account" if "Ahmed" in r["partner"] else "not recorded",
            "machine": machine,
            "where": where,
            "role": r["role"],
            "task": r["thread_or_task"],
            "model": model_label(r["model_last_served"], r["model_configured"]),
            "effort": r["effort"],
            "start": iso(r["start_utc"]),
            "end": iso(r["last_update_utc"]) or iso(r["snapshot_utc"]),
            "input": num(r["input"]), "cw": num(r["cache_write"]),
            "cr": num(r["cache_read"]), "out": num(r["output"]),
            "weighted": w, "cost": num(r["cost_usd_reported"]),
            "snapshot": r["snapshot_utc"], "source": r["source"], "notes": r["notes"],
            "running": bool(re.search(r"still running|running at the snapshot|^running", r["notes"], re.I)),
        })
    sessions.sort(key=lambda s: s["start"] or "")

    # Observed dollars per million weighted tokens, by model, from rows that report a cost.
    rates = {"Opus 5.5": [], "Sonnet 5.5": []}
    for s in sessions:
        if s["cost"] and s["weighted"] and s["model"] in rates:
            rates[s["model"]].append(s["cost"] / s["weighted"] * 1e6)
    rate_range = {k: (min(v), max(v)) for k, v in rates.items() if v}

    est_low = est_high = 0.0
    for s in sessions:
        s["est"] = None
        if s["cost"] is not None or not s["weighted"]:
            continue
        parts = []
        if s["model"] == "Opus + Sonnet 5.5":
            mo = re.search(r"Opus 5\.5: [^;]*?weighted ([\d,]+)", s["notes"])
            ms = re.search(r"Sonnet 5\.5: [^;]*?weighted ([\d,]+)", s["notes"])
            if mo and ms:
                parts = [("Opus 5.5", float(mo.group(1).replace(",", ""))),
                         ("Sonnet 5.5", float(ms.group(1).replace(",", "")))]
        elif s["model"] in rate_range:
            parts = [(s["model"], s["weighted"])]
        if not parts:
            continue
        lo = sum(wt * rate_range[m][0] / 1e6 for m, wt in parts)
        hi = sum(wt * rate_range[m][1] / 1e6 for m, wt in parts)
        s["est"] = [round(lo, 2), round(hi, 2)]
        est_low += lo
        est_high += hi

    # Measured readings (screens), newest first in quota-readings.csv: first match of each wins.
    quota = read("quota-readings.csv")

    def first(pred):
        return next((q for q in quota if pred(q)), None)

    measured = {"five": None, "week": None, "credit": None, "project": None}
    q = first(lambda q: q["tool"] == "Claude" and "five" in q["limit"] and "%" in q["reading"])
    if q:
        measured["five"] = {"pct": float(re.search(r"(\d+)%", q["reading"]).group(1)), "resets": q["resets_utc"], "time": q["time_utc"], "by": q["read_by"]}
    q = first(lambda q: q["tool"] == "Claude" and "weekly" in q["limit"] and "%" in q["reading"])
    if q:
        measured["week"] = {"pct": float(re.search(r"(\d+)%", q["reading"]).group(1)), "resets": q["resets_utc"], "time": q["time_utc"], "by": q["read_by"]}
    q = first(lambda q: "credit" in q["limit"] and re.search(r"USD (\d+) of USD (\d+) left", q["reading"]))
    if q:
        m = re.search(r"USD (\d+) of USD (\d+) left", q["reading"])
        measured["credit"] = {"left": float(m.group(1)), "total": float(m.group(2)), "expires": q["resets_utc"], "time": q["time_utc"], "by": q["read_by"]}
    q = first(lambda q: "project usage" in q["limit"])
    if q:
        measured["project"] = {"reading": q["reading"], "time": q["time_utc"]}

    screen = read("project-usage-screen.csv") if (LEDGER / "project-usage-screen.csv").exists() else []

    reported = sum(s["cost"] for s in sessions if s["cost"] is not None)
    snap = max([s["snapshot"] for s in sessions] + [q["time_utc"] for q in quota])
    snap_iso = iso(snap)

    def tot(k):
        return sum(s[k] or 0 for s in sessions)

    by_model = {}
    by_machine = {}
    for s in sessions:
        for d, k in ((by_model, s["model"]), (by_machine, s["machine"])):
            e = d.setdefault(k, {"n": 0, "weighted": 0, "out": 0, "cost": 0.0, "costed": 0})
            e["n"] += 1
            e["weighted"] += s["weighted"] or 0
            e["out"] += s["out"] or 0
            if s["cost"] is not None:
                e["cost"] += s["cost"]
                e["costed"] += 1

    data = {
        "snapshot": snap_iso,
        "credit": {"total": CREDIT_TOTAL_USD, "expires": CREDIT_EXPIRES_UTC},
        "measured": measured,
        "screen": [{"thread": r["thread_title"], "model": r["model_shown"], "last": r["last_activity_shown"],
                    "tokens": r["tokens_shown"], "cache": r["cache_hit_shown"], "share": r["share_shown"],
                    "time": r["reading_utc"]} for r in screen],
        "reported": round(reported, 2),
        "estimate": [round(est_low, 2), round(est_high, 2)],
        "rates": {k: [round(v[0], 2), round(v[1], 2)] for k, v in rate_range.items()},
        "totals": {"input": tot("input"), "cw": tot("cw"), "cr": tot("cr"),
                   "out": tot("out"), "weighted": tot("weighted")},
        "byModel": by_model,
        "byMachine": by_machine,
        "sessions": sessions,
        "quota": [
            {"time": q["time_utc"], "tool": q["tool"], "account": q["account"], "limit": q["limit"],
             "reading": q["reading"], "resets": q["resets_utc"], "by": q["read_by"], "notes": q["notes"]}
            for q in quota
        ],
        "codex": read("codex-runs.csv"),
        "window": {"start": "2026-11-07T19:00:00Z", "end": "2026-11-08T00:00:00Z"},
        "built": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%MZ"),
    }
    html = TEMPLATE.read_text(encoding="utf-8").replace(
        "/*LEDGER_DATA*/null", json.dumps(data, ensure_ascii=False))
    Path(out).write_text(html, encoding="utf-8")
    print(f"{len(sessions)} sessions, reported USD {reported:.2f}, estimate {est_low:.2f} to {est_high:.2f}, "
          f"weighted {tot('weighted'):,.0f}, written to {out}")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
