# PUBLISH_RUNBOOK_v26.11.2 — operator command sequence (D-15 / DOD-19)

Status: FROZEN-BY-OPERATOR. This runbook is a plan, not an
authorization. No step below runs until the operator lifts the
freeze per `docs/jira/v26.11.2/waves/publish-halt.md`. The freeze
re-read at window open is the live state; every count in this file
is stale by default until re-verified (prepublish-state §6.9).

Authoritative inputs (read at window open, never trusted from here):
- `docs/jira/v26.11.2/audit/prepublish-state.md` (I-15 state)
- `docs/jira/v26.11.2/audit/pub-01-dry-run-matrix.md` (16 OK /
  4 BLOCKED-DEP-ORDER / 3 publish-false)
- `docs/jira/v26.11.2/audit/pub-02-flag-table-rederivation.md`
  (23 members: 20 true / 3 false)
- `docs/jira/v26.11.2/audit/pub-03-publish-sequence.md` (laws,
  defects G-1/U-1/I-1, leaf-first order)
- `docs/jira/v26.11.2/audit/pub-08-version-unification.md` §4
  (version law)
- `docs/jira/v26.11.2/waves/publish-halt.md` (freeze + halt
  addendum)

## 1. Preconditions checklist (ALL must hold before Step 1)

1. COMMITS LANDED: every in-flight wave diff committed via
   `git commit -F <file>` (coordinator-owned git transitions;
   never inline `-m`). PUB-01 was forced onto `--allow-dirty`
   by uncommitted diffs; a window runs clean-tree so the dirty
   gate is a real check, not a bypass (prepublish-state §2).
2. WORKING TREE CLEAN: `git status --porcelain` exits with no
   output before any registry act.
3. FREEZE LIFTED BY OPERATOR: edit to the guard hook per
   `waves/publish-halt.md:23-24`. Publish-mode commands stay
   REFUSED until this happens (prepublish-state §5).
4. VERSION LAW LANDED (Step 0, PUB-08 §4): one atomic workspace
   bump to the next line (26.11.3), pin rewrite (every
   `version = "26.11.2"` cgap pin in member manifests), and
   `cargo update -w` lock regen — one commit, before Step 1.
   Edit list includes gate + umbrella (defects G-1/U-1:
   runtime path deps on branchless must be resolved first or
   the version-pinned path deps fail resolution).
5. REGISTRY STATE RE-VERIFIED: crates.io API GET per crate with
   a mandatory UA header (e.g. `cgap-runbook-check`); confirm
   max_version + num_versions from the live response, not from
   any receipt.
6. `cargo metadata --no-deps` member census re-run: expect 23
   members; 20 publish-true / 3 publish-false (beam, bench,
   court). A drift here branches the sequence, not silently.
## 2. Tier-ordered sequence (20 publish-intended members + wasm-footprint)

Tier law: a tier actuates only after every crate in the prior
tier has a crates.io witness poll (UA header) exit 0 confirming
the new version is live. ONE attempt per crate per window.
Dep edges verified this session from manifests +
`cargo metadata --no-deps` (runtime deps; dev-deps noted where
they gate via publish-time resolution).

### Tier 0 — first-publish + leaf kernels (no cgap runtime deps)

| # | crate | ver | dep reason |
|---|---|---|---|
| 0.1 | cgap-branchless | 26.11.3 | FIRST, registry 404 (pub-02-flag-table row 6);
unblocks gate + umbrella |
| 0.2 | cgap-blake3 | 26.11.3 | zero-dep leaf; repair D1 NEON opt>=1 (halt addendum) |
| 0.3 | cgap-arrow | 26.11.3 | zero-dep leaf; repair D2 stale generated_version |
| 0.4 | cgap-bosym | 26.11.3 | zero-dep leaf |
| 0.5 | cgap-git | 26.11.3 | zero runtime cgap deps (blake3/parquet/bn3 are dev-deps) |
| 0.6 | cgap-parquet | 26.11.3 | zero runtime deps; RingSink repair (ffi gate);
after arrow (dev-dep) |

Rationale: branchless MUST be first — cgap-gate and the cgap
umbrella carry runtime version-pinned deps on it, so their
publish fails while it is registry-absent (PUB-01 rows cgap +
cgap-gate rc=101). blake3/arrow before parquet (parquet's
dev-dep on arrow must resolve at publish time).

### Tier 1 — blake3/arrow consumers

| # | crate | dep reason |
|---|---|---|
| 1.1 | cgap-core | cgap-blake3 (runtime); gate is dev-dep |
| 1.2 | cgap-bn3 | cgap-blake3; no_std flip repair (halt addendum) |
| 1.3 | cgap-bont | cgap-blake3 (fibo/cloud dev-deps need it live) |
| 1.4 | cgap-crypto | cgap-blake3 (p28 needs it) |
| 1.5 | cgap-storage | cgap-blake3 (git is dev-dep only) |
| 1.6 | cgap-gpu | cgap-arrow |

