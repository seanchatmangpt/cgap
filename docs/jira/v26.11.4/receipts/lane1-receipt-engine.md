# LANE-1 — Forensic receipt engine + census enforcement

Standing: ALIVE

Date: 2026-11-04 (UTC ~21:10Z close). Repo: /Users/sac/cgap. Lane: LANE-1
(receipt engine + census enforcement), staggered vs benchmark lanes on the
one canonical checkout.

## Plane law block

This receipt covers ONLY the LANE-1 receipt-engine surface:
`crates/cgap-receipt/` (new, standalone single-crate workspace),
`scripts/receipt_header.py`, `scripts/receipt_verbatim_verify.py`,
`scripts/receipt_engine_court.py`, and the additive `--receipt-verbatim`
arm of `scripts/profile_guard_census.py`. Never cross-cite onto bench
lanes' benches (C1/C2 own those Cargo.toml surfaces; no existing bench
file was edited).

## What landed

- `crates/cgap-receipt` — zero-dep Rust engine: self-contained receipt
  header (plane const, live anchored `git rev-parse HEAD`, target binary,
  build profile, cores, optional alloc signature, 1/5/15-min load averages,
  UTC civil timestamp from SystemTime). Profile law identical to
  `cgap_aoq::profile_guard` axes: debug profile -> typed
  `ReceiptError::DisqualifiedProfile` + `DISQUALIFIED_PROFILE` ledger line,
  never a warning. 7 unit courts, both profiles.
- `scripts/receipt_header.py` — Python twin emitter, same field contract;
  fail-closed `--require-release` arm; poison/clean `--self-check`.
- `scripts/receipt_verbatim_verify.py` — VERBATIM_MISMATCH enforcement:
  diff-checks a receipt's cited verbatim block against the raw log line
  range it cites (BM-3 precedent mechanized). Exit 1 on byte drift /
  miscited range / source-less claim; loud `[SKIP]` (exit 3) for a missing
  raw log; `--self-check` poison/clean twins.
- `scripts/profile_guard_census.py` — additive opt-in `--receipt-verbatim`
  composite arm (default census exit ABI byte-identical; a vanished /tmp
  log loud-skips instead of turning the timing census red). 3 edited
  regions only.
- `scripts/receipt_engine_court.py` — anti-vacuity shim-court for the
  engine itself (5 legs, below).

## Witnessed defect fixed in the engine itself

Unanchored git identity: an early probe run from a foreign cwd
(`/Users/sac/arcus`) minted a FOREIGN repo's SHA (arcus HEAD f88ce140…) as
receipt identity. Fix: identity is anchored — Rust `capture`/`emit` take
`repo_dir` (bin: `CGAP_REPO_DIR` env; anchored re-probe returned cgap HEAD
e251f258…, rc=0), Python emitter uses `git -C REPO_ROOT` derived from the
script location. Court leg M3's anchor arm plus the new unit court
`identity_anchor_refuses_non_repo_dir` pin the law.

## Commands + exits (this session)

```bash
cargo test --manifest-path crates/cgap-receipt/Cargo.toml            # exit 0 (7 passed, debug)
cargo test --release --manifest-path crates/cgap-receipt/Cargo.toml  # exit 0 (7 passed, release)
python3 scripts/receipt_header.py --self-check                       # exit 0 (SELF-CHECK OK)
python3 scripts/receipt_verbatim_verify.py --self-check              # exit 0 (SELF-CHECK OK)
python3 scripts/receipt_engine_court.py                              # exit 0 (COURT PASS, 5/5 legs flipped)
python3 scripts/profile_guard_census.py --self-check                 # exit 0 (SELF-CHECK OK)
python3 scripts/profile_guard_census.py                              # exit 1 (see census disclosure)
```

Release binary probe (rc, verbatim): debug bin -> rc=1 with
`LEDGER [DISQUALIFIED_PROFILE] DISQUALIFIED_PROFILE: non-release profile:
debug_assertions=true opt-level=unset`; release bin ->
rc=0, full header incl. `build-profile: release`, cores 16,
alloc-signature, load-avg, UTC.

## Anti-vacuity court witness (SHIM-COURT protocol)

`python3 scripts/receipt_engine_court.py` -> exit 0, all 5 legs FLIPPED,
restores sha256 byte-exact:

- m1-compare-disable: verifier's drift comparison disabled in a sandbox
  copy -> its own `--self-check` goes VACUOUS (rc=1).
- m2-profile-axis-disable: emitter's debug axis disabled in a sandbox copy
  -> `--self-check` goes VACUOUS (rc=1).
- m3-rust-emit-permit: sandbox copy of the crate with the fail-closed
  profile check dead -> `cargo test` (debug) FAILS the fail-closed court
  (rc=101).
- m4-receipt-field-corrupt: one digit of the BM-3 receipt's 50k-span
  number corrupted in a sandbox copy -> production verifier refuses with
  VERBATIM_MISMATCH.
- m5-restore-corrupt: botched restore detected via sha256 mismatch (the
  restore-detection is not vacuous).

## Census disclosure (pre-existing, another lane's subject)

Plain census this session: `77 TIMING files, 67 guard-composing files,
12 allowlist entries` — exit 1 on
`crates/cgap-ffi/tests/aoq_flip_ladder_court.rs` (un-allowlisted TIMING
file, in-flight from another lane). The gate is working as designed; NOT
allowlisted by this lane (new allowlist entries only with a named ticket,
and the ticket belongs to the file's owner). My composite arm inherits the
same fail-closed behavior.

## Pre-existing finding surfaced by the verifier (not repaired — other lane's artifact)

`bm3-chain-scale-v2.md` cites "Source of record … (lines 11-23)" but its
16-line verbatim block is byte-faithful to `/tmp/bm3_runs_v2.log` lines
9-24. The measured numbers are byte-faithful; the cited RANGE is miscited
by 2 at the head. Typed finding: VERBATIM_MISMATCH (exit 1). Left
untouched: the receipt belongs to the BM-3 lane; repairing another lane's
receipt silently would violate lane ownership. The BM-3 owner should
re-cite the range as 9-24 (content verified identical).

## Honest residue

- `crates/cgap-receipt` is deliberately NOT a root-workspace member (root
  members list is coordinator-scaffolded; lanes never edit the root
  manifest). Consumers use a path dependency
  (`cgap-receipt = { path = "../cgap-receipt" }`). Consequence: the census
  (which enumerates root-workspace members) does not scan this crate's
  files — none of them are timing files, so the census surface is
  unaffected.
- My crate's `target/` build lease is deleted at lane close (cleanup law).
- Head moved during the session (e251f258 landed mid-flight from another
  lane); receipts here are anchored to the working-tree subject, exact
  commit SHA recorded at commit time below.

## Status

ALIVE: engine built, tested both profiles, court 5/5 flipped with
byte-exact restores, census self-check green, plain census exit 1 on a
pre-existing other-lane finding (disclosed, not mine to fix).
