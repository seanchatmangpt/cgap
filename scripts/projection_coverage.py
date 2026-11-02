#!/usr/bin/env python3
"""projection_coverage.py — AC/DC instrument (transition 2,
docs/jira/v26.11.1/phase-change-charter.md; mechanics:
docs/jira/v26.11.1/dflss/phase-4-design-phase-change.md D2-2).

Classifies tracked files into GENERATED (a declared regen path exists:
ggen manifests' template `to:` targets, the ggen.lock files ggen sync
rewrites, the repo CLAUDE.md generated-artifacts table, and the
manifest_patch.py consumer seam) vs HAND-AUTHORED; replays every
declared regen path in a /tmp sandbox copy (the shared tree is never
mutated); reports % of tracked bytes regenerable byte-identically
(the AC->DC baseline number).

Exit contract: 0 pass / 1 gate fail (replay failure, byte drift,
missing declared file, undeclared writer) / 2 subject missing (no
ggen manifest) / 3 loud skip (ggen tool absent).

Anti-vacuity court: --self-check builds a normalized sandbox, flips
one byte in a generated file, REQUIRES the gate to exit 1 (a gate
that does not flip exits nonzero as VACUOUS), restores byte-exact
(sha256-verified, mtime bumped), and requires the gate green again.

No Process.sleep: bounded subprocess waits on observables only.
"""
import argparse
import hashlib
import os
import re
import shutil
import subprocess
import sys
import tempfile

EXIT_PASS = 0
EXIT_FAIL = 1
EXIT_MISSING = 2
EXIT_SKIP = 3
GGEN_TIMEOUT = 600  # seconds per regen step (bounded, no sleeps)


def run(cmd, cwd, timeout=GGEN_TIMEOUT):
    """Bounded subprocess wait on a real observable (the exit), never sleep."""
    p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout)
    return p.returncode, p.stdout, p.stderr


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def git_ls_files(git_dir):
    rc, out, err = run(["git", "--git-dir", git_dir, "ls-files", "-z"], cwd="/tmp")
    if rc != 0:
        raise RuntimeError("git ls-files failed rc=%d: %s" % (rc, err.strip()))
    return sorted(x for x in out.split("\0") if x)


def read_bytes(root, rel):
    with open(os.path.join(root, rel), "rb") as f:
        return f.read()


def brace_expand(tok):
    m = re.fullmatch(r"(.*)\{([^{}]+)\}(.*)", tok)
    if not m:
        return [tok]
    head, body, tail = m.groups()
    return [head + part + tail for part in body.split(",")]


def parse_toml_ish(text):
    """Tolerant [section] parser for ggen.toml (declared surface only)."""
    secs = {}
    cur = None
    for line in text.splitlines():
        s = line.strip()
        m = re.match(r"^\[([^\]]+)\]$", s)
        if m:
            cur = m.group(1)
            secs.setdefault(cur, "")
            continue
        if cur and s:
            secs[cur] += line + "\n"
    return secs


def find_manifests(root, tracked):
    out = []
    for p in tracked:
        if p == "ggen.toml" or (p.endswith("/ggen.toml") and "/target" not in p):
            out.append(p)
    return sorted(out)


def manifest_info(root, rel):
    """Parse a ggen.toml: templates dir, ontology source, pack paths."""
    text = read_bytes(root, rel).decode()
    secs = parse_toml_ish(text)
    mdir = os.path.dirname(rel)
    tdir = "gen-templates"
    osrc = None
    packs = []
    for key, body in secs.items():
        if key == "templates":
            m = re.search(r'^\s*dir\s*=\s*"([^"]+)"', body, re.M)
            if m:
                tdir = m.group(1)
        elif key == "ontology":
            m = re.search(r'^\s*source\s*=\s*"([^"]+)"', body, re.M)
            if m:
                osrc = m.group(1)
        elif key == "packs":
            for pm in re.finditer(r'path\s*=\s*"([^"]+)"', body):
                packs.append(pm.group(1))
    return {
        "manifest": rel,
        "dir": mdir,
        "templates_dir": os.path.normpath(os.path.join(mdir, tdir)),
        "ontology_source": os.path.normpath(os.path.join(mdir, osrc)) if osrc else None,
        "pack_paths": packs,
    }


