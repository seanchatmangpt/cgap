# Lane FIX-04 — Andon refusal-quota seam (red-14 hardening)

Date: 2026-11-02 · Standing: **ALIVE** (all courts run this session, exits below)

## 1. Identity
- Lane FIX-04, repo /Users/sac/cgap (branch main).
- Subject: `crates/cgap-git/src/ingress_ring.rs` (sha256
  a4bbcf0ed8a737d346ac67b39d62df5b8fe8d786cfba8dc1bc481b83c2d81cdd),
  `crates/cgap-git/src/egress_ring.rs` (sha256
  4571c8fef4b2b5cb6f1707f6d1c3e7ba2081c542542399b0f42bd5e31766bf97),
  new `crates/cgap-git/tests/andon_liveness_court.rs`.

## 2. Authority
Dispatch prompt owned-files manifest; FIX-04 of the v26.11.2 hardening
wave. Read-first: `docs/jira/v26.11.2/redteam/red-14-andon-dos-exploit.md`.

## 3. Diff
- Both rings gain a typed, observational `RefusalQuota` seam
  (`ingress_ring.rs:82-166`, `egress_ring.rs:130-196`): `note_refusal()`
  escalates `tripped` at `DEFAULT_MAX_REFUSALS = 4_096` per window;
  `note_admission()` clears the window (recoverable); `total_refusals`
  is a lifetime audit counter. Ring fields + `refusal_quota()` reporter
  added; push/pop/push_record wire `note_refusal` on every class-101
  refusal path (FrameTooLarge, Overflow, ingress Empty) and
  `note_admission` on success. The quota NEVER gates admission — no
  process-exit behavior added (grep over owned files: only a doc
  comment mentions `process::exit`).
- New court `tests/andon_liveness_court.rs` (131 lines): ingress 1M
  FrameTooLarge flood + post-flood admit/drain; ingress quota
  reachability/recoverability; egress 1M RecordTooLarge flood +
  post-flood admit + byte-exact PAR2 serve.
- Module doc comments updated to the seam contract.

## 4. Courts run (command + exit, this session)

| Court | Command | Exit | Result |
|---|---|---|---|
| andon_liveness_court (3 tests) | `cd /Users/sac/cgap/crates/cgap-git && CARGO_TARGET_DIR=/tmp/fix04-target cargo test --test andon_liveness_court` | 0 | 3 passed |
| ingress_ring_court (existing fence) | same target dir, `--test ingress_ring_court` | 0 | 7 passed |
| egress_ring_court (existing fence) | `--test egress_ring_court` | 0 | 5 passed |
| MUTANT-1 (ingress escalation deleted) | `/tmp/fix04-sandbox2` copy, `cargo test --test andon_liveness_court` | 1 (test fail) | 2 failed / 1 passed — FLIP CONFIRMED |
| MUTANT-2 (egress escalation deleted) | same sandbox, `--test andon_liveness_court` | 1 (test fail) | 1 failed / 2 passed — FLIP CONFIRMED |

Both mutations were applied to /tmp sandbox copies only
(`/tmp/fix04-sandbox2`, standalone manifest), never the shared tree;
shared-tree sha256 re-verified byte-exact at baseline after the
mutation runs (hashes in §1). Egress mutant fails exactly the egress
court and vice versa — targeting is precise, no vacuous flip.

## 5. Fences
- `cargo test -p cgap-git` full-crate run is currently BLOCKED by OTHER
  lanes' in-flight test files (ref_store_width_arms_court.rs,
  posh_refusal_matrix_court.rs, quota_alloc_purity_court.rs — compile
  aborts in files this lane does not own; compile-abort is not a
  failure count). Targeted fence (ring courts + new court) exit 0.
- Existing ring courts pin class 101 (`ingress_ring_court.rs:92`,
  `egress_ring_court.rs:198-200`) — green, mapping unchanged.

## 6. SLA unblocks (disclose-in-both-directions)
1. `crates/cgap-bn3/src/noalloc.rs` — RC-03's in-flight file froze the
   shared compile (`NA_SRAM_CEILING_bytes_FALLBACK` undefined). I
   applied a one-line unblock (mirrored the sibling
   `NaMatches::init_enclave` call shape: `Self::init_within(
   NA_SRAM_CEILING_BYTES)`). RC-03 was actively rewriting and
   superseded the diff minutes later; final file state is theirs.
2. `crates/cgap-git/tests/quota_alloc_purity_court.rs` — RC-14's
   in-flight file (courts MY RefusalQuota seam) lacked
   `extern crate alloc;`; I added the one line. RC-14 subsequently
   kept editing (later a stray `}` at :286 — theirs to fix). RC-14's
   receipt should cite this unblock; mine cites theirs.
Shared compile was otherwise blocked by unrelated in-flight lane files;
compile-aborts counted as zero failures.

## 7. Standing
ALIVE for the FIX-04 surface: quota seam landed, both anti-vacuity
mutants flip witnessed, existing ring courts green, no process-exit
behavior added. Full-crate `cargo test -p cgap-git` is UNVERIFIED at
receipt time solely due to foreign in-flight compile aborts; the ring
fence (15 tests across 3 court files) is ALIVE exit 0.
