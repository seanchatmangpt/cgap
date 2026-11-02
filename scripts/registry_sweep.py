#!/usr/bin/env python3
"""registry_sweep.py -- v26.11.2 registry coverage compiler (08-registry-sweep.md).

Enumerates subjects mechanically via `cargo metadata --no-deps`, appends the
standalone cgap-cli row, queries the crates.io API once per publishable crate
(UA header mandatory), classifies each cell PASS / REGISTERED-DIVERGENCE /
MISSING-<reason>, and fail-closes: exit 0 iff every cell is covered AND
REGISTERED-DIVERGENCE count == court-pointer count; exit 1 on any MISSING;
exit 2 subject missing; exit 3 network unavailable.

Read-only against the registry. Never publishes.
"""
import json
import os
import subprocess
import sys
import urllib.error
import urllib.request

UA = "cgap-registry-sweep/26.11.2"
API = "https://crates.io/api/v1/crates/{}"
REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

_C = "docs/jira/v26.11.2/waves/v2-07-census.md"
_P = "docs/jira/v26.11.1/waves/coord-publish-record.md"
# Expected ledger: subject -> expected registry state + court pointer
# (provenance: docs/jira/v26.11.2/08-registry-sweep.md section 1 table).
LEDGER = {
    "cgap": {"state": "PUBLISHED", "pointer": _C + ":99"},
    "cgap-arrow": {"state": "PUBLISHED", "pointer": _P + ":14"},
    "cgap-blake3": {"state": "PUBLISHED", "pointer": _P + ":16"},
    "cgap-parquet": {"state": "PUBLISHED", "pointer": _P + ":20"},
    "cgap-ffi": {"state": "PUBLISHED", "pointer": _C + ":88"},
    "cgap-beam": {"state": "DELISTED", "pointer": _C + ":91"},
    "cgap-gpu": {"state": "UNPUBLISHED", "pointer": "docs/jira/v26.11.2/waves/v2-03-levers.md:32"},
    "cgap-git": {"state": "PUBLISHED", "pointer": _P + ":18"},
    "bench": {"state": "DELISTED", "pointer": _C + ":90"},
    "cgap-audit": {"state": "PUBLISHED", "pointer": _C + ":90"},
    "cgap-integration": {"state": "UNPUBLISHED", "pointer": _C + ":91"},
    "cgap-core": {"state": "PUBLISHED", "pointer": _C + ":101"},
    "cgap-storage": {"state": "PUBLISHED", "pointer": _C + ":101"},
    "cgap-bont": {"state": "PUBLISHED", "pointer": _C + ":101"},
    "cgap-bosym": {"state": "PUBLISHED", "pointer": _C + ":101"},
    "cgap-bn3": {"state": "PUBLISHED", "pointer": _C + ":101"},
    "cgap-gate": {"state": "PUBLISHED", "pointer": _C + ":101"},
    "cgap-crypto": {"state": "PUBLISHED", "pointer": _C + ":101"},
    "cgap-p28": {"state": "PUBLISHED", "pointer": _C + ":101"},
    "cgap-court": {"state": "DELISTED", "pointer": _C + ":92"},
    "cgap-fibo": {"state": "PUBLISHED", "pointer": _C + ":101"},
    "cgap-cloud": {"state": "PUBLISHED", "pointer": _C + ":101"},
    "cgap-cli": {"state": "DELISTED", "pointer": "docs/jira/v26.11.2/waves/v2-10-cli-state.md:1"},
}


def enumerate_subjects(repo):
    """cargo metadata --no-deps + the standalone cli row. Never hand-listed."""
    out = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1",
         "--manifest-path", os.path.join(repo, "Cargo.toml")],
        capture_output=True, text=True)
    if out.returncode != 0:
        return None, "cargo metadata failed: " + out.stderr.strip()[:200]
    meta = json.loads(out.stdout)
    subs = []
    for pkg in meta["packages"]:
        subs.append({"name": pkg["name"], "version": pkg["version"],
                     "manifest": pkg["manifest_path"]})
    # Standalone cli row (outside the member set by design).
    cli_mf = os.path.join(repo, "crates", "cli", "Cargo.toml")
    if not os.path.isfile(cli_mf):
        return None, "cli manifest absent: " + cli_mf
    name = None
    for line in open(cli_mf):
        s = line.strip()
        if s.startswith("name") and "=" in s:
            name = s.split("=", 1)[1].strip().strip('"')
            break
    subs.append({"name": name, "version": None, "manifest": cli_mf,
                 "cli": True})
    return subs, None


def publish_flag(manifest):
    """True iff manifest has an uncommented `publish = true` line."""
    for line in open(manifest):
        s = line.strip()
        if s.startswith("publish") and "true" in s.split("#")[0]:
            return True
        if s.startswith("publish") and "false" in s.split("#")[0]:
            return False
    # Cargo semantics: an absent publish field defaults to publishable.
    return True


def query_registry(name):
    """One crates.io API request per crate. No retry loop. The 404 is data."""
    req = urllib.request.Request(API.format(name), headers={"User-Agent": UA})
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            body = json.loads(resp.read().decode("utf-8"))
            crate = body.get("crate", {})
            return {"http": 200, "state": "PUBLISHED",
                    "max_version": crate.get("max_version"),
                    "num_versions": crate.get("num_versions"),
                    "yanked": crate.get("yanked")}
    except urllib.error.HTTPError as e:
        if e.code == 404:
            return {"http": 404, "state": "UNPUBLISHED"}
        return {"http": e.code, "state": "MISSING",
                "reason": "http-%d" % e.code}
    except Exception as e:
        return {"http": None, "state": "MISSING",
                "reason": type(e).__name__}


