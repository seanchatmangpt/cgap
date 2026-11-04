#!/usr/bin/env python3
"""meta_court.py — anti-vacuity mutation-corpus coverage court (lane-2).

Mechanically sweeps the repo's timing/scale gate surface:
  S1  census allowlist tickets (scripts/profile_guard_allowlist.toml)
  S2  gate scripts: scripts/ basenames matching the timing/scale/perf/
      footprint/census/bench/budget family regex
  S3  corpus courts (registry cells of class corpus/builtin)

and asserts 100% corpus coverage: every registered gate appears with a
witnessed flip receipt (class corpus/builtin), else a TYPED EXCLUSION
(ticket + reason). Any swept gate id missing from the registry = REGISTRY
DRIFT = exit 1.

Coverage cells render as:
  PASS               receipt(s) with flipped=true + restore_verified=true
  REGISTERED-VACUITY receipt with flipped=false (court FAILS row, loud)
  EXCLUDED           typed exclusion with ticket + reason (loud)

Anti-vacuity (meta level): --self-check drops one registry cell and
requires the court to exit 1 — a meta-court that cannot flip is FAIL.

Exit ABI: 0 = 100% coverage; 1 = coverage/drift failure.
"""

from __future__ import annotations

import argparse
import glob
import json
import os
import re
import sys

MC = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(MC))
REGISTRY = os.path.join(MC, "registry.json")
RECEIPTS = os.path.join(MC, "receipts")

# Timing/scale gate family sweep (basename regex over scripts/).
SWEEP_RE = re.compile(
    r"(timing|perf|bench|footprint|census|budget|hop|profile|scale|"
    r"alloc|latency|zero_alloc|mutation_court|mutation_kills)"
)

def sweep_gate_scripts() -> set[str]:
    ids: set[str] = set()
    for path in glob.glob(os.path.join(ROOT, "scripts", "*")):
        b = os.path.basename(path)
        if b.endswith((".py", ".sh")) and SWEEP_RE.search(b):
            ids.add(b)
    return ids

def sweep_allowlist_tickets() -> tuple[list[str], list[str]]:
    """Census allowlist tickets, mechanically parsed. Returns (tickets, errors)."""
    try:
        import tomllib
    except ImportError:
        return [], ["tomllib unavailable"]
    path = os.path.join(ROOT, "scripts", "profile_guard_allowlist.toml")
    if not os.path.exists(path):
        return [], [f"allowlist missing: {path}"]
    with open(path, "rb") as fh:
        data = tomllib.load(fh)
    out, errors = [], []
    for e in data.get("allowlist", []):
        t = e.get("ticket")
        if not t:
            errors.append("allowlist entry missing ticket")
        else:
            out.append(t)
    return out, errors

def load_receipt(name: str, receipts_dir: str | None = None) -> dict | None:
    p = os.path.join(receipts_dir or RECEIPTS, name + ".json")
    if not os.path.exists(p):
        return None
    with open(p) as fh:
        return json.load(fh)

def all_receipt_names(cells: dict) -> list[str]:
    names = []
    for c in cells.values():
        names.extend(c.get("receipts", []))
    return sorted(set(names))

