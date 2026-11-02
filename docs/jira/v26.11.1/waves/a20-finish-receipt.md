# A20 Finish Receipt

Lane A20 (court verdict), wave 4, session 2026-11-01 ~15:00-16:01 PST. Repo
`/Users/sac/cgap`, canonical checkout, HEAD `cfd085a9` at session start.
File ownership honored: `crates/cgap-court/`, `tests/court_verdict.rs`,
`scripts/boundary_gate.py`, this receipt. No git writes.

## Status

PARTIAL-ALIVE leaning ALIVE: every §9 DoD command witnessed green this
session with true exit codes, on the exact subject sha below; the mutation
campaign crossed threshold (kill_rate 1.0 >= 0.98) with the non-vacuity
row first. Two caveats keep it from a clean ALIVE: (1) the /tmp sandbox
delete was refused by the permission system (unfinished cleanup, recorded
below); (2) the tree is shared with other lanes landing concurrent edits
(witnessed a foreign red-baseline window and a foreign compile break mid-
session; the DoD sweep was re-witnessed after the tree settled).

## Exact subject

- HEAD at session start: `cfd085a9fd230696b9dcdb437aa3e6b0aea2e60e`
- Court kernel (campaign subject) sha256:
  `7ec519fd8870e9e4eb20676701879adc6215f51d19313452113b4548134a5250`
  (matches the ledger's `baseline_sha256["tests/court_verdict.rs"]`,
  re-read at use time).
- Ledger: `crates/cgap-court/a20-kill-ledger.json` (abi_version 1).

## §9 DoD commands (all run this session, true exits, no pipes)

| DoD | Command | Exit | Witness |
|---|---|---|---|
| 1 | `cargo test -p cgap-court --test court_verdict` | 0 | 13 passed; 0 failed; 13.45s |
| 2 | `python3 scripts/boundary_gate.py --mutate --report crates/cgap-court/a20-kill-ledger.json` | 0 | 19/19 killed, kill_rate 1.0000 >= 0.98, 5 equivalent, 0 survivor defects |
| 2 | `python3 scripts/boundary_gate.py --shim-court-only` | 0 | killed by `a04_identity_court` |
| 2 | `python3 scripts/boundary_gate.py --shim-court-only --shim-inert` | 1 | `VACUOUS GATE` typed refusal (negative control behaved) |
| 3 | `python3 scripts/boundary_gate.py --deps` | 0 | E=0 across 15 covered crates |
| 4 | `python3 scripts/boundary_gate.py --strings` | 0 | 0 hits, A07 register v2 (sha 7778078dfe11), cross-gate scan GREEN |
| 5 | corpus receipt | in DoD-1 | `corpus_determinism_receipt` court pins the regenerated corpus sha256 in-test |

Gate self-regression courts (exercised because I edited the gate's
scanner): `--cross-gate-court` exit 0 (G green / R real-red fixture / F
seam-neuter flip witnessed, fixtures self-cleaned); `--interference-court`
exit 0 (G quiet / F foreign-write detection / V cmp-swap inversion).

## Mutation campaign ledger (run 6, canonical)

- Sampled 24 of 1185 generated mutants, stride 50 (recorded in
  `config.sampling`, never hidden). Baseline PASS row first, then the
  §6.3 `ShimCourt` row (killed by `a04_identity_court`), then campaign.
- Verdicts: 19 killed (14 `KilledByTest`, 5 `KilledByBuild`), 5
  `Equivalent` (registered triage below), 0 `SurvivorDefect`.
- kill_rate = 19 / (24 - 5) = 1.0 >= 0.98 -> gate PASSES.

## Session-introduced changes (disclosures)

1. `scripts/boundary_gate.py` — scanner repair (defect found by the
   campaign itself): `code_regions` never advanced `start` past consumed
   segments and appended `(0, len(line))` as the tail region, so digits
   INSIDE string literals counted as code; three run-2 "survivors" were
   mutants of diagnostic-string text (`"256 B slots"` -> `"255 B slots"`,
   `"2-tier mode"` -> `"3-tier mode"`, `"8 of 32 B"` -> `"8 of 33 B"`),
   not code. Fix: cross-line string state (`string_state_at_line_starts`)
   threaded through `code_regions` / `find_matches` / `mutate_line` /
   `enumerate_mutants` and the campaign apply site; string interiors and
   multi-line string continuations are now excluded. Regression-witnessed
   by the gate's own `--cross-gate-court` and `--interference-court`
   after the fix.
