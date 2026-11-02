# Lane GAP-02 — cgap-blake3 chained-receipt tamper courts

W1 SUBJECT: /Users/sac/cgap, branch main. New file only:
`crates/cgap-blake3/tests/chain_mutation_court.rs` (200 lines, sha256
`a7f0a5b5dcb2eb4e6cbfa09272519cb0f8a84e51154be489cda0aa6c322e02c9`).
No src edits. Receipt: `docs/jira/v26.11.2/lanes/lane-gap-02-blake3-tamper.md`.

W2 AUTHORITY: LANE GAP-02 dispatch. Port source: external court
`crates/csct/src/bin/cscb_c7_receipt_sealing.rs` (`verify_against_chain`
:73-96, bitflip court :174-202). Gap of record:
`docs/jira/v26.11.2/audit/dod-cgap-blake3.md:57-67` (T3: no in-crate
bitflip-rejection or parent-digest-chain mutation test).

W3 CONSEQUENCE: GAP closed — cgap-blake3 now carries in-crate chained
tamper courts. Test-file census: 5 files BEFORE (official_vectors,
multi_chunk_vectors, small_batch_vectors, thread_differential,
zero_alloc), 6 files AFTER (adds chain_mutation_court.rs); pre-existing 5
untouched (scoped `git status --porcelain -- crates/cgap-blake3/` shows
only the new untracked file). Root workspace untouched.

W4 REPLAY (commands + exits, this session):
- `CARGO_TARGET_DIR=/tmp/gap02-target cargo test -p cgap-blake3` → exit 0.
  Full transcript: lib 3 passed; chain_mutation_court 4 passed
  (court0_clean_chain_verifies, court_a_child_payload_bitflip_rejected,
  court_b_duplicate_digest_substitution_typed_fail,
  court_c_root_mutation_rederivation_mismatch); multi_chunk_vectors 7
  passed/1 ignored; official_vectors 6 passed; small_batch_vectors 3
  passed; thread_differential 3 passed; zero_alloc 5 passed; doc-tests 0.
  28 file-level tests + 3 lib, all green.
- Scoped git status: only `?? crates/cgap-blake3/tests/chain_mutation_court.rs`.
- Build root /tmp/gap02-target deleted at lane end (lease law).

W5 COURTS + FLIP TRANSCRIPTS (all witnessed BEFORE/AFTER in-test):
- Court 0 (baseline, non-vacuity control): 16-receipt chain verifies
  clean BEFORE every mutation. Chain structure: 8B nonce LE + 32B parent
  + 32B payload = 72B frame; parent = previous digest; genesis parent =
  zero digest; digests via real `cgap_blake3::hash`.
- Court A (bitflip): flip byte 47 (bit 3) of receipt 7's payload.
  BEFORE: clean chain verifies. Land-check: frame byte changed
  (assert_ne). AFTER: verify_chain rejects, error cites receipt 7 or 8
  (digest mismatch at 7, then parent break at 8); differential witness —
  stored-digest-only flip at receipt 7 also rejected.
- Court B (duplicate digest substitution): receipt 5's stored digest :=
  receipt 4's digest. BEFORE: digests distinct (assert_ne, substitution
  not vacuous), clean chain verifies. AFTER: typed FAIL naming "digest
  mismatch" (exact substring asserted).
  Structure ported from cscb_c7's identical-digest typed FAIL
  (:186-190) and ACCEPTED-by-court typed FAIL (:191-196) as in-test
  expect_err analogues.
- Court C (root mutation + re-derive): flip byte 31 of stored root
  digest of the 16-receipt chain. BEFORE: `hash(frame)` == stored root.
  AFTER: typed "digest mismatch" rejection AND direct re-derivation
  mismatch asserted (the exit-1-class analogue of cscb_c7's
  `process::exit(1)`), plus assert_ne proving the mutation landed.
- Anti-vacuity posture: every mutation carries a land-check assert_ne;
  every court asserts the typed rejection; no skips ([SKIP] count: 0).
  Every court names its mutation: A = payload byte 47 bit 3; B = digest
  := parent digest at receipt 5; C = root byte 31 bit 7.
- Non-vacuity caveat (honest): the courts mutate the in-test chain
  struct, not crate source — the flip-proves-gate-works direction is
  witnessed here; the src-level mutation court (shim-court with
  sha256-exact restore over src/) remains the external court's job and
  is out of this lane's file ownership.

W6 STANDING: **ALIVE** for the named surface (in-crate chained tamper
courts, cargo test -p cgap-blake3 exit 0 this session, 4 new court tests
green alongside 24 pre-existing file-level tests). Compile-freeze SLA: no
shared-file edits, nothing to unblock. Fences: existing 5 test files
could not regress — untouched on disk (sha256s recorded, W3);
`cargo test -p cgap-blake3` green confirms.

W7 DIVERGENCES:
- Dispatch said "audit/dod-cgap-blake3.md"; actual path is
  `docs/jira/v26.11.2/audit/dod-cgap-blake3.md` — read at that path.
- One mid-write corruption of the new test file (heredoc append landed a
  stray Python token block, lines 160-168) was caught by token sweep and
  repaired in the same session by full rewrite; final on-disk sha256
  recorded in W1. Recorded for the agent-corruption ledger.
- Test count language: dispatch said "cargo test -p cgap-blake3 fully
  green" — full transcript recorded in W4; doc-tests report 0 (cargo
  counts the doctest harness run separately; that is cargo's output,
  not a skip).