def template_outputs(root, info):
    """Declared `to:` targets of every *.tmpl under the templates dir."""
    outs = []
    troot = os.path.join(root, info["templates_dir"])
    if not os.path.isdir(troot):
        return outs
    for name in sorted(os.listdir(troot)):
        if not name.endswith(".tmpl"):
            continue
        text = open(os.path.join(troot, name)).read()
        m = re.match(r"^---\n(.*?)\n---", text, re.S)
        if not m:
            continue
        tm = re.search(r'^to:\s*"([^"]+)"', m.group(1), re.M)
        if tm:
            outs.append({
                "path": os.path.normpath(os.path.join(info["dir"], tm.group(1))),
                "via": "ggen.toml template %s" % name,
            })
    return outs


CMD_TOKEN = re.compile(r"`([^`]+)`")
PATHISH = re.compile(r"^[A-Za-z0-9_./{}-]+$")


def claude_artifact_rows(root):
    """Rows of the CLAUDE.md 'Generated artifacts' table: (outputs, cmd)."""
    text = read_bytes(root, "CLAUDE.md").decode()
    m = re.search(r"^## Generated artifacts.*?$(.*?)(?=^## |\Z)",
                  text, re.M | re.S)
    if not m:
        return []
    rows = []
    for line in m.group(1).splitlines():
        if not line.strip().startswith("|"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) != 2 or cells[0] in ("File", ":---", "---"):
            continue
        if set(cells[0]) <= set("-: "):
            continue
        rows.append((cells[0], cells[1]))
    return rows


def interpret_cmd(cmd):
    """Split a declared `cd X && a && b` chain into machine steps.

    Fail-closed: an unrecognized verb becomes ('unknown', tok) and the
    caller must loud-skip the outputs tied to that plan.
    """
    steps = []
    cwd = "."
    for part in cmd.split("&&"):
        part = part.strip().strip("`").strip()
        if not part:
            continue
        if part.startswith("cd "):
            cwd = part[3:].strip()
            steps.append(("cd", cwd))
        elif part == "rm Cargo.toml":
            steps.append(("rm", "Cargo.toml"))
        elif part == "ggen sync run":
            steps.append(("ggen", part))
        elif part.startswith("python3 ") or part.startswith("python "):
            steps.append(("py", part.split(None, 1)[1]))
        else:
            steps.append(("unknown", part))
    return steps


def parse_chain(cell):
    """Cell -> (base_dir, steps). Recognizes two shapes: an explicit
    `cd X && ...` chain, and prose naming a replayable script/manifest
    ("repo-root `ggen sync run`", "patched by `manifest_patch.py`").
    Returns steps==[] when the cell declares no verb (informational)."""
    if "&&" in cell:
        steps = interpret_cmd(cell)
        base = next((a for k, a in steps if k == "cd"), ".")
        return base, steps
    steps = []
    base = "." if "repo-root" in cell else None
    for t in CMD_TOKEN.findall(cell):
        if t.startswith("cd "):
            base = t[3:].strip()
            steps.append(("cd", base))
        elif t == "ggen sync run":
            steps.append(("ggen", t))
        elif t.startswith(("python3 ", "python ")):
            steps.append(("py", t.split(None, 1)[1]))
    return base, steps


def replay_chains(root, tracked):
    """Tracked *.py scripts carrying a `Replay: <chain>` docstring line
    (e.g. manifest_patch.py) -> {basename: (script_rel, base, steps)}."""
    chains = {}
    for p in tracked:
        if not p.endswith(".py"):
            continue
        try:
            text = read_bytes(root, p).decode()
        except UnicodeDecodeError:
            continue
        m = re.search(r'^Replay:\s*(.+)$', text, re.M)
        pm = re.search(r'^P\s*=\s*"([^"]+)"', text, re.M)
        if not (m and pm):
            continue
        base, steps = parse_chain(m.group(1))
        chains[os.path.basename(p)] = {
            "script": p, "dir": os.path.dirname(p) or ".",
            "target": os.path.normpath(os.path.join(os.path.dirname(p) or ".", pm.group(1))),
            "steps": steps,
        }
    return chains