2. `scripts/boundary_gate.py` — EQUIVALENT_TRIAGE register refresh per
   the register's own fail-closed law ("refresh from that run's ledger,
   never widen"): the 9be9109d (+591-line) expansion re-strided the
   sample; run 3 refused 3 stale keys (removed, empty set). After the
   scanner fix + court repairs, run 5's ledger yielded 5 survivors, all
   triaged EQUIVALENT with property-based justifications (unsigned
   tautology with sibling pin; provably-redundant conjunct implied by
   `op - sp >= 2`; Sigil-ban fixture var identity; zero-alloc sweep
   template row identity; mutation-witness row identity).
3. `tests/court_verdict.rs` — two single-point-of-failure repairs found
   by the campaign (each adds one redundant sibling pin):
   (a) corpus receipt court now asserts all FOUR BN3 families present
   (the `zero > 0 && ...` conjuncts were the normal-family population's
   only pin); (b) the a05 engine pins the final adjacent truncation
   offset pair explicitly, outside the windows(2) sweep (a
   windows(2)->windows(3) mutation silently dropped exactly that pair;
   bounds + k=N coverage do not imply it).
4. `tests/court_verdict.rs` — a14 row-5 spoliation corpus: added
   `("att magic", b[48] = b'X')` (array 7 -> 8). The campaign's :1574
   survivor showed row 4's magic flip was the att-magic law's ONLY pin.
   Suite green after (13 passed) — the real ingress refuses the
   corpus-path magic flip.

## Meta-flip (suite detects its own corruption)

APFS-cloned the repo (minus `.git`/`target`) into `/tmp/a20_sandbox` with
a clonefile `target/`; pre-flip kernel sha256 matched the campaign subject
exactly (`7ec519fd...a5250`); flipped
`assert_eq!(KILL_RATE_MIN_PERMILLE, 980)` -> `assert_ne!` at
court_verdict.rs:719; `cargo test -p cgap-court --test court_verdict` in
the sandbox: exit 101, `wave1_abi_freeze_receipt` FAILED (12 passed /
1 failed). The suite is not decoration.

## Unfinished cleanup (loud)

`/tmp/a20_sandbox` (3.6 GB) — deletion DENIED by the permission system
(two `rm -rf` refusals, honored); scratch `/tmp/a20_*.out` files remain
for the same reason. Coordinator should delete, or re-dispatch with the
permission granted. Nothing inside the shared tree is affected.

## Divergences (spec/reality)

1. Spec §10 calls both artifacts PROPOSED; they exist, run, and are
   mature (waves 1-9 landed them). The lane doc predates the tree.
2. Dispatch calls 0xBF58_476D_1CE4_E5B9 the "canonical seed"; it is the
   SplitMix64 mix constant #2, not the seed — the seed is
   `CORPUS_SEED = 0xA20C_0FFE_E5A7_E000`. Both canonical on disk (exactly
   one occurrence each; no variants — the known corruption class is
   absent), compile-pinned by `wave1_abi_freeze_receipt`.
3. `cgap-court` is a `[[test]]`-only package by design: no `src/`, no
   lib.rs; its only target points at `../../tests/court_verdict.rs`.
   `cargo build -p cgap-court` exit 0. Nothing to fill; the harness home
   is where the spec puts it.
4. Run 1 of the campaign (all-24-killed, 0 survivors) was VOIDED this
   session: verdicts did not reproduce under the fixed scanner (it
   counted string-digit mutants and produced spurious build-kill rows).
   The canonical campaign is run 6; runs 2-5 are the observation trail.
5. Concurrent-lane turbulence: a foreign red-baseline window (bench
   manifest mid-write, 15:4x) refused two of my sweep attempts per the
   gate's own law; the tree settled (build OK after a 287s bounded poll)
   and the whole DoD sweep was re-witnessed green on the settled tree.

## Standing

PARTIAL-ALIVE: all §9 DoD commands green first-hand this session on the
exact subject sha; kill_rate 1.0 with non-vacuity witnessed both ways
(shim kill + inert refusal); meta-flip witnessed. Held short of full
ALIVE only by the permission-refused sandbox cleanup and the shared-tree
turbulence above. The five courts' claims: corpus receipt, BN3 closure,
alloc budget, power-cut interleave, 1M-row seal — all asserted against
real collaborators (cgap-core/blake3/bn3/gate/storage/p28/crypto/ffi/
parquet), no mocks, Chicago discipline throughout.



