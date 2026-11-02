# W2-01: zero-datafusion gate (scripts/zero_datafusion_gate.py)

Lane: W2-01. Interface per frozen W1-05 spec §4.1
(`waves/w1-05-doctor-extension-spec.md`); consensus rule per dispatch.

## What landed

- `scripts/zero_datafusion_gate.py` (stdlib-only python3, 314 lines):
  three probes, consensus-classified.
- P1 metadata: `cargo metadata --format-version 1` JSON — no package or
  dependency (any kind) whose name contains `datafusion`.
- P2 tree: `cargo tree --workspace --edges all` (CARGO_NET_OFFLINE=1)
  stdout text scan.
- P3 literal: substring scan of root Cargo.toml, every member Cargo.toml
  (explicit members list), and Cargo.lock.
- Latent opt-in channel folded into all three probes: root
  `[workspace.dependencies]` keys — an unused ws-dep is a one-line
  opt-in for any member and is flagged.
- Classification: all CLEAN -> 0; any ERROR -> 2 GATE-REFUSED; all HIT
  -> 1 VIOLATION (file:line + dep-path to stderr); mixed HIT/CLEAN
  (probe disagreement) -> 2 GATE-REFUSED.
- `--self-check`: /tmp sandbox copy (ignore .git/target/docs), two arms.
  Clean twin must classify 0; poison arm appends dummy
  `datafusion = "1"` under `[workspace.dependencies]` in the copied
  root Cargo.toml and MUST classify 1; else real gate exits 3 VACUOUS.
  Both arms recorded in the `--json` receipt.
- Final line `ZERO-DATAFUSION: PASS` / `ZERO-DATAFUSION: FAIL n`
  (release_check.py:15 style). Exit contract 0/1/2/3.

## Run receipts (this session, exact subject)

| command | exit |
|---|---|
| `python3 scripts/zero_datafusion_gate.py` | 0 |
| `python3 scripts/zero_datafusion_gate.py --self-check --verbose` | 0 |
| `python3 scripts/zero_datafusion_gate.py --json --verbose` | 0 |
| `python3 scripts/zero_datafusion_gate.py --self-check --json` | 0 |

- Production run: all three probes CLEAN, verdict PASS, zero
  violations; receipt matches the removal receipt
  (`docs/jira/v26.11.1/waves/cgap-datafusion-removal.md`: tree pruned,
  22 members, no datafusion entries).
- Self-check: `clean_twin_green: true`, `poison_killed: true` (both
  arms witnessed in the `--json` receipt; poison arm classified exit 1
  on the sandbox copy, sandbox rmtree'd). Non-vacuity: the gate's
  predicate demonstrably flips on the planted poison; a gate that
  could not flip would exit 3 VACUOUS by construction.

## Anti-vacuity mutation

Named mutation (spec §6): add a datafusion ws-dependency line to a
sandbox Cargo.toml -> gate flips to exit 1. Implemented as the
`--self-check` poison arm, executed this session (poison_killed=true).
Sandbox copies only; shared tree untouched (no writes outside the two
lane files).

## Standing

- This lane: ALIVE (table above, command+exit, exact subject
  `scripts/zero_datafusion_gate.py` on the current tree).
- Doctor-chain wiring (justfile d11 row, doctor.rs c07 SCRIPTS row) is
  NOT this lane's dispatch; remains open for the integration lane.
- Perf figures: none claimed. Compile-freeze: none incurred. No git
  commands run; no manifests edited.

## Divergences

- Dispatch narrows spec §4.1's scan corpus ("any cgap-datafusion
  reference ... use, path dep, doc pointer") to member Cargo.tomls +
  Cargo.lock literal scan; implemented per dispatch (no .rs/doc scan).
- Spec §6 mutation says "add a `cgap-datafusion` path dep line";
  dispatch overrides with the dummy `datafusion = "1"` ws-dep into the
  copied root Cargo.toml — dispatch taken (registry-free, flips all
  three probes via the ws-dep opt-in channel).
- Tree probe additionally folds in root ws-dep keys so the consensus
  rule ("all three must agree") stays satisfiable on the poison arm;
  documented in the script docstring.
