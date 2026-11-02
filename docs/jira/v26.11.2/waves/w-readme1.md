# W-README1 — kernel-crate README reconciliation

Lane: W-README1. Date: 2026-11-01. No git commands run.

## Precondition (bounded poll)

Bounded poll satisfied immediately: census receipts at
`docs/jira/v26.11.2/waves/crate-reality/{cgap-blake3,cgap-arrow,cgap-git,cgap-parquet}.md`
each witness `crateVersion 26.11.2`, and root `Cargo.toml:42` pins workspace
`26.11.2` (grep, exit 0). No waiting required.

## Archive-before-overwrite (sha256 fail-closed)

Archived to `crates/<crate>/readme-archive/<crate>-2026-11-01.md` (header
line + blank line prepend; body byte-identical to pre-edit README):

| crate | sha256 of body (pre-edit) |
|---|---|
| cgap-blake3 | 29bd5416d243b636ff39173ab49e6d23f5ecb78ecaff8270846ab425c37435a1 |
| cgap-arrow | bc4fad5a9ee0c6f85ec025fdea5247c519f17a87977130426b4c8738f5c60c81 |
| cgap-git | f2708fa137313f9f6e276f6336fea2142cc0bcfda28d22f5243f9d690a741fac |
| cgap-parquet | 9526e415552cfd05552954c616863c5d206ed2d210cfb06e6b3c4cdf64152529 |

Restore check: `tail -n +3 <archive> | shasum -a 256` reproduced each hash
exactly (exit 0, all four).

## Edits applied (per README)

- Status block: PUBLISHED-26.11.1 prose -> WORKING-26.11.2 pointing at the
  crate-reality census; 26.11.1 publish history retained as historical
  citations in the lane-receipt section.
- Test counts -> reality-census numbers, citing
  `waves/crate-reality/<crate>.md`: blake3 27/0, arrow 142/0 (134 unit +
  1 mutation-kill + 1 property + 6 validation), git 89/0 (61 inline +
  28 integration, 2 loud ignored), parquet 222/0 (1 loud ignored).
- Install: `cargo add <crate>@26.11.1` -> `@26.11.2`.
- Lane-receipt section now cites the 26.11.2 census first, v26.11.1
  receipts demoted to "historical:".
- All lines <= 100 chars (asserted in the edit script).

## Diff summary

blake3 39->41 lines, arrow 36->40, git 38->41, parquet 36->38; 5
substitutions per crate (status, census pointer, suite line, install,
lane receipt), each asserted present before replace.

## Standing

ALIVE: every command above ran this session with exit 0 against the exact
subjects. The `@26.11.2` install line is a version reference only — no
registry probe was run, so crates.io presence of 26.11.2 is UNVERIFIED.

{"lane":"W-README1","files_written":["crates/cgap-blake3/README.md","crates/cgap-blake3/readme-archive/cgap-blake3-2026-11-01.md","crates/cgap-arrow/README.md","crates/cgap-arrow/readme-archive/cgap-arrow-2026-11-01.md","crates/cgap-git/README.md","crates/cgap-git/readme-archive/cgap-git-2026-11-01.md","crates/cgap-parquet/README.md","crates/cgap-parquet/readme-archive/cgap-parquet-2026-11-01.md","docs/jira/v26.11.2/waves/w-readme1.md"],"commands":["grep workspace version (exit 0, 26.11.2)","grep crate-reality census crateVersion (exit 0, 26.11.2)","cp+sha256 archives x4 (exit 0)","python3 edit script (exit 0, 16/16 substitutions asserted)","tail|shasum restore check x4 (exit 0)","100-char line assert (exit 0)"],"courts_run":["archive restore sha256 byte-exact check (anti-vacuity: a mutated archive body would flip the hash)"],"standing":"ALIVE","divergences":["26.11.2 crates.io registry presence UNVERIFIED (no network probe run; install line is a version reference)","readme-archive/ is a new directory per crate"]}
