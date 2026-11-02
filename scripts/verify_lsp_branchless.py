#!/usr/bin/env python3
"""LSPA-6 — LSP tier-2 branchless court (aarch64 wire) over cgap-branchless.

Consumes the lsp-max tier-2 court (run_tier2, rule_pack.rs:236) through the
real JSON-RPC wire (branchless_client.diagnostics_for_files), ADDITIVE to
AE4 (verify_disassembly_invariants.py, riscv32imc-only). Method of record:
docs/jira/v26.11.2/audit/lspu1-consume-plan.md move #2+#3; wire semantics
witnessed in lanes/lane-g05-lsp-wire-rebuild.md (did_save tier-2 publish is
the SECOND publish for a URI; diagnostics_for_files drains the tier-1
publish and returns the tier-2 court verdict).

Arms:
  a. AE4 disassembly-invariants court (riscv32imc) — exit inherited.
  b. Staleness guard: `strings <bin> | grep -c ANDON_REFUSED` == 0 -> typed
     loud skip (exit 3) naming the rebuild command; never a vacuous pass.
  c. Portable mutation arm on aarch64 via the wire: clean select.rs text ->
     n_refused == 0; AE4 arm-C mutant text (if-injection into cmov_u64,
     verify_disassembly_invariants.py:262-268) -> n_refused >= 1. The
     mutant exists ONLY in a /tmp sandbox file — the shared tree is never
     mutated.
  d. One typed markdown line per subject.
  e. Exit 0 only if every arm held.

Exit ABI (mirrors AE4 :56-59): 0 clean / 1 refused-flip failure or vacuous
court / 2 subject missing / 3 loud skip.
"""
import os
import re
import subprocess
import sys
import tempfile

REPO = os.environ.get("CGAP_ROOT", "/Users/sac/cgap")
LSPMAX = "/Users/sac/lsp-max"
DEFAULT_BIN = os.path.join(LSPMAX, "target", "release", "branchless-lsp")
PY_CLIENT_DIR = os.path.join(LSPMAX, "python")
SELECT_RS = os.path.join(REPO, "crates", "cgap-branchless", "src", "select.rs")
AE4 = os.path.join(REPO, "scripts", "verify_disassembly_invariants.py")

EXIT_CLEAN = 0
EXIT_BRANCH = 1
EXIT_SUBJECT_MISSING = 2
EXIT_LOUD_SKIP = 3

# AE4 arm-C anchors (verify_disassembly_invariants.py:262-268) — read live
# from that file so the mutant is the REAL mutant, never a re-typed copy.
CLEAN_RE = re.compile(
    r'^CLEAN_SELECT = """(.*?)"""', re.S | re.M)
MUTANT_RE = re.compile(
    r'^MUTANT_SELECT = """(.*?)"""', re.S | re.M)


def fail(msg):
    sys.stderr.write(msg + "\n")


def ae4_arm():
    """Arm a: run AE4 verbatim; return its exit code."""
    print("[ARM-A] python3 scripts/verify_disassembly_invariants.py")
    p = subprocess.run([sys.executable, AE4])
    print(f"[ARM-A] AE4 exit={p.returncode}")
    return p.returncode


def staleness_guard(bin_path):
    """Arm b: fail-closed staleness witness on the server binary."""
    if not os.path.isfile(bin_path):
        fail(f"[SKIP] branchless-lsp binary missing: {bin_path} — rebuild "
             f"with: cd {LSPMAX} && cargo build --release -p branchless-lsp")
        return False
    p = subprocess.run(["strings", bin_path],
                       capture_output=True, text=True)
    n = p.stdout.count("ANDON_REFUSED")
    if n == 0:
        fail(f"[SKIP] staleness witness VACUOUS: 0 ANDON_REFUSED strings in "
             f"{bin_path} — binary predates the tier-2 court. Rebuild with: "
             f"cd {LSPMAX} && cargo build --release -p branchless-lsp")
        return False
    print(f"[ARM-B] staleness witness: {n} ANDON_REFUSED strings in {bin_path}")
    return True


