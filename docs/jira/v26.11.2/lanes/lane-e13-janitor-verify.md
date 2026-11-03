# Lane E-13 — janitor-state verification (archival transcription)

Date: 2026-11-02 · Repo: /Users/sac/cgap · Standing: TRANSCRIBED (original
verification executed by the E-13 Explore lane; read-only). Its findings
were folded into the C-09 extended-grammar sweep + REG-01.

## W1 /tmp census — 65 paths

61 ACTIVE (<2h lane leases, listed not judged), 4 cold df4-cpk*
RETAINED-BY-RECEIPT (S-14's disclosed-retained list, lane-s14:39-44).

## W2 findings (all dispositioned)

1. gate_battery.py digest drift vs R-18 — legitimate, receipted
   (R-ADV05FIX: sha 379bab50 post-fix at 18:23:31 vs R-18's e38d5210);
   adv-18's Probe-4 "no drift" claim was the erroneous transcription —
   erratum filed in adv-18's receipt by WARN-01's ERRATA 2.
2. S-14 grammar gap (f01*/ocel* arms missing) — folded into C-09's
   extended grammar.
3. Census staleness (~15 post-S-14 ACTIVE paths) — expected under
   concurrent lanes; C-09's later sweep reclaimed the cold set.
4. Untracked/modified gate scripts — flagged for the commit lane
   (FS-15's staging plan owns landing).

## W3 clean items

Repo target-* clean (rc=1); anchored hog pgrep clean (rc=1); retained
trio (adv02 / adv11-probe / adv11-disasm.txt) present-intact at audit
time; df4-cpk cold residue disclosed-retained.

## W4 standing

TRANSCRIBED, landed by the coordinator. Original audit: E-13 Explore lane
(read-only; reported the landing blocker rather than writing).
