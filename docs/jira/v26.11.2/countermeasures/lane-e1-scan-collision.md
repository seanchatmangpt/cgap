# Lane E1 — scan-collision fix + torn-write verify

Date: 2026-11-02 · Repo: /Users/sac/cgap · Branch: main (no git writes)

## 1. Torn write (closure_alloc_court.rs:101) — VERIFY ONLY

Current state: **compile FIXED-BY-OWNER** (F9 landed a splice fix before E1
touched nothing in that file; E1 made zero edits to it).

Command + results:

```
CARGO_TARGET_DIR=/tmp/e1-target cargo test -p cgap-storage --test closure_alloc_court
# compile: green (Finished test profile; executable built)
# 3 tests, finished in 1845.23s:
#   court_f9b_seen_set_semantics       ... ok
#   court_f9a_alloc_falsifier_under_1000 ... FAILED
#       alloc falsifier: 7899209 allocs / 3419485750 bytes; ceiling 1000
#       (crates/cgap-storage/tests/closure_alloc_court.rs:86)
#   court_f9c_churn_bounded              ... FAILED
# test result: FAILED. 1 passed; 2 failed
```

Verdict: **compile FIXED-BY-OWNER; courts f9a/f9c FAIL (real, executed
failures — not compile-abort)**. These two budget/churn courts are F9's
in-flight optimization work (allocation ceiling 1000 vs measured 7.9M
allocs). E1 does not edit F9's file per lane contract.

## 2. Scan collision — OPTION (a): scanner exemption, justification-required

tracking_alloc.rs (`crates/cgap-core/src/tracking_alloc.rs`) is
`#[path]`-included by `crates/cgap/tests/alloc_boundary.rs` (test-only
collaborator) but lives physically in `cgap-core/src/`, so the
source_scan_gate directory scan trips on `core::alloc::` (banned token
`alloc::`).

Fix: per-token, justification-required exemption marker in the scanner.

- `crates/cgap-core/tests/source_scan_gate.rs`:
  - new `SOURCE-SCAN-EXEMPT: <token> : <justification>` marker parse
    (separator is `" : "` with spaces so `alloc::` survives the left side);
  - marker requires (i) token in the BANNED table and (ii) justification
    >= 20 chars, else an `INVALID-EXEMPTION` violation is reported (a
    marker without justification is a hole, not a pass);
  - exemption is per-token per-file; every other banned token still trips;
  - `scan_dir` reports violations per file via `scan_file`;
  - BANNED table and no_std invariant UNCHANGED.
- `crates/cgap-core/src/tracking_alloc.rs`: one marker line added above the
  `use core::alloc::` import (comment only, no code change).

Anti-vacuity courts added (all run green, same binary):

- `exemption_requires_justification_and_known_token` — short justification
  or unknown token => INVALID-EXEMPTION violation (marker-without-proof is
  flagged).
- `exemption_is_scoped_not_a_blanket_pass` — a valid `alloc::` marker does
  NOT suppress `vec!` / `std::` in the same file.
- `real_tracking_alloc_marker_present_and_effective` — the real
  tracking_alloc.rs scans clean WITH its marker and re-trips `alloc::`
  with the marker REMOVED (mutation flip; the /tmp-reintroduced-violation
  flip requirement is witnessed by this flip plus the existing
  file-level injected-vec positive control, which still runs).

## 3. Gates re-run this session

```
CARGO_TARGET_DIR=/tmp/e1-core-target cargo test -p cgap-core --test source_scan_gate
# 8 passed; 0 failed (5 original + 3 new anti-vacuity courts)

CARGO_TARGET_DIR=/tmp/e1-core-target cargo test -p cgap-core --test manifest_court
# 3 passed; 0 failed (dod1, dod10_urn, dod10_no_spdx)
```

## 4. Files written by E1

- crates/cgap-core/tests/source_scan_gate.rs (exemption mechanism + 3 courts)
- crates/cgap-core/src/tracking_alloc.rs (marker comment line only)
- docs/jira/v26.11.2/countermeasures/lane-e1-scan-collision.md (this receipt)

No other files touched. No git writes. No Process.sleep (bounded polls
only, marked POLL-STEP in-session; no sleeps landed in any source file).

## Standing

- Scan-collision fix: ALIVE (commands + exits above, this session).
- closure_alloc_court f9a/f9c: FAIL (real executed failures, F9-owned);
  f9b ALIVE-passing; compile FIXED-BY-OWNER.
