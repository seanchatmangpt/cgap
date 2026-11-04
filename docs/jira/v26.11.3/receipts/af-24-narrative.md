# af-24-narrative

AF-24 implementation lane (AF-3 post-fix narrative propagation), v26.11.3.
AF-3's fix changed the story: the "4x SIMD inversion" in ab-12 was a
leaf-kernel mismatch (scalar `hash`'s fast path secretly used NEON leaves;
`hash_wide_simd` hashed leaves scalar), fixed via `compress_chunks_4way`
batching — post-fix wide wins ~3x at 128 KiB-64 MiB (1134 MiB/s @64 KiB,
2.7x), residual scalar-x2 edge at exactly 64 KiB documented follow-on.
Source of truth: `af-3-simd-profile.md` (same dir). Never ran git.

## 1. cgap ab-12 receipt — POST-FIX section appended

`/Users/sac/cgap/docs/jira/v26.11.3/receipts/ab-12-blake3-stress.md`
— new "POST-FIX (af-3, 2026-11-03)" section appended before Standing:
inversion root-caused + fixed, wide wins 3x at scale (post-fix numbers
from af-3), the receipt's finding 1 stands as the pre-fix record, 64 KiB
residual named as documented follow-on, FR-06 gate back to passing.
No existing text altered.

## 2. arcus dflss-aoq-claims-audit.md — cross-ref appended

`/Users/sac/arcus/docs/jira/v26.11.3/dflss-aoq-claims-audit.md`
— cross-ref paragraph appended directly under the claims-audit table
(rows 1-2 are the scalar-blake3 curve rows; there is no dedicated
SIMD-discussion section, so the table is the SIMD-adjacent surface).
No row verdicts changed: those rows audited the dossier's claimed
numbers against the receipts, not the inversion mechanism. Cites
`af-3-simd-profile.md`.

## 3. arcus cscb-prompt-library.md — Prompt 4 rewritten

`/Users/sac/arcus/docs/jira/v26.11.3/cscb-prompt-library.md`
(Prompt 4, "The Physics Inversion" — AF-11's landed file; this lane
owns the one-file edit). Case-study narrative rewritten from
"pin scalar to avoid the inversion" to the stronger post-fix story:
the profiling courts caught a leaf-kernel vector mismatch invisible
to standard CI, remediated it, wide-vector throughput now ~3x scalar
across all operational payload tiers (>=128 KiB), 64 KiB residual
honestly named (~1.63 GiB/s scalar x2 vs ~1.13 GiB/s wide 4-way).
The diagnostic machine catching its own wrong story is the sales
point. Every claim cites af-3 (ab-3, ab-12 retained for ChaCha bands
and the 16 MiB GPU crossover). Teach / Emotional impact / New way /
Impact proof / Take control all updated; no other prompt touched.

## 4. arcus cbmqs-standard.md — verified UNAFFECTED, no edit

`/Users/sac/arcus/docs/jira/v26.11.3/cbmqs-standard.md`
— grep for wide/hash/kernel/throughput/inversion/scalar/SIMD/MiB: the
only throughput cites are G4's ab-3 ChaCha (616.3 / 367.7 MB/s) and
ab-2 SHAKE absorption (4.301 ns/B), plus ab-10/ab-19 ring/IPC floors.
No G-gate cites the blake3 wide-vs-scalar inversion or carries a
blake3 threshold. Edit only-if-needed law: not needed.

## Verification

- All 4 files render clean (Read re-check post-edit; no corruption,
  no truncated tables).
- Line-length gate: all inserted lines <= 100 chars
  (`awk 'length > 100'` over the three edited files: 0 hits).
- Cite check: every inserted claim names `af-3-simd-profile.md` or its
  af-3 numbers as landed in that receipt.