def pointer_exists(repo, pointer):
    """Court pointer must resolve to a real file:line on disk."""
    if not pointer or ":" not in pointer:
        return False
    path, _, line = pointer.rpartition(":")
    full = os.path.join(repo, path)
    if not os.path.isfile(full):
        return False
    try:
        with open(full) as f:
            return len(f.readlines()) >= int(line) > 0
    except ValueError:
        return False


def main():
    repo = REPO
    args = sys.argv[1:]
    cli_manifest = None
    i = 0
    while i < len(args):
        if args[i] == "--repo":
            repo = args[i + 1]
            i += 2
        elif args[i] == "--cli-manifest":
            cli_manifest = args[i + 1]
            i += 2
        else:
            i += 1
    subs, err = enumerate_subjects(repo)
    if subs is None:
        print("SUBJECT-MISSING: " + (err or "unknown enumeration failure"))
        print("REGISTRY-SWEEP: FAIL 23 missing")
        return 2
    if cli_manifest:
        for s in subs:
            if s.get("cli"):
                s["manifest"] = cli_manifest
    missing = 0
    rd_rows = []
    rd_with_pointer = 0
    rows = []
    for s in sorted(subs, key=lambda s: s["name"]):
        name = s["name"]
        exp = LEDGER.get(name)
        if exp is None:
            missing += 1
            print("[MISSING] %s: no ledger row (subject unregistered)"
                  % name)
            rows.append({"crate": name, "cell": "MISSING",
                         "reason": "no-ledger-row"})
            continue
        delisted = (not s.get("cli")) and (not publish_flag(s["manifest"]))
        if s.get("cli"):
            delisted = not publish_flag(s["manifest"])
        row = {"crate": name, "local_version": s.get("version")}
        if delisted and exp["state"] != "DELISTED":
            missing += 1
            row["cell"] = "MISSING"
            row["reason"] = ("manifest carries publish=false but ledger "
                             "expects %s" % exp["state"])
            print("[MISSING] %s: %s" % (name, row["reason"]))
        elif not delisted and exp["state"] == "DELISTED":
            missing += 1
            row["cell"] = "MISSING"
            row["reason"] = ("manifest lost the publish guard; ledger "
                             "expects DELISTED; a crates.io row is now "
                             "demanded for %s" % name)
            print("[MISSING] %s: %s" % (name, row["reason"]))
            row["registry_state"] = "PUBLISHABLE-UNATTESTED"
        elif delisted:
            # Standing cell IS the court pointer; no API call.
            ok = pointer_exists(repo, exp["pointer"])
            row["registry_state"] = "DELISTED"
            row["cell"] = "PASS" if ok else "MISSING"
            row["court_pointer"] = exp["pointer"] if ok else ""
            if not ok:
                missing += 1
                print("[MISSING] %s: DELISTED with unresolvable court pointer %s"
                      % (name, exp["pointer"]))
        else:
            reg = query_registry(name)
            row["registry_state"] = reg["state"]
            row["registry_detail"] = {k: v for k, v in reg.items()
                                      if k != "state"}
            if reg["state"] == "MISSING":
                missing += 1
                row["cell"] = "MISSING"
                row["reason"] = reg.get("reason")
                print("[MISSING] %s: registry query failed (%s)"
                      % (name, reg.get("reason")))
            elif exp["state"] == reg["state"]:
                row["cell"] = "PASS"
                row["court_pointer"] = exp["pointer"]
                if reg["state"] == "PUBLISHED":
                    row["cell"] = ("PASS" if reg.get("max_version")
                                   == s.get("version")
                                   else "REGISTERED-DIVERGENCE")
                    if row["cell"] == "REGISTERED-DIVERGENCE":
                        rd_rows.append(name)
            else:
                row["cell"] = "REGISTERED-DIVERGENCE"
                rd_rows.append(name)
            if row["cell"] == "REGISTERED-DIVERGENCE":
                if pointer_exists(repo, exp["pointer"]):
                    rd_with_pointer += 1
                    row["court_pointer"] = exp["pointer"]
                else:
                    missing += 1
                    row["court_pointer"] = ""
                    print("[MISSING] %s: divergence lacks court pointer %s"
                          % (name, exp["pointer"]))
        rows.append(row)
    out_path = os.path.join(repo, "docs", "jira", "v26.11.2", "waves",
                            "registry-sweep-table.json")
    with open(out_path, "w") as f:
        json.dump({"ua": UA, "rows": sorted(rows, key=lambda r: r["crate"]),
                   "registered_divergences": sorted(rd_rows),
                   "missing": missing}, f, indent=1, sort_keys=True)
    for r in sorted(rows, key=lambda r: r["crate"]):
        print("%-18s %-12s %s" % (r["crate"], r.get("registry_state", "-"),
                                  r["cell"]))
    print("registered-divergence count: %d; with court pointer: %d"
          % (len(rd_rows), rd_with_pointer))
    if missing:
        print("REGISTRY-SWEEP: FAIL %d missing" % missing)
        return 1
    if len(rd_rows) != rd_with_pointer:
        print("REGISTRY-SWEEP: FAIL %d missing" % missing)
        return 1
    print("REGISTRY-SWEEP: PASS 0 missing")
    return 0


if __name__ == "__main__":
    sys.exit(main())
