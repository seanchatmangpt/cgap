# AF-8 — sandbox-court copier recursion (E0583 fix)

Lane: AF-8. Date: 2026-11-03. No git commands run.

## Subject

- `crates/cgap-ffi/src/sovereign.rs` — T1 sandbox fixture copy loop
  (`xip_t1_riscv_elf_size_court`) and the status-ledger walk roots
  (`status_court_reachability_ledger`)
- `crates/cgap-ffi/src/xip_tables.rs` — r2 `stage_sandbox` copy loop
  (`r2_production_link_court`)
- `crates/cgap-ffi/src/aoq/kind_enums.rs` — two `std::error::Error` impls
  (disclosed minimal fix, outside nominal ownership; owner = aoq lane)

## Root cause (re-derivation corrected AB-15's single-cause attribution)

AB-15 attributed all 4 `--lib` failures to the root-level `src/*.rs` copy
loop breaking on the `src/aoq/` landing. Re-derived per-test this session:

1. `xip_t1_riscv_elf_size_court`, `r2_production_link_court` — E0583 as
   claimed: both /tmp sandboxes copied only root-level `src/*.rs`, so
   `pub mod aoq;` (`src/lib.rs:357`) died with `file not found for module
   'aoq'` (witnessed in a single-test run). Fix: replaced both loops with
   `copy_src_recursive` (read_dir recursion, `.rs` filter, relative paths
   preserved via per-level `file_name()` joins, `create_dir_all` per dir).
   Second-order: with `aoq/` landing, the no_std sandbox then failed on
   `impl std::error::Error` in `aoq/kind_enums.rs:37,79` (E0433 `std` on
   riscv32) — flipped to `core::error::Error` (stable 1.81; nightly
   toolchain).
2. `status_court_reachability_ledger` — NOT E0583 (fails identically
   without the sandbox): walk roots `["crates", "ontologies"]` name a
   directory that no longer exists; the member dir is `ontology-packs/`,
   so the Wave-6 pin (`ontology-packs/cloud/src/lib.rs` must be in the
   walked set) was the failing witness. One-word root fix, disclosed.
3. `wave9_a07_zero_string_gate_over_all_built_crates` — NOT E0583: the
   A07 gate refuses (exit 2, `GATE-REFUSED: execution-crate-no-artifacts`)
   when doc-execution crates have zero scanned debug artifacts — an
   environment precondition; satisfied by `cargo build --workspace`.

## Gates (commands + exits, this session)

- BEFORE: `cargo test -p cgap-ffi --lib` → **54 passed; 4 failed**, exit 101
  (the exact AB-15 four).
- Compile evidence: `cargo test -p cgap-ffi --lib xip_t1` →
  `error[E0583]: file not found for module 'aoq'` (src/lib.rs:357).
- After recursion only: 53 passed; 5 failed — the four surfaced their
  true per-test causes.
- FINAL: `cargo test -p cgap-ffi --lib -- --test-threads=1` → **rc=0,
  58 passed; 0 failed** (102.67s), after `cargo build --workspace` (rc=0).
  4 failures → 0. ACHIEVED.
- Anti-vacuity: all four courts carry executed-in-court mutation arms
  (T1 links the section-absent xip.ld mutant; r2 links its production
  mutant; the ledger executes the crates-only negative control; the A07
  gate carries its own self-check) — green in the final run, so the
  mutation arms fired and passed, non-vacuous. The cgap-crypto
  ct_scan_court sandbox-add-token kills are exercised by the
  cgap-crypto run below.
- `cargo test -p cgap-crypto` ×2: BLOCKED at this receipt's writing —
  another lane's in-flight cgap-crypto edit (`dbg5` test:
  `module 'sha3' is private`, E0603; earlier: kem.rs E0425/unclosed
  delimiter, then recovered in-SLA). cgap-crypto compile poll running in
  background; will be updated to final state when it clears. My diff
  touches no cgap-crypto file.

## Compile-freeze disclosures (same-checkout fan-out)

- cgap-crypto kem.rs (E0425/unclosed delimiter) and later dbg5 sha3
  visibility — other lanes' in-flight edits; I never touched cgap-crypto.
- cgap-aoq Cargo.toml duplicated `[dev-dependencies]` — other lane; I
  never touched it; recovered in-SLA (~3 min).
- My only cross-lane file edits: `aoq/kind_enums.rs` (std→core::error,
  2 lines) and `sovereign.rs` walk-root fix — both disclosed above;
  the walk-root fix is one word in a file adjacent to my owned seam.
