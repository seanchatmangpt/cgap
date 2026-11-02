# Lane EX2 — cgap-p28 exhaustive census (v26.11.2)

Read-only census, session 2026-11-02. Writes: this file only.

## A. File inventory (src + tests, cgap-p28) + lease coverage

| File | LOC | Branch sites | Hot/cold |
|---|---|---|---|
| `src/lib.rs` | 23 | 0 (`cfg` only, no runtime branch) | — |
| `src/asc450.rs` | 2363 | 75 `if/match/while/for`; 433 branch-family hits | mostly COLD (refusals); hot: `plan_event`/`commit_event` |
| `src/attestation.rs` | 1373 | 76 `if/match/while/for`; 240 branch-family hits; 17 `Err(` sites | ALL COLD (per BR2/FA2/w6-b8: every refusal path cold) |
| `src/trap.rs` | 768 | 25 `if/match/while/for`; 106 branch-family hits | COLD (halt path, by definition) |
| `tests/lever_courts.rs` | 222 | 3 | test-only |
| `tests/p28_invariant_courts.rs` | 397 | 12 | test-only |
| `crates/cgap-crypto/src/lease.rs` (covered here — p28 lease) | 219 | 3 runtime branches (69, 117, 138) | mixed |

**Absent names, loud**: `lease.rs`, `counter.rs`, `tripwire.rs` do NOT exist in
cgap-p28 (`find crates/cgap-p28 -name '*.rs'` = 3 files above). Lease
implementation lives at `crates/cgap-crypto/src/lease.rs` and is covered in
section C. `counter.rs`/`tripwire.rs` have no counterpart anywhere in the
crate; no candidate files.

## B. Reality checks (dispatch claims verified)

- **No saturating sites**: CONFIRMED. `grep -nE 'checked_|saturat|wrapping'`
  over `src/asc450.rs` → only `checked_add` at asc450.rs:827, :831 plus
  comments at :1491, :1613-1614 (named mutation documentation). Zero
  `saturating_add`/`wrapping` in src. forbid-comment at asc450.rs:824-834 and
  :1613-1614 present as claimed.
- **trap.rs:36 const pin**: CONFIRMED — `ERR_P28_SPOLIATION: u32 = 0x5032_38FF`
  at trap.rs:36, ABI-frozen against the attestation magic 'P','2','8'+0xFF,
  compile-time derivation pinned in the ABI-freeze block (trap.rs:38-45).
- **"abort path no production caller"**: PARTIALLY DISPROVED — the halt has
  production callers inside the crate: `cgap_p28_invariant_check`
  (trap.rs:103 null-guard, trap.rs:112 Err-arm). But it is dead across the
  workspace: cgap-ffi references `_cgap_p28_invariant_check` only in comments
  (sovereign.rs:357, :911 — "stays UNSUPPORTED"); no symbol use anywhere.
  Verdict: **workspace-dead production path**, cold as claimed, but the
  dispatch wording "no production caller" is in-crate false, cross-crate true.
  trap.rs:198/:334 callers are `#[cfg(test)]`.
- **All refusal sites COLD**: CONFIRMED for attestation.rs (all 17 `Err(`
  sites are validation refusals on untrusted input — cold by definition).
  asc450: hot path is `plan_event`/`commit_event` only; every `Err` arm
  (Overflow, chain-balance c3/c5, never-emit c3) is cold.

## C. Tick accounting path — zero-conditional-branch claim

DISPROVED. `Lease::tick` (lease.rs:115-127) contains exactly ONE conditional
branch: `if t >= self.tick_ceiling` at **lease.rs:117**, panic-arm at :118-124,
accounting at :125 (`self.tick.0 = t + 1`) is branch-free (add + store).
`Lease::phi` (lease.rs:136-151) adds a second guard branch at **lease.rs:138**,
same predicate, defense-in-depth — not on the tick path proper. Hot/cold:
- lease.rs:117 — HOT, but 100%-predictable not-taken until authority is spent;
  last iteration flips to panic (trap discipline, §5).
- lease.rs:125 — HOT, branch-free accounting.
- lease.rs:69 (`Lease::new` ceiling==0 trap) — COLD (constructor-time).
- lease.rs:138 — COLD in the tick path (only caller `chatman::seal_unsealed`,
  chatman.rs:489 `PhiLease::from_bytes` on decode; seal-time only).
- lease.rs:191-218 `to_bytes`/`from_bytes` — HOT at seal/verify, branch-free
  (copy_from_slice panics on slice-len mismatch — compile-proven lengths here).

## D. Substitution candidates / KEEP with court mutants

| Site | Candidate | Mutant (court flips) |
|---|---|---|
| asc450.rs:827/:831 | KEEP (checked_add, forbid-comment in place) | named mutant: `checked_add→saturating_add` — lever_courts.rs/p28_invariant_courts.rs exact-state assertions kill it (asc450.rs:1613-1614) |
| lease.rs:117 | KEEP (panic trap, not error return — §5 forbids error return) | mutant `if t >= c` → `if t > c` off-by-one: current_tick()==ceiling then tick() returns t instead of trapping; add court asserting `tick()` must panic at t==ceiling |
| lease.rs:125 | KEEP branch-free | mutant `t + 1` → `t | 1` (parity corrupt): current_tick parity court flips |
| lease.rs:69 | KEEP | mutant drop ceiling==0 panic → court constructs 0-ceiling lease, expects panic |
| attestation.rs (all) | KEEP, all COLD | per-check mutants covered by p28_invariant_courts.rs one-bit mutation corpus |
| trap.rs:36 | KEEP const pin | ABI-freeze `const _` asserts fail build on drift |

## E. Coverage ledger (100% target)

| Item | Status |
|---|
| 6 p28 .rs files enumerated (incl. lib.rs) + lease.rs | YES |
| Branch census per file | YES (counts in table A) |
| Saturating-site check | ran grep, exit 0, zero sites |
| trap.rs:36 pin | read this session, byte-cited |
| Zero-branch tick claim | DISPROVED at lease.rs:117/:138 |
| Substitution/KEEP table | YES, every site dispositioned |
| Anti-vacuity mutant named per candidate | YES (section D) |
| Builds run | NONE (read-only lane) |
| Verdict | PARTIAL — census ALIVE; court mutant for lease.rs:117 not executed (lane is read-only, listed as candidate) |
