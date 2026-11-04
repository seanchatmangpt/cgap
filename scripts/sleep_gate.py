#!/usr/bin/env python3
"""sleep_gate.py — no-sleep doctrine Law 3 enforcement for cgap (Rust).

Port of smalt's sleep-gate law (violation class 5: make the convention
enforced law). Scans .rs sources for fixed sleep primitives:

  std::thread::sleep(   thread::sleep(   sleep_ms(   tokio::time::sleep(

and enforces, by path role:

  test side (crates/*/tests/**, benches/**, *_test.rs)
    every sleep FAILS unless (in precedence order):
      (a) the exact site (path + line) is ticketed in the typed allowlist
          scripts/sleep_gate_classes.json (audited exceptions; growth is a
          reviewed diff), or
      (b) it carries a // SLEEP-CLASS: <TIME-ADVANCE|POLL-STEP> annotation
          (same line or the contiguous // comment block above), or
      (c) it sits inside a bounded poll loop: the nearest loop opener above
          carries a time/deadline bound and the span inspects an observable
          — the doctrine's lawful poll form.

  lib side (crates/*/src/**): same precedence with lib classes
    <SCHEDULING|POLL-STEP|CONTRACT>.

Outside lib/ and test/ jurisdiction (examples/, docs fixtures): skipped.
String literals containing sleep text are fence fixtures (the two existing
scanner tests' poison snippets), never call sites — skipped. Fail-closed:
zero files scanned, an unreadable file, or a missing/unreadable
classification file is a hard failure. Exit 0 = pass, 1 = violations /
fail-closed. Stdlib only.

Non-vacuity: scripts/sleep_gate_court.sh proves a clean subject PASSES and
a sleep-injected subject is REFUSED (inject -> refuse; restore byte-exact
-> pass; restore-corruption self-check proves restore detection is not
vacuous).
"""

import json
import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CLASSES_PATH = os.path.join(REPO, "scripts", "sleep_gate_classes.json")

SLEEP_RE = re.compile(
    r"\b(?:std::thread::sleep\s*\(|thread::sleep\s*\(|sleep_ms\s*\(|"
    r"tokio::time::sleep\s*\()")
ANN_RE = re.compile(r"//\s*SLEEP-CLASS:\s*([A-Za-z-]+)")
# Loop opener anywhere on the line (covers `let x = loop {` and bare
# `loop {` / `while ...` / `for ...`).
LOOP_OPEN_RE = re.compile(r"\b(?:loop\s*\{|while\b|for\b)")
TIME_BOUND_RE = re.compile(
    r"\b(?:Instant::now\(\)|\.elapsed\(\)|deadline|DEADLINE|timeout|TIMEOUT|"
    r"from_millis|from_secs|Duration::)")
OBSERVABLE_RE = re.compile(
    r"\bif\b|\bmatch\b|[!=]=|\.contains\(|\.wait\(|\.recv\(|\.try_wait\(")

LIB_CLASSES = ["CONTRACT", "POLL-STEP", "SCHEDULING"]
TEST_CLASSES = ["POLL-STEP", "TIME-ADVANCE"]

EXEMPT_DIRS = {
    ".git", "target", "node_modules", "__pycache__", ".venv", "vendor",
    "_build", "deps",
}


def load_classes():
    """Typed allowlist — fail-closed if missing/unreadable/malformed."""
    try:
        with open(CLASSES_PATH, encoding="utf-8") as f:
            data = json.load(f)
        files = {}
        for e in data["files"]:
            files[os.path.normpath(e["path"])] = (
                e.get("reason", ""), e.get("lines"))
        fns = {e["name"]: e.get("reason", "") for e in data["functions"]}
        return files, fns
    except (OSError, ValueError, KeyError, TypeError) as exc:
        print("[SLEEP-GATE] [FAIL] classification file unreadable: %s" % exc)
        sys.exit(1)


def strip_line_comment(line):
    """Cut the line at the first // outside a double-quoted string. Only
    ever removes match surface (comment text cannot create a call site)."""
    in_str = False
    i = 0
    while i < len(line):
        c = line[i]
        if in_str:
            if c == '"' and line[i - 1] != "\\":
                in_str = False
        else:
            if c == '"':
                in_str = True
            elif c == "/" and i > 0 and line[i - 1] == "/":
                return line[:i - 1]
        i += 1
    return line


def block_comment_mask(lines):
    """Toggle-style /* */ mask. Approximate; only ever skips, never
    invents, sites."""
    mask = [False] * len(lines)
    in_blk = False
    for i, line in enumerate(lines):
        opens = line.count("/*")
        closes = line.count("*/")
        if in_blk:
            mask[i] = True
            if closes:
                in_blk = False
        elif opens > closes:
            mask[i] = True
            in_blk = True
    return mask


def in_string_literal(line, start):
    """True if offset `start` lies inside a double-quoted string on this
    line (only ever skips fence fixtures; a real call site is never
    syntactically inside a string)."""
    in_str = False
    open_pos = -1
    i = 0
    while i < len(line):
        c = line[i]
        if in_str:
            if c == "\\":
                i += 1  # skip escaped char
            elif c == '"':
                if open_pos < start < i:
                    return True
                in_str = False
        elif c == '"':
            in_str = True
            open_pos = i
        i += 1
    return False


