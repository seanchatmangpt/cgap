# Flip a court

Prove a court can fail by mutating a /tmp sandbox copy — never the shared
tree. Protocol as witnessed in `waves/a6-flip-rerun.md` (A6 NEUTER flip) and
the A17/GAP-1 court patterns.

## Copy the crate to a sandbox

```bash
mkdir -p /tmp/flipcrate/tests
cp tests/court_verdict.rs /tmp/flipcrate/tests/
shasum -a 256 tests/court_verdict.rs > /tmp/flip-shared.sha
```

Hash the shared file before and after; the final hash must match the
baseline byte-exact (fail-closed restore proof). The sandbox manifest
mirrors `crates/cgap-court`'s dev-deps as absolute path deps into the
shared tree (read-only consumption), so the copy still builds.

## Run the baseline

```bash
cd /tmp/flipcrate && cargo test --test court_verdict <court_name> -- --exact --nocapture
```

Exit 0, `1 passed`. Record the printed ledger line — the flip must change
it.

## Apply ONE named mutation

Edit only the sandbox copy. Mark it:

```rust
// A6 NEUTER — SHIM-COURT mutation (sandbox only)
```

Example (witnessed): replace the sovereign eval call block
`let status = unsafe { eval_fn(...) }` with
`let status = CGAP_OK; let count = 0usize;`. One mutation per flip; a
flip that needs two edits proves nothing.

## Predict, then run the mutated court

State the predicted failure mode BEFORE running: for the A6 neuter, exit
101, `0 passed; 1 failed`, panic at the eval-hop assert. Then run the same
command. The court must fail exactly as predicted; an unpredicted failure
or no failure both mean re-derive before claiming.

## Restore and prove it

```bash
cp /tmp/flipcrate/tests/court_verdict.rs /tmp/flip-mutated.rs   # evidence
cp <pristine copy> tests/court_verdict.rs
shasum -a 256 tests/court_verdict.rs
```

The post hash must equal `/tmp/flip-shared.sha`. Optionally touch the file
(mtime bump) so downstream mtime-based flip gates re-check it. Re-run the
baseline command: exit 0 again.

## Anti-vacuity

A court that cannot flip is FAIL. If your named mutation leaves the court
green, the court's asserts are tautological — file it, do not weaken the
mutation.
