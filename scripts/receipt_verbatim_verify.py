#!/usr/bin/env python3
"""receipt_verbatim_verify.py — VERBATIM_MISMATCH enforcement (LANE-1).

Diff-checks a receipt's cited verbatim output block against the raw log
file it references (the BM-3 precedent: "Source of record for run output:
`/tmp/bm3_runs_v2.log` (lines 11-23); the block above is a byte-faithful
copy of those lines").

Recognition contract (per receipt .md):
  * a "Verbatim run output" marker introduces a ```text fenced block;
  * a "Source of record for run output: `PATH` (lines A-B)" line names the
    raw log and the cited 1-based, INCLUSIVE line range.
A marker without a source line, or a source line without a block, is a
typed fail-closed finding (a verbatim claim that cannot be replayed is
not a receipt).

Exit ABI:
  0  all verbatim blocks byte-faithful (or no verbatim claims present)
  1  VERBATIM_MISMATCH / VERBATIM_SOURCE_MISSING / VERBATIM_RANGE /
     RECEIPT_UNREADABLE  (drift or unplayable claim — fail-closed)
  3  loud skip: VERBATIM_LOG_MISSING (raw /tmp log no longer on disk —
     typed per receipt, never silent; aggregate exit 3 only when no exit-1
     finding exists)

`--self-check` builds clean/poison twins (receipt + raw log in a temp
workdir) and runs the production checker on both; exits nonzero printing
`VACUOUS GATE` unless poison HITS (byte drift detected) and clean passes.
"""

from __future__ import annotations

import argparse
import os
import re
import sys
import tempfile

EXIT_OK = 0
EXIT_GATE_FAIL = 1
EXIT_LOUD_SKIP = 3

MARKER_RE = re.compile(r"Verbatim run output")
SOURCE_RE = re.compile(
    r"Source of record for run output:\s*`([^`]+)`\s*\((?:lines?\s+)?(\d+)-(\d+)\)")


def check_receipt(path: str, root: str) -> tuple[list[str], list[str]]:
    """Production predicate. Returns (hard_findings, loud_skips).

    hard_findings exit 1; loud_skips (log missing) are typed and aggregate
    to exit 3 only absent hard findings.
    """
    hard: list[str] = []
    skips: list[str] = []
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            text = fh.read()
    except OSError as e:
        return [f"RECEIPT_UNREADABLE {path}: {e}"], []

    src = SOURCE_RE.search(text)
    has_marker = bool(MARKER_RE.search(text))
    if has_marker and not src:
        hard.append(
            f"VERBATIM_SOURCE_MISSING {path}: verbatim claim names no "
            f"raw-log source-of-record line")
        return hard, skips

    if not src:
        return [], []  # no verbatim claim -> nothing to replay

    log_path = os.path.normpath(os.path.join(root, src.group(1)) \
        if not os.path.isabs(src.group(1)) else src.group(1))
    lo, hi = int(src.group(2)), int(src.group(3))
    if lo < 1 or hi < lo:
        hard.append(
            f"VERBATIM_RANGE {path}: cited range {lo}-{hi} is not a "
            f"non-empty 1-based inclusive range")
        return hard, skips

    try:
        with open(log_path, encoding="utf-8", errors="replace") as fh:
            log_lines = fh.read().split("\n")
    except OSError:
        return hard, [f"VERBATIM_LOG_MISSING {path}: raw log absent: {src.group(1)}"]

    if hi > len(log_lines):
        hard.append(
            f"VERBATIM_RANGE {path}: cited lines {lo}-{hi} exceed log "
            f"length {len(log_lines)} ({log_path})")
        return hard, skips

    # The verbatim block: the first ```text fence after the marker
    # (or before the source line — BM-3 layout has block then source).
    cited = log_lines[lo - 1:hi]
    block = _verbatim_block(text)
    if block is None:
        hard.append(
            f"VERBATIM_SOURCE_MISSING {path}: source-of-record cites lines "
            f"but no ```text verbatim block exists")
        return hard, skips

    if block != cited:
        first = next(
            (i for i, (a, b) in enumerate(zip(block, cited)) if a != b),
            min(len(block), len(cited)),
        )
        hard.append(
            f"VERBATIM_MISMATCH {path}: block line {first + 1} drifts from "
            f"{log_path}:{lo + first} (block has {len(block)} lines, log "
            f"range has {len(cited)})")
    return hard, skips


def _verbatim_block(text: str) -> list[str] | None:
    """First ```text fenced block; lines with the fence rows removed."""
    lines = text.split("\n")
    for i, line in enumerate(lines):
        if line.strip() == "```text":
            for j in range(i + 1, len(lines)):
                if lines[j].strip() == "```":
                    return lines[i + 1:j]
            return None  # unterminated fence
    return None


def scan(paths: list[str], root: str) -> int:
    hard_all: list[str] = []
    skip_all: list[str] = []
    total = 0
    for p in paths:
        hard, skips = check_receipt(p, root)
        total += 1
        hard_all += hard
        skip_all += skips
    print(f"verbatim verify: {total} receipt(s), {len(hard_all)} hard "
          f"finding(s), {len(skip_all)} loud skip(s)")
    for f in hard_all:
        print(f"FAIL: {f}")
    for s in skip_all:
        print(f"[SKIP] {s}")
    if hard_all:
        return EXIT_GATE_FAIL
    if skip_all:
        return EXIT_LOUD_SKIP
    print("PASS: every cited verbatim block is byte-faithful to its raw log")
    return EXIT_OK


