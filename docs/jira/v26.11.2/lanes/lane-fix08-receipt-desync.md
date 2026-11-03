# Lane FIX-08 — receipt-chain desync hardening (RED-13 items)

2026-11-02 | owner: FIX-08 | subject: 7f7ccdf7 (main) + session diff
over `crates/cgap-ffi/src/receipt_chain.rs` only

## 1. Files written

- `crates/cgap-ffi/src/receipt_chain.rs` (only owned file edited)
- `docs/jira/v26.11.2/lanes/lane-fix08-receipt-desync.md` (this receipt)

## 2. Chosen paths per finding

### Finding 1 (doc-code divergence, RED-13 §4) — IMPLEMENT

The doc comment at the old receipt_chain.rs:111-119 claimed an
append-time cross-check against the sealer's own digest that the body
never performed. Implemented, not corrected: `append` now runs
`cgap_parquet::footer::verify_seal(bytes, sealed.digest())` after the
license gate and refuses typed (`Spoliation`, chain unchanged) on any
disagreement. The chain's own BLAKE3 link hashes whatever bytes it is
handed, so it cannot see a post-seal tamper; the sealer's recorded FNV
checksum + content digest can. Also removed the duplicated doc
paragraph (old :114-119).

### Finding 2 (chain/commit desync, RED-13 §3) — STATED BOUNDARY, no scaffold

`cgap_commit_parquet` seals with `seal(None)` (no P28 attestation
surface), keeps its lease thread-local with no chain handle, and has
no FFI return path for receipts — wiring `RingReceiptChain` in is not
a small coherent diff. Per dispatch law, nothing was scaffolded. The
commit-leg non-repudiation claim stays UNSUPPORTED; recorded as a
stated boundary in the `append` doc comment ("COMMIT-LEG BOUNDARY,
RED-13 §3"). sovereign.rs is outside this lane's file ownership.

### Finding 3 (courts green) — ALIVE

See §4. The 5/5 chain court is now 6/6 (one court added).

## 3. Commands + exits (this session)

- `cargo test -p cgap-ffi --lib receipt_chain` → exit 0, 6 passed /
  0 failed (was 5; `append_cross_checks_sealer_digest` added).
- Full lib suite minus the pre-existing failure:
  `cargo test -p cgap-ffi --lib -- --skip wave9_a07` → exit 0,
  49 passed / 0 failed.
- `cargo test -p cgap-ffi --test fa9_commit_gating` → 3 passed /
  1 failed (`fa9_release_after_owner_thread_exit`: same-path attempt
  while owner alive returned 0x0, expected 0x4347_0006).
- Flip battery in /tmp/fix08-sandbox (workspace dep-closure copy,
  excludes cli/csct/ontologies; root manifest trimmed of ontology
  members):
  - baseline 6/6 exit 0;
  - MUTATION (python3 replace, exact-match assert): remove the
    verify_seal gate from `append` → court fails, exactly
    `append_cross_checks_sealer_digest` (5 passed / 1 failed,
    `error: test failed` exit nonzero);
  - RESTORE: `cp` from shared tree, sha256-exact
    (`f207f96cf2bb160dfd067e8fdf9099874fbe7803ede6d6170ba47ce1d7e9f701`
    matched), mtime bumped, rerun 6/6 exit 0.

## 4. Courts run + anti-vacuity

- `append_cross_checks_sealer_digest` (new): baseline pristine-block
  arm (always-refusing append = VACUOUS GATE), data-level mutation arm
  (one committed body byte flipped through the block buffer's live
  bytes → typed Spoliation, receipt NOT minted, prior chain refuses),
  plus the code-level sandbox flip witness in §3.
- Existing tamper/append-gate/empty courts: green unchanged.

## 5. Fences (must-not-regress surfaces named + run)

- cgap-ffi lib courts (50 tests): 49/49 green excluding the A07 court;
  the chain court family is green 6/6.
- `fa9_commit_gating` integration court: ran; 3/4 with the failure
  named in §6. My diff (receipt_chain.rs only) cannot touch the spill
  gate; pre-existence witnessed at HEAD in the sandbox.

## 6. Pre-existing failures (NOT session-introduced — witnessed)

1. `sovereign::wave9_courts::wave9_a07_zero_string_gate_over_all_built_crates`
   — A07 gate refuses on `GATE-REFUSED: sections-unreadable
   .../lib.rmeta_` (ar-extracted rustc metadata member). Witnessed
   failing against HEAD code in the pristine sandbox AND with the
   shared-tree default target: exit nonzero in both. Environment/
   artifact-dependent; sovereign.rs scope, not this lane.
2. `fa9_release_after_owner_thread_exit` — same-path attempt while
   owner alive returned CGAP_OK (0x0) instead of
   CGAP_ERR_CONCURRENT_WRITER_DENIED (0x4347_0006). This is RED-13 §2
   (spill-gate path/ownership breach, BREACH-CONFIRMED) surfacing in
   the court. Witnessed failing against HEAD code in the pristine
   sandbox: exit nonzero. sovereign.rs scope — belongs to the RED-13
   §2 fix lane, not FIX-08.

## 7. Standing

ALIVE (finding 1 implemented + court flip witnessed; finding 3
verified). Finding 2: UNSUPPORTED by design (stated boundary, no
scaffold). Divergences: pre-existing A07 + FA9 failures recorded in
§6; sandbox root manifest trimmed of ontology members (copied tree
only — no shared-tree edit).
