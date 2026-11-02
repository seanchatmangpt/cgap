# Run the court suites

Prove the campaign's gates are non-vacuous: the 13 court cases, the boundary
gate's mutation kill-rate, and the inert-shim negative control. Every command
below is witnessed at exit 0 in
`docs/jira/v26.11.1/waves/f3-04-courts-green.md`.

## Run the verdict courts

```bash
cargo test -p cgap-court --test court_verdict
```

Exit 0 with 13 passed / 0 failed in 13.63s (witnessed run, f3-04 leg 1,
build root `/tmp/f3-4-target`). The suite counts are 13/14: 13 passing
court cases out of the 14-case surface — the 14th slot is the negative
control exercised by the boundary gate below, not a Rust test.

## Run the mutation kill-rate

```bash
python3 scripts/boundary_gate.py --mutate --report /tmp/ledger.json
```

Exit 0 requires kill_rate >= 0.98. The witnessed run killed 19/19
(kill_rate 1.0000), 5 equivalent mutants, 0 survivors. Read the ledger
back from disk and confirm `kill_rate` before claiming a pass — stdout
alone is not the receipt.

## Run the inert-shim negative control

```bash
python3 scripts/boundary_gate.py --shim-court-only
python3 scripts/boundary_gate.py --shim-court-only --shim-inert
```

The first must exit 0 (inert shim killed by the a04_identity_court). The
second must exit 1 with `VACUOUS GATE` — that exit 1 is the required
result. If the inert shim survives, every pass the court could produce is
void (f3-04 legs 3-4, gate §6.3).

## Confirm the zero-string gate self-check

```bash
python3 scripts/zero_string_gate.py --self-check
```

Exit 0, `poison fired=True hits=2; clean hits=0; RESULT PASS` (f3-04 leg 5).
A self-check that cannot fire its own poison is a vacuous gate — treat exit
0 without the poison line as a fail.

## Troubleshoot

- Exit 2: subject missing — check you are at the repo root.
- Exit 3: loud skip, named reason in output; never read as a pass.
- Build cache: use a fresh `CARGO_TARGET_DIR=/tmp/<lane>-target` and delete
  it after the run.
