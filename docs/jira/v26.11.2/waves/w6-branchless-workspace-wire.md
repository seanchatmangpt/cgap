# W6 Lane W1 — Branchless Workspace Wire

Date: 2026-11-02. Branch: main. Repo: /Users/sac/cgap (canonical checkout).

## Diff summary

| File | Change |
|---|---|
| `Cargo.toml` | Added `[workspace.dependencies]` section (none existed) with `cgap-branchless = { path = "crates/cgap-branchless", default-features = false }` |
| `crates/cgap-gate/Cargo.toml` | Path dep normalized to `cgap-branchless = { workspace = true }` |
| `crates/cgap-branchless/Cargo.toml` | No edit needed — B1 scaffold already conforms (workspace-relative license-file, explicit `publish = false`). |
| `Cargo.lock` | Resolution updated: cgap-branchless as workspace dep (7 insertions). |
| `crates/cgap-p28/Cargo.toml` | NOT edited — B8 verdict stands: no consumption site for branchless in cgap-p28; an unused dep is vacuous manifest debt. Deferral recorded here per dispatch. |

## Commands + exits

| Command | Exit |
|---|---|
| `make gate0` (before edits) | 2 |
| `CARGO_TARGET_DIR=/tmp/w1-target cargo check -p cgap-branchless -p cgap-gate` | 0 |
| `cargo test -p cgap-branchless` | 0 — 26 passed; 0 failed (unittests; census test target 0 tests, declared) |
| `cargo test -p cgap-gate` | 0 — 12 + 3 passed; 0 failed |
| `cargo check --workspace --target riscv32imc-unknown-none-elf` (CARGO_TARGET_DIR=/tmp/w1-target) | 101 — mixed; failing crates are typed below |
| `cargo check -p cgap-branchless -p cgap-gate --target riscv32imc-unknown-none-elf` | 0 — gate + branchless compile clean on target |
| `make gate0` (after edits) | 2 |

## riscv32imc typed failure rows (D5-backlog, std-dependent)

| Crate | Status |
|---|---|
| cgap-branchless, cgap-gate | compile clean on target (exit 0, targeted check) |
| cgap-court, cgap-crypto, cgap-p28, cgap-ffi, cgap-beam | UNVERIFIED individually (workspace check aborted on backlog crates) |
| cgap-git, cgap-parquet, cgap-core, cgap-bosym, cgap-storage, cgap-bont, cgap-bn3, cgap-integration | BLOCKED — std-dependent (`Vec`/std paths), expected D5-backlog; not caused by this lane |

## Divergences

1. `make gate0` fails identically before (exit 2) and after (exit 2) edits:
   `[FM-PACK-008]` pack `cgap-pack` content-hash mismatch — `ggen.lock`
   pins `blake3:e085b8d1…` but the pack on disk hashes to
   `blake3:22ba9f10…`. PRE-EXISTING, not introduced by W1. Remediation
   (delete ggen.lock to re-lock) touches generated artifacts outside
   this lane's ownership — left to coordinator. Byte-stable replay is
   therefore UNVERIFIED this session (sync aborts at validation, both
   runs identical, so at least it is deterministic).
2. `cargo check -p cgap-gate` emits `unused_dependencies` warning for
   cgap-branchless — consistent with B7's note that no penstock body
   site swaps landed yet; the dep edge is the deliverable, not the body
   use.

## p28 deferral rationale

No symbol from cgap-branchless is referenced anywhere in
`crates/cgap-p28/src` (B8's finding, confirmed unchanged). Adding the
dep would produce an unused-dependency warning and zero capability —
vacuous manifest debt. Deferred until a real consumption site lands.

## Standing

ALIVE for the wiring change itself (all commands above run this session
with exits cited). gate0 byte-stable replay: BLOCKED (pre-existing
FM-PACK-008, divergence row 1).
