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
- `cargo test -p cgap-crypto` ×2: **BLOCKED — foreign landed defect**.
  Evidence chain (this session): (1) compile-freeze windows (kem.rs E0425
  unclosed delimiter, dbg5 `sha3 is private` E0603) — cleared by their
  owner in-SLA; (2) one flaky AE8 court death (lease.rs:430, E0460 stale
  rmeta race under concurrent rebuilds) — cleared by a coherent
  `cargo clean -p cgap-blake3 -p cgap-crypto` + rebuild (my transition on
  the shared target, disclosed); after which the lib suite is 25/25 green
  and every suite green EXCEPT `mlkem_sanity::roundtrip_sanity`
  (tests/mlkem_sanity.rs:16, ss1≠ss2). `src/kem.rs` (mtime 22:31) and
  `tests/mlkem_sanity.rs` (22:15) have been idle >1h while their lane
  iterated KAT-flip courts (kat_flip_demo.rs 23:01, green) — the ML-KEM
  FO roundtrip in another lane's landed kem.rs is broken, repro
  `cargo test -p cgap-crypto --test mlkem_sanity` → exit 101, deterministic.
  I own no cgap-crypto file; the fix belongs to that lane. My diff touches
  no cgap-crypto file.

## Compile-freeze disclosures (same-checkout fan-out)

- cgap-crypto kem.rs (E0425/unclosed delimiter) and later dbg5 sha3
  visibility — other lanes' in-flight edits; I never touched cgap-crypto.
- cgap-aoq Cargo.toml duplicated `[dev-dependencies]` — other lane; I
  never touched it; recovered in-SLA (~3 min).
- Shared-target transition: I ran `cargo clean -p cgap-blake3
  -p cgap-crypto` (148.5 MiB) to clear the AE8 stale-rmeta race; lanes
  mid-run against those artifacts should rebuild.
- My only cross-lane file edits: `aoq/kind_enums.rs` (std→core::error,
  2 lines) and `sovereign.rs` walk-root fix — both disclosed above;
  the walk-root fix is one word in a file adjacent to my owned seam.

## Standing

- cgap-ffi sandbox-court E0583 fix + walk-root fix: **ALIVE** — commands +
  exits cited above (BEFORE 4 failed / FINAL 58 passed, exit 0, this
  session, exact subject).
- cgap-crypto ×2 green: **BLOCKED** — foreign landed defect in
  `src/kem.rs` ML-KEM FO roundtrip (`mlkem_sanity::roundtrip_sanity`,
  deterministic, repro cited); crypto-lane owner required. All other
  cgap-crypto suites green after coherent rebuild (lib 25/25; ct_scan
  sandbox-add-token anti-vacuity kills among them).
