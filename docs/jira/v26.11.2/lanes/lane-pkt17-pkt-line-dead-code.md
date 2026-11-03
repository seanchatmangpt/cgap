# Lane PKT-17 — pkt_line dead-code cleanup

Date: 2026-11-02 · Owner file: `crates/cgap-git/src/pkt_line.rs` (only file touched)

## W1 Subject

8 dead-code warnings in `crates/cgap-git/src/pkt_line.rs` per FS-17 census
(live line numbers shifted by +1/+2 after edits; pre-edit numbers cited).

## W2 Evidence (baseline, this session)

`CARGO_TARGET_DIR=/tmp/pkt17-target cargo build -p cgap-git` → exit 0,
lib warnings = **11**, of which 8 pkt_line dead-code sites:

| Pre-edit line | Item | Repo-wide usage (grep) |
|---|---|---|
| :27 | `PKT_BUF_LEN` | in-file only (encode_* signatures); flagged transitively |
| :164 | `err_message`/`is_flush` | `tests/pkt_line_court.rs:178,179,185` |
| :186 | `encode_pkt` | `tests/pkt_line_court.rs:121,151,183,194,201` |
| :212 | `HEX` (census said "encode_err second signature" — actually the HEX const) | in-file only (encode nibble emission) |
| :215 | `encode_flush` | `tests/pkt_line_court.rs:153` |
| :221 | `encode_delim` | `tests/pkt_line_court.rs:155` |
| :227 | `encode_response_end` | `tests/pkt_line_court.rs:157` |
| :233 | `encode_err` | `tests/pkt_line_court.rs:166` |

## W3 Disposition: 8/8 `#[allow(dead_code)]` + reason, 0 deletions

Deletion was ruled out repo-wide: every encode/decode item is referenced by
the integration court `tests/pkt_line_court.rs` (10 tests, all pass
post-edit); `PKT_BUF_LEN`/`HEX` are in-file dependencies of the court-tested
encode surface. Removal would break the fence. Each site got a targeted
`#[allow(dead_code)]` with a one-line reason naming its court use, per the
dispatch's wire-format-completeness preference.

## W4 Commands (this session, /tmp build root)

- baseline build exit 0 (11 lib warnings, 8 pkt_line)
- `cargo build -p cgap-git` post-edit exit 0 — **lib warnings 11 → 1**
  (only remaining: pre-existing `unused variable: bound` in another module;
  pkt_line contributes 0)
- `cargo test -p cgap-git` exit 0 (all suites 0 failed; doc-tests 0/0)
- `cargo test -p cgap-git --test pkt_line_court` exit 0 — **10 passed / 0 failed**
- Build-root lease `/tmp/pkt17-target` deleted post-run.

## W5 Divergence from dispatch

- Census item ":212 encode_err second signature" was actually the `HEX`
  const at :212 (there is exactly one `encode_err` at :233). Recorded as a
  census mislabel, not a code defect.
- Warning delta measured 11 → 1 (dispatch predicted 11 → 3; concurrent
  lanes had already cleared the deprecated-fetch_update / no_std-attr
  warnings, or the census was of an earlier tree state).

## W6 Standing

ALIVE — commands + exits run this session against the exact owned file;
shared tree untouched outside the owned file; no git write commands; no
sandbox mutation (behavioral no-op diff: 9 inserted attribute/comment
lines, 0 semantic edits). Zero-alloc sentry court behaviorally untouched
(`pkt_line_court` 10/10 green).
