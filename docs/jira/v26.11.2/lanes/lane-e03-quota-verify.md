# Lane E-03 — R-ADV12FIX quota repair verification (archival transcription)

Date: 2026-11-02 · Repo: /Users/sac/cgap · Standing: TRANSCRIBED (original
verification executed by the E-03 Explore lane in the coordinator session;
read-only).

## W1 level-triggered tripped — CONFIRMED

- ingress_ring.rs:108-112 doc ("LEVEL-TRIGGERED, not edge-triggered");
  note_admission :167-179 only increments clean_streak while tripped.
- Egress twin mirrors exactly: :137-141 doc, note_admission :195-204.
- Structs field-for-field identical (:114-128 vs :143-157) per the zero-dep
  "restated here" contract (:131-133).

## W2 acknowledge() — CONFIRMED

- :180-187 / :206-213 — clears tripped, window_refusals, clean_streak.
  Pinned at andon_liveness_court.rs:67-70.

## W3 1024-clean-admission hysteresis — CONFIRMED

- CLEAN_ADMISSIONS_TO_CLEAR = 1024 (:139/:167); mid-streak refusal resets
  (:158/:185); exactly-1024 + mid-streak restart pinned
  (andon_liveness_court.rs:168-222, both twins).

## W4 saturating window / wrapping total — CONFIRMED

- window_refusals saturating_add (:156/:183); total wrapping_add (:157/:184);
  saturation leg (:231-253) pins u32::MAX-1 → no panic, lifetime counting
  intact (:241).

## W5 old assertion gone — CONFIRMED (with nuance)

- Remaining "admission resets" hits (:65/:242) are assert MESSAGES on
  window_refusals == 0 paired with tripped()-STAYS-true asserts — they pin
  preserved window-reset under the new level-triggered trip, not the old
  edge-triggered self-clear. adv-12's one-admit-clears defect is directly
  contradicted by landed code + tests.

## W6 quota_alloc_purity coherence — CONFIRMED

- Sticky asserts at :171-181/:247-249; in-file credit
  "lane R-ADV12FIX minimal unblock, disclosed in both receipts" (:171-174);
  receipt cites within a couple of lines of landed blocks (drift only).
  Zero-alloc fence intact (:256-263).

## W7 finding (informational, pre-existing)

- RefusalQuota derives Copy; the court's acknowledge leg tests a copied
  value; refusal_quota(&self) exposes no &mut path — acknowledge() only
  reachable on an owned copy from outside the crate. FIX-04-era API shape,
  not a repair defect.

## W8 standing

TRANSCRIBED, landed by the coordinator. Original verification: E-03 Explore
lane.
