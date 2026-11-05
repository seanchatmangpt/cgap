# RX-3 — RFC-0089 §6 Scorecard (measured)

Standing: UNVERIFIED

- **Lane**: RX-3 IMPLEMENTATION, RFC-0089 §6 automated G1–G7 evaluation
  table generator. Repo: cgap. No git operations. Diff is one new test
  file + one cross-ref line (arcus standard) + this receipt.
- **Date**: 2026-11-03.

## Deliverable

`/Users/sac/cgap/crates/cgap-aoq/tests/rfc0089_scorecard.rs` (new,
1,148 lines, zero warnings, compiles debug + release). One module, three
tests:

- `rfc0089_scorecard_emits_sealed_table` — runs all seven gates, prints
  the §6 table (per-gate: dimension, sovereign target, legacy tolerance,
  measured value, verdict, verification cite), emits the sealed JSON
  (AF-14 seal pattern), asserts structure + seal self-verification +
  tamper detection.
- `rfc0089_disqualification_witnessed_by_injected_unmet` — sandbox-
  injected UNMET (test fixture, no tree mutation): all-MET fixture →
  QUALIFIED; inject ONE UNMET → `DISQUALIFIED { by: ["G3"] }`; a
  SKIP/UNMEASURED degrades to PROVISIONAL, never DISQUALIFIED.
- `rfc0089_profile_guard_loud_skip` — §5.2 profile guard: debug build
  refused by `cgap_aoq::profile_guard::assert_release()`, release
  admitted; G2 (and G7 leg A) are SKIP-typed §5.2 under debug.

## Verdict vocabulary + admission law (typed, never softened)

- `MEASURED-MET` / `MEASURED-UNMET` / `TARGET-UNMEASURED` /
  `SKIP-typed` — per row.
- Unweighted: **any single gate UNMET → overall DISQUALIFIED** (RFC-0089
  §6); SKIP/UNMEASURED degrade to PROVISIONAL, never disqualify and
  never soften.

## Gate map (composes landed surfaces; methods from AF-10's cbmqs_gates)

| Gate | Dimension | Composes | Release verdict (this session) |
|---|---|---|---|
| G1 | zero-alloc hot path | AB-9 counting-allocator court (positive control + 100k windows, 0 allocs / 0 bytes) + AB-19 slab floor leg (release-only, §5.2) | MEASURED-MET |
| G2 | wire framing ceiling | AB-6 amortized-min bench (AF-25 converged method), release-only §5.2 | MEASURED-MET (86.2 ns combined, min-of-7×100k, release) |
| G3 | spec-drift compile court | AB-6 TTL==generated + injected offset mutation flips | MEASURED-MET |
| G4 | KAT + flip rejection | AB-2 SHAKE128/256 KATs + AB-3 RFC 8439 §2.5.2 + flip witnesses | MEASURED-MET |
| G5 | storm soak 50k | AB-10 partial-accept storm + AB-11 FrameTooLarge typed refusal (real wire-shm segment) | MEASURED-MET (13,200,000 accepts; FrameTooLarge max=4080) |
| G6 | churn endurance | AB-17 cross-repo citation (disk-grounded) + in-process 200-cycle wire-shm ring churn w/ per-cycle byte identity + typed backpressure re-witness | MEASURED-MET |
| G7 | receipt chain | AF-21 AoqChain method (leg A, release) + AB-14 receipt re-read from disk (leg B) | **MEASURED-UNMET** |

## G7 — the honest borderline number, exactly as instructed

- **Leg B (receipted on disk, re-read this run)**: AB-14's landed receipt
  (`/Users/sac/ggen/docs/jira/v26.11.3/receipts/ab-14-chain-scale.md`) —
  50k span **1276.01 ms** (min-of-3, debug profile, cgap-ffi
  RingReceiptChain plane) → 1276.01 ms / 50,000 = **25.5202 µs/link** vs
  the RFC-0089 ≤25 µs/link target → **MEASURED-UNMET**, typed, never
  softened. Profile disclosure rides the claim (debug, min-of-3,
  different plane than leg A).
  > ERRATA 2026-11-04: G7 leg B standing superseded by ZP-17's branch-(a)
  > MEASURED-MET release re-measure (1.26394 µs/link, min-of-7, same plane) —
  > see `cgap/docs/jira/v26.11.4/receipts/zp-17-gate10-settling.md` and the
  > v4 corrected-claims ledger
  > (`ggen/docs/jira/v26.11.4/receipts/v4-corrected-claims-ledger.md` row 5).
- **Leg A (in-process, release)**: AF-21 method — AoqChain bound to
  sovereign `cgap_blake3::hash`, 50k links, min-of-3 append + full-chain
  verify green + tamper flip witnessed → **within target** (MET leg).
