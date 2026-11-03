# Lane R-ADV12FIX — ADV-12 quota defect repair

Date: 2026-11-02 · Standing: **ALIVE** (all commands + exits this session)

## 1. Identity

- Repo `/Users/sac/cgap` (branch main). ADV-12 spoilers → REPAIRED-RECEIPTED.
- Owned files written: `crates/cgap-git/src/ingress_ring.rs`,
  `crates/cgap-git/src/egress_ring.rs`,
  `crates/cgap-git/tests/andon_liveness_court.rs`.
- Disclosed minimal unblock: `crates/cgap-git/tests/quota_alloc_purity_court.rs`
  (NOT in this lane's ownership list — its self-clear assertions pinned the
  old defect semantics and deterministically flipped under the required
  level-triggered fix; three assertions updated, see §5).

## 2. Diff

- Both `RefusalQuota` twins (ingress + egress): escalation is now
  LEVEL-TRIGGERED. `tripped` clears only via (a) new explicit
  `acknowledge()` API, or (b) `CLEAN_ADMISSIONS_TO_CLEAR: u32 = 1024`
  (named const) consecutive clean admissions tracked in a new private
  `clean_streak` field. One admitted frame no longer erases the signal.
- `window_refusals` uses `saturating_add` — typed saturation at `u32::MAX`,
  no debug panic / release wrap. Window semantics preserved (resets on
  admission). `total_refusals` hardened to `wrapping_add` (u64 wrap
  unreachable per ADV-12 §3 arithmetic).
- Court: `andon_liveness_court.rs` 3→5 tests. Two existing legs updated to
  level-triggered semantics (assert `tripped()` STAYS true after 1 and
  after 4 admissions, window still resets; `acknowledge` path pinned).
  New legs: `quota_escalation_clears_only_after_sustained_quiet_window`
  (exactly-1024 hysteresis + mid-streak refusal restart, both twins) and
  `window_refusals_saturate_without_overflow` (public-field seam pins
  `u32::MAX - 1`, refuses past it, no panic).

## 3. Mutation battery (all in /tmp sandbox `/tmp/adv12fix-mut`,
sha256-verified against the shared tree, shared tree never mutated)

| Mutant | Flip witness |
|---|---|
| M1 ingress self-clear revert | andon court FAILED: 2 legs flipped |
| M2 ingress `saturating_add`→`+=` | saturation leg FAILED, "attempt to add with overflow" |
| M3 egress self-clear revert | 2 legs flipped (quiet-window + egress flood) |
| M4 egress `+=` revert | saturation leg FAILED (overflow) |

Green (unmutated): andon 5 passed; purity court 1 passed.

## 4. Gate

- `cargo test -p cgap-git` (full, lease root /tmp/radv12fix-target): ALL
  parent test summaries ok — 63 lib + every court; 0 parent failures. The
  lone `FAILED ... 4 filtered` line under octopus_merge_court is the
  expected-failing CHILD re-exec spawned by
  `mutation_conflict_injection_flip` (`tests/octopus_merge_court.rs:336-370`
  — child must exit 101); the parent court passes (5 passed).
- Shared-tree hashes byte-identical pre/post mutation battery (recorded
  sha256 for all four touched files, re-verified after).
- Lease cleanup: /tmp/radv12fix-target and /tmp/adv12fix-mut deleted.

## 5. Disclose-in-both-directions

quota_alloc_purity_court.rs belongs to another (landed) lane; its legs at
(now) :170-180 and :243-249 asserted `!tripped()` after one admission —
the exact ADV-12 defect. Minimal unblock: assert sticky `tripped()` +
window reset instead. Owning lane should supersede this diff on landing;
this receipt is the disclosure record.

## 6. Fences re-run

- `quota_alloc_purity_court` (zero-alloc fence) exit 0, 1 passed —
  zero-alloc preserved.
- `posh_refusal_matrix_court`, `ref_store_*`, `pack_demux_*` all green in
  the full run — no regression surface hit.
