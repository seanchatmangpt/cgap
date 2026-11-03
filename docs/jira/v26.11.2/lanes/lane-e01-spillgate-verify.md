# Lane E-01 — R-ADV10FIX spill-gate re-keying verification (archival transcription)

Date: 2026-11-02 · Repo: /Users/sac/cgap · Standing: TRANSCRIBED (original
verification executed by the E-01 Explore lane in the coordinator session;
read-only, no files written by that lane). Closes the E-wave's spill-gate
verification record.

## W1 identity_key mechanism — VERIFIED

- `identity_key` = FNV-1a over "I" + dev LE + ino LE from `fs::metadata`:
  `crates/cgap-ffi/src/sovereign.rs:976-986`.
- Grant-arm ordering: `claim` (:762) → `verify` (:775-781) → `File::create`
  (:784) → `rebind` (:789), all inside one synchronous commit call (:770-795).
- `rebind` (:1071-1086) removes BOTH the text key and identity key for the
  tid before pushing the identity entry — no stale dual-entry accumulation.

## W2 court coverage — VERIFIED (courts, not prose)

- `fa9_rename_race_denied` (fa9_commit_gating.rs:400): real fs::rename
  mid-lease (:464), racer on the new path (:470-488); asserts racer_ok == 0
  naming the (dev,ino) rebind (:493-497), racer_deny >= 1 anti-vacuity
  (:498-501), byte-level drained-prefix preservation (:504-514).
- `fa9_symlink_keyflip_denied` (:529): dangling `x -> y` planted before y
  exists; typed refusal is the sole defense (design rationale :516-528);
  asserts DENIED typed (:551-555) + y never created through the link
  (:556-559); present-target variant delegated to identity_key (:525-527).
- Corollary leg `fa9_rebind_no_stale_ownership` (:562) on disk.

## W3 residuals — loudly stated (with one gap, since closed)

- Case-variant portability residual: honestly disclosed in
  corrosion/adv-10-fmea-risk-laundering.md:73-82 ("PROTECTED ON THIS HOST,
  UNVERIFIED by construction") and the dossier — but ABSENT from the
  adv10fix receipt's residual list and code doc at audit time (finding).
  Subsequent state: FM-01 row refreshed by F-02 to
  CLOSED-RECEIPTED-WITH-RESIDUAL naming both residuals; case-variant
  ASCII-fold fix planned (plan-p08-case-variant-fold.md, Design A).
- verify→create intra-syscall swap residual + O_NOFOLLOW|O_EXCL note at
  sovereign.rs:887-890/:1042-1046; FNV 64-bit collision bound unchanged.

## W4 SLA unblock — CONSISTENT

- R-14's disclosed OsStrExt unblock (lane-r14 D2) superseded-or-consistent:
  function-local imports with disclosure comments at :932-935/:977/:996; the
  landed code is a superset retaining the unblock lines; no E0252 remains.

## W5 flip-witness caveat

- MUTANT-ADV10/ADV17 exit-101 flips are receipt-cited (lane-r-adv10fix W3),
  not re-run by E-01 (read-only).

## W6 standing

TRANSCRIBED. Original verification: E-01 Explore lane, coordinator session.
PFMEA re-status it requested: since closed by F-02.
