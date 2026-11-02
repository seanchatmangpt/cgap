# dod-five-gates

Morning DoD five-gate aggregate — R17 (court-auditor, read-only except this
marker). Repo `/Users/sac/cgap`, HEAD `cfd085a9` at session start (no git
writes). Poll window: opened 2026-11-02T01:21:55Z, 25-30s cadence, bounded
45 min; gate-1 found 01:43:47Z (poll-complete), gate-3 completion seen
01:47:52Z, gate-5 production section seen 01:59:30Z (mtime 17:43 local —
it landed in the gap between my two pollers' baselines and is read below).
All five gate markers landed inside the window; every marker was read
first-hand and byte-swept (0 NUL/control bytes, newline-terminated).

## Aggregate verdict

G1 GREEN, G2 GREEN, G3 ALIVE (4-of-4), G4 ALIVE, G5 ALIVE-with-recorded-
divergence. No gate claims ALIVE whose underlying marker says otherwise:
each ALIVE row below carries its marker's own counts plus at least one
first-hand command+exit from this lane.

| gate | standing | marker (mtime local) | this-lane first-hand witness |
|---|---|---|---|
| G1 workspace purity | GREEN | gate-1-workspace.md 17:42 | census delta (OPEN-3) |
| G2 binary/symbol purity | GREEN | f3-04 + gate-5 nm | 5 legs 0, inert 1, nm 4x T |
| G3 1M-row ledger | ALIVE | gate-3-ledger-proof 17:47 | 14/0 exit 0 incl. eval court |
| G4 640-bit spoliation | ALIVE | gate-4-spoliation 16:47 | p28 42/0 exit 0 |
| G5 bare-metal silicon | ALIVE* | gate-5-silicon 17:43 | ELF 150/65536, 0/32768 |

G5 *: ALIVE on the size law only; the production .oabi/.bn3 sections in the
linked ELF are court DEMO bytes, the raw production-object link still
refuses on the xip.ld ASSERTs, and XIP rule execution stays
BLOCKED-on-owner (A12 unifier alloc API, no allocator in the XIP budget) —
all recorded in the marker, not laundered.

## G1 — workspace purity: GREEN

- Marker: gate-1-workspace.md (R1, verdict GREEN). Census: 121 suites,
  1025 passed / 0 failed / 3 ignored, 0 panics;
  `cargo build --workspace` exit 0, `cargo test --workspace --no-fail-fast`
  exit 0 (isolated /tmp/r1-target, created 17:30:41, 197M).
- Sole prior RED resolved by replacement, not skip:
  `xip_t1_riscv_staticlib_size_court` replaced by
  `xip_t1_riscv_elf_size_court` (subject re-anchored to the xip.ld-linked
  §4 ELF; budgets unchanged 64 KiB / 32 KiB).
- This lane did not re-run the full census (heavy; bounded window). Named
  below as OPEN-3 with the delta-coverage argument; not rounded to ALIVE
  first-hand.

## G2 — binary/symbol purity: GREEN

- Marker legs (f3-04:15-19): court_verdict 13/0, kill_rate 1.0000 (19/19
  killed, 0 survivors, 5 equivalent), shim kill exit 0, inert control
  exit 1, self-check PASS.
- This lane first-hand (all 2026-11-02T01:2x-01:5xZ):
  - `python3 scripts/zero_string_gate.py --self-check` -> 0 (poison hits=2
    fired=True; clean hits=0; RESULT PASS)
  - `python3 scripts/zero_string_gate.py` -> 0 (scanned=96 crates=21
    hits=0 emitter_hits=2 adv_hits=0 gate=1.4.1)
  - `python3 scripts/boundary_gate.py --strings` -> 0 (0 hits x15;
    CROSS-GATE canonical A07 scan GREEN exit=0 scanned=96 hits=0)
  - `python3 scripts/boundary_gate.py --shim-court-only` -> 0 (killed by
    a04_identity_court — non-vacuous)
  - `python3 scripts/boundary_gate.py --shim-court-only --shim-inert` -> 1
    (VACUOUS GATE fired — the required negative control)
  - `nm -gU /tmp/r17-target/debug/libcgap_ffi.dylib` (own build, exit 0):
    all four sovereigns T @0x1e1c/0x21b0/0x2630/0x32e8 — byte-matches
    gate-3's marker addresses; GAP-1's nm row and R11's host-cdylib row
    agree (marker-cited).
- nm 0/0 alloc/panic purity on the riscv cgap-arrow rlib: marker-cited
  (gate-5, two passes, toolchain-internal llvm-nm); not re-run here
  (no cross artifact in this lane's target dir) — named OPEN-4.

## G3 — 1M-row sovereign ledger proof: ALIVE (4-of-4)

- Marker: gate-3-ledger-proof.md — prior PARTIAL record preserved; R10
  completion section (17:47) witnesses the new `a19_seal_1m_eval_court`:
  sovereign admit over 1M lane-triple frames -> sovereign eval closure
  over ADMITTED views (1M lanes, per-row differential pin) -> commit ->
  twin determinism -> disk read-back -> flips -> receipt tamper -> verify.
  Single-court run exit 0 (2.47s, witness line "SEAL+EVAL 1M LEDGER
  (4-of-4)"); full suite exit 0, 14 passed / 0 failed.
- This lane first-hand: quiescence-gated rerun at 01:47-01:49Z —
  `CARGO_TARGET_DIR=/tmp/r17-target cargo test -p cgap-court --test
  court_verdict` -> exit 0, 14 passed / 0 failed (26.05s), including
  `a19_seal::a19_seal_1m_eval_court ... ok`. The 1M stress prints
  (powl verify 0.624s; ocel2 rows=1_000_000; etc.) and the
  cgap-audit/gate/bn3 stage counts are marker-cited (37/0, 11/0, 111/0).
- Anti-vacuity: R10's /tmp sandbox flip (eval hop neutered -> exit 101 at
  the closure-count leg, shared-tree sha 8184d1e9...c9cb055c invariant) is
  marker-cited; this lane did not reconstruct the sandbox (bounded window)
  — loud skip, named OPEN-2. In-leg mutations M1-M7 + disk flips + receipt
  tamper executed inside this lane's own 14/0 run (in-vivo).
- Race disclosure: this lane's FIRST a19_seal run (launched ~01:25:57Z
  during R10's harness edits) exited 101 on a §5.1 corpus-receipt mismatch
  (generated e4a14987... != pinned 03ed29d2..., 0 passed/1 failed, 12
  filtered). court_verdict.rs mtime 01:27:53Z/01:35:50Z/01:43:48Z proves
  mid-edit state; the quiescence-gated rerun on the settled tree is
  exit 0 14/0 with the pin unchanged at 03ed29d2. Classified mid-edit
  race (f3-03 leg-3 precedent), superseded; recorded, not hidden.

## G4 — 640-bit spoliation + liability: ALIVE

- Marker: gate-4-spoliation.md (ALIVE): 640-bit sweep root+derivative —
  640 executed / 629 refused / 11 tolerated per record == the predicted
  D-A17-1 CLEAR-flip class (popcount(5_000_000_000)=11, re-derived); three
  distinct vacuity probes all distinguishable from the 11-pin; stripped/
  truncated carriers -> real child SIGABRT, stdout staged=0x503238ff;
  2560 mutations / 2516 real deaths; liability boundary at the real 5e9
  ceiling; `_cgap_verify_p28` in the fresh cdylib (nm -gU).
- This lane first-hand: `CARGO_TARGET_DIR=/tmp/r17-target cargo test -p
  cgap-p28 -- --test-threads=1` -> exit 0, 42 passed / 0 failed
  (36 lib + 6 integration), including `bit_flip_sweep_640_integration`,
  `sweep_vacuity_probes`, `strip_halt_child`, `halt_court_subprocess_
  sigabrt` — the sweep and halt courts ran inside my own 42/0.

## G5 section: silicon fit — ALIVE on the size law (divergence recorded)

- Marker: gate-5-silicon.md (17:43) — cgap-arrow ALIVE (30,157 B flash /
  0 SRAM rlib; lto variant 0 subject), cgap-ffi FLIPPED PARTIAL -> ALIVE on
  the ELF size law by R11's production re-measure; production link leg A
  (raw object) exit 1 REFUSED on both xip.ld ASSERT texts; leg B (§4 ELF
  subject, /tmp sandbox) exit 0 -> FLASH 150 / 65,536 B, SRAM 0 / 32,768 B,
  placement 0x42000020/0x42010000/0x42020000 == A19 receipt.
- This lane first-hand: `CARGO_TARGET_DIR=/tmp/r17-target cargo test -p
  cgap-ffi --lib xip_t1_riscv_elf_size_court -- --nocapture` -> exit 0,
  1 passed: "T1 ELF SIZE ... flash=150 B / 65536 B, sram=0 B / 32768 B —
  budgets met" (+ CONTEXT staticlib 444,213 B row, not asserted). Matches
  GAP-1's in-court receipt and R11's measurement byte-for-byte (150/0).

## Loud skips

- [SKIP] xtensa T2 — toolchain absent (rc=1 grep), witnessed in gate-5;
  T2 UNVERIFIED, loud, never silent. T1 (RISC-V) is the primary gate.
- [SKIP] R10 flip reconstruction — reproducing the neutered-eval /tmp flip
  needs a full sandbox-workspace build (~10+ min); exceeds this lane's
  bounded window. R10's flip row (exit 101 + shared-tree sha invariance)
  stands marker-cited. Reason stated; not a silent pass.
- [SKIP] G1 census re-run — same window reasoning; delta-coverage named in
  OPEN-3.

## OPEN set (named, never rounded)

1. G5 production .oabi/.bn3 byte sources (A06 rule tables / A11 networks
   into the none-target object): future dispatch. The raw production-object
   link still refuses; demo-byte divergence recorded in gate-5.
2. R10 eval-neuter flip: marker-cited only (see loud skips).
3. G1 census: R1's census predates the final court_verdict.rs edit
   (01:43:48Z); the only post-census edit is tests/court_verdict.rs, and
   this lane's quiescent 14/0 exit-0 run (01:49Z) covers exactly that
   crate — the green census holds for the 01:49Z tree by delta, not by
   full re-run.
4. riscv rlib nm 0/0: marker-cited (gate-1: none; gate-5 marker-cited),
   not re-run by this lane.
5. §4 SRAM worst-case eval-stack term: UNVERIFIED (no hardware/qemu),
   labeled in-court per GAP-1 item 8.
6. Publish chain: f3-09..13 BLOCKED at ledger draft (26.10.29 x5 on
   registry through 00:52:53Z); re-dispatch past the now-green gate is
   post-window and unwitnessed here.

## Divergences (recorded)

1. This lane's own mid-flight a19_seal exit 101 (corpus receipt mismatch,
   generated e4a14987... != pinned 03ed29d2...) during R10's harness edits
   at 01:27:53Z — race class (f3-03 leg-3 precedent), superseded by the
   quiescence-gated exit-0 14/0 rerun on the settled tree, pin unchanged.
2. Campaign ledger section 4 GATE-5 row says "gate-5-silicon.md ABSENT at
   draft" while the file landed 16:54:58, before the ledger mtime 16:58 —
   as-of-date conflict; this aggregate supersedes: the marker exists and
   reads ALIVE-with-divergence.
3. f3-13 out-of-chain-order landing (named by gap-8; publish NOT attempted;
   carried, not re-adjudicated).
4. gate-3's status line honestly read PARTIAL between GAP-1's body landing
   (17:20-17:28) and R10's completion (17:47) — an underclaim window, not
   an overclaim; no ALIVE-overclaim exists anywhere in the chain.

## files_written

- docs/jira/v26.11.1/waves/dod-five-gates.md (this marker; sole write of
  this lane; two-chunk append, byte-swept 0 NUL/control)
