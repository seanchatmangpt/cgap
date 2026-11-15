#!/usr/bin/env python3
"""affidavit_bundle.py — cgap gate runs -> omni-evidence/v1 bundle for affidavit.

cgap judges itself with fail-closed gates (exit 0 pass / 1 fail / 2 subject
missing / 3 loud skip). This script runs a gate set and records each run as
one affidavit operation-event, so `affi` can seal the set into a BLAKE3
receipt chain and certify it. cgap observes; affidavit certifies; neither
decides for the other. No hashing of payloads happens here: affidavit computes
`blake3(payload)` at emit time.

Bundle layout (omni-evidence/v1; schema in affidavit
`schemas/omni-evidence-batch.v1.schema.json`):

  <out>/batch.json          [{"event_type","objects","payload"}...]
  <out>/payloads/NNNN.json  canonical JSON: gate, command, exit, verdict, output

event_type is `cgap.gate.<verdict>` (pass|fail|missing|skip|unknown);
objects are `gate-<name>:cgap-gate:<command>` and
`<HEAD sha>:git-commit:seanchatmangpt/cgap`.

Usage:
  python3 scripts/affidavit_bundle.py --out DIR [--gate NAME=CMD ...]
  python3 scripts/affidavit_bundle.py --self-check

Without --gate the default set runs (zero-string gate + boundary --deps/--strings).
Certify:
  affi receipt emit-batch --batch-file DIR/batch.json
  affi receipt assemble --out receipt.json && affi verify receipt.json

Exit: 0 bundle written (whatever the gates said — the verdicts are the
evidence) / 1 self-check failed / 2 bad usage or no gates / 124 gate timeout
recorded as verdict `unknown` (never a fake pass).
"""
import json
import os
import shlex
import shutil
import subprocess
import sys
import tempfile

REPO = os.environ.get("CGAP_REPO") or os.path.dirname(
    os.path.dirname(os.path.abspath(__file__)))
FORMAT = "omni-evidence/v1"
VERDICTS = {0: "pass", 1: "fail", 2: "missing", 3: "skip"}
DEFAULT_GATES = [
    ("zero-string", "python3 scripts/zero_string_gate.py"),
    ("boundary-deps", "python3 scripts/boundary_gate.py --deps"),
    ("boundary-strings", "python3 scripts/boundary_gate.py --strings"),
]
TIMEOUT_S = int(os.environ.get("CGAP_BUNDLE_GATE_TIMEOUT", "1800"))


def head_sha(repo):
    out = subprocess.run(["git", "-C", repo, "rev-parse", "HEAD"],
                         capture_output=True, text=True, check=False)
    return out.stdout.strip() if out.returncode == 0 else "unknown"


def run_gate(name, cmd, repo):
    """Run one gate; return its record. A timeout is verdict `unknown`."""
    try:
        proc = subprocess.run(shlex.split(cmd), cwd=repo, capture_output=True,
                              text=True, check=False, timeout=TIMEOUT_S)
        code, output = proc.returncode, proc.stdout + proc.stderr
    except subprocess.TimeoutExpired as exc:
        code = 124
        raw = exc.stdout or b""
        output = raw.decode("utf-8", "replace") if isinstance(raw, bytes) else raw
    except OSError as exc:
        code, output = 127, "spawn failed: %s" % exc
    return {
        "gate": name,
        "command": cmd,
        "exit": code,
        "verdict": VERDICTS.get(code, "unknown"),
        "output": output,
    }


def canonical(obj):
    """Sorted-key compact JSON, the same canonical form affidavit uses."""
    return json.dumps(obj, sort_keys=True, separators=(",", ":"),
                      ensure_ascii=False).encode("utf-8")