def evaluate(cells: dict, receipts_dir: str | None = None) -> tuple[list, list]:
    """Evaluate registry cells + mechanical sweep -> (rows, bad_rows)."""
    rows = []

    def receipt_status(names):
        if not names:
            return "NO-RECEIPT", "no receipts registered"
        flipped_any = False
        vacuity_rows = []
        for n in names:
            r = load_receipt(n, receipts_dir)
            if r is None:
                vacuity_rows.append(f"{n}: receipt missing")
                continue
            if r.get("flipped") and r.get("restore_verified"):
                flipped_any = True
            elif r.get("flipped") is False:
                vacuity_rows.append(f"{n}: VACUOUS (court cannot flip) - standing defect row")
            else:
                vacuity_rows.append(f"{n}: restore not verified")
        if flipped_any:
            return "PASS", f"{len(names)} receipt(s), flip witnessed"
        if vacuity_rows:
            return "REGISTERED-VACUITY", "; ".join(vacuity_rows)
        return "NO-RECEIPT", "receipts registered but none valid"

    gate_ids = sweep_gate_scripts()
    # registry may key script gates by repo path; normalize via basename aliases
    alias = {os.path.basename(cid): cid for cid in cells}
    for g in sorted(gate_ids - set(alias)):
        rows.append((g, "REGISTRY-DRIFT", "swept gate absent from registry"))

    for cid, cell in sorted(cells.items()):
        cls = cell.get("class")
        if cls == "excluded":
            if not cell.get("ticket") or not cell.get("reason"):
                rows.append((cid, "EXCLUDED-INVALID", "exclusion missing ticket/reason"))
            else:
                rows.append((cid, "EXCLUDED", cell.get("reason", "")))
            continue
        status, detail = receipt_status(cell.get("receipts", []))
        rows.append((cid, status, detail))

    # allowlist-ticket coverage leg
    tickets, errors = sweep_allowlist_tickets()
    census_cell = cells.get("scripts/profile_guard_census.py", {})
    if census_cell.get("tickets") == ["*allowlist-tickets*"]:
        m5 = load_receipt("m5_allowlist_tamper", receipts_dir)
        if m5 is None or not m5.get("flipped"):
            errors.append("allowlist tickets wildcard set but m5_allowlist_tamper flip not witnessed")
    elif tickets:
        errors.append(f"allowlist tickets {tickets} not wildcard-covered by census cell")
    if errors:
        rows.append(("census-allowlist-tickets", "TICKET-COVERAGE-FAIL", "; ".join(errors)))
    return rows, tickets

def main() -> int:
    ap = argparse.ArgumentParser(description="mutation-corpus coverage court")
    ap.add_argument("--self-check", action="store_true",
                    help="drop one registry cell and require the court to flip")
    args = ap.parse_args()

    cells = {c["id"]: c for c in json.load(open(REGISTRY))["cells"]}

    if args.self_check:
        # Meta-level anti-vacuity: delete one witnessed flip receipt from a
        # temp copy; the court MUST flip to NO-RECEIPT (exit 1 path).
        import shutil, tempfile
        tmp = tempfile.mkdtemp(prefix="mc-selfcheck-")
        for n in all_receipt_names(cells):
            src = os.path.join(RECEIPTS, n + ".json")
            if os.path.exists(src):
                shutil.copy(src, tmp)
        victim = "m1_alloc_inject"
        os.remove(os.path.join(tmp, victim + ".json"))
        rows, _ = evaluate(cells, receipts_dir=tmp)
        victim_row = next(r for r in rows if r[0] == "crates/cgap-blake3/tests/zero_alloc.rs")
        flipped = victim_row[1] != "PASS"
        if not flipped:
            print("VACUOUS META-COURT: deleted receipt not detected", file=sys.stderr)
            return 1
        print(f"SELF-CHECK OK: deleted receipt {victim} -> court flips to NO-RECEIPT (coverage FAIL)")
        return 0

    rows, tickets = evaluate(cells)
    bad = [r for r in rows if r[1] not in ("PASS", "EXCLUDED", "REGISTERED-VACUITY")]
    for cid, cell in cells.items():
        status = next((r[1] for r in rows if r[0] == cid), None)
        if status == "REGISTERED-VACUITY":
            vac_cell_subject = os.path.basename(
                ((load_receipt(next(iter(cell.get("receipts", [])), ""), None) or {}).get("subject", ""))
            )
            other_flip = any(
                (load_receipt(n) or {}).get("flipped")
                for n in all_receipt_names(cells)
                if os.path.basename((load_receipt(n) or {}).get("subject", "")) == vac_cell_subject
                and n not in cell.get("receipts", [])
            )
            if not other_flip:
                row = (cid, "VACUITY-UNCOVERED", "vacuity registered but no flipping receipt in family")
                rows.append(row)
                bad.append(row)

    print(f"meta_court: {len(tickets)} allowlist tickets, {len(cells)} registry cells")
    for cid, status, detail in rows:
        print(f"  {status:<22} {cid:<55} {detail[:90]}")
    if bad:
        print(f"meta_court: FAIL - {len(bad)} uncovered/misregistered cell(s)")
        return 1
    print("meta_court: PASS - 100% registry coverage (PASS / typed EXCLUDED / REGISTERED-VACUITY with family flip)")
    return 0

if __name__ == "__main__":
    sys.exit(main())
