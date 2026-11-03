# Lane XC-03 — ΔS+Φ≡R (mod Σ) prose-only gap: position invariant lands

2026-11-02 | owner: XC-03 | subject: main (7f7ccdf7) + session diff over
`crates/cgap-ffi/src/receipt_chain.rs` only

## 1. Files written

- `crates/cgap-ffi/src/receipt_chain.rs` (only owned file edited)
- `docs/jira/v26.11.2/lanes/lane-xc03-position-invariant.md` (this receipt)

## 2. Code vs prose — the honest split

- STILL PROSE-ONLY (marked in the module doc, citing red-13 §1 and
  `audit/cgapprd-02-equation-grounding.md:47-55`): `ΔS_admitted`,
  `Φ_lease`, and `Σ_PQC` have NO referent in any block payload this
  chain hashes — Σ_PQC is audit-confirmed INVENTED LORE (zero Rust
  hits tree-wide). Per dispatch law the missing quantities were NOT
  invented. ΔS/Φ exist only as cgap-git commit trailers
  (commit_synth.rs), not in SealedBlock/ChainReceipt.
- NOW CODE: the real link law `R_i = BLAKE3(prev_hash || block_bytes ||
  license_bytes)` (pre-existing) PLUS the strict per-link position
  invariant (this lane): in `verify`, receipt i must carry
  `index == i` (strictly increasing, zero gaps) and
  `prev_hash == prior link's hash` (genesis anchor at i = 0); violation
  is the ONE opaque `Spoliation` class. Doc comment updated to name
  the invariant and the prose-only remainder.

## 3. Commands + exits (this session)

- Shared tree: `cargo test -p cgap-ffi --lib receipt_chain`
  (CARGO_TARGET_DIR=/tmp/xc03-target) -> exit 0, 7 passed / 0 failed
  (6/6 FIX-08 baseline + `chain_position_monotonicity_invariant`).
- Full fence: `cargo test -p cgap-ffi --lib -- --skip wave9_a07`
  -> exit 0, 50 passed / 0 failed (wave9_a07 = pre-existing A07
  environment failure, witnessed by FIX-08 §6).
- Sandbox /tmp/xc03-sandbox (dep-closure copy: 13 crates + trimmed root
  manifest; topology guard refused a full-repo rsync, so the copy is
  crates-only + generated root manifest):
  - baseline: exit 0, 7/7;
  - MUTATION (python3 exact-match replace, count==1 asserted): remove
    the `r.index != i || r.prev_hash != prev` check from `verify` ->
    exit 101, `chain_position_monotonicity_invariant` FAILED (also
    tampered court, expected — the check carries the stored
    prev-hash tamper arm);
  - RESTORE: cp from shared tree, sha256-exact
    `f3b91d5789fffe88d729c7384b8bf9f11619dcca8c44452fb42cf6beb99042c4`
    matched, mtime bumped (touch), rerun exit 0, 7/7.

## 4. Courts + anti-vacuity

- `chain_position_monotonicity_invariant` (new): baseline k=4 pristine
  chain verifies OK with zero-gap positions (always-refusing verify =
  VACUOUS GATE); three data-level mutation arms over in-memory copies:
  (A) rewound index (link 1 relabeled 0 — every digest internally
  consistent, ONLY the position invariant can see it), (B) gap (link 2
  advanced), (C) position permutation (links 2/3 swap). Each refuses
  typed `Spoliation`, final state asserted.
- Code-level sandbox flip witness in §3 — the court cannot pass without
  the invariant.

## 5. Fences (must-not-regress surfaces named + run)

- cgap-ffi lib courts 50/50 green excluding the pre-existing wave9_a07
  failure; chain family 7/7.
- The new verify check subsumes the stored prev-hash tamper arm the
  tamper court previously caught only implicitly; tamper court green
  unchanged.

## 6. Divergences / disclosures

- Another lane's in-flight `crates/cgap-ffi/src/stats.rs` was stale in
  my first sandbox copy (compile error E0425/E0277); refreshed from the
  shared tree once to get a compiling baseline. No shared-tree edit; no
  unblock applied to it — its owner's mid-flight edits supersede.
- Anti-vacuity mutation ran on the /tmp sandbox copy only; shared tree
  mutated zero times (sha256 of shared file unchanged across the flip:
  f3b91d57...).

## 7. Standing

ALIVE — every claim above carries the command + exit run this session.