def derive_declared(root, tracked):
    """Union of every declared-generated path with provenance + plan."""
    manifests = [manifest_info(root, m) for m in find_manifests(root, tracked)]
    chains = replay_chains(root, tracked)
    declared = {}
    plans = {}

    def add(path, source, plan=None):
        path = os.path.normpath(path)
        e = declared.setdefault(path, {"sources": [], "plans": set()})
        e["sources"].append(source)
        if plan:
            e["plans"].add(plan)

    def plan_for(pid, base, steps, frm):
        if steps and pid not in plans:
            plans[pid] = {"dir": base, "steps": steps, "from": frm}

    # 1) ggen manifests: template `to:` targets + sibling ggen.lock files.
    for i in manifests:
        for o in template_outputs(root, i):
            add(o["path"], o["via"])
        lock = os.path.normpath(os.path.join(i["dir"], "ggen.lock"))
        if lock in set(tracked):
            head = read_bytes(root, lock).decode(errors="replace")[:120]
            if "generated by `ggen sync`" in head:
                add(lock, "ggen.lock header (ggen sync rewrites it)")

    # 2) CLAUDE.md generated-artifacts table rows.
    for out_cell, cmd_cell in claude_artifact_rows(root):
        base, steps = parse_chain(cmd_cell)
        named = [t for t in CMD_TOKEN.findall(cmd_cell)
                 if t in chains or t.endswith("manifest_patch.py")]
        pid_dir = None
        if steps and base is not None:
            pid_dir = os.path.normpath(base)
        elif named:
            pid_dir = os.path.normpath(chains[named[0]]["dir"])
        pid = ("regen@%s" % pid_dir) if pid_dir is not None else None
        tracked_set = set(tracked)
        for tok in CMD_TOKEN.findall(out_cell):
            for t in brace_expand(tok):
                if not PATHISH.match(t):
                    continue
                bp = os.path.normpath(os.path.join(base or ".", t))
                rp = os.path.normpath(t)
                # Row paths mix two anchors (some carry crates/cli/ in the
                # token, siblings are cd-relative): prefer the reading that
                # lands on a tracked path; else the cd-relative one (a
                # declared-but-untracked path surfaces as drift later).
                p = bp if bp in tracked_set else (rp if rp in tracked_set else bp)
                if any(i["ontology_source"] == p for i in manifests):
                    continue  # graph source, not a projection
                s = "CLAUDE.md generated-artifacts table"
                if not steps and named:
                    s += " (replay via %s)" % chains[named[0]]["script"]
                add(p, s, plan=pid)
    return manifests, chains, declared, plans


def attach_plans(root, manifests, chains, declared):
    """Build regen plans (one per manifest dir) and attribute every
    declared path to the plan that owns its replay. Precedence for a
    dir's chain: CLAUDE.md row > script docstring > manifest default."""
    plans = {}
    rows = claude_artifact_rows(root)

    def plan(pid, base, steps, frm):
        if pid not in plans:
            plans[pid] = {"dir": base, "steps": steps, "from": frm,
                          "declares": set()}
        return plans[pid]

    row_plan = {}
    for out_cell, cmd_cell in rows:
        base, steps = parse_chain(cmd_cell)
        if not steps:
            named = [t for t in CMD_TOKEN.findall(cmd_cell) if t in chains]
            if named:
                base = chains[named[0]]["dir"]
                steps = chains[named[0]]["steps"]
        if steps and base is not None:
            row_plan.setdefault(os.path.normpath(base),
                                (steps, "CLAUDE.md: %s" % cmd_cell))

    for m in manifests:
        d = os.path.normpath(m["dir"] or ".")
        pid = "regen@%s" % d
        default = ([("cd", d), ("ggen", "ggen sync run")],
                   "ggen.toml at %s (default sync)" % m["manifest"])
        steps, frm = row_plan.get(d, default)
        e = plan(pid, d, steps, frm)
        for o in template_outputs(root, m):
            declared[o["path"]]["plans"].add(pid)
        lock = os.path.normpath(os.path.join(m["dir"] or ".", "ggen.lock"))
        if lock in declared:
            declared[lock]["plans"].add(pid)
        if "manifest_patch.py" in chains and \
                os.path.normpath(chains["manifest_patch.py"]["dir"]) == d:
            declared[chains["manifest_patch.py"]["target"]]["plans"].add(pid)

    for p, e in declared.items():
        if not e["plans"]:
            d = os.path.normpath(os.path.dirname(p) or ".")
            if "regen@%s" % d in plans:
                e["plans"].add("regen@%s" % d)
    return plans
