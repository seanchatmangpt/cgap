# Lane AE3 — mandatory-execution test gate (`gate_cargo_test.py`)

Date: 2026-11-02 · Lane: AE3 · Branch: main · Subject:
`scripts/gate_cargo_test.py` (new, sole owned file; receipt is this
document).

## Standing

ALIVE — every claim below cites a command + exit run this session
against the exact subject.

## Design

Exit ABI 0/1/2/3. Wraps `cargo test -p <crate>` with stdout/stderr
merged at the source (`stderr=subprocess.STDOUT`) — concatenating the
streams after the fact desyncs cargo's interleaved Running-headers vs
test results (witnessed and fixed mid-lane; see Divergences #3).

Structural laws:

- Parses every `test result:` line via segment-split on `Running` /
  `Doc-tests` headers. A Doc-tests segment with `0 passed;
  0 filtered out` is lawful (no doctests), NOT evasion.
- Any `Running` segment with `passed==0 && filtered_out==0` →
  `VACUOUS_TEST_EVASION`, exit 3.
- Any crate under quota → exit 1 with the deficit.
- Compile-abort (0 targets parsed + cargo rc != 0) → exit 1 loudly
  (`COMPILE_ABORT ... counted as FAILURE, never a pass`).
- Missing cargo → exit 2.

Quotas at ~80% of measured counts (`--quota CRATE=N` overrides):

| crate           | measured (this session) | quota |
|-----------------|-------------------------|-------|
| cgap-branchless | 35                      | 28    |
| cgap-arrow      | 192                     | 153   |
| cgap            | 15–17                   | 12    |
| cgap-parquet    | UNMEASURABLE            | 50    |

## Court matrix (all run this session, real collaborators)

| #   | Scenario                                        | Expected | Observed                                                            | Exit |
|-----|-------------------------------------------------|----------|---------------------------------------------------------------------|------|
| S1  | real run: branchless, arrow, cgap               | no false vacuous; quotas met where measurable | branchless 35>=28 pass; arrow 192>=153 pass; cgap exit 3 (real zero-test bin target, Divergences #4) | 3 |
| S2  | /tmp/ae3-empty: empty lib, quota 1              | 3        | VACUOUS_TARGET [ae3-empty]: passed==0 filtered_out==0               | 3    |
| S2b | /tmp/ae3-one: 1 trivial test, quota 1           | 0        | PASS: 1 crates met quota, 0 vacuous                                 | 0    |
| S3  | --quota cgap-branchless=9999                    | 1        | QUOTA_DEFICIT: 35 < 9999 (deficit 9964)                             | 1    |
| S4  | cgap-parquet (compile-broken shared tree)       | 1 loud   | COMPILE_ABORT [cgap-parquet]: 0 tests executed, cargo rc=101        | 1    |

Anti-vacuity: S2 flips the gate; a gate that accepted the empty crate
would be vacuous. Sandbox crates and the `/tmp/ae3-target` build-root
lease were deleted at lane end (`shutil.rmtree`, exit 0).

## Commands (exit receipt)

- `cargo test -p <each crate>` under `CARGO_TARGET_DIR=/tmp/ae3-target`
  → rc 0 (branchless/arrow/cgap) / 101 (parquet compile abort).
- `python3 scripts/gate_cargo_test.py --crates
  cgap-branchless,cgap-arrow,cgap --json` → exit 3 (real finding).
- S2 → 3; S2b → 0; S3 → 1; S4 → 1.
- `pyright scripts/gate_cargo_test.py` → 0 errors, 0 warnings,
  0 informations.
- `ast.parse` token-sweep → SYNTAX OK; on-disk 6612 bytes.

## Divergences (doc yields to reality; all recorded)

1. cgap-parquet does not compile in the shared tree (17 errors, `RingSink`
   undeclared at `src/mmap_writer.rs:1821,1867` — in-flight edits from
   another lane). Its measured count is UNMEASURABLE; the dispatch floor
   quota 50 stands and the gate fails loudly on it until the tree
   compiles. Not an AE3 defect.
2. cgap has pre-existing failing tests (`patent_behavioral_court`: 2
   passed / 1 failed; `self_play_full_closed_loop`: 2 failed under
   parallel runs) — pre-existing shared-tree state, not introduced by AE3.
3. Two mid-lane bugs found and fixed by the gate's own courts: (a)
   `RESULT_RE` lacked `re.MULTILINE` (0 targets parsed); (b) post-hoc
   stdout+stderr concatenation broke cargo's header/test interleaving →
   streams now merge at spawn. Both surfaced as vacuous parses; S2
   would have caught either.
4. Real finding landed by S1: the `cgap` bin target
   `enclave_self_play_entry` executes 0 tests, 0 filtered — the
   structural law fires (exit 3) on real shared-tree state. The gate
   doing its job, not a regression.
5. Dispatch said the 1-trivial-test crate "passes quota check only if
   quota allows": S2b confirms (quota 1 → exit 0); S3 shows the
   deficit path.

## Files written

- `/Users/sac/cgap/scripts/gate_cargo_test.py` (sole owned file)
- `/Users/sac/cgap/docs/jira/v26.11.2/countermeasures/lane-03-test-execution-gate.md` (receipt)

Build-root lease `/tmp/ae3-target` deleted. Shared tree otherwise
untouched; no git write commands run.
