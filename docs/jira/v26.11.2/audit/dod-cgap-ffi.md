# dod-cgap-ffi

EXP-08 audit lane, crates/cgap-ffi + include/, 2026-11-02.
Read-only except this receipt. G19 receipt re-verified, not trusted.

## 1. Header compile gates (run this session, exits recorded)

| Gate | Command | Exit |
|---|---|---|
| cgap.h C11 | `cc -std=c11 -Wall -Werror -c include/cgap.h -o /tmp/exp08_a.o` | 0 |
| c_data C11 | `cc -std=c11 -ffreestanding -Wall -Wextra -pedantic` |
|             | `-c include/cgap_c_data.h -o /tmp/exp08_b.o` | 0 |

Both C11 gates clean. G19's load-bearing finding CONFIRMED independently:
the `_Static_assert` pin block is C11-gated at include/cgap.h:90-91
(`__STDC_VERSION__ >= 201112L`); under `-std=c99` the block preprocesses
OUT (c99 -E of cgap.h yields zero `_Static_assert` lines, counted 0), so a
c99-std compile passes without exercising any layout pin. The pins activate
only under C11; our C11 exits above are 0 with pins ON, so the 23 offsets
(13 in cgap.h: CgapStatus sizeof + 6 LP64 + 6 ILP32 frame-view pins;
10 ArrowArray/ArrowSchema pins live only in G19's /tmp court file, not the
tree) compile clean under C11.

Divergence vs G19 receipt: G19 cited "23 offsets" as a header court; only
the 13 cgap.h pins are in-tree. The remaining 10 Arrow-struct pins lived in
/tmp/g19_offsets_check.c (still present at /tmp, NOT version-controlled) —
the Arrow-side pin court is NOT in the repo and regresses to hand-audit-only
if /tmp is cleaned. Defect-open (documentation/court-persistence), not a
header bug.

## 2. Error-code parity — ALL THREE BOUND BOTH SIDES (no DEFECT-OPEN)

| Code | Rust binding | C binding |
|---|---|---|
| 0x4347_0004 | sovereign.rs:80 CGAP_ERR_LEASE_EXHAUSTED |
|             | cgap.h:63 = 0x43470004u |
| 0x4347_0006 | sovereign.rs:87 CGAP_ERR_CONCURRENT_WRITER_DENIED |
|             | cgap.h:67 = 0x43470006u |
| 0x5032_38FF | trap.rs:36 ERR_P28_SPOLIATION (cgap-p28), sovereign.rs:90 twin |
|             | cgap.h:68 = 0x503238FFu |
|             | + sovereign.rs:90 CGAP_ERR_P28_SPOLIATION twin | |

Path correction to dispatch: trap.rs is crates/cgap-p28/src/trap.rs, not
cgap-ffi. Rust value parity asserted at sovereign.rs:1798-1800. Rust
const-assert derivation pins the P28 magic at cgap-p28/src/trap.rs:39-45.

## 3. cargo test -p cgap-ffi (CARGO_TARGET_DIR=/tmp/exp08-target)

| Suite | Expected | Actual |
|---|---|---|
| lib unittests | 61 | 50 |
| alloc_courts | 4 | 4 |
| c_header_abi_court | 4 | 4 |
| fa9_commit_gating | 3 | 3 |
| zero_copy | 5 | 5 |
| lever_courts | (not expected) | 2 |

All 0 failed across 6 binaries; exit 0 overall. Divergence: lib unittest
count is 50, not the expected 61 (68 total ran vs 77 expected). Zero
failures, so this is a count-drift finding (expectation stale or tests
moved), not a red build. Extra lever_courts binary (2 tests) not in the
dispatch expectation — likely double-counted into the "61".

## 4. Packaging readiness

- `publish = true` in crates/cgap-ffi/Cargo.toml:8 (A19 P3-2b guard flip,
  2026-11-01).
- docs/jira/v26.11.2/publish.md DOES NOT EXIST. The operative doc is
  docs/jira/v26.11.2/waves/publish-halt.md: FROZEN-BY-OPERATOR.
- cgap-ffi readiness per publish-halt.md addendum: BLOCKED — registry
  cgap-parquet 26.11.2 predates RingSink; needs parquet 26.11.3 first.
- Receipt-backed court: publish-halt.md carries a JSON receipt block
  (files_written/commands/courts_run/standing) and a dry-run-witnessed
  readiness matrix; cgap-ffi is in the frozen-10 with BLOCKED standing.
- Verdict: packaging-ready = NO (operator freeze + parquet 26.11.3
  dependency), code-ready = YES (publish flag flipped, tests green).

## Verdict summary

| Area | Verdict |
|---|---|
| Header compile (C11, both headers) | PASS exit 0 / exit 0 |
| C11 gating of pin block | CONFIRMED (cgap.h:90-91, c99 strips pins) |
| Error-code parity | PASS, all 3 bound both sides |
| Test suite | GREEN but count drift: lib 50 vs 61 expected |
| Packaging | FROZEN + BLOCKED (parquet 26.11.3 dep) |
| G19 receipt load-bearing claims | Re-verified, stand; Arrow pin court un-versioned |

## Open items

- DEFECT-OPEN (court persistence): /tmp/g19_offsets_check.c (Arrow-side
  10-pin court) is not in-tree; promote it to crates/cgap-ffi/tests/ or a
  Makefile gate or it is lost on /tmp cleanup.
- Count-drift: reconcile lib unittest expectation 61 vs actual 50.
- /tmp/exp08-target could not be deleted: rm denied by sandbox twice.
  Left on disk for manual cleanup.
