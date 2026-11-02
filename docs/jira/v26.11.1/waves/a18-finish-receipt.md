# A18 Finish Receipt

- **Lane**: A18 (`crates/cgap-p28/src/asc450.rs`), finish pass, v26.11.1
  campaign, 2026-11-01
- **Repo**: `/Users/sac/cgap`, one canonical checkout, branch `main`
- **Subject**: `crates/cgap-p28/src/asc450.rs` sha256
  `a08ccfec473e23e04a9d34f82077448193f0a524819997fa70b8d3d5be874ef6`
  (post-finish; pre-finish wave-9 file was `a31bdc5d…5858`; the delta is
  this lane's two added courts, +118 lines, no behavior change)
- **Verdict**: **ALIVE** — canonical gate 36/36 exit 0 this session;
  dispatched boundary court (a) and self-attestation court (b) green in
  canonical and sandbox; one adversarial flip witnessed in a /tmp sandbox
  with byte-exact restore; float sweep zero-tolerance clean.

## ASSESS finding (dispatch asked ASSESS first)

The module was already landed by waves 1-9 (see `a18-wave3-audit-receipt.md`,
`a18-wave9-audit-receipt.md`; wave-9 verdict ALIVE at 34/34). The finish
lane's work is therefore: (1) close the one court gap against the dispatch —
C1 proved the §4.2 boundary law only at a toy ceiling (100 cents); the
dispatched numbers (exactly 5_000_000_000, then 5_000_000_001) had constant
pins (`consts_freeze_court`) but no behavioral court; (2) add the dispatched
self-attestation court (recompute the proof from emitted state); (3) re-run
everything this session; (4) one sandbox flip.

## Files written (lease)

- `crates/cgap-p28/src/asc450.rs` — added
  `f1_default_ceiling_boundary_court` + `f2_proof_self_attestation_court`
  (test module only; no production-code change; `git diff` = +118 lines).
- `docs/jira/v26.11.1/waves/a18-finish-receipt.md` — this file.

## Courts added (dispatch courts (a) and (b))

### F1 — default-ceiling boundary court (dispatch court (a))

- Meter at the frozen `DEFAULT_SAFE_HARBOR_CEILING_CENTS` (5_000_000_000).
- 2.5B + 2.5B unlicensed cents → NO emit, unlatched: exactly at the ceiling
  is still inside the safe harbor (PASS).
- +1 cent → EMIT: `crossing_total_cents == 5_000_000_001`, seq/tick 3,
  latched. Post-crossing event accumulates (5_000_000_008) and bumps seq,
  never re-emits.
- Named anti-vacuity mutation: the `>=` ceil-vs-floor off-by-one emits at
  exactly 5_000_000_000; killed by the `is_none` assert. The strict-`>` half
  is killed by the emission assert at 5_000_000_001.

### F2 — proof self-attestation court (dispatch court (b))

- Emit one real proof at the frozen ceiling (single 5_000_000_001-cent
  event), then re-derive everything from the emitted 168 bytes alone:
- (1) `from_bytes(to_bytes)` round trip byte-identical; (2) a FRESH
  `LeaseEpochSealer` over the same epoch root — pure verify, consumes no
  lease authority — recomputes the seal over span 0..104 and it MATCHES
  (`.verify` → `Ok`); (3) `proof_digest` of the emitted bytes equals the
  meter's chain head (`prev_proof_digest`); the attestation anchor re-derives
  as BLAKE3 of the real A17 instrument bytes; ceiling and genesis-chain
  fields re-derive; (4) in-court negative control: one flipped byte in the
  signed span fails the same recompute (the match is not vacuous).

## Session commands + exits (this receipt's evidence)

```
CARGO_TARGET_DIR=/tmp/cgap-lane-A18-target cargo test --locked -p cgap-p28
    # exit 0 — 36 passed / 0 failed (34 prior + F1 + F2);
    # f1_default_ceiling_boundary_court ... ok,
    # f2_proof_self_attestation_court ... ok in the passing list
grep -c "f32\|f64\|f16\|f128" crates/cgap-p28/src/asc450.rs
    # 0 hits, grep exit 1 — zero-tolerance float sweep clean
SB=/tmp/a18-finish-sb  (cp -R cgap-p28 cgap-blake3 cgap-crypto + minimal
    root manifest + LICENSE.md + LICENSE-P28-SIL.md + rust-toolchain.toml;
    sandbox asc450.rs sha256 == canonical a08ccfec…74ef6 pre-mutation)
cd /tmp/a18-finish-sb && CARGO_TARGET_DIR=/tmp/a18-finish-sb-target \
    cargo test -p cgap-p28 --lib
    # BASELINE: exit 0 — 36/36 (sandbox run without --locked: standalone
    # workspace has no Cargo.lock to lock against; zero-dep)
python3 count==1 anchored patch: "next > self.ceiling" -> "next >= self.ceiling"
    # SHIM-COURT: exit 101 — 29 passed / 7 FAILED
    # killers: c1, c2b, c3, c4, f1_default_ceiling_boundary_court,
    #          w9_a05, w9_a20
restore from canonical + shasum -a 256 (pre == post, a08ccfec…74ef6)
    # RESTORE: exit 0 — 36/36 again
```

## Courts run (this session)

| Court | Mutation named in court | Result |
|---|---|---|
| F1 default-ceiling boundary | `>=` off-by-one | green; FLIPS (exit 101) |
| F2 proof self-attestation | no-recompute verify; span byte-flip | green incl. neg. control |
| C1-C7, C7b (prior) | per in-code witnesses | green in 36/36 baseline |
| W9-A04/A05/A20 | per in-code witnesses | green; w9_a05/w9_a20 re-flipped by this mutant |

Suite-level never-emit mutant (C3) is in the flip set of this session's
off-by-one mutant — the suite is not vacuous.

## Standing

- `crates/cgap-p28/src/asc450.rs` boundary + self-attestation mechanism,
  courts C1-C7/C7b, W9 slice, F1/F2: **ALIVE** — witnessed this session
  (commands + exits above).
- Float zero-tolerance: **ALIVE** — 0 hits (grep exit 1) on the final file.
- Perf numbers: none named by the module; none measured. The 1M-event court
  is a correctness court, not a benchmark.

## Divergences (recorded, doc yields)

1. **Dispatch paraphrase vs spec §4.2**: the dispatch reads "at
   5_000_000_001 → REFUSED/halt"; the authoritative lane spec's law is
   EMIT the signed proof when `next > ceiling` (the meter refuses only
   overflow/invalid-attestation events). Implemented and courted per spec;
   the dispatch wording yields.
2. **C1 scale gap closed here**: the spec's C1 design ran at a toy ceiling;
   the dispatched boundary numbers (5e9 / 5e9+1) had only constant pins
   before this lane added F1.
3. **Sandbox runs without `--locked`**: the standalone sandbox workspace has
   no Cargo.lock to lock against (zero-dep); canonical gate ran `--locked`.
4. **Pre-existing lane build roots** `target-a18/`, `target-a18w9/` at the
   repo root (prior waves; wave-9 audit divergence 5 recurrence) — not this
   session's debris, left for coordinator cleanup.
5. **Concurrent fan-out traffic**: the working tree carries other lanes'
   in-flight edits (bn3/bont/bosym/core/ffi/git, zero_string_gate.py, other
   finish receipts) — untouched by this lane; this lane's shared-tree writes
   are exactly `asc450.rs` + this receipt.
6. **Wave-9 audit divergence 1 closed here**: a standalone A18 wave-record
   file now exists (`a18-finish-receipt.md`); the wave-9 receipts remain
   in-code at the cited lines.

## Fence compliance

- No git write commands; only read-only `git status`/`diff`.
- No `Process.sleep`; no mocks (real meter, real A16 lease sealer, real
  pinned instrument bytes; grep for mock/sleep/SystemTime = 1 hit, the
  "no mocks" comment itself).
- Lease honored: shared-tree writes are `asc450.rs` (lane-owned module) +
  this receipt; `/tmp` debris (`/tmp/cgap-lane-A18-target`,
  `/tmp/a18-finish-sb`, `/tmp/a18-finish-sb-target`) deleted before
  returning.
- Mutations applied only in the sandbox copy; canonical `asc450.rs`
  byte-identical pre/post flip (sha256 `a08ccfec…74ef6`).
