# A17 Finish Receipt

- **Lane**: A17 (P28 Derivative Attestation) finisher, v26.11.1, 2026-11-01
- **Subject**: `/Users/sac/cgap` @ `cfd085a9` + working tree (this lane's
  uncommitted tests/ addition), one canonical checkout, branch `main`
- **Lease**: `crates/cgap-p28/src/` + `crates/cgap-p28/tests/` +
  `docs/jira/v26.11.1/waves/a17-finish-receipt.md`
- **Spec**: `docs/jira/v26.11.1/lanes/A17-p28-derivative.md` (authoritative)

## Verdict

**ALIVE.** `cargo test -p cgap-p28` → **exit 0, 36 unit + 6 integration
passed / 0 failed** (final gate, this session, after all sandbox work). The
640-bit sweep is landed and witnessed in both forms: the 4x640
subprocess-per-mutation sweep (`trap::tests::spoliation_loader_court`, real
SIGABRT deaths carrying `0x503238ff`) and the 640x2 wire sweep at the public
integration seam with counts printed to stdout. The dispatch's premise
("tests/ absent ⇒ the 640-bit sweep is the highest-value missing piece") was
re-verified FALSE: tests/ was indeed absent, but the sweep had already landed
inline in `src/trap.rs` + `src/attestation.rs` and ran green before this
session wrote anything.

## Files written (this session)

- `crates/cgap-p28/tests/p28_invariant_courts.rs` (new, 397 lines,
  sha256 `ea865f631d82fe37d48745ae686117577cf1bf3814d58b8cb678f02ad865f8ce`)
  — the
  previously-absent integration target: public-API-only courts over the REAL
  pinned instrument (the unit wire sweep uses a synthetic instrument; this
  target does not), counts witnessed in stdout, real-process strip halt.
- `docs/jira/v26.11.1/waves/a17-finish-receipt.md` (this file).
- No file under `crates/cgap-p28/src/` was modified
  (`attestation.rs` `e750ed43…`, `trap.rs` `835d9d50…` sha256 pre == post).

## Commands and exits (this session)

- `cargo test -p cgap-p28` → exit 0 — baseline (pre-write): 36 unit passed,
  incl. `spoliation_loader_court` (2560 subprocess mutations, 2516 real
  SIGABRT deaths, 44 predicted survivors = 4 x 11 D-A17-1 class)
- `cargo test -p cgap-p28 --test p28_invariant_courts -- --nocapture` → exit 0,
  6 passed; witnessed stdout:
  `root: executed 640 single-bit flips, 629 refused, 11 tolerated (D-A17-1
  ceiling set-bit class)`; `derivative: executed 640 … 629 refused, 11
  tolerated`; `bit_flip_sweep_640_integration totals: 1280 mutations
  executed, 1258 refused`

- `cargo test -p cgap-p28` → exit 0 — post-write full gate: 36 + 6 = 42
  passed / 0 failed (re-run after sandbox work, same result)
- Sandbox anti-vacuity battery (git-archive sandbox, shared tree untouched):
  - `cd /tmp/a17-vac3 && CARGO_TARGET_DIR=/tmp/a17-vac3-target cargo test -p
    cgap-p28 --test p28_invariant_courts` → **exit 101** with
    `bit_flip_sweep_640_integration` FAILED ("survivors must be exactly the
    D-A17-1 ceiling set-bit class") and `ceiling_boundary_court` FAILED
    (MAX+1 admitted) — the neutered-trap mutation (15 halt sites →
    unconditional Ok in the SANDBOX copy of `attestation.rs`) cannot pass
    this court. Shared tree sha256 pre == post (fail-closed).
- `shasum -a 256 LICENSE-P28-SIL.md` →
  `79107c7ade61eb747098f93f1a5641ba665460b76daa1f235bd7894ea4e2484f`
  (instrument pin, §3/§8.4; digest pin
  `e144c48a49a2c4e40953a4b1eb4b4b938e280e0f92b8680fe3bb0aa22127b0b4`
  asserted green by `origin_digest_determinism_and_pin`)

## Courts run (name → evidence)

- `layout_frozen_integration` — PASS — any offset/size/flag/constant edit
  flips it + the compile-time pins.
- `ceiling_boundary_court` — PASS — MAX admits, MAX+1 refused; flips under
  the sandbox neuter (exit 101).
- `bit_flip_sweep_640_integration` — PASS — count law 640/record; survivors
  == the 11-bit D-A17-1 class; positive control first.
- `sweep_vacuity_probes` — PASS — accept-all (640 survivors), truncated
  (100), reject-all (0): all distinguishable.
- `strip_header_entirely_halts` — PASS — empty + 79-byte artifacts: real
  child SIGABRT deaths staging 0x503238ff.
- `spoliation_loader_court` (inline, re-witnessed) — PASS — 4x640 subprocess
  mutations, real cc-ELF + wat2wasm artifacts (mutations named in trap.rs).

## Mission items vs reality (spec A17 authoritative)

1. Attestation layout frozen 80B — landed prior wave (compile-time pins +
   `offset_table_matches_frozen_layout`); re-witnessed at the public seam by
   `layout_frozen_integration`. Green.
2. Exhaustive 640-bit sweep as a real test — landed prior wave INLINE
   (`attestation::bit_flip_sweep_court` 640x2 wire + `trap::spoliation_loader_court`
   4x640 subprocess). This session landed the integration-seam form
   (`bit_flip_sweep_640_integration`) with counts witnessed in stdout. Green.
3. Ceiling boundary court — landed prior wave (MAX admits via
   `validate_positive_controls` at-ceiling root; MAX+1 refuses via
   `validate_rejects_each_s5_step_violation`); this session consolidated
   the pair in one integration court (`ceiling_boundary_court`). The asc450
   meter boundary is A18's module and spec, left to A18.
4. Extra sandbox flip beyond the sweep — strip-the-header-entirely witnessed
   as real SIGABRT deaths (`strip_header_entirely_halts`), plus the
   sandbox-level neutered-trap mutation (exit 101) proving the vacuous case
   exits nonzero.
5. Receipt — this file.

## Divergences (recorded, not silent)

- **Dispatch premise corrected**: "tests/ absent ⇒ sweep missing" was FALSE.
  tests/ was absent; the 640-bit sweep had already landed inline in src/
  (prior wave). This session therefore did NOT duplicate the 4x640
  subprocess sweep in tests/ (O(N^2) drift law); it landed the missing
  integration-seam courts instead.
- **D-A17-1**: "all 640 bits halt" is arithmetically unachievable under §5.8
  `<=` semantics — CLEAR-flips of the 11 set bits of the ceiling at MAX stay
  <= MAX. The court pins survivors == exactly that class (11/record), kills
  629/640 per record. Dispatch phrasing "every flip must FAIL" holds only
  under this re-derivation, already recorded in the lane spec's own §7.1
  re-derivation clause.
- **D-A17-2**: fail-closed lineage readings (Root+parent / Derivative-without-
  parent = spoliation), landed prior wave.
- Sandbox lease debris: `/tmp/a17-vac{,2,3}` + `*-target` dirs remain (the
  permission system denied rm; /tmp is volatile; shared tree unaffected).
- No [SKIP] fired this session: `cc` (Apple clang 17) and `wat2wasm` were on
  PATH; all courts executed.

## Standing

**ALIVE** — every claim above carries a command+exit run this session; no
UNVERIFIED residue in this lane's scope.
