#!/usr/bin/env python3
"""lease_janitor.py — lane build-root lease enforcement (v26.11.2 DX-3).

Law (same-checkout-fanout cleanup law): lane build roots are leases, not
assets.  Lanes that had `rm` permission-blocked accumulated /tmp debris
(exp0*-target, fix0*-target, sandboxes, coverage dirs, probe dirs).

Grammar (documented, explicit, conservative — a path not matching is
REFUSED, never deleted; fail-closed):

    *-target        lane cargo build-root leases        e.g. exp02-target
    *-target-*      suffixed build-root variants        e.g. a1-target-2
    *-sandbox*      lane sandbox dirs (incl. -sandbox-target)
    *-cov           coverage dirs
    *-scratch*      scratch dirs
    *-probe         probe dirs
    csct-l*-*, csct-l*-  lane-keyed csct debris
    lspa*           lspa lane debris
    dx*             dx lane debris

Active vs orphaned: mtime within ACTIVE_MIN (60 min) => active lease,
never deleted.  Orphaned => reclaimable.  Only DIRECTORIES under /tmp are
considered (files are left to their owners — conservative).

Usage:
    python3 scripts/lease_janitor.py            # dry-run (default): list + sizes
    python3 scripts/lease_janitor.py --clean    # delete orphaned ONLY, print bytes
"""

import argparse
import fnmatch
import os
import shutil
import sys
import time

TMP = "/tmp"
ACTIVE_MIN = 60  # minutes; mtime younger than this = active lease

# Directory-name grammar. fnmatch patterns over the BASENAME, case-sensitive.
GRAMMAR = [
    "*-target",
    "*-target-*",
    "*-sandbox",
    "*-sandbox-*",
    "*-sandbox-target",
    "*-cov",
    "*-scratch",
    "*-scratch-*",
    "*-probe",
    "csct-l*-*",
    "lspa*",
    "dx*",
]


def matches_grammar(name: str) -> bool:
    return any(fnmatch.fnmatchcase(name, pat) for pat in GRAMMAR)


def dir_size(path: str) -> int:
    total = 0
    for root, _dirs, files in os.walk(path, onerror=lambda e: None):
        for f in files:
            try:
                total += os.lstat(os.path.join(root, f)).st_size
            except OSError:
                pass
    return total


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--clean", action="store_true",
                    help="delete orphaned lease dirs (default: dry-run)")
    ap.add_argument("--tmp", default=TMP, help="scan root (default /tmp)")
    ap.add_argument("--active-min", type=int, default=ACTIVE_MIN)
    args = ap.parse_args()

    now = time.time()
    cutoff = args.active_min * 60
    matched, refused = [], 0
    for name in sorted(os.listdir(args.tmp)):
        path = os.path.join(args.tmp, name)
        if not os.path.isdir(path):
            continue
        if not matches_grammar(name):
            refused += 1
            continue
        age_min = (now - os.lstat(path).st_mtime) / 60.0
        state = "ACTIVE" if age_min < args.active_min else "ORPHANED"
        matched.append((name, path, state, age_min, dir_size(path)))

    total_reclaim = 0
    reclaimed = []
    for name, path, state, age_min, size in matched:
        print(f"{state:9s} {age_min:8.1f}m {size:>12,d}B  {path}")
        if state == "ORPHANED":
            total_reclaim += size
            reclaimed.append((name, path, size))

    print(f"\nmatched-grammar dirs: {len(matched)}  "
          f"(unclassified dirs skipped: {refused})")
    print(f"orphaned: {len(reclaimed)}  reclaimable: {total_reclaim:,} bytes")

    if not args.clean:
        print("DRY-RUN (default). Pass --clean to delete orphaned only.")
        return 0

    print("\n--clean: deleting orphaned only; unclassified paths are refused "
          "by construction (grammar filter above).")
    errors = 0
    for name, path, size in reclaimed:
        try:
            shutil.rmtree(path)
            print(f"removed {size:>12,d}B  {path}")
        except OSError as e:
            errors += 1
            print(f"FAILED {path}: {e}", file=sys.stderr)
    print(f"\nreclaimed total: {total_reclaim:,} bytes "
          f"({len(reclaimed)} dirs, {errors} failures)")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
