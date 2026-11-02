# Lane GAP-12 — aarch64 anti-vacuity pin-stabilization (LSPA-6 companion)

Date: 2026-11-02. Companion receipt to
`lanes/lane-lspa-06-just-branchless.md` (landed while this lane was
reading the tree — VERIFY-AND-EXTEND, not duplicate). Trigger: DOCTRINE-1
(`audit/doctrine-swarm-and-isel.md:50`) found the aarch64 court script
VERIFIED but its receipt MISSING. This receipt is that paper trail; no
code changed in either repo. lsp-max `rule_pack.rs` untouched (dispatch
allowed it only if needed — see Pin state).

## Court runs (this session)

| command | exit | witness |
|---|---|---|
| `python3 scripts/verify_lsp_branchless.py` | 0 | `LSPA-6 branchless wire court: CLEAN (AE4 clean + staleness witness + aarch64 clean 0-refused + mutant flip all held)` |
| `just branchless` | 0 | identical CLEAN verdict line; staleness witness 4 |

## Flip transcript (fresh run, release binary staleness count 4)

| subject | tier-2 aarch64 verdict |
|---|---|
| clean select.rs | n_refused=0 PASS |
| mutant_ae4 (M-AE4-1 if-injection into cmov_u64, pinned AE4 arm-C text) | n_refused=0 ADMITTED (aarch64 -O csel-lowered + seeded `repaired_hot` row) |
| mutant_portable (M-LSPA6-1 = M-AE4-1 + black_box forward while-branch + `#[export_name = "lspa6_mutant_cmov_u64"]` seed-defeating) | n_refused=3 PASS (flip) |

M-LSPA6-1 refusal text (verbatim, x2): `ANDON_REFUSED: fn
'lspa6_mutant_cmov_u64' carries a forward conditional branch; annotate
with #[allow_isel_residual] or make it branchless`.

## Pin state (dispatch task 3)

The portable mutant IS the anti-vacuity of record and is pinned in the
DRIVER, not ad-hoc: `scripts/verify_lsp_branchless.py:114-137`
(`build_subjects`) constructs M-LSPA6-1 deterministically from the live
AE4 arm-C anchors + two fail-closed site checks, with typed `[SKIP]`
(exit 3) if either injection site vanishes — the court's anti-vacuity
does NOT depend on a riscv-specific mutation surviving, and does NOT
depend on rule_pack.rs. Driver-side pinning is dispatch-preferred; no
lsp-max server edit made.

## Confirmations and honest notes

- DOCTRINE-1's finding is confirmed by re-run: the AE4 arm-C mutant does
  not flip on aarch64 (n_refused=0; -O csel + seeded repaired_hot).
  Receipt closure answers `audit/doctrine-swarm-and-isel.md:50` ("LSPA-6
  aarch64 court: script VERIFIED, receipt MISSING") — now receipted.
- AE4's linked-staticlib arm remains riscv32imc-only. The aarch64
  residual risk (isel resurrection on aarch64) is owned by AE4, NOT
  closed here; this lane only receipts the wire-court alternative.
- aarch64 clean-0-branch assertion: ALIVE (clean n_refused=0, this
  session). Portable flip: ALIVE. AE4 arm ran exit 0 inside both court
  runs (census + linked witness + M-AE4-1 + self-check + binary mutant).

## Fences run (must-not-regress)

- AE4 gate as arm (a): exit 0 both runs (shared tree untouched, /tmp
  scratch lease per run).
- `just branchless` exit 0: justfile recipe parse + recipe intact.
- No files outside the two receipts written; no git commands.

## Standing

Court ALIVE (commands + exits above, this session). Divergences: none
beyond the dispatch's own anticipated receipt race (LSPA-6's receipt
landed first; this receipt extends, does not supersede).

## See Also

docs/jira/v26.11.2/lanes/lane-lspa-06-just-branchless.md ·
docs/jira/v26.11.2/audit/doctrine-swarm-and-isel.md ·
scripts/verify_lsp_branchless.py