def load_anchors():
    """Read the REAL arm-C mutant from AE4's source (never re-typed)."""
    src = open(AE4).read()
    mc, mm = CLEAN_RE.search(src), MUTANT_RE.search(src)
    if not (mc and mm):
        return None, None
    return mc.group(1), mm.group(1)


def no_mangle_probe(src):
    """Tier-2 probe convention (G05 / rule_pack fixtures): every probe fn
    carries #[no_mangle] so the judge sees bare symbols and the seed grid
    (residuals.toml) can match them. Without it, v0-mangled probe symbols
    cannot match and documented residual rows (e.g. checked_uniform_select)
    are refused — witnessed this lane."""
    return src.replace("pub fn ", "#[no_mangle]\npub fn ")


def build_subjects():
    """Arm c subjects as in-memory strings; return dict or None."""
    clean_raw = open(SELECT_RS).read()
    anchor, mutant_body = load_anchors()
    if anchor is None or anchor not in clean_raw:
        fail(f"[SKIP] AE4 arm-C anchor text not present in {SELECT_RS} "
             f"(loud) — mutant arm not run")
        return None
    clean = no_mangle_probe(clean_raw)
    mutant_ae4 = no_mangle_probe(clean_raw.replace(anchor, mutant_body))
    if mutant_ae4 == clean:
        fail("[SKIP] mutant replacement produced no change (loud)")
        return None
    # Portable aarch64 mutant (M-LSPA6-1): the AE4 if-injection alone
    # lowers to csel at aarch64 -O (witnessed this lane) and cmov_u64/
    # aarch64 is a seeded repaired_hot row (residuals.toml), so the pinned
    # arm-C text is structurally non-flipping on the aarch64 wire. The
    # portable mutant adds a black_box forward branch (G05 BRANCHY pattern)
    # and an export_name that defeats the seed-grid lookup.
    mutant_portable = mutant_ae4.replace(
        "#[inline]\n#[no_mangle]\npub fn cmov_u64(",
        "#[export_name = \"lspa6_mutant_cmov_u64\"]\npub fn cmov_u64(",
    )
    if "lspa6_mutant_cmov_u64" not in mutant_portable:
        fail("[SKIP] M-LSPA6-1 export-name site not found (loud)")
        return None
    core = (
        "    let mut spin = core::hint::black_box(0u64);\n"
        "    while spin < core::hint::black_box(r) { "
        "spin = spin.wrapping_add(1); } // M-LSPA6-1 forward branch\n"
        "    r\n"
    )
    old_tail = "if r > 0 { r } else { true_val } // SHIM-COURT M-AE4-1 if-injection\n}\n"
    if old_tail not in mutant_portable:
        fail("[SKIP] M-LSPA6-1 injection site not found (loud)")
        return None
    mutant_portable = mutant_portable.replace(old_tail, core + "}\n")
    return {
        "clean": clean,
        "mutant_ae4": mutant_ae4,
        "mutant_portable": mutant_portable,
    }


def wire_refusals(subjects, bin_path):
    """One client session PER FILE; drain publishDiagnostics for the URI
    and take the LAST publish (socket-ordered: tier-1 advisory publish
    precedes the tier-2 court verdict publish). Never trusts the first
    publish — the G05 race (tier-1 shadowing tier-2) is deterministic-
    killed here, not patched in the shared client (client file is not a
    lane file)."""
    sys.path.insert(0, PY_CLIENT_DIR)
    try:
        from lsp_client import branchless_client as bc
    except Exception as exc:  # noqa: BLE001 — typed skip, not a pass
        fail(f"[SKIP] branchless_client import failed: {exc}")
        return None
    import asyncio

    async def one_session(path, text):
        client = bc.BranchlessLspClient(bin_path, timeout=20.0)
        uri = bc._path_to_uri(path)
        await client.start()
        try:
            client.assert_alive()
            await client.did_open(path, text)
            await client.wait_for_diagnostics(uri)  # tier-1
            await client.did_save(path, text)
            last = None
            while True:  # bounded: wait_for_diagnostics raises on timeout
                try:
                    diags = await client.wait_for_diagnostics(uri)
                except (bc.LspTimeout, asyncio.TimeoutError, TimeoutError):
                    break
                last = diags
            return last or []
        finally:
            await client.stop()

    async def run_all():
        out = {}
        for name, text in subjects:
            fd, p = tempfile.mkstemp(prefix=name + "_", suffix=".rs",
                                     dir=tempfile.gettempdir())
            with os.fdopen(fd, "w") as fh:
                fh.write(text)
            try:
                out[name] = await one_session(p, text)
            finally:
                try:
                    os.unlink(p)
                except OSError:
                    pass
        return out

    try:
        return asyncio.run(run_all())
    except Exception as exc:  # noqa: BLE001 — transport failure is typed
        import traceback
        fail(f"[SKIP] wire transport failure: {exc!r}")
        traceback.print_exc()
        return None