def build_sandbox(root, git_dir, parent):
    """Copy TRACKED files into parent/tree (the shared tree is never
    touched). Returns (sandbox_root, copied_bytes_map)."""
    tracked = git_ls_files(git_dir)
    sbx = os.path.join(parent, "tree")
    os.makedirs(sbx)
    copied = {}
    for rel in tracked:
        dst = os.path.join(sbx, rel)
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        data = read_bytes(root, rel)
        with open(dst, "wb") as f:
            f.write(data)
        copied[rel] = data
    # Declared empty dirs (ggen requires the templates dir to exist;
    # git does not track empty dirs) and relative pack paths escaping
    # the tree get created/linked at the exact path ggen resolves.
    for mrel in find_manifests(sbx, tracked):
        info = manifest_info(sbx, mrel)
        os.makedirs(os.path.join(sbx, info["templates_dir"]), exist_ok=True)
        for pp in info["pack_paths"]:
            if os.path.isabs(pp):
                continue  # absolute pack paths resolve outside the box
            raw = pp
            real = os.path.normpath(os.path.join(root, info["dir"], raw))
            if not os.path.isdir(real):
                log("[SKIP] declared pack dir missing outside tree: %s"
                    % raw)
                continue
            link = os.path.normpath(os.path.join(sbx, info["dir"], raw))
            os.makedirs(os.path.dirname(link), exist_ok=True)
            if not os.path.lexists(link):
                os.symlink(real, link)
    return sbx, copied


def execute_plan(sbx, plan):
    """Run one regen plan inside the sandbox. Returns list of step
    receipts (kind, arg, rc, tail). Shared-tree state is never read or
    written here; the sandbox is the subject."""
    receipts = []
    cwd = os.path.join(sbx, plan["dir"] or ".")
    for kind, arg in plan["steps"]:
        if kind == "cd":
            receipts.append(("cd", arg, 0, ""))
            continue
        if kind == "rm":
            tgt = os.path.join(cwd, arg)
            if os.path.lexists(tgt):
                os.remove(tgt)
            receipts.append(("rm", arg, 0, ""))
        elif kind == "ggen":
            rc, out, err = run(["ggen", "sync", "run"], cwd)
            receipts.append(("ggen", "%s@%s" % (arg, plan["dir"]), rc,
                             (out + err).strip().splitlines()[-1:]
                             and (out + err).strip().splitlines()[-1] or ""))
        elif kind == "py":
            rc, out, err = run(["python3", arg], cwd)
            receipts.append(("py", arg, rc, (out + err).strip()[:200]))
        else:
            receipts.append(("unknown", arg, None, "uninterpretable declared step"))
    return receipts


def replay_all(sbx, plans):
    per_plan = {}
    for pid in sorted(plans):
        per_plan[pid] = execute_plan(sbx, plans[pid])
    return per_plan


def post_state(sbx, tracked):
    """Bytes of every tracked path after replay (None if absent)."""
    st = {}
    for rel in tracked:
        p = os.path.join(sbx, rel)
        st[rel] = open(p, "rb").read() if os.path.isfile(p) else None
    return st