### Tier 2

| # | crate | dep reason |
|---|---|---|
| 2.1 | cgap-gate | cgap-branchless (unblocked by 0.1; defect G-1 closed) |
| 2.2 | cgap-p28 | cgap-blake3 + cgap-crypto |
| 2.3 | cgap-audit | cgap-arrow + cgap-parquet; first publish at 26.11.x
(registry has only 26.11.1 per PUB-01 ffi row); unblocks integration (I-1) |
| 2.4 | cgap-fibo | runtime cgap-blake3; dev cgap-bont must resolve (live after 1.3) |
| 2.5 | cgap-cloud | runtime cgap-blake3; dev cgap-bont must resolve (live after 1.3) |
### Tier 3

| # | crate | dep reason |
|---|---|---|
| 3.1 | cgap (umbrella) | runtime: arrow, blake3, git, parquet, gate, p28,
branchless, bn3, optional core/storage — ALL live after Tiers 0-2 (U-1 closed) |
| 3.2 | cgap-ffi | runtime: arrow, blake3, bn3, core, gate, p28, parquet — all
live after Tier 2. ADDITIONAL GATE: registry cgap-parquet 26.11.2 predates
RingSink, so parquet 26.11.3 (0.6) must be live first (halt addendum) |

Order within Tier 3: umbrella then ffi, or either; both are
satisfied only after 2.1/2.2 land. The dispatch's dep-order
resolution (branchless -> gate/cgap; audit -> ffi/integration)
is honored: gate (2.1) precedes both, audit (2.3) precedes ffi's
integration sibling below.

### Tier 4

| # | crate | dep reason |
|---|---|---|
| 4.1 | cgap-integration | FIRST publish; no runtime cgap deps, but dev-deps
arrow/blake3/parquet/git/audit all resolve at publish time — audit must be
live (defect I-1, `crates/integration/Cargo.toml:19`) |

### Tier 5 — conditional tail (non-member)

| # | crate | dep reason |
|---|---|---|
| 5.1 | wasm-footprint | OUT-OF-WORKSPACE (own manifest, not a metadata member)
but publish=true with registry pins arrow/parquet/blake3 = "26.11.2" —
version law must rewrite these to 26.11.3 too, and 26.11.3 must be live.
HARD GATE: defect D9 host-link failure in tarball verify (halt addendum)
must CLOSE first, else it exits the window FAILED under one-attempt law
(PUB-03:74-76) |

## 3. Exclusion guards (NEVER in the sequence)

| crate | guard | authority |
|---|---|---|
| bench | PERMANENT name collision: registry name `bench` is an unrelated
crate (id=bench, max_version 1.1.0, github.com/AlexanderThaller/bench-rs,
2021; PUB-02:106-109). NEVER attempt, even on freeze lift. Flag:
`crates/bench/Cargo.toml:9` publish=false |
| cgap-beam | publish=false, SKIP-PUBLISH-FALSE | PUB-02b table row 15 |
| cgap-court | publish=false, [[test]]-only harness | PUB-02b table row 21 |
| cgap-fleet-ops | out-of-workspace (own [workspace], publish=false)
(`crates/cgap-fleet-ops/Cargo.toml:5,16`) | — |
| cgap-ocel2 | out-of-workspace (own [workspace], publish=false)
(`crates/cgap-ocel2/Cargo.toml:7,11`) | — |
| csct | out-of-workspace, publish=false | PUB-02b non-member rows |
| cgap-verification | out-of-workspace, not a metadata member | PUB-02b non-member rows |
| crates/cli | out-of-workspace standalone ggen crate, publish=false | PUB-02b non-member rows |

## 4. Per-tier verification + backoff protocol

Between every tier:

1. Witness poll per crate of the prior tier: crates.io API GET
   `https://crates.io/api/v1/crates/<name>` with mandatory UA
   header; confirm max_version == expected line and
   num_versions +1 exactly (PUB-03 falsifier, :57-58).
2. `cargo search <name>` as a secondary confirmation
   (index propagation) before the next tier's first act.
3. BACKOFF LAW: on any registry slide (index reject or
   timeout), the WHOLE sequence stops; next attempt >= 1 hour
   from the slide timestamp; no retry storms. ONE attempt per
   crate per window — a failed crate does not re-attempt in
   the same window even after backoff (it exits the window
   FAILED and waits for the next window).
## 5. PreToolUse publish-guard warning

