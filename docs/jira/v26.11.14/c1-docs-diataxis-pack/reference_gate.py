#!/usr/bin/env python3
"""Stdlib reference model of gates 010-040 + SUMMARY projection.
NOT the SPARQL (ggen absent); mirrors its semantics over the restricted
one-statement-per-line Turtle subset used by qualification/consumer.ttl.
Usage: reference_gate.py FILE.ttl -> exit 0 admit / 1 refused (rows on stderr)."""
import re, sys
from collections import defaultdict
ORDER = {"Tutorial": 1, "HowTo": 2, "Reference": 3, "Explanation": 4}
REQ = {"Tutorial": "goal", "HowTo": "task", "Reference": "subject", "Explanation": "topic"}

def parse(text):
    pages = {}
    for line in text.splitlines():
        m = re.match(r"^(ex:\w+) a ddx:Page ;(.*)\.\s*$", line)
        if not m:
            continue
        props = defaultdict(list)
        for part in re.split(r' ;(?=(?:[^"]*"[^"]*")*[^"]*$)', m.group(2).strip()):
            part = part.strip()
            if not part: continue
            k, rest = part.split(" ", 1)
            vals = [rest.strip('"')] if rest.startswith('"') else [v.strip() for v in rest.split(",")]
            props[k] += vals
        pages[m.group(1)] = props
    return pages

def parse_nav(text):
    navs = {}
    for line in text.splitlines():
        m = re.match(r"^(ex:\w+) a ddx:Nav ; ddx:navFor (ex:\w+) ; ddx:navOrder (\d+) \.\s*$", line)
        if m: navs[m.group(1)] = (m.group(2), int(m.group(3)))
    return navs

def gates(pages, navs=None):
    rows = []
    paths = defaultdict(list); pos = defaultdict(list)
    for s, p in pages.items():
        for k in ("dcterms:title", "ddx:position", "ddx:relPath", "ddx:body", "ddx:quadrant"):
            if not p.get(k): rows.append(("010", s, "missing " + k))
        qs = p.get("ddx:quadrant", [])
        if len(qs) > 1: rows.append(("020", s, "multi-quadrant"))
        for q in qs:
            name = q.replace("ddx:", "")
            if name not in ORDER: rows.append(("020", s, "quadrant-not-in-enum"))
            elif not p.get("ddx:" + REQ[name]): rows.append(("010", s, "missing ddx:" + REQ[name]))
        for r in p.get("ddx:relPath", []):
            paths[r].append(s)
            if not r.endswith(".md"): rows.append(("040", s, "not-markdown"))
            if r.startswith("/") or "../" in r or r == "SUMMARY.md": rows.append(("040", s, "escapes-root-or-reserved"))
        for q in qs:
            for v in p.get("ddx:position", []): pos[(q, v)].append(s)
    for r, ss in paths.items():
        if len(ss) > 1: rows += [("040", s, "duplicate-relPath") for s in ss]
    for k, ss in pos.items():
        if len(ss) > 1: rows += [("030", s, "duplicate-position-in-quadrant") for s in ss]
    for n, (pg, no) in (navs or {}).items():
        p = pages.get(pg)
        if p and p.get("ddx:quadrant") and p.get("ddx:position"):
            q = p["ddx:quadrant"][0].replace("ddx:", "")
            if q in ORDER and no != ORDER[q] * 1000 + int(p["ddx:position"][0]):
                rows.append(("030", n, "nav-order-disagrees-with-position"))
    return rows

def text_docs(t): return len(re.findall(r"^ex:\w+ a ddx:Doc\b", t, re.M))

def summary(pages):
    ent = []
    for s, p in pages.items():
        q = p["ddx:quadrant"][0].replace("ddx:", "")
        ent.append((ORDER[q], int(p["ddx:position"][0]), q, p["dcterms:title"][0], p["ddx:relPath"][0]))
    out, cur = ["# Summary"], None
    for qo, po, q, t, rp in sorted(ent):
        if q != cur: out += ["", "## " + q]; cur = q
        out.append("- [%s](%s)" % (t, rp))
    return "\n".join(out) + "\n"

if __name__ == "__main__":
    pages = parse(open(sys.argv[1]).read())
    if text_docs(open(sys.argv[1]).read()) > 1: print("FM-PACK-013 gate 050 doc doc-not-singleton", file=sys.stderr); sys.exit(1)
    if not pages: print("REFUSED: zero pages parsed (vacuous)", file=sys.stderr); sys.exit(2)
    text = open(sys.argv[1]).read()
    rows = gates(pages, parse_nav(text))
    for r in rows: print("FM-PACK-013 gate %s %s %s" % r, file=sys.stderr)
    if rows: sys.exit(1)
    sys.stdout.write(summary(pages))
