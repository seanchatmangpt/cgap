# pub-16 — excluded surface audit (publish=false + standalone)

Audit date: 2026-11-02. Scope: the publish=false set (bench, cgap-beam,
cgap-branchless, cgap-court) plus standalone (cli, csct,
cgap-verification). Question: is each exclusion DELIBERATE (receipt /
manifest / CLAUDE.md citation), does the code still compile, and is any
exclusion reason STALE (first-publish candidate post-freeze, like
cgap-integration)?

## 1. Method (all commands this session, exit 0 unless noted)

- Manifest read: crates/{bench,cgap-beam,cgap-branchless,cgap-court,
  cli,csct,cgap-verification}/Cargo.toml — all seven carry an explicit
  `publish = false` on disk (cli line 9, csct line 5, verification
  line 8, beam line 8, bench line 9, branchless line 7, court line 7).
- Root Cargo.toml members list cites cli as deliberately non-member
  ("standalone ggen six-pack generated crate (its own [workspace])").
- Receipt/CLAUDE.md citations: CLAUDE.md:39-46 (publish law, re-derived
  2026-11-02); docs/jira/v26.11.2/08-registry-sweep.md §4 guarded
  table; waves/v2-07-census.md:88-101 (live API sweep rows);
  waves/publish-halt.md (operator freeze + readiness matrix);
  waves/w6-branchless-workspace-wire.md (branchless wire receipt).
- Compile: `cargo check -p bench -p cgap-beam -p cgap-branchless
  -p cgap-court` (shared repo target) exit 0;
  `CARGO_TARGET_DIR=/tmp/pub16-tgt-cli cargo check` in crates/cli
  exit 0; same for crates/csct (/tmp/pub16-tgt-csct) exit 0;
  same for crates/cgap-verification (/tmp/pub16-tgt-verify) exit 0.
- Registry: live crates.io API (UA cgap-pub16-audit, 2026-11-02):
  cgap-branchless / cgap-court / cgap-beam / cgap-cli / csct /
  cgap-verification all HTTP 404. bench: registry `bench` 1.1.0 (2021)
  is a squatter, not ours (census:90, v2-07 f3-20 re-verify row).

## 2. Per-excluded-crate table

| crate | exclusion reason (citation) | valid? | compile | candidate? |
|---|---|---|---|---|
| bench | name collision: `bench` 1.1.0 (2021) squatter (T:9; census:90) | YES | OK | NO |
| cgap-beam | guarded membrane; 404 row (T:8; census:91; S4) | YES | OK | MAYBE |
| cgap-branchless | on-disk flag, reason UNDOCUMENTED (T:7; not in S4) | STALE | OK | YES |
| cgap-court | [[test]]-only, no lib surface (T:7-13; S4) | YES | OK | NO |
| cgap-cli | standalone ggen crate, non-member (root T; S4 r23) | YES | OK | NO |
| csct | standalone crate, own [workspace] (T:5+comment) | YES | OK | MAYBE |
| cgap-verification | on disk, non-member (S4; CLAUDE.md:62) | PARTIAL | OK | MAYBE |

Key: T = crates/<crate>/Cargo.toml (line of the publish=false flag);
S4 = docs/jira/v26.11.2/08-registry-sweep.md section 4; census =
waves/v2-07-census.md; root T = /Users/sac/cgap/Cargo.toml.

Detail (wrapped per crate):

- bench: registry name collision is PERMANENT — `bench` 1.1.0 (2021)
  is a squatter, not ours, re-confirmed live in the v2-07 f3-20
  re-verify row (census:90) and by CLAUDE.md:39-40. Exclusion valid.
- cgap-beam: guarded membrane, registry 404 (census:91, S4, live
  404 this session). Publishable in principle (cdylib NIF) but deps
  are path+version pairs needing registry rewrite; low priority.
- cgap-branchless: the ONLY stale row. publish=false is on disk
  (Cargo.toml:7) but no receipt states why; S4 omits it entirely and
  the sweep compiler emitted `MISSING no-ledger-row`
  (registry-sweep-table.json:118, sole missing subject, W-SWEEP
  exit 1). No structural blocker: zero-external-dep no_std kernel,
  consumed by cgap-gate as a workspace dep
  (w6-branchless-workspace-wire.md:9-11), riscv32imc target-clean
  (same receipt:24), registry 404 (live). Same shape as
  cgap-integration pre-flip (S4: UNPUBLISHED 404, blocker CLEARED,
  [TARGET-v26.11.2]). FIRST-PUBLISH CANDIDATE post-freeze,
  conditional on: (a) a written reason receipt, (b) an S4 row,
  (c) closing the no-ledger-row sweep defect.
- cgap-court: structural reason still true — manifest is
  [[test]]-only (court_verdict), no [lib] target. Nothing to
  publish by design. Valid.
- cgap-cli: standalone ggen six-pack generated crate, non-member by
  design (root Cargo.toml NOTE; S4 row 23; v2-10-cli-state.md:1;
  CLAUDE.md:50-54). Publishing would freeze generated output. Valid.
