#!/usr/bin/env python3
"""Count the Claude tokens in one or more Claude Code transcripts, for the Notaya usage ledger.

Usage (read-only, standard library only, writes nothing):

    python3 -I tools/usage-ledger/scan_claude_transcript.py TRANSCRIPT.jsonl [MORE.jsonl ...]
        [--since 2026-11-07T20:00:00Z] [--until 2026-11-07T21:00:00Z] [--csv]

A Claude Code transcript is the session's `.jsonl` file (on Windows under
`%USERPROFILE%\\.claude\\projects\\<folder>\\`, in a cloud container under
`~/.claude/projects/<folder>/`). Each Claude reply can be written to it more
than once while it streams, so every reply is counted once, by its message id.

What it prints, per model and in total:
- replies: Claude replies counted
- input, cache_write, cache_read, output: the raw token counts the API reported
- weighted: input x1 + cache_write x1.25 + cache_read x0.1 + output x5
  (the Notaya ledger's comparison unit; see docs/ledger/README.md)

It never prints message text, prompts, file contents or tool output: only
token counts, model names and the first and last reply time.
"""

import argparse
import json
import sys
from datetime import datetime, timezone

WEIGHTS = {"input": 1.0, "cache_write": 1.25, "cache_read": 0.1, "output": 5.0}


def parse_time(value):
    """Return an aware UTC datetime for an ISO 8601 string, or None."""
    if not value:
        return None
    try:
        text = value.strip()
        if text.endswith("Z"):
            text = text[:-1] + "+00:00"
        moment = datetime.fromisoformat(text)
    except ValueError:
        return None
    if moment.tzinfo is None:
        moment = moment.replace(tzinfo=timezone.utc)
    return moment.astimezone(timezone.utc)


def replies(paths):
    """Yield (key, model, usage, time) once per Claude reply across all files."""
    seen = {}
    for path in paths:
        with open(path, "r", encoding="utf-8", errors="replace") as handle:
            for line in handle:
                line = line.strip()
                if not line:
                    continue
                try:
                    record = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if not isinstance(record, dict):
                    continue
                message = record.get("message")
                if not isinstance(message, dict):
                    continue
                usage = message.get("usage")
                if not isinstance(usage, dict):
                    continue
                key = message.get("id") or record.get("requestId") or record.get("uuid")
                if not key:
                    continue
                # The last copy of a streamed reply carries the final counts.
                seen[key] = (message.get("model") or "unknown", usage, parse_time(record.get("timestamp")))
    for key, (model, usage, moment) in seen.items():
        yield key, model, usage, moment


def number(usage, name):
    value = usage.get(name, 0)
    return int(value) if isinstance(value, (int, float)) else 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("transcripts", nargs="+", help="one or more Claude Code .jsonl transcripts")
    parser.add_argument("--since", help="count only replies at or after this UTC time (ISO 8601)")
    parser.add_argument("--until", help="count only replies before this UTC time (ISO 8601)")
    parser.add_argument("--csv", action="store_true", help="print CSV rows instead of a table")
    args = parser.parse_args(argv)

    since = parse_time(args.since) if args.since else None
    until = parse_time(args.until) if args.until else None
    if (args.since and since is None) or (args.until and until is None):
        parser.error("--since and --until take an ISO 8601 time such as 2026-11-07T20:00:00Z")

    totals = {}
    first = last = None
    for _key, model, usage, moment in replies(args.transcripts):
        if since and (moment is None or moment < since):
            continue
        if until and (moment is None or moment >= until):
            continue
        row = totals.setdefault(model, {"replies": 0, "input": 0, "cache_write": 0, "cache_read": 0, "output": 0})
        row["replies"] += 1
        row["input"] += number(usage, "input_tokens")
        row["cache_write"] += number(usage, "cache_creation_input_tokens")
        row["cache_read"] += number(usage, "cache_read_input_tokens")
        row["output"] += number(usage, "output_tokens")
        if moment is not None:
            first = moment if first is None or moment < first else first
            last = moment if last is None or moment > last else last

    columns = ["replies", "input", "cache_write", "cache_read", "output"]
    grand = {name: sum(row[name] for row in totals.values()) for name in columns}

    def weighted(row):
        return round(sum(row[name] * WEIGHTS[name] for name in WEIGHTS))

    stamp = lambda moment: moment.strftime("%Y-%m-%dT%H:%M:%SZ") if moment else "not recorded"
    if args.csv:
        print("model," + ",".join(columns) + ",weighted,first_reply_utc,last_reply_utc")
        for model in sorted(totals):
            row = totals[model]
            print(model + "," + ",".join(str(row[n]) for n in columns) + f",{weighted(row)},{stamp(first)},{stamp(last)}")
        print("TOTAL," + ",".join(str(grand[n]) for n in columns) + f",{weighted(grand)},{stamp(first)},{stamp(last)}")
        return 0

    print(f"First reply: {stamp(first)}   Last reply: {stamp(last)}")
    header = f"{'model':<28}" + "".join(f"{n:>14}" for n in columns) + f"{'weighted':>14}"
    print(header)
    print("-" * len(header))
    for model in sorted(totals):
        row = totals[model]
        print(f"{model:<28}" + "".join(f"{row[n]:>14,}" for n in columns) + f"{weighted(row):>14,}")
    print(f"{'TOTAL':<28}" + "".join(f"{grand[n]:>14,}" for n in columns) + f"{weighted(grand):>14,}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
