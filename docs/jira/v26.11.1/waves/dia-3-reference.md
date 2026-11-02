# DIA-3 Reference Receipt

- **Lane**: DIA-3-reference (Diataxis reference tier), repo /Users/sac/cgap
  (canonical checkout), 2026-11-01.
- **Lease**: `docs/diataxis/reference/` (new files only) +
  `docs/jira/v26.11.1/waves/dia-3-reference.md` (this receipt).
- **Files written**:
  - `docs/diataxis/reference/crate-index.md` (22 crates + 2 ontology
    packs; live crates.io curl, UA `cgap-publish-receipt`)
  - `docs/diataxis/reference/sovereign-c-abi.md`
  - `docs/diataxis/reference/authority-pins.md`
  - `docs/diataxis/reference/error-codes.md`
  - `docs/diataxis/reference/bench-surface.md`
- **Commands + exits**:
  - `ls crates docs/diataxis; ls ~/ggen-marketplace/packs/` — exit 0
    (scout: no docs/reference pack found; step 1 result: none to follow)
  - grep sweeps over `crates/cgap-ffi/src/sovereign.rs`,
    `include/cgap.h`, `crates/cgap-core/src/frame.rs`,
    `crates/cgap-bn3/src/sigil.rs`, `crates/cgap-bn3/src/arena.rs`,
    `crates/cgap-bont/src/primitives.rs`, `crates/bench/Cargo.toml` — exit 0/1 mixed
  - `for c in ...; curl -s -A cgap-publish-receipt crates.io/api/v1/crates/$c` —
    exit 0, 25 lookups (22 crates + 3 name-collision re-checks)
  - `awk 'length > 100'` sweep — exit 0 (findings below)
  - `python3` cite-fix pass on authority-pins.md — exit 0
- **Verification**: every symbol/constant/line-number cited from a file
  read this session; crates.io versions are live-curl receipts, not
  recalled. `crates-and-apis.md` and `api-surface.md` predate this lane
  and were left untouched.
- **Divergences**:
  - Marketplace scout: no `docs/reference` ggen pack exists in
    `~/ggen-marketplace/packs/` — proceeded without a pack.
  - crates.io name collisions: `bench` (1.1.0), `audit` (0.8.0), and
    `cli` (0.0.1) are unrelated crates; the repo's packages are
    `bench` (local-only), `cgap-audit` (published 26.11.1), `cgap-cli`
    (unpublished). Re-checked `cgap-audit`/`cgap-cli`/`cgap-integration`
    by exact package name.
  - Markdown lines >100 chars occur inside table rows in the new files
    (entries with long signatures/receipt paths); the pre-existing
    `crates-and-apis.md` violates the same way — tables were kept
    intact rather than broken.
  - Four entrypoint body line numbers were taken from the grep of
    `#[no_mangle]` definitions; signature fn-pointer lines (168/181/
    192/201) witnessed directly.