- csct: standalone crate (own [workspace], Cargo.toml:5 + inline
  comment citing the cli pattern and lane CSCT-L5). Exclusion
  deliberate, but it has no S4 row — add one if it stays excluded.
  Minor: cargo check warns unused dep cgap-core (exit 0).
- cgap-verification: on disk, not in the 22-member cargo metadata
  list (S4; CLAUDE.md:62). Its own [workspace] pin is annotated
  "Remove this table when root-membership lands" — membership never
  landed, so the comment is stale-by-deferral (documented, not
  accidental). Decide membership first, then publish law.

## 3. Findings

F1. All seven exclusions are DELIBERATE (explicit publish=false on
disk + at least one receipt/manifest/CLAUDE.md citation each). None is
accidentally ignored. Verified by manifest read + citation table above.

F2. All seven compile clean, exit 0, this session: the four members
via one `cargo check -p ...` invocation (shared repo target), the
three standalone crates via individually leased /tmp target dirs.

F3. STALE FLAG — cgap-branchless: its publish=false is on disk but
UNDOCUMENTED. It is absent from the 08-registry-sweep.md §4 guarded
table, and the sweep compiler emitted it as `MISSING no-ledger-row`
(registry-sweep-table.json:118 — the sole missing subject in the
W-SWEEP exit-1 run). Unlike bench (name collision), beam (guarded
membrane), and court (no lib surface), branchless has no structural
blocker: it is a zero-external-dep no_std kernel, consumed by
cgap-gate (workspace dep, w6-branchless-workspace-wire.md:9-11),
riscv32imc target-clean, and registry-absent (404, live check). It is
the same shape as cgap-integration pre-flip (UNPUBLISHED 404,
blocker CLEARED, publish-eligible [TARGET-v26.11.2] per 08-sweep §4).
Verdict: first-publish candidate post-freeze, CONDITIONAL on (a) a
written reason receipt, (b) a §4 guarded-table row, (c) closure of the
no-ledger-row sweep defect.

F4. Secondary gaps (do not block, disclose only):
- csct and cgap-verification have no §4 guarded-table row at all;
  only branchless is MISSING in the sweep JSON. Add rows if they stay
  excluded past the freeze.
- cgap-verification's own [workspace] pin is annotated "Remove this
  table when root-membership lands" — membership never landed, so the
  pin comment is itself stale-by-deferral (documented deferral, not an
  accidental exclusion).
- csct: cargo check warns `unused_dependencies` on cgap-core
  (help: consider removing) — manifest debt, disclosed, exit 0.
- member check (bench/beam/branchless/court) also surfaced an unused
  cgap-storage dep warning inside one member; exit 0, cosmetic.
- Live 404s this session confirm the registry-absent rows still hold
  (branchless/court/beam/cli/csct/verification); bench collision row
  unchanged (squatter 1.1.0).

F5. Publish law anchor: operator freeze still in force
(waves/publish-halt.md, standing FROZEN-BY-OPERATOR). No publish act
is authorized for any row in this audit; the branchless candidacy in
F3 is POST-FREEZE only, sequenced per 08-sweep §3 (one attempt per
crate, dependency-ordered).

## 4. Verdict summary

- DELIBERATE exclusions: 7/7 (none accidental).
- Compile-clean: 7/7 (exit 0, this session).
- Stale exclusion reasons: 1 (cgap-branchless — undocumented reason,
  no §4 row, sweep MISSING no-ledger-row).
- First-publish candidates post-freeze: cgap-branchless (primary,
  cgap-integration-shaped), cgap-verification + csct (secondary,
  need membership/publish-law decisions first).
- Permanent non-candidates: bench (name collision), cgap-court (no
  lib surface by design), cgap-cli (generated artifact).

## JSON footer

```json
{
  "lane": "pub-16",
  "files_written": [
    "docs/jira/v26.11.2/audit/pub-16-excluded-surface.md"
  ],
  "commands": [
        "manifest reads x7 (crates/{bench,cgap-beam,cgap-branchless,",
    "  cgap-court,cli,csct,cgap-verification}/Cargo.toml)",
    "cargo check -p bench -p cgap-beam -p cgap-branchless -p cgap-court (exit 0)",
    "CARGO_TARGET_DIR=/tmp/pub16-tgt-cli cargo check in crates/cli (exit 0)",
    "CARGO_TARGET_DIR=/tmp/pub16-tgt-csct cargo check in crates/csct (exit 0)",
    "CARGO_TARGET_DIR=/tmp/pub16-tgt-verify cargo check in crates/cgap-verification (exit 0)",
    "crates.io API x6, UA cgap-pub16-audit (404 x6)"
  ],
  "courts_run": [],
  "standing": "PASS-WITH-1-STALE-FLAG",
  "divergences": [
        "cgap-branchless publish=false reason undocumented; absent",
    "  from 08-sweep S4 table; sweep row MISSING no-ledger-row",
        "csct unused dep warning (cgap-core); one member unused",
    "  cgap-storage warning; both cosmetic, exit 0",
    "verification [workspace] pin comment stale-by-deferral (membership never landed)"
  ]
}
```