- Unweighted worst-of: **G7 MEASURED-UNMET → overall DISQUALIFIED**.
- Honest outcome statement: the 25.5202 number is a DEBUG-profile,
  min-of-3, cgap-ffi-plane measurement. It is what the landed receipt
  carries, so it is what the scorecard records. A release-profile,
  min-of-3 50k-span re-measurement of the cgap-ffi plane would settle
  whether the borderline unmet is profile- or plane-artifact; that
  re-measurement is NOT done here (no invented number).

## Gates (exit records)

- Release pass 1: `cargo test -p cgap-aoq --test rfc0089_scorecard
  --release -- --nocapture` → `test result: ok. 3 passed; 0 failed`,
  exit 0, 0.46s. Honest overall: **DISQUALIFIED (unmet: G7)**.
- Release pass 2: same command → `test result: ok. 3 passed; 0 failed`,
  exit 0, 0.47s. Same honest overall **DISQUALIFIED (G7)**.
- Flake-margin pass 3: same command → `3 passed; 0 failed`, exit 0
  (recorded as margin, gate is ×2).
- Debug pass: same command minus `--release` → `3 passed; 0 failed`,
  exit 0, 2.12s; G1 floor + G2 + G7 leg A loud-SKIP §5.2 as designed;
  overall DISQUALIFIED (G7 leg B is profile-independent).
- Seals (BLAKE3-32, per-run body): release p1
  `726943ea…494da11`, release p2 `15ed4edf…e697`, debug
  `f4adf510…23fca`. Per-run hashes differ because the body carries
  that run's measured content (G2 ns, G5 accept census) — the seal
  binds the report to its own content; cross-run identity is not a
  seal requirement.

## Anti-vacuity witnesses (all green this session)

- G1 positive control (alloc counter must move).
- G3 injected offset mutation flips the equality.
- G4 two flip witnesses (SHAKE256 input flip, Poly1305 message flip).
- G5 partial-accept witness (13.2M calls > 1× frames) + FrameTooLarge.
- G6 typed backpressure re-witness at cycles 0/50/100/150.
- G7 tamper flip on a 50k-link chain (mutated copy refuses typed).
- Seal tamper detection: one flipped body byte breaks seal
  verification.
- Driver disqualification path: sandbox-injected UNMET flips QUALIFIED →
  DISQUALIFIED, typed, naming the gate; SKIP→PROVISIONAL witnessed.

## Defects caught during bring-up (non-vacuity receipts)

1. Vacuous verify: `AoqChain::new` resets `len` to 0 — the post-rep
   verify over a freshly constructed chain verified an EMPTY chain
   (vacuous green) and the tamper leg panicked index 25000/len 0.
   Fixed by re-appending the full span before `verify_records` over the
   exact written slab; the vacuous-verify trap is now named in the code
   comment.
2. G6/G5 cross-thread segment collision: the three tests run in
   parallel threads of ONE process (same pid) — `{pid}-{cycle}` paths
   collided (StaleOrdinal at cycle 8, witnessed in release pass 1 of
   the pre-salt bytes) and G5 carried the same risk. Fixed with atomic
   salts on the /tmp segment paths. Both defects are structural, not
   flake-noise: parallel-test hygiene is now salted by construction.

## Cross-ref line

- `docs/cbmqs-standard.md` does not exist in cgap: the standard's
  canonical location is
  `/Users/sac/arcus/docs/jira/v26.11.3/cbmqs-standard.md`
  (AF-21's coordinator correction). The RFC-0089 scorecard cross-ref
  line was added to the arcus standard's See Also section (typed cite
  of the scorecard implementation + this receipt).

## Disclosures

1. Driver asserts on structural law, not on an all-MET outcome: the
   honest verdict (DISQUALIFIED via G7 leg B) is recorded, printed, and
   the driver stays green on structure + seal + injection witness.
2. G7 leg A measures the cgap-aoq `AoqChain<Blake3Digest>` plane at
   50k links (release, min-of-3): the MET leg. G7 leg B cites AB-14's
   debug-profile cgap-ffi number. The two planes/profiles are
   disclosed per-row, never conflated.
3. G6's in-process churn is a NEW court (this lane): 200 wire-shm ring
   cycles in-process, composed with AB-17's disk-grounded cross-repo
   citation for the leak witnesses (fd/RSS) which are NOT re-measured
   in-process (lsof spawn would re-measure; cited instead, disclosed).
4. Seals are per-run-content-bound, not cross-run-deterministic: two
   release runs produce different hashes when measured content varies
   (G2 ns, G5 accepts). This is the AF-14 pattern's semantics: the seal
   binds the report to its own content.
5. `Cargo.toml` untouched (cgap-blake3 dev-dep already landed by AF-14);
   `cbmqs_gates.rs` untouched (AF-10/AF-25's artifact). Diff is
   additive: one new test file, one See-Also entry in the arcus
   standard, this receipt.
6. The lane law said "docs/cbmqs-standard.md RFC-0089 cross-ref line"
   with repo cgap — the file doesn't exist in cgap; the standard lives
   in arcus (AF-21's correction, followed here).