def check(root, git_dir, verbose=True):
    """One full projection-coverage pass. Returns (exit_code, report)."""
    rep = {"root": root, "manifests": [], "plans": [], "files": [],
           "totals": {}, "skips": [], "fails": []}
    if git_dir is None and not os.path.exists(os.path.join(root, ".git")):
        log("[SKIP] no .git under %s" % root)
        return EXIT_MISSING, rep
    git_dir = git_dir or os.path.join(root, ".git")
    if shutil.which("ggen") is None:
        log("[SKIP] ggen not on PATH - regen paths unreplayable")
        return EXIT_SKIP, rep
    try:
        tracked = git_ls_files(git_dir)
    except RuntimeError as e:
        log("[SKIP] %s" % e)
        return EXIT_MISSING, rep
    manifests, chains, declared, _ = derive_declared(root, tracked)
    if not manifests:
        log("[SKIP] no ggen.toml manifest under %s" % root)
        return EXIT_MISSING, rep
    plans = attach_plans(root, manifests, chains, declared)
    rep["manifests"] = [m["manifest"] for m in manifests]
    rep["plans"] = [{"id": pid, "dir": p_["dir"], "from": p_["from"],
                     "steps": [[k, a] for k, a in p_["steps"]]}
                    for pid, p_ in sorted(plans.items())]
    # ---- replay in a sandbox; the shared tree is never mutated ----
    parent = tempfile.mkdtemp(prefix="pcov-")
    try:
        sbx, copied = build_sandbox(root, git_dir, parent)
        per_plan = replay_all(sbx, plans)
        post = post_state(sbx, tracked)
    finally:
        shutil.rmtree(parent, ignore_errors=True)
    rep["regen_steps"] = [
        {"plan": pid, "kind": k, "arg": a, "rc": rc, "tail": t}
        for pid, rs in per_plan.items() for (k, a, rc, t) in rs]
    for r in rep["regen_steps"]:
        if r["rc"] != 0:
            rep["fails"].append({"kind": "REPLAY_STEP_FAILED",
                                 "plan": r["plan"], "step": r["arg"],
                                 "rc": r["rc"], "tail": r["tail"][:160]})
    # ---- classify + compare (declared generated vs replayed bytes) ----
    bad_plans = set()
    for pid, p_ in plans.items():
        for k, a, rc, _t in per_plan.get(pid, []):
            if rc is None or rc != 0 or k in ("unknown",):
                bad_plans.add(pid)
    skipped_declared = []
    for rel in tracked:
        e = declared.get(rel)
        size = len(copied.get(rel) or b"")
        if e is None:
            rep["files"].append({"path": rel, "class": "hand", "bytes": size})
            continue
        owning = e["plans"] & set(plans.keys())
        if owning and owning <= bad_plans or not owning:
            rep["files"].append({"path": rel, "class": "generated",
                                 "bytes": size, "status": "UNREPLAYED"})
            skipped_declared.append(rel)
            continue
        sbx_bytes = post.get(rel)
        if sbx_bytes is None:
            rep["fails"].append({"path": rel, "kind": "MISSING_IN_REPLAY"})
            rep["files"].append({"path": rel, "class": "generated",
                                 "bytes": size, "status": "MISSING"})
            continue
        if sbx_bytes != copied[rel]:
            rep["fails"].append({"path": rel, "kind": "DRIFT",
                                 "on_disk": sha256(copied[rel]),
                                 "regen": sha256(sbx_bytes)})
            rep["files"].append({"path": rel, "class": "generated",
                                 "bytes": size, "status": "DRIFT"})
        else:
            rep["files"].append({"path": rel, "class": "generated",
                                 "bytes": size, "status": "IDENTICAL"})
    # ---- undeclared writers: replayed bytes changed a tracked path the
    # declared set does not own (single-writer law) ----
    for rel in tracked:
        if rel in declared:
            continue
        if post.get(rel) != copied.get(rel):
            rep["fails"].append({"path": rel, "kind": "UNDECLARED_WRITE"})
    gen = [f for f in rep["files"] if f["class"] == "generated"]
    hand = [f for f in rep["files"] if f["class"] == "hand"]
    rep["totals"] = {
        "tracked_files": len(tracked),
        "tracked_bytes": sum(len(copied.get(r) or b"") for r in tracked),
        "generated_files": len(gen),
        "generated_bytes": sum(f["bytes"] for f in gen),
        "matched_bytes": sum(f["bytes"] for f in gen if f["status"] == "IDENTICAL"),
        "hand_files": len(hand),
        "hand_bytes": sum(f["bytes"] for f in hand),
    }
    t = rep["totals"]
    t["pct_regenerable"] = round(100.0 * t["matched_bytes"] / t["tracked_bytes"], 4) \
        if t["tracked_bytes"] else 0.0
    if rep["fails"]:
        for f in rep["fails"]:
            log("FAIL: %s" % f)
        return EXIT_FAIL, rep
    if skipped_declared:
        for rel in skipped_declared:
            log("[SKIP] %s - no clean replayable declared regen path" % rel)
        return EXIT_SKIP, rep
    return EXIT_PASS, rep


def log(msg):
    print(msg, flush=True)


