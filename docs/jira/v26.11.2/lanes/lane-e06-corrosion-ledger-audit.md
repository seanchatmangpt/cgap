# Lane E-06 — corrosion corpus consistency audit (archival transcription)

Date: 2026-11-02 · Repo: /Users/sac/cgap · Standing: TRANSCRIBED (original
audit executed by the E-06 Explore lane; read-only).

## W1 integrity — no silent demotion, no post-hoc inflation

Every verdict change in the corpus is disclosed in-file with an ERRATA /
errata-against marker or an explicit register (adv-06:51, adv-18:82,
adv-16:84-90 confirming R-16's errata follows ERRATA-never-rewrite). The
five SPOILER elevations are first-session findings with command+exit.

## W2 R-19-F1 (shm_open) — CONFIRMED LANDED

fix-19-corpus-audit.md:150/:190-197 flag the spoiler; errata landed in BOTH
red-13-receipt-desync-exploit.md:87-97 (UNSUPPORTED → REFUTED-AS-FILED,
citing cgap-beam/src/shm.rs:47, 7 hits) and
red-20-hardened-survivor-synthesis.md:265-276 (row 75 REFUTED-PARTIALLY,
count corrected to 12 verified + 1 refuted). Both append-only.

## W3 repair receipts on disk

lane-r-adv05fix-gate-battery-utf8 (post-fix sha 379bab50 chain-verifiable
against adv-05's pre-fix e38d5210), lane-r-adv09fix-contention-doc-wiring,
lane-r-adv10fix-spill-gate-rename-toctou, lane-r-adv11fix-implcell-slide,
lane-r-adv12fix-quota-repair, lane-r-dodfix-2313-qualifier.

## W4 definitive ledger

01 IMMUNE / 02 IMMUNE (the :126 SPOILER-CONFIRMED is the court flip, not a
defect) / 03 IMMUNE / 04 typed-immunity / 05 REPAIRED-RECEIPTED +
REGISTERED-BOUND residual / 06 IMMUNE / 07 REPAIRED-RECEIPTED / 08
HONEST-REFUTED / 09 REPAIRED-RECEIPTED (R1 registered-bound residual) /
10 REPAIRED-RECEIPTED / 11 REPAIRED-RECEIPTED / 12 REPAIRED-RECEIPTED ×2 /
13 IMMUNE / 16 CLEAN / 17 REPAIRED-RECEIPTED / 18 IMMUNE + errata-vs-R-18.

## W5 conclusion

Every SPOILER-CONFIRMED item has a repair receipt on disk; no spoiler
silently demoted; the only downgrades are append-only erratas. ADV-09's
machine gate honestly PROCEDURAL/TARGET. Ledger clean.

## W6 standing

TRANSCRIBED, landed by the coordinator. Original audit: E-06 Explore lane.
