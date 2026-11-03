# Lane E-11 — breach-regression court verification (archival transcription)

Date: 2026-11-02 · Repo: /Users/sac/cgap · Standing: TRANSCRIBED (original
verification executed by the E-11 Explore lane; read-only). Post-audit
closures: the adv-14 dossier gap filled by F-09; the stale leg1 comment
fixed by I-13.

## W1 verdict — ALL 5 LEGS PASS

M1-M5 flips witnessed in lane-r14 (:56-60), each restore sha256-exact; all
six sha256s re-verified byte-exact against the live tree by E-11.

## W2 coverage confirmations

- LEG 1 two-layer confirmed: M4 = canonical_key raw + identity_key None
  (per D5 :103-108). Single-layer revert no longer flips — the ADV-10
  inode keying is the redundant second layer.
- LEG 5 dual witness: M5 gate-budget neutralization PLUS the in-gate
  bn3-pin channel (verify_cscb_sram.py bn3-pin mutate reports DIVERGED on
  a passing gate; live script channel at :377-446 — the :136-159 cite was
  the not-yet-landed lane doc).
- LEG 4 contract-change disclosure honored (D1 :71-85): level-triggered
  ADV-12 semantics asserted (:309-335).

## W3 findings at audit time — disposition

1. adv-14 dossier absent → filled by F-09 (independent attack: 5/5 flips,
   leg1 two-layer no-single-point-of-failure).
2. Stale leg1 mutation comment → fixed by I-13 (on-disk :90-96 now names
   the two-layer revert).
3. Cosmetic line-count drift (312 recorded vs 340 on disk) — W2 prose
   stale, sha-consistent.

## W4 standing

TRANSCRIBED, landed by the coordinator. Original audit: E-11 Explore lane.
