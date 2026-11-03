# CE-06 — Charter §4 branchless/SWAR verification

Lane CE-06, 2026-11-02. Canonical checkout `/Users/sac/cgap`, branch main.
Read-only except this file.

## 1. Subject claim under verification

"compiles to branchless bitmasks" (charter §4 phrasing, as dispatched).

## 2. Charter-file location search (grep, this session)

- `grep -rn "branchless bitmasks" /Users/sac/cgap/dflss` — no such dir;
  grep exit 1 (no matches anywhere under docs/ or a dflss/ tree).
- `grep -rn "compiles to branchless\|branchless bitmasks"
  docs/jira/v26.11.2/` — 0 hits.
- `14-dflss-charter.md` on disk contains ZERO occurrences of
  "branchless" (grep exit 1). The phrasing exists in the session
  record only, not on disk.
- Per dispatch contract: session-record-only ⇒ this verify doc IS the
  landing of the corrected claim. No charter file was edited.

## 3. Corrected claim (pinned truth)

Pinned per row 14 of `docs/jira/v26.11.2/11-measured-claims.md:38`
(MEASURED, receipt `waves/w3-t2-asm-census.md:23-29`):

- 0 INDIRECT branches: blr=0, br xN=0, brk=0 across the three census
  regions — cgap-gate `penstock::ingress` (158-line body, 2 arms, hot
  arm fallthrough+ret), cgap-bn3 `noalloc` region (4 functions,
  direct `bl` only), cgap-p28 `asc450` region (10 functions, direct
  `bl` only). All targets direct symbolic `bl` or intra-region
  conditional (b./cb./tb.).
- Scope: census, not a court. The census receipt itself states
  "MUTATION-FREE, so it is a census, not a court" (w3-t2-asm-census.md,
  Verdict section). Mutation-free census ⇒ no vacuity claim; the anti-
  vacuity surface lives in the flip courts (rows 23/24/25).
- NOT "zero branches": conditional arms are lawful. Row 29 pins the
  "constant-time"/"validation-without-branches" wording UNSUPPORTED
  (11-measured-claims.md:53); BR3 rejected the 0-conditional law —
  `hardening/lane-11-arrow-swar-validate.md:85-93`: "A validator's
  branches ARE its typed refusals"; removing them removes the
  refusal capability itself (vacuous-branch-hiding or an
  unconditional-accept validator, fail-closed destroyed).
- "branchless" in cgap-branchless/crate names is a crate identity, not
  a claim of 0 conditional branches anywhere in the workspace.

## 4. Per-claim verdicts

| # | Claim | Verdict | Basis |
|---|---|---|---|
| 1 | charter "branchless bitmasks" phrase | NOT-ON-DISK | 0 grep hits; session-record only |
| 2 | "0 branches" 0-conditional law | REJECTED (BR3) | row 29; lane-11:85-93 |
| 3 | 0 INDIRECT branches 3 regions | MEASURED | row 14; census receipt :23-29 |
| 4 | bitmask-compile guarantee | UNSUPPORTED | no measured claim; row 14 is indirect-only |

## 5. Receipt-integrity checks (this session)

- w3-t2-asm-census.md exists on disk (read this session; JSON footer
  standing ALIVE, verdict PASS).
- hardening/lane-11-arrow-swar-validate.md lines 85-93 read this
  session — BR3 rejection text confirmed verbatim.
- 11-measured-claims.md rows 14/29 read this session at lines 38/53.

## 6. Standing

ALIVE for the correction landing: the on-disk grep census (exit-1
zero-hit results witnessed above) and receipt reads ran this session.
The underlying asm claim (row 14) is MEASURED per its cited receipt;
this lane re-read the receipt from disk rather than recalling it.
No command re-measured the asm — the census receipt is cited, not
re-derived; row 14 remains MEASURED-as-of its own receipt date
(2026-11-01).