def entries(records, sha):
    out = []
    for rec in records:
        out.append({
            "event_type": "cgap.gate." + rec["verdict"],
            "objects": [
                "gate-%s:cgap-gate:%s" % (rec["gate"], rec["command"]),
                "%s:git-commit:seanchatmangpt/cgap" % sha,
            ],
            "payload_bytes": canonical(dict(rec, format=FORMAT, subject=sha)),
        })
    return out


def write_bundle(ents, out_dir):
    os.makedirs(os.path.join(out_dir, "payloads"), exist_ok=True)
    batch = []
    for i, ent in enumerate(ents):
        rel = "payloads/%04d.json" % i
        with open(os.path.join(out_dir, rel), "wb") as fh:
            fh.write(ent["payload_bytes"])
        batch.append({"event_type": ent["event_type"],
                      "objects": ent["objects"], "payload": rel})
    with open(os.path.join(out_dir, "batch.json"), "w", encoding="utf-8") as fh:
        json.dump(batch, fh, indent=2, ensure_ascii=False)
        fh.write("\n")
    return out_dir


def self_check():
    """Non-vacuity: a pass gate and a fail gate must yield different event
    types and their exit codes must survive into the payloads; the bundle must
    be byte-deterministic. Mutation named: mapping every exit to `pass`
    (VERDICTS = {}-style collapse) makes check 1 fail."""
    sbx = tempfile.mkdtemp(prefix="cgap-bundle-selfcheck-")
    ok = True
    try:
        gates = [("ok", "%s -c 'raise SystemExit(0)'" % sys.executable),
                 ("bad", "%s -c 'raise SystemExit(1)'" % sys.executable),
                 ("gone", "%s -c 'raise SystemExit(2)'" % sys.executable)]
        recs = [run_gate(n, c, sbx) for n, c in gates]
        ents = entries(recs, "0" * 40)
        types = [e["event_type"] for e in ents]
        checks = [
            ("verdicts distinguish pass/fail/missing",
             types == ["cgap.gate.pass", "cgap.gate.fail", "cgap.gate.missing"]),
            ("exit codes recorded in payloads",
             [json.loads(e["payload_bytes"])["exit"] for e in ents] == [0, 1, 2]),
            ("object refs carry the gate command in the qualifier",
             all(e["objects"][0].split(":", 2)[2] == c
                 for e, (_, c) in zip(ents, gates))),
        ]
        a = write_bundle(ents, os.path.join(sbx, "a"))
        b = write_bundle(entries(recs, "0" * 40), os.path.join(sbx, "b"))
        same = all(open(os.path.join(a, f), "rb").read()
                   == open(os.path.join(b, f), "rb").read()
                   for f in ["batch.json", "payloads/0000.json",
                             "payloads/0001.json", "payloads/0002.json"])
        checks.append(("bundle is byte-deterministic", same))
        for label, passed in checks:
            print("[SELF-CHECK] %s: %s" % (label, "ok" if passed else "FAIL"))
            ok = ok and passed
    finally:
        shutil.rmtree(sbx, ignore_errors=True)
    return 0 if ok else 1


def main(argv):
    if "--self-check" in argv:
        return self_check()
    if "--out" not in argv:
        print(__doc__)
        return 2
    out_dir = argv[argv.index("--out") + 1]
    gates = []
    for i, arg in enumerate(argv):
        if arg == "--gate":
            spec = argv[i + 1]
            if "=" not in spec:
                print("bad --gate %r (want NAME=CMD)" % spec)
                return 2
            name, cmd = spec.split("=", 1)
            gates.append((name, cmd))
    gates = gates or DEFAULT_GATES
    sha = head_sha(REPO)
    recs = []
    for name, cmd in gates:
        rec = run_gate(name, cmd, REPO)
        print("[BUNDLE] %s exit=%d verdict=%s" % (name, rec["exit"], rec["verdict"]))
        recs.append(rec)
    write_bundle(entries(recs, sha), out_dir)
    print("[BUNDLE] %d events -> %s/batch.json (%s)" % (len(recs), out_dir, FORMAT))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