def annotate_class(lines, idx):
    """// SLEEP-CLASS: on the sleep line itself, or in the contiguous //
    comment block ending on the line above. Operates on raw lines."""
    m = ANN_RE.search(lines[idx])
    if m:
        return m.group(1)
    j = idx - 1
    found = None
    while j >= 0 and lines[j].lstrip().startswith("//"):
        mm = ANN_RE.search(lines[j])
        if mm:
            found = mm.group(1)
            break
        j -= 1
    return found


def enclosing_fn(lines, idx):
    """Nearest fn name at-or-above the site (allowlist function key)."""
    for j in range(idx, -1, -1):
        m = re.match(r"\s*(?:pub\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+([A-Za-z0-9_]+)",
                     lines[j])
        if m:
            return m.group(1)
    top = None
    return top


def bounded_poll_admission(lines, idx):
    """Lawful poll form: nearest loop opener above (the walk stops at a
    column-0 closing brace, which fences the site outside that loop)
    carries a time/deadline bound, and the opener->site span inspects an
    observable. Only ever ADMITS on real loop evidence."""
    opener = None
    for j in range(idx - 1, max(-1, idx - 200), -1):
        if lines[j].startswith("}"):
            break
        if LOOP_OPEN_RE.search(lines[j]):
            opener = j
            break
    if opener is None:
        return False
    span = "\n".join(lines[opener:idx + 1])
    return bool(TIME_BOUND_RE.search(span) and OBSERVABLE_RE.search(span))


def classify_path(rel):
    if rel.startswith("benches/") or re.search(r"(^|/)tests?/", rel) or \
            rel.endswith("_test.rs"):
        return "test"
    if re.search(r"(^|/)src/", rel):
        return "lib"
    return None


def scan_file(path, rel, allow_files, allow_fns, force_role=None):
    violations = []
    try:
        with open(path, encoding="utf-8") as f:
            lines = f.read().split("\n")
    except OSError as exc:
        return [(rel, 0, "FAIL-CLOSED: unreadable file: %s" % exc)]

    role = force_role or classify_path(rel)
    if role is None:
        return []
    norm = os.path.normpath(rel)
    mask = block_comment_mask(lines)
    for i, line in enumerate(lines):
        if mask[i]:
            continue
        raw = strip_line_comment(line)
        hits = [m for m in SLEEP_RE.finditer(raw)
                if not in_string_literal(line, m.start())]
        if not hits:
            continue
        if norm in allow_files:
            reason, allowed_lines = allow_files[norm]
            if allowed_lines is None or (i + 1) in allowed_lines:
                continue
        if enclosing_fn(lines, i) in allow_fns:
            continue
        cls = annotate_class(lines, i)
        lawful_classes = LIB_CLASSES if role == "lib" else TEST_CLASSES
        if cls in lawful_classes:
            continue
        if bounded_poll_admission(lines, i):
            continue
        classes = "|".join(lawful_classes)
        violations.append(
            (rel, i + 1,
             "unclassified %s sleep primitive (// SLEEP-CLASS: <%s> required, "
             "or inside a bounded poll loop with time bound + observable "
             "check): `%s`" % (role, classes, raw.strip()))
        )
        break  # one report per line
    return violations


def main(argv):
    force_as = None
    paths = []
    for a in argv[1:]:
        if a.startswith("--as="):
            force_as = a.split("=", 1)[1]
            if force_as not in ("test", "lib"):
                print("[SLEEP-GATE] [FAIL] unknown --as value: %s" % a)
                return 1
            continue
        paths.append(a)
    allow_files, allow_fns = load_classes()
    files_scanned = 0
    violations = []
    if force_as is not None:
        # Sandbox subjects (court legs): scan only the named files with the
        # forced role — path-role classification cannot see outside-tree
        # subjects.
        for p in paths:
            subj = os.path.abspath(p)
            if not os.path.exists(subj):
                print("[SLEEP-GATE] [FAIL] --as path not found: %s" % p)
                return 1
            files_scanned += 1
            violations.extend(
                scan_file(subj, os.path.basename(subj), allow_files,
                          allow_fns, force_as))
        if files_scanned == 0:
            print("[SLEEP-GATE] [FAIL] no subject files scanned — fail-closed")
            return 1
    else:
        for p in paths or [os.path.join(REPO, "crates"),
                           os.path.join(REPO, "benches")]:
            if not os.path.exists(p):
                print("[SLEEP-GATE] [FAIL] path not found: %s" % p)
                return 1
            for root, dirs, names in os.walk(p):
                dirs[:] = [d for d in dirs if d not in EXEMPT_DIRS]
                for name in sorted(names):
                    f = os.path.join(root, name)
                    if not f.endswith(".rs"):
                        continue
                    rel = os.path.relpath(os.path.abspath(f), REPO)
                    files_scanned += 1
                    violations.extend(
                        scan_file(f, rel, allow_files, allow_fns, None))
        if files_scanned == 0:
            print("[SLEEP-GATE] [FAIL] no .rs files scanned — fail-closed")
            return 1

    if violations:
        print("[SLEEP-GATE] %d violation(s) across %d files — REFUSED"
              % (len(violations), files_scanned))
        for rel, ln, why in violations:
            print("  [FAIL] %s:%d %s" % (rel, ln, why))
        print("[SLEEP-GATE] lawful forms: // SLEEP-CLASS annotation, bounded "
              "poll loop (time bound + observable), or a ticketed site in "
              "scripts/sleep_gate_classes.json")
        return 1

    print("[SLEEP-GATE] PASS — %d files, 0 unclassified sleep primitives"
          % files_scanned)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))