def self_check(root, git_dir):
    """Anti-vacuity court, /tmp sandbox only. Sequence: normalize a
    sandbox (replay in place), require green; flip ONE byte in a
    generated file, REQUIRE exit 1 (a gate that does not flip is
    VACUOUS and fails the court); restore byte-exact (sha256-verified,
    mtime bumped); require green again."""
    git_dir = git_dir or os.path.join(root, ".git")
    verdicts = []
    parent = tempfile.mkdtemp(prefix="pcov-court-")
    try:
        sbx, _ = build_sandbox(root, git_dir, parent)
        manifests, chains, declared, _ = derive_declared(sbx, git_ls_files(git_dir))
        plans = attach_plans(sbx, manifests, chains, declared)
        replay_all(sbx, plans)  # normalize to regenerated state
        rc, _ = check(sbx, git_dir)
        verdicts.append(("green_baseline", rc, rc == EXIT_PASS))
        targets = sorted(p for p in declared
                         if os.path.isfile(os.path.join(sbx, p)))
        if not targets:
            log("[SKIP] no generated file on disk to mutate")
            return EXIT_SKIP, verdicts
        tgt = targets[0]
        orig = read_bytes(sbx, tgt)
        digest = sha256(orig)
        mutated = bytearray(orig)
        mutated[0] ^= 0x20  # ASCII-preserving flip: exercises the byte
        # DRIFT comparison, not a generator decode refusal.
        with open(os.path.join(sbx, tgt), "wb") as f:
            f.write(bytes(mutated))
        rc, _ = check(sbx, git_dir)
        flipped = (rc == EXIT_FAIL)
        verdicts.append(("flip_detected", rc, flipped))
        with open(os.path.join(sbx, tgt), "wb") as f:
            f.write(orig)
        os.utime(os.path.join(sbx, tgt), None)  # mtime bump on restore
        restored = sha256(read_bytes(sbx, tgt)) == digest
        verdicts.append(("restore_sha256_exact", digest[:16], restored))
        rc, _ = check(sbx, git_dir)
        verdicts.append(("green_after_restore", rc, rc == EXIT_PASS))
        ok = all(v[2] for v in verdicts)
        return (EXIT_PASS if ok else EXIT_FAIL), verdicts
    finally:
        shutil.rmtree(parent, ignore_errors=True)


def print_report(rep):
    t = rep.get("totals") or {}
    if not t:
        for f in rep.get("fails", []):
            log("FAIL: %s" % f)
        return  # early-return path: the [SKIP] line already printed
    log("PROJECTION COVERAGE - AC/DC baseline "
        "(phase-change-charter.md transition 2)")
    log("root: %s" % rep["root"])
    log("manifests: %s" % ", ".join(rep["manifests"]))
    for p_ in rep.get("plans", []):
        log("plan %s: %s" % (p_["id"], p_["from"]))
    for r in rep.get("regen_steps", []):
        log("  step plan=%s kind=%s arg=%s rc=%s" % (r["plan"], r["kind"],
                                                     r["arg"], r["rc"]))
    for f in rep["files"]:
        if f["class"] == "generated":
            log("  %-9s %7d  %s" % (f.get("status", "?"), f["bytes"],
                                    f["path"]))
    log("generated (declared): %d files, %d bytes" %
        (t["generated_files"], t["generated_bytes"]))
    log("hand-authored:        %d files, %d bytes" %
        (t["hand_files"], t["hand_bytes"]))
    log("TOTAL tracked:        %d files, %d bytes" %
        (t["tracked_files"], t["tracked_bytes"]))
    log("regenerable byte-identical: %.2f%% of tracked bytes" %
        t["pct_regenerable"])
    for f in rep["fails"]:
        log("FAIL: %s" % f)


def main(argv=None):
    ap = argparse.ArgumentParser(
        description="AC/DC projection-coverage instrument "
                    "(exit 0 pass / 1 fail / 2 missing / 3 skip)")
    ap.add_argument("--root", default=os.getcwd())
    ap.add_argument("--git-dir", default=None)
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--self-check", action="store_true",
                    help="anti-vacuity court in /tmp sandbox copies")
    a = ap.parse_args(argv)
    root = os.path.abspath(a.root)
    if a.self_check:
        rc, verdicts = self_check(root, a.git_dir)
        for name, got, ok in verdicts:
            log("COURT %-22s got=%-8s %s" %
                (name, got, "OK" if ok else "REQUIRED-BUT-MISSED"))
        if rc != EXIT_PASS:
            log("VACUOUS GATE: self-check court did not flip - FAIL")
        else:
            log("SELF-CHECK COURT PASS")
        return rc
    rc, rep = check(root, a.git_dir)
    print_report(rep)
    if a.json:
        import json
        log(json.dumps(rep, indent=1))
    if rc == EXIT_PASS:
        log("exit 0 (pass)")
    return rc


if __name__ == "__main__":
    sys.exit(main())
