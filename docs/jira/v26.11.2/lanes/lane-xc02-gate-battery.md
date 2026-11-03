# Lane XC-02 — gate battery rollup

Date: 2026-11-02 · Branch: main · Owner file: `scripts/gate_battery.py` (NEW, only file written)

## W1 Subject

`scripts/gate_battery.py` — fail-closed rollup battery over the four house
gates (boundary_gate.py, zero_string_gate.py, verify_cscb_sram.py,
ffi_surface_gate.py). Exit ABI: 0 all-pass / 1 any gate fail / 2 missing
gate script (named row) / 3 loud skip rows only inside --with-self-check.

## W2 Commands + exits (this session, run_checked tee)

- `python3 scripts/gate_battery.py` -> **exit 1** (3/4 PASS; ffi-surface
  FAIL: `cgap_stats_snapshot` prototyped in include/cgap.h but not
  exported — an in-flight cgap-ffi/stats.rs from another lane)
- `python3 scripts/gate_battery.py --with-self-check` -> **exit 1**
  (6/8 PASS; ffi-surface + ffi-surface:self-check FAIL, this time exit 3:
  `could not compile cgap-ffi (lib) due to 14 previous errors` — the
  same in-flight lane moved between runs; recorded honestly, not a
  battery defect)
- `python3 scripts/gate_battery.py --prove-refusal` -> **exit 0** (court
  HOLDS: NA_MAX_MATCHES 2048->1024 injected into a
  tempfile.mkdtemp sandbox copy, gate exit 1 observed via
  `--subject bn3 --bn3-source <sandbox>`, battery FAIL rollup witnessed;
  shared noalloc.rs sha256 `e039d3ab88b2d111...` identical before/after,
  mtime untouched — no write ever occurred on the shared tree)
- `python3 scripts/gate_battery.py --only nonexistent-gate` -> **exit 2**
  (typed unknown/missing-gate row)
- Battery registry uses `boundary_gate.py --deps` as the canonical
  boundary gate pass (bare invocation is a typed EXIT_REFUSE; cite:
  `closure/lane-04-boundary-gate-witness.md:8`,
  `lanes/lane-g16-boundary-gate.md:9`).

## W3 Courts run

- prove-refusal (battery anti-vacuity) — FLIPPED, exit 0 (above).
- Each gate's own self-check also ran green: boundary DEPS SELF-CHECK
  ("both arms flipped"), zero-string --self-check, cscb-sram
  --mutate bn3-pin (mutant refused).

## W4 Fences (must-not-regress surface of this diff)

New file only; no gate script edited (RC-08/RC-10 files invoked
read-only). The four gates ARE the fences — all four ran this session
(exit rows above). No lib.rs/Cargo.toml touched; no git commands run.

## W5 Divergences / notes

- Dispatch suggested flipping the SRAM pin "on a /tmp copy"; the
  sram-bloat mutate mode never reads a copied tree (it builds a bloat
  caller in /tmp), so prove-refusal instead uses the bn3-source sandbox
  seam (also inside verify_cscb_sram.py) — same fail-closed law, real
  /tmp copy, witnessed restore-by-construction.
- First run exposed a polarity bug in my own prove-refusal (exited 1 on
  court success); fixed to house convention (court HOLDS = exit 0) and
  rerun — the exit-1 run is preserved in the command log above.
- RC-08's rumored undefined-name diagnostic in zero_string_gate.py did
  not manifest: `--self-check` and the default pass both exit 0.
- The ffi-surface FAIL is a real mid-wave cross-lane state
  (untracked crates/cgap-ffi/src/stats.rs, include/cgap.h prototype
  already landed); battery reporting it as exit 1 is the battery
  working as designed. Standing for the battery itself: ALIVE; the
  4/4 PASS rollup is UNVERIFIED until the cgap-ffi lane lands.

## W6 Standing

ALIVE (every claim above carries a command + exit run this session).
