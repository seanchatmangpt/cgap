#!/usr/bin/env python3
"""HOP-RATIO: machinery-share instrument (v26.11.1 phase-change charter, transition 3 "Steam").

Reads a workflow journal.jsonl, counts event types (launched/started/result/failed;
the schema has no labels - keys are v2:<sha> content hashes, unique keys are distinct
agent calls), and emits the run's machinery share:

    machinery share = 1 - (agent_hops / contract_hops)

agent_hops    unique `started` keys (retried/resumed calls share one dispatch hash).
contract_hops the phase shape's designed agent count: --contract-hops N, or --auto
              (looked up in a sidecar JSON by workflow id, journal basename, then the
              "contract_hops" default key).
loud_gaps     `failed` events not superseded by a later `result` for the same key
              (a result before the failure does not supersede; only a later one).

Emits exactly: {"journal", "agent_hops", "contract_hops", "share", "loud_gaps"}.

Exit contract (repo gate law): 0 pass / 1 gate fail / 2 subject missing / 3 loud skip.
  0  share computed; journal closed (no loud gaps); share >= 0
  1  measured-but-failing: loud_gaps nonempty, or share < 0 (run exceeded its shape)
  2  subject missing: journal absent, malformed line (named), zero `result` events,
     or contract_hops <= 0 (never a divide-by-zero)
  3  loud skip: zero `started` events (nothing measurable), or --auto found no
     contract_hops entry for this journal

Usage:
  hop_ratio.py <journal.jsonl> --contract-hops N
  hop_ratio.py <journal.jsonl> --auto --sidecar <contracts.json>
"""

import argparse
import json
import os
import sys
from collections import Counter

EXIT_OK = 0
EXIT_GATE_FAIL = 1
EXIT_SUBJECT_MISSING = 2
EXIT_LOUD_SKIP = 3

_TERMINAL = ("started", "result", "failed")


class JournalDefect(Exception):
    """A journal line that is not parseable JSON or carries no 'type' field."""

    def __init__(self, line_no, reason):
        super().__init__("malformed line %d: %s" % (line_no, reason))
        self.line_no = line_no
        self.reason = reason


def parse_args(argv):
    p = argparse.ArgumentParser(
        prog="hop_ratio.py",
        description="Machinery-share instrument over a workflow journal.jsonl.",
    )
    p.add_argument("journal", help="path to journal.jsonl")
    p.add_argument("--contract-hops", type=int, default=None,
                   help="designed agent count of the phase shape")
    p.add_argument("--auto", action="store_true",
                   help="agent-hops from unique started keys; contract from sidecar")
    p.add_argument("--sidecar", default=None,
                   help="contract sidecar JSON (used with --auto)")
    return p.parse_args(argv)


def scan_journal(path):
    """One pass over the journal.

    Returns (counts, events, started_keys) where events is the ordered
    (line_no, kind, key) triple list for started/result/failed lines.
    Raises JournalDefect on the first malformed line.
    """
    counts = Counter()
    events = []
    started_keys = set()
    with open(path, "r", encoding="utf-8") as fh:
        for line_no, raw in enumerate(fh, 1):
            line = raw.strip()
            if not line:
                continue
            try:
                obj = json.loads(line)
            except json.JSONDecodeError as exc:
                raise JournalDefect(line_no, str(exc))
            if not isinstance(obj, dict) or "type" not in obj:
                raise JournalDefect(line_no, "event object has no 'type' field")
            kind = obj["type"]
            counts[kind] += 1
            if kind in _TERMINAL:
                key = obj.get("key")
                events.append((line_no, kind, key))
                if kind == "started":
                    started_keys.add(key)
    return counts, events, started_keys


def loud_gaps_of(events):
    """`failed` events not superseded by a later `result` for the same key.

    A result BEFORE the failure does not supersede it - only a strictly later
    result (higher file position) closes the gap. Each unclosed failure is one
    loud gap.
    """
    last_result_pos = {}
    for pos, (_line_no, kind, key) in enumerate(events):
        if kind == "result":
            last_result_pos[key] = pos
    gaps = []
    for pos, (line_no, kind, key) in enumerate(events):
        if kind != "failed":
            continue
        rp = last_result_pos.get(key)
        if rp is None or rp < pos:
            gaps.append({"key": key, "failed_line": line_no})
    return gaps


def workflow_id(path):
    """Extract the wf_* id from a journal path; fall back to the basename."""
    base = os.path.basename(path)
    for part in path.replace("\\", "/").split("/"):
        if part.startswith("wf_"):
            return part
    return os.path.splitext(base)[0]


def resolve_contract(args, journal):
    """Returns (contract_hops or None, source-or-skip-reason string)."""
    if args.contract_hops is not None:
        return args.contract_hops, "flag:--contract-hops"
    if not args.auto:
        return None, ("no contract source: pass --contract-hops N or "
                      "--auto --sidecar <contracts.json>")
    if not args.sidecar:
        return None, "--auto given without --sidecar: no contract source named"
    try:
        with open(args.sidecar, "r", encoding="utf-8") as fh:
            table = json.load(fh)
    except (OSError, ValueError) as exc:
        return None, "sidecar unreadable: %s" % exc
    if not isinstance(table, dict):
        return None, "sidecar is not a JSON object"
    wid = workflow_id(journal)
    for k in (wid, os.path.basename(journal), "contract_hops"):
        if k in table:
            return int(table[k]), "sidecar:%s" % k
    return None, "sidecar has no entry for %s" % wid


def main(argv=None):
    args = parse_args(argv)
    journal = args.journal
    if not os.path.isfile(journal):
        sys.stderr.write("SUBJECT MISSING: no such journal: %s\n" % journal)
        return EXIT_SUBJECT_MISSING
    try:
        counts, events, started_keys = scan_journal(journal)
    except JournalDefect as defect:
        sys.stderr.write("SUBJECT MISSING: %s (%s)\n"
                         % (defect, journal))
        return EXIT_SUBJECT_MISSING
    if counts["result"] == 0:
        sys.stderr.write("SUBJECT MISSING: zero result events in %s "
                         "(share undefined; refusing divide-by-zero)\n" % journal)
        return EXIT_SUBJECT_MISSING
    if counts["started"] == 0:
        sys.stderr.write("LOUD SKIP: zero started events in %s - nothing "
                         "measurable\n" % journal)
        return EXIT_LOUD_SKIP
    agent_hops = len(started_keys)
    contract, source = resolve_contract(args, journal)
    if contract is None:
        sys.stderr.write("LOUD SKIP: %s\n" % source)
        return EXIT_LOUD_SKIP
    if contract <= 0:
        sys.stderr.write("SUBJECT MISSING: contract_hops must be > 0, got %d\n"
                         % contract)
        return EXIT_SUBJECT_MISSING
    share = 1.0 - (agent_hops / contract)
    gaps = loud_gaps_of(events)
    emit = {
        "journal": journal,
        "agent_hops": agent_hops,
        "contract_hops": contract,
        "share": round(share, 4),
        "loud_gaps": gaps,
    }
    sys.stdout.write(json.dumps(emit) + "\n")
    if gaps:
        return EXIT_GATE_FAIL
    if share < 0:
        return EXIT_GATE_FAIL
    return EXIT_OK


if __name__ == "__main__":
    sys.exit(main())