## F2-2 concurrent-close addendum (lane F2-2-A20-closer, 2026-11-01 ~15:38-16:04 PST)

Dispatch: the F1 finisher (receipt above) was in flight at F2-2 dispatch;
bounded poll 8 min + one 8-min extension found no receipt by 15:58, so the
close-out ran in this lane per dispatch 2b, disclosing the concurrency.
Both lanes' campaigns agree on every number (deterministic stride-50
sampling over the same subject bytes, same triage register).

First-hand witnesses this lane (all `CARGO_PROFILE_DEV_OPT_LEVEL=2`,
`CARGO_TARGET_DIR=/tmp/lane-f2-2-target` where cargo runs):

| Command | Exit | Witness |
|---|---|---|
| `cargo test -p cgap-court --test court_verdict` | 0 | 13 passed; 0 failed; 2.49s (DoD 1+5: `corpus_determinism_receipt` ok in run #1) |
| `python3 scripts/boundary_gate.py --deps` | 0 | E=0 x 15 covered crates |
| `python3 scripts/boundary_gate.py --strings` | 0 | 0 hits; A07 register v2 sha 7778078dfe11 |
| `python3 scripts/boundary_gate.py --shim-court-only --target-dir /tmp/lane-f2-2-target` | 0 | killed by `a04_identity_court`; kernel sha256-restored |
| `python3 scripts/boundary_gate.py --shim-court-only --shim-inert --target-dir /tmp/lane-f2-2-target` | 1 | `VACUOUS GATE: the INERT control was not killed` — the no-kill control run exits nonzero (vacuity property) |
| `python3 scripts/boundary_gate.py --shim-inert` (standalone) | 1 | `no mode` fail-closed CLI guard: `--shim-inert` is a modifier, not a mode |
| `python3 scripts/boundary_gate.py --mutate --report crates/cgap-court/a20-kill-ledger.json --target-dir /tmp/lane-f2-2-target` | 0 | 19/19 killed (1.0000 >= 0.98), 5 equivalent, 0 survivor defects; ledger row 0 Baseline, row 1 ShimCourt (non_vacuity row_index 1) |

Ledger on disk (`crates/cgap-court/a20-kill-ledger.json`, mtime 16:01:15)
is THIS lane's campaign; its verdict rows match the run-6 canonical ledger
described above row-for-row. Subject pins this lane: kernel
`7ec519fd8870e9e4eb20676701879adc6215f51d19313452113b4548134a5250`
(byte-exact before/after every cargo and gate run, corruption-marker
grep E5BA=0 at rest), gate `eafba9735e2779ef49c12350981aeb84e7463926ae2ca6794a3553649c4c5a5d`.

Concurrent-session incidents witnessed independently by this lane:

1. Compile-freeze 15:42:40-15:56:04: `crates/bench/Cargo.toml` (outside
   both lanes' leases) declared `[[bench]] bn3`/`ingress` before their
   files landed; every workspace cargo parse failed ("can't find `bn3`
   bench"). Cleared by the owning lane (ingress.rs 15:55:53, bn3.rs
   15:56:04); no out-of-lease edit applied by this lane. Same incident as
   the "foreign compile break" caveat above.
2. Live-mutant window 15:57:45-15:57:54: campaign lock HELD and the shim
   corruption marker (seed `...E5B9` -> `...E5BA`) present mid-shim-court;
   this lane held all cargo/gate runs until lock-free + marker-gone, then
   re-pinned the kernel sha before re-verifying. No court ever compiled a
   live mutant (before/after sha pins on every run).
3. Kill-rate semantics cited at use time: `kill_rate = killed /
   (sampled - equivalent)`, KILL_RATE_MIN 0.98 (boundary_gate.py:143),
   refusal for untriaged survivors and for rate < 0.98; non-vacuity is a
   mechanical pointer (`non_vacuity.row_index`), not row-order convention.

Standing: ALIVE for the full A20 §9 DoD surface (all five items with
first-hand command+exit this session). The F1 receipt's caveat 2 (shared
concurrency) is bounded by this addendum's post-settle re-witness; caveat
1 (F1's sandbox cleanup debt) remains the F1 lane's, untouched here.
