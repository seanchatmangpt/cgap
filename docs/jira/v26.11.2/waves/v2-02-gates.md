# V2-02 Gates Receipt (lane V2-GATES)

Lane: V2-GATES (gate-runner dispatch blocks, eight release gates G1-G8).
Charter: CHARTER-CGAP-STARDF-v26.11.2, section GOVERNING (cited).
Subject: docs/jira/v26.11.2/01-release-gates.md

## files_written

- docs/jira/v26.11.2/01-release-gates.md (G1-G8 dispatch blocks)
- docs/jira/v26.11.2/waves/v2-02-gates.md (this receipt)

## commands + exits (this session)

- `grep -ri datafusion` both repos -> exit 0, one stale-comment hit
  /Users/sac/cgap/Cargo.toml:33; Cargo.lock datafusion count = 0.
- Workspace version read: 26.11.1 (not 26.11.2) -> G1 self-fail state.
- crates/bench [[bench]] inventory: no mat-closure, no DRED target.
- Subject existence checks: verify_ost_design_around.py absent both repos;
  purrdf/scripts lacks stardf_f1_census.py / stardf_golden.py / kd_flip.py.

## courts_run

None executed this session (lane scope = transcription; court subjects
missing -> loud-skip paths documented per gate, not faked):
- G1 flip: not run (release_check.py version pin must move to 26.11.2 first)
- G2-G4 flip: not run (bench targets absent -> exit 2 loud skip)
- G5 flip: not run (riscv32imc path unwired -> exit 2 loud skip)
- G6 flip: not run; gate body run live (see commands) -> PASS-pending
  (stale comment must be deleted to exit 0)
- G7: BLOCKED (script absent, exit 3 loud skip documented)
- G8: BLOCKED (purrdf census scripts absent, exit 2 loud skip documented)

## standing

G6 PARTIAL (gate body verified live; flip court pending) |
G1 PARTIAL (subject present, version skew blocks pass) |
G2-G5 UNVERIFIED (targets absent, loud-skip documented) |
G7-G8 BLOCKED (named subjects absent).

## divergences

- Dispatch says workspace at 26.11.2 for G1; disk shows 26.11.1.
- Dispatch says court/cli/bench "lawfully delisted"; release_check.py has
  no delist table yet ( REQUIRED_VERSION also stale at 26.10.29 ).
- verify_ost_design_around.py + purrdf stardf scripts named by dispatch
  do not exist; recorded as BLOCKED, not invented.
- Charter CHARTER-CGAP-STARDF-v26.11.2 is not on disk in either repo;
  cited by reference per dispatch, contents UNVERIFIED.