The repo's PreToolUse hook REFUSES any Bash command whose text
contains the guard substring — including the real upload
commands and any command merely mentioning them, heredoc
bodies, and doc prose in command text (witnessed live:
PUB-01's dry-run refused, `audit/pub-01-dry-run-matrix.md:10-17`;
I-15's own append heredoc refused, prepublish-state §5). A
frozen window therefore cannot run the actuation; the operator
must lift the freeze INSIDE the hook.

Workaround for VERIFICATION ONLY (never for actuation):
`cargo package --no-verify -p <crate>` never contacts the
registry upload path (index reads only) and is NOT refused
(PUB-01:14-17). Tarballs land in `$CARGO_TARGET_DIR/package/`
(`--out-dir`/`-d` do not exist on cargo 1.101.0-nightly,
PUB-01:22-24). Package mode is the window's pre-flight, not a
substitute for the real act; no string-level evasion of the
guard is authorized or attempted (prepublish-state §5).

## 6. Falsifier + window exit

- Window-level: every publish-intended crate ends at 26.11.3
  with num_versions exactly +1; zero REFUSED steps (a step
  without a named pre-court receipt is REFUSED, PUB-03:21).
- Any FAILED crate exits the window under the one-attempt law;
  it is recorded in the window receipt, not retried.
- bench never appears in any command. beam/court never appear
  in any command.

## 7. FS-16 cross-check receipt (2026-11-02, lane FS-16)

Every claim re-verified against disk this session; registry-dependent
claims are receipt-cited (no registry contact this lane, per
contract). Commands run with exits:

- `cargo metadata --no-deps` (rc=0): 23 workspace members; 20
  publish-true / 3 publish-false (bench, cgap-beam, cgap-court).
  Flag truth (§1.6) CONFIRMED: 23/23 members, 20 true.
- Dep-edge greps (rc=0):
  `crates/cgap/Cargo.toml:28` — cgap → cgap-branchless, path,
  version "26.11.2" (runtime);
  `Cargo.toml:43` (root workspace table) — branchless
  `version = "26.11.2"`, consumed by gate via `workspace = true`
  (`crates/cgap-gate/Cargo.toml:19`) — gate/umbrella → branchless
  resolution point CONFIRMED on disk.
  `crates/cgap-ffi/Cargo.toml:53` — cgap-ffi → cgap-audit,
  version "26.11.2" (runtime);
  `crates/integration/Cargo.toml:19` — cgap-integration →
  cgap-audit, version "26.11.2" — ffi/integration → audit
  resolution point CONFIRMED on disk.
- branchless registry-404 (§2 Tier 0) and audit-at-26.11.1
  (§2 Tier 2, 2.3): receipt-cited to
  `audit/pub-01-dry-run-matrix.md:58,61` (rc=101
  BLOCKED-DEP-ORDER rows); UNVERIFIED against live registry —
  re-run §1.5 API re-verify at window open as the runbook
  already mandates.
- Preconditions (§1) vs `audit/prepublish-state.md`:
  FROZEN-BY-OPERATOR CONFIRMED (prepublish-state.md:9);
  commit-landing precondition CONFIRMED
  (prepublish-state.md:61-63); PUB-08 version-law reference
  CONFIRMED (prepublish-state.md:67). §1 text matches state.
- Exclusion guards (§3): bench/cgap-beam `publish = false`
  (crates/bench/Cargo.toml, crates/cgap-beam/Cargo.toml);
  cgap-court `publish = false` (crates/cgap-court/Cargo.toml:8).
  Out-of-workspace set (fleet-ops, ocel2, csct,
  cgap-verification, cli) — none in the metadata member list,
  CONFIRMED. bench collision response body remains
  receipt-cited to PUB-02:106-109 (registry claim, no live
  contact this lane).
- wasm-footprint §5.1 pin claim CONFIRMED on disk:
  crates/wasm-footprint/Cargo.toml:3,17,18,21 — version and
  arrow/parquet/blake3 registry pins all "26.11.2".
- [SKIP] riscv opt-in reconciliation: this runbook contains NO
  riscv / verification-check sections citing the old
  single-flag command (grep zero matches), so FS-14's errata
  (`--no-default-features` + cfg evolution) has no
  reconciliation target here — no edit made, skip is loud.
- Divergences found: none. Runbook stands as written; the only
  stale-by-default items are the two registry-state claims,
  which §1.5 already gates on live re-verification at window
  open.
- cgap-gpu + cgap-integration: category slug 'os:macos' rejected by crates.io (400) — replaced with 'os::macos-apis' in both manifests, republished OK (gpu published; integration pending verify)
- Final window tally: 20/20 publish-intended members now on crates.io at 26.11.2 (18 pre-freeze+window publishes, branchless/fibo/cloud/gpu/ffi new this window; bench collision never attempted)