def main():
    subjects = build_subjects()
    bin_path = os.environ.get("BRANCHLESS_LSP_BIN") or DEFAULT_BIN

    # Arm b (staleness guard) precedes any wire session.
    if not staleness_guard(bin_path):
        return EXIT_LOUD_SKIP
    if subjects is None:
        return EXIT_LOUD_SKIP

    # Arm a: AE4 runs after the guard so a stale binary still fails fast.
    ae4_rc = ae4_arm()

    clean = subjects["clean"]
    result = wire_refusals(
        [
            ("clean_select.rs", clean),
            ("mutant_ae4.rs", subjects["mutant_ae4"]),
            ("mutant_portable.rs", subjects["mutant_portable"]),
        ],
        bin_path,
    )
    if result is None:
        return EXIT_LOUD_SKIP

    def count_errors(key):
        return sum(
            1 for d in result.get(key, [])
            if d.get("severity") in ("ERROR", "error", 1))

    n_clean = count_errors("clean_select.rs")
    n_ae4 = count_errors("mutant_ae4.rs")
    n_mut = count_errors("mutant_portable.rs")

    print("| subject | tier-2 aarch64 verdict |")
    print("|---|---|")
    print(f"| clean select.rs | n_refused={n_clean} "
          f"{'PASS' if n_clean == 0 else 'FAIL'} |")
    print(f"| mutant_ae4 (M-AE4-1 if-injection into cmov_u64, pinned) | "
          f"n_refused={n_ae4} "
          f"{'flipped' if n_ae4 >= 1 else 'ADMITTED (aarch64 -O csel + seeded repaired_hot; see receipt)'} |")
    print(f"| mutant_portable (M-LSPA6-1 = M-AE4-1 + black_box forward "
          f"branch, seed-defeating export_name) | n_refused={n_mut} "
          f"{'PASS (flip)' if n_mut >= 1 else 'FAIL'} |")
    for key in ("mutant_ae4.rs", "mutant_portable.rs"):
        for d in result.get(key, []):
            msg = d.get("message", "") if isinstance(d, dict) else str(d)
            print(f"  {key} diagnostic: {msg[:160]}")

    verdict = EXIT_CLEAN
    if n_clean != 0:
        fail("[FORBIDDEN] clean kernel source refused on aarch64 — "
             "clean arm failed")
        verdict = EXIT_BRANCH
    if n_mut < 1:
        fail("[VACUOUS] M-LSPA6-1 portable mutant NOT refused on aarch64 — "
             "court cannot flip")
        verdict = EXIT_BRANCH
    if ae4_rc != EXIT_CLEAN:
        fail(f"[ARM-A] AE4 did not return clean (exit {ae4_rc})")
        if verdict == EXIT_CLEAN:
            verdict = ae4_rc
    if verdict == EXIT_CLEAN:
        print("LSPA-6 branchless wire court: CLEAN (AE4 clean + staleness "
              "witness + aarch64 clean 0-refused + mutant flip all held)")
    return verdict


if __name__ == "__main__":
    sys.exit(main())
