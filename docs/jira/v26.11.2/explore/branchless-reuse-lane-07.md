# Branchless reuse map — lane BR7 (bn3 join probe)

Divergence note: dispatch anchors (bucket fn :728-731, probe loop :1226-1310,
lane_eq_masks :1375-78) have drifted; reality on this tree is bucket fn
`unify.rs:825-828`, probe loop :1391-1423, `lane_eq_mask` :678-687,
candidate build/cache gate :1252-1302. Doc yields; anchors below are as-read.

## 1. Bucket hash — swap `radix_bucket` → `cgap_branchless::radix::radix_bucket_12`

- Current: `unify.rs:825-828` private `fn radix_bucket` (payload-masked
  xor-shift, 3 ALU ops). Replica: `cgap-branchless/src/radix.rs:45-48`,
  byte-equal body, B5 differential court pins equality
  (radix.rs:121-142 edge + tagged-sigil court).
- Proposed: make `radix_bucket_12` the single kernel; bn3 calls it.
  Blocked on visibility, not semantics: the shipped fn is private with no
  pub re-export (radix.rs:39-43 documents the same wall). Options: `pub`
  + re-export from bn3, or copy-with-sha-pin (B5's answer, already done).
- Gain class: dedup/audit, NOT ns — 3 ALU ops either way; both are
  `[inline(always)]` so codegen is identical. Honest claim only.
- Court mutant: flip one limb of the xor-shift (`y >> 16` → `y >> 8`) in a
  /tmp sandbox copy → B5 differential court fails (bucket assignments
  differ on tagged words). Already exists; reuse, do not rebuild.
- Standing: NOT proposed for the hot path by BR7 — RADIXFIX lane owns the
  Default flip (`JoinProbeScratch::default` radix:true, :789+); BR7 does
  not duplicate.

## 2. Triple-field equality — already bit-parallel; the branchy part is the verdict branch, not the compare

- `unify.rs:1402-1405`: `ok = MASK_ALL; for bound_slots: ok &=
  lane_eq_mask(...)`. This IS SWAR word-parallel across all bound slots
  (FA3's "DEEP, all bound slots" confirmed), zero data-dependent branches
  in the compare itself; `lane_eq_mask` (:678-687) is branch-free
  xor/neg/sub.
- The residual branch is the single verdict test `if ok != MASK_ALL
  { continue }` (:1406-1409) and the prefilter branch (:1397-1400).
  Candidates: fold prefilter into `ok` (`ok &= mask_word[j]` style) to
  kill one of two branches; verdict itself must stay a branch — push
  decisions gate on it (structural, single, well-predicted on skew).
- No `pred_inc`-style branchless counting exists at emission: match
  emission (:1410-1420) is unconditional pushes gated by the verdict
  branch, not counted. `cgap-branchless` has no pred_inc primitive
  (swar.rs exports swar_lt/eq/has_zero_byte/range_mask/splat only), so a
  counting rewrite is new-kernel work, not reuse. Gain class: micro;
  only worth it if profiling shows the verdict branch mispredicts.
- Court mutant: replace `lane_eq_mask` accumulator AND with OR in a
  sandbox copy → any multi-slot candidate with one mismatched slot is
  wrongly emitted → differential vs nested arm fails.

## 3. Chain walk (:1394-1422) — keep, structural

- Pointer chase `while c != u32::MAX` with sentinel branch + two
  mid-loop `continue` branches. A chain walk over a singly-linked list
  inherently needs the sentinel branch; branchless alternatives
  (speculative full-list scans) destroy the O(cur+cand) contract.
- Gain class: structural, no change proposed. Mark as KEEP.
