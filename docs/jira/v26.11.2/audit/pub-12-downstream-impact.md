# pub-12-downstream-impact

PUB-12 downstream impact audit (2026-11-02, read-only; fresh reads
of every consumer manifest; greps filtered of _build/deps/target).

## W0 Headline

cgapprd-08b's "zero cgap deps" verdict holds for all four Elixir
repos, extended to lsp-max at the dependency level: no Cargo.toml
or Cargo.lock in any of the five trees names cgap. The only real
coupling is NON-dependency, inside lsp-max: (1) a byte-exact
embedded copy of cgap's countermeasures residual grid (sha256
verified equal this audit), (2) the shared
`.cgap/receipts/branchless/` path convention.

## W1 Per-consumer impact table

| repo | pins | cgap dep? | impact |
|---|---|---|---|
| xaas | ash_a2a git ref 86214551 (mix.exs:116-120, override); ash_graphlaw/-surface/-affidavit path deps (:131-133) | NONE | none — greps + lock: 0 cgap hits |
| beam4pm | ash_a2a hex cap ~>26.9.30 OR git ref 99280b85 (mix.exs:133-148); graphlaw_ex test path (:114) | NONE | none |
| smalt | no ash_a2a dep (pin-mirror comment mix.exs:176); graphlaw_ex dev/test path (:112) | NONE | none |
| ash_a2a | ash_graphlaw path dep (mix.exs:350) | NONE | none — lib/native/docs: 0 cgap hits |
| lsp-max | path "../.." only in branchless-lsp (Cargo.toml:16); NO cgap path/git dep | NO (convention-coupled) | none breaking; 2 soft heads-ups (W3) |

## W2 Change-by-change verdicts

1. cgap-git 15 additive modules (c0ebd750): exactly 15 new files,
   nothing removed; zero external cgap-git consumers (five-repo
   Cargo.lock sweep clean) — no break surface. The A2A-named
   modules have no Elixir referents; the 08b "bidirectionally
   absent" edge remains absent.
2. cgap-parquet: the real session change made the previously
   DECORATIVE `std` feature load-bearing (lib.rs:15-18 — without
   it the std Error impl failed E0433). default=[] / std=[]
   unchanged; default no_std builds unchanged. Erratum-level for
   future consumers only.
3. cgap-core license-file: metadata-only, landed PRE-session
   (d03a6b4d, Oct 31). Packaging-only fix. Heads-up: P28-SIL is a
   sovereign license instrument, not permissive — future
   consumers inherit it.
4. branchless publish=false: moot externally (no registry or path
   consumer). Same-day flag state: court/verification/bench also
   publish=false; audit/ffi/bn3/core/storage FLIPPED true
   2026-11-01 (P3-2b). No consumer path-deps into cgap anywhere
   (lsp-max's one cross-repo path dep points at lsp-max itself).

## W3 Heads-up errata

- lsp-max H1: branchless-lsp embeds cgap's countermeasures/
  residuals.toml via include_str! (attributes.rs:284); copy
  byte-exact TODAY. If cgap edits the grid, lsp-max's seed
  silently diverges — the "source of truth: cgap" claim has NO
  sync mechanism. Erratum: sha-pin check or copy court.
- lsp-max H2: `.cgap/receipts/branchless/` (receipts.rs:195-197,
  gate-branchless-lsp.py:85) is a NAME-space convention implying
  cgap ownership of a path cgap does not read/write; rename or
  document as lsp-max-local.
- xaas/beam4pm/smalt/ash_a2a: no errata — sibling pins untouched.

## W4 Verification commands (all run)

grep -i cgap per mix.exs/mix.lock: 0 hits. Per-repo cgap greps
minus build dirs: only lsp-max/branchless-lsp + docs/scripts.
shasum residuals.toml both sides: identical. git log/show:
c0ebd750, 194010b6, d03a6b4d; parquet Cargo.toml history.

## See Also

audit/cgapprd-08b-crossrepo-referents.md ·
audit/pub-08-version-unification.md
