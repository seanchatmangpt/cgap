# Lane RC-01 — ref_store seal-counter reentrancy repair (red-11 BREACH fix)

Standing: **ALIVE** (all commands below run this session, exits captured).
Subject: `crates/cgap-git/src/ref_store.rs` + new
`crates/cgap-git/tests/ref_store_reentrancy_court.rs`.
Dispatch contract honored: no git write commands; shared-tree mutations none
(sandbox-only, sha256-verified restore); only the two owned files written.

## W1 Subject & authority
- red-11 receipt: `docs/jira/v26.11.2/redteam/red-11-riscv-reentrancy-corruption.md`
  (BREACH-CONFIRMED: check-then-act window between ceiling check and RMW;
  at bound >= WIDTH the fetch_add wraps to 0).
- DOD-04 parent: commit a0431b70 "ref_store width-gated atomics".

## W2 Diff (mu)
1. `stamp` no longer loads/checks the counter outside the RMW. It calls the
   new private [`SealRegister::reserve`], an indivisible check-and-increment:
   the ceiling guard `c < bound` lives INSIDE the fetch_update closure, so a
   full counter yields None -> typed `CounterExhausted`; wrap is impossible
   by construction (closure never emits c+1 at c == bound <= WIDTH). Slot
   copy + FNV chain fold moved AFTER the reservation (a reentrant observer
   sees a reserved lease, never a duplicated one).
2. `Counter::fetch_add` removed (unused; it was the unguarded RMW).
3. implcell arm (truly single-core, `Cell`-backed): interrupt-safety
   contract made loud + typed — `seal_atomic::implcell::critical` is the
   seam firmware must bind (real enter/exit-critical); the non-atomic
   `Counter::fetch_update` arm wraps its get/set in `critical(|| ...)`.
   Contract documented at ref_store.rs (implcell mod + fetch_update arm).
4. New court file, 5 Chicago courts over the real code via #[path] include:
   - C1 rc01_overspend_through_rmw_is_typed_refusal — ceiling overspend
     refuses typed; counter never exceeds; authority spent-not-stuck.
   - C2 rc01_control_no_interrupt_no_overspend — no-preemption control:
     1..5 legal, then typed exhaust; chain fold determinism.
   - C3 rc01_lease_uniqueness_invariant — 64 stamps, strict monotonic,
     spent lease not re-mintable (StaleLease).
   - C4 rc01_exhausts_never_wraps_law — ceiling == WIDTH stamps once,
     chains, host arm = impl64.
   - C5 rc01_preempted_window_replay — THE flip witness: context A observes
     counter < bound, ISR spends remaining authority via real stamps, A
     resumes AT the RMW through a `#[doc(hidden)] pub fn reserve_probe`
     court seam that runs the identical production `reserve` path. Fixed:
     typed refusal, counter stays 5. Mutated: mints lease 6.
5. Errata vs red-11 receipt line "the check is not part of the RMW": fixed
   by construction — see W2.1.

## W3 Commands / exits (this session)
| cmd | exit |
|---|---|
| cargo test -p cgap-git --test ref_store_court | 0 (5 passed) |
| cargo test -p cgap-git --test ref_store_reentrancy_court | 0 (5 passed) |
| riscv32imc build --no-default-features | 0 |
| MUTATION battery: separate-check/unguarded-RMW revert in /tmp sandbox copy of ref_store.rs, reentrancy court | 101 (3 courts red: C1, C2, C5) |
| restore (current file recopied into sandbox) + rerun | 0 (5 passed); shared tree sha256 4d010011... never mutated |

## W4 Fences run
- ref_store_court (pre-existing DOD-04 court, 5 tests): green, exit 0.
- Full `cargo test -p cgap-git`: exit 101 — NOT this lane's surface. Both
  errors live in another lane's in-flight files (present in git status at
  session start): `tests/quota_alloc_purity_court.rs:286` (unexpected
  closing delimiter) and its #[path]-included
  `src/egress_ring.rs:650` (::alloc not extern). Disclosed here; the owning
  lane must run this fence after its landing.
- All other test targets in the full run compiled green; ref_store courts
  pass on both host arms exercised.

## W5 Anti-vacuity
- Flip law witnessed: reverted shape -> exit 101, C1+C2+C5 red; restored ->
  0. A court that cannot flip is FAIL; this one flips.
- No sleeps, no mocks; final-state asserts on the real register.

## W6 Divergences
1. Pre-existing, NOT owned: quota_alloc_purity_court.rs:286 syntax error +
   egress_ring.rs:650 ::alloc resolution failure (full-crate fence is
   BLOCKED on the owning lane, not on RC-01).
2. red-11 named impl16/implcell "UNVERIFIED arm": still UNVERIFIED by
   execution (host=impl64, riscv32imc=impl32); this lane compiles the
   implcell `critical` seam only via the not-atomic8 cfg — UNVERIFIED by
   execution, now with a typed contract instead of a silent assumption.
3. `reserve_probe` is a doc(hidden) court seam on a published-kernel type —
   disclosed as deliberate; removal requires relocating the flip witness.
4. Full-crate fence BLOCKED on the foreign-lane break (W4).

## W7 Replay
- Files: crates/cgap-git/src/ref_store.rs, crates/cgap-git/tests/ref_store_reentrancy_court.rs
- Shared-tree ref_store.rs sha256 at close: 4d0100118ce23884df5e32ca1252f98f6757d84c21aa85dcd450d48ad0ab51c8
- Mutation battery replay: copy tree to /tmp, apply the W3 mutation
  (unguard fetch_update in reserve), expect exit 101 on
  ref_store_reentrancy_court; restore byte-exact (verified by cmp).
