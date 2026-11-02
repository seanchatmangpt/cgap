# Lane LSPA-6 — `just branchless` court (LSP tier-2 aarch64 wire, AE4-composed)

Date: 2026-11-02. Repo: /Users/sac/cgap (lane files only:
`scripts/verify_lsp_branchless.py` NEW, `justfile` one recipe after
`courts`, this receipt). lsp-max consumed read-only (server binary +
python client); no git commands; shared tree never mutated (mutants live
in /tmp sandbox files).

## Landed

- `/Users/sac/cgap/scripts/verify_lsp_branchless.py` — stdlib driver,
  exit ABI 0/1/2/3 mirrored from AE4
  (`scripts/verify_disassembly_invariants.py:56-59`): 0 clean / 1
  refused-flip failure or vacuous / 2 subject missing / 3 loud skip.
  Arms: (a) AE4 verbatim, exit inherited; (b) staleness guard —
  `strings <bin> | count ANDON_REFUSED` == 0 -> exit 3 naming
  `cargo build --release -p branchless-lsp`; (c) wire mutation court on
  aarch64; (d) typed markdown line per subject; (e) exit 0 only if all
  arms held.
- `justfile` — `branchless` recipe (justfile:55-56), single line after
  `courts`.

## Court runs (this session)

| command | exit | witness |
|---|---|---|
| `python3 scripts/verify_lsp_branchless.py` | 0 | flip + clean held, see below |
| `BRANCHLESS_LSP_BIN=/nonexistent python3 scripts/verify_lsp_branchless.py` | 3 | typed skip: `[SKIP] branchless-lsp binary missing: /nonexistent — rebuild with: cd /Users/sac/lsp-max && cargo build --release -p branchless-lsp` |
| `just branchless` | 0 | same CLEAN verdict line |

## Mutant-flip transcript (release binary, mtime Nov 2 13:44, staleness count 4)

Per-file sessions; per-URI publishDiagnostics drained; LAST publish taken
(socket-ordered tier-1 then tier-2, killing the G05 first-publish race).

| subject | tier-2 aarch64 verdict |
|---|---|
| clean select.rs (select.rs content, probe-convention #[no_mangle]) | n_refused=0 PASS |
| mutant_ae4 (M-AE4-1 if-injection into cmov_u64, pinned AE4 arm-C text) | n_refused=0 — ADMITTED (aarch64 -O csel-lowered AND cmov_u64/aarch64 is a seeded `repaired_hot` row in residuals.toml) |
| mutant_portable (M-LSPA6-1 = M-AE4-1 + black_box forward branch + `#[export_name = "lspa6_mutant_cmov_u64"]` defeating the seed) | n_refused=3 PASS (flip) |

ANDON_REFUSED transcript (mutant_portable, verbatim, x2 + 1 blocking row):
```
ANDON_REFUSED: fn `lspa6_mutant_cmov_u64` carries a forward conditional branch; annotate with #[allow_isel_residual] or make it branchless
ANDON_REFUSED: fn `lspa6_mutant_cmov_u64` carries a forward conditional branch; annotate with #[allow_isel_residual] or make it branchless
A blocking diagnostic was emitted without a corresponding ANDON push event.
```

Stability: two independent probe runs + full court + `just branchless`
gave clean=0 / ae4=0 / portable=3 identically (pre-fix runs gave
clean=31/0 — the race was publish-selection, fixed by drain-last-publish
per-URI).

## Divergences (dispatch vs reality — doc yields)

1. `audit/` path: dispatch cited `audit/lspu1-consume-plan.md`; actual
   `docs/jira/v26.11.2/audit/lspu1-consume-plan.md`.
2. **The pinned AE4 arm-C mutant does NOT flip on aarch64** (consume-plan
   move #3 premise): (i) the injected `if r > 0 { r } else { true_val }`
   lowers to `csel` at aarch64 -O (0 branch sites), and (ii) cmov_u64/
   aarch64 is a seeded `repaired_hot branch_count=0` row
   (lsp-max residual-grid lookup, attributes.rs:436-463). Even a genuine
   branch in cmov_u64/aarch64 classifies HintResidual, not refused.
   The court keeps the pinned arm as an informational ADMITTED row and
   the anti-vacuity mutation of record is M-LSPA6-1 (same if-injection +
   G05 BRANCHY black_box forward while + export_name defeating the seed
   lookup). The flip is real and reproducible (2 identical runs + the
   full court + `just branchless` = 4 witnesses).
   → ERRATA-candidate for lspu1-consume-plan.md move #3 (errata note, not
   a rewrite of the audit file — owner's call; recorded here first).
3. **Seed grid cannot match non-no_mangle probe symbols** (witnessed:
   un-annotated full select.rs text drew 31 refusals, all naming the
   v0-mangled `..._022checked_uniform_select`, defeating the
   skip_reference seed row). Subjects therefore carry the tier-2 probe
   convention `#[no_mangle]` per fn (same convention as G05 fixtures and
   rule_pack.rs test fixtures). Without it the clean arm is unprovable
   on this server build.
4. G05's `diagnostics_for_files` was found RACY for this use (run 1:
   clean n_refused=0 vs true verdict 31): the tier-1/tier-2 publish race
   means "drain one publish then read one" can return tier-1 empties.
   The lane's driver therefore runs per-file sessions and drains,
   taking the LAST publish (socket-ordered). The shared client file was
   NOT edited (not a lane file).
5. Staleness count is 4, not G05's 3 — count method (substring
   occurrences in `strings` output) differs from G05's `grep -c` line
   count; binary rebuilt Nov 2 13:44 (post-G05).

## Fences run (must-not-regress)

- AE4 gate (`scripts/verify_disassembly_invariants.py`) ran as arm (a)
  inside every court run: exit 0 each time (census + linked witness +
  M-AE4-1 + self-check + arm E binary mutant all held). Shared tree
  untouched; AE4 builds into a fresh /tmp scratch lease per run.
- Root workspace untouched; justfile edit is additive (one recipe);
  `just branchless` exit 0 witnesses the recipe parses.
- G05's shared client untouched — wire behavior exercised via its public
  API only.

## Standing

- Court ALIVE: commands + exits above, run this session.
- aarch64 clean-0-branch assertion: ALIVE (clean select.rs, n_refused=0
  on aarch64 — the assertion AE4 cannot make, consume-plan finding 2).
- Portable mutation arm: ALIVE (M-LSPA6-1 flips, named ANDON_REFUSED).
- Pinned AE4-mutant-on-aarch64 flip: UNSUPPORTED (divergence 2) —
  structurally non-flipping, kept as informational ADMITTED row.
- Wire codeAction / classify-exact-count arms: UNVERIFIED (out of lane
  scope).

## See Also

docs/jira/v26.11.2/audit/lspu1-consume-plan.md ·
docs/jira/v26.11.2/lanes/lane-g05-lsp-wire-rebuild.md ·
scripts/verify_disassembly_invariants.py