def self_check() -> int:
    """Poison/clean twins: receipt + raw log in a temp workdir."""
    body_lines = [
        "    Finished `release` profile [optimized] target(s) in 5.75s",
        "running 3 tests",
        "[run 0] append 100k: 201.2ms",
        "test result: ok. 3 passed; 0 failed",
    ]
    block_src = "\n".join(body_lines)
    receipt_tpl = (
        "# BM-X receipt\n\n"
        "Verbatim run output:\n\n"
        "```text\n{block}\n```\n\n"
        "Source of record for run output: `{log}` (lines 1-4);\n"
        "the block above is a byte-faithful copy of those lines.\n"
    )

    ok = True
    with tempfile.TemporaryDirectory(prefix="rvv-selfcheck-") as tmp:
        log_path = os.path.join(tmp, "raw.log")
        with open(log_path, "w", encoding="utf-8") as fh:
            fh.write(block_src + "\n")

        # Clean twin: byte-faithful block -> must PASS.
        clean = os.path.join(tmp, "clean.md")
        with open(clean, "w", encoding="utf-8") as fh:
            fh.write(receipt_tpl.format(block=block_src, log=log_path))
        hard, skips = check_receipt(clean, tmp)
        if hard or skips:
            print(f"VACUOUS GATE: clean twin hit: {hard or skips}", file=sys.stderr)
            ok = False

        # Poison twin 1: drift one byte in the receipt's block -> MUST HIT.
        poison = os.path.join(tmp, "poison.md")
        drifted = block_src.replace("201.2ms", "201.3ms")
        with open(poison, "w", encoding="utf-8") as fh:
            fh.write(receipt_tpl.format(block=drifted, log=log_path))
        hard, _ = check_receipt(poison, tmp)
        if not any(h.startswith("VERBATIM_MISMATCH") for h in hard):
            print(f"VACUOUS GATE: poison byte-drift not detected: {hard}",
                  file=sys.stderr)
            ok = False

        # Poison twin 2: raw log drifts under a faithful receipt -> MUST HIT.
        poison2 = os.path.join(tmp, "poison2.md")
        with open(poison2, "w", encoding="utf-8") as fh:
            fh.write(receipt_tpl.format(block=block_src, log=log_path))
        with open(log_path, "a", encoding="utf-8") as fh:
            fh.write("extra line\n")
        hard, _ = check_receipt(poison2, tmp)
        # (log growth BEYOND the cited range is not drift of the claim —
        # the cited lines 1-4 are still byte-faithful; no hit expected here.)
        # Now corrupt WITHIN the cited range:
        with open(log_path, "r+", encoding="utf-8") as fh:
            lines = fh.read().split("\n")
            lines[1] = "running 4 tests"  # inside lines 1-4
            fh.seek(0)
            fh.truncate()
            fh.write("\n".join(lines))
        hard, _ = check_receipt(poison2, tmp)
        if not any(h.startswith("VERBATIM_MISMATCH") for h in hard):
            print(f"VACUOUS GATE: in-range log corruption not detected: {hard}",
                  file=sys.stderr)
            ok = False

        # Poison twin 3: marker without source line -> MUST HIT.
        poison3 = os.path.join(tmp, "poison3.md")
        with open(poison3, "w", encoding="utf-8") as fh:
            fh.write("Verbatim run output:\n\n```text\nx\n```\n")
        hard, _ = check_receipt(poison3, tmp)
        if not any(h.startswith("VERBATIM_SOURCE_MISSING") for h in hard):
            print(f"VACUOUS GATE: source-less verbatim claim not detected: {hard}",
                  file=sys.stderr)
            ok = False

        # Loud-skip twin: missing log -> typed skip, no hard finding.
        poison4 = os.path.join(tmp, "poison4.md")
        with open(poison4, "w", encoding="utf-8") as fh:
            fh.write(receipt_tpl.format(block=block_src,
                                        log=os.path.join(tmp, "gone.log")))
        hard, skips = check_receipt(poison4, tmp)
        if hard or not skips:
            print(f"VACUOUS GATE: missing-log must be loud skip, got hard={hard} "
                  f"skips={skips}", file=sys.stderr)
            ok = False

    if not ok:
        print("VACUOUS GATE", file=sys.stderr)
        return EXIT_GATE_FAIL
    print("SELF-CHECK OK: byte-drift HIT, in-range log corruption HIT, "
          "source-less claim HIT, missing log loud-skips, clean PASSes")
    return EXIT_OK


def main() -> int:
    ap = argparse.ArgumentParser(
        description="VERBATIM_MISMATCH enforcement: receipt block vs raw log (LANE-1)")
    ap.add_argument("receipts", nargs="*", help="receipt .md files to verify")
    ap.add_argument("--dir", default=None,
                    help="scan a receipts directory for *.md (default: none)")
    ap.add_argument("--root", default=os.getcwd(),
                    help="root for resolving relative log paths")
    ap.add_argument("--self-check", action="store_true")
    args = ap.parse_args()

    if args.self_check:
        return self_check()

    paths = list(args.receipts)
    if args.dir:
        for name in sorted(os.listdir(args.dir)):
            if name.endswith(".md"):
                paths.append(os.path.join(args.dir, name))
    if not paths:
        print("usage: pass receipts or --dir", file=sys.stderr)
        return 2
    return scan(paths, args.root)


if __name__ == "__main__":
    sys.exit(main())
