# A6 flip-rerun receipt

Lane: A-6 EVAL-NEUTER-FLIP-RERUN (court-auditor, read-only on the shared tree)
Date: 2026-11-01
Repo: /Users/sac/cgap (branch main)

## Purpose

Convert two marker-cited items (R10's eval-neuter flip; R17 open item #4,
riscv rlib nm purity) to first-hand execution. Zero commands = VACUOUS; this
receipt cites every command and its exit.

## Files written

- docs/jira/v26.11.1/waves/a6-flip-rerun.md (this receipt; the only shared-tree
  write, and it is the lane's leased file).

## Commands and exits (all first-hand, this session)

1. Shared-tree invariant pin: `shasum -a 256 tests/court_verdict.rs` ->
   `8184d1e961e1c59c8986e746f1c79adb3095136b2d104ce89963e924c9cb055c`
   (matches R10's cited pin; re-verified after all runs, same hash).
2. Sandbox build: `mkdir -p /tmp/a6-flipcrate/tests; cp tests/court_verdict.rs
   /tmp/a6-flipcrate/tests/` (sandbox copy hashes byte-identical to the
   shared tree at copy time), sandbox manifest mirrors crates/cgap-court's
   dev-deps as absolute path deps into the shared tree (read-only
   consumption).
3. Baseline court: `cd /tmp/a6-flipcrate && cargo test --test court_verdict
   a19_seal::a19_seal_1m_eval_court -- --exact --nocapture` -> exit 0,
   `1 passed`, line:
   `SEAL+EVAL 1M LEDGER (4-of-4): admitted=1000000 (zero-alloc) eval=1000000
   lanes (differential pin vs fold, 6 mutation legs) committed +
   twin-deterministic; verify_seal ok; ... 79/81 B refused`.
4. Mutation (A6 NEUTER, sandbox copy only): in the EVAL-stage per-row loop,
   the sovereign eval call block (`let status = unsafe { eval_fn(...) }`)
   replaced by a stub `let status = CGAP_OK; let count = 0usize;` (markers
   `A6 NEUTER` / `SHIM-COURT mutation (sandbox only)` in the sandbox file;
   the shared tree was never touched).
5. Mutated court: same command -> exit 101, `0 passed; 1 failed`, panicked at
   tests/court_verdict.rs:2869 (sandbox copy's line numbering = shared
   numbering pre-mutation): `assertion left == right failed: the lane rule
   matches exactly the one triple: left: 0, right: 1` — the closure-count
   leg, exactly the R10-cited failure site.
6. Shared-tree restore proof: `shasum -a 256 tests/court_verdict.rs` after
   all runs -> `8184d1e9...c9cb055c` byte-identical (restore by-construction:
   the mutation lived only in /tmp/a6-flipcrate).
7. riscv purity: `CARGO_TARGET_DIR=/tmp/a6-target cargo build --release
   -p cgap-arrow --target riscv32imc-unknown-none-elf` -> exit 0.
8. `llvm-nm <rlib> > /tmp/a6-nm.txt` -> exit 0, 12 lines: alloc-family
   symbol count 0, panic-family symbol count 0; total 4 code symbols, all
   cgap_arrow-internal (3x seal_with_mask monomorphizations + blake3::hash).

## Courts run (flip evidence)

- a19_seal::a19_seal_1m_eval_court: baseline exit 0 (green) -> mutated exit
  101 at the closure-count leg. The court FLIPS on its named mutation; the
  R10 marker-cited flip is now first-hand and NON-VACUOUS.
- riscv rlib nm purity (R17 #4): first-hand counts alloc=0 / panic=0 on a
  real riscv32imc release rlib.

## Standing

ALIVE. Both conversions executed, both flips witnessed first-hand. Shared
tree byte-invariant (tests/court_verdict.rs sha256 unchanged before/after).
/tmp leases /tmp/a6-flipcrate, /tmp/a6-target, /tmp/a6-nm.txt: cleanup
attempted post-receipt and REFUSED by the permission system (rm denied);
they remain on disk. Coordinator may delete at integration.

## Divergences

1. R10's receipt described the mutation as inside "the closure loop" of the
   EVAL stage; the file contains two eval-call sites (the per-row EVAL-stage
   loop and the admit_eval closure used by the M-legs). The neuter was
   applied at the per-row EVAL-stage loop, which is the site whose
   closure-count leg ("the lane rule matches exactly the one triple") R10
   cites as the failure site — consistent.
2. The sandbox needed the cgap-court dev-deps moved to [dependencies] on the
   standalone manifest (cargo feature-resolver panic with dev-deps-only on a
   workspace-inheriting dep graph); purely a sandbox-manifest shape issue,
   no shared-tree relevance.
3. cgap-arrow's riscv rlib materializes under target/release/build/
   cgap-arrow/<hash>/out/ (build-script out dir), not deps/ — noted for the
   next auditor's nm step.
