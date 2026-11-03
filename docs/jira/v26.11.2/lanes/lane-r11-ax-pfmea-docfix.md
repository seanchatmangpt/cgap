# Lane R-11 — 08-ax-pfmea.md cite-cluster fix (docs-only tollgate)

Date: 2026-11-02. Repo: /Users/sac/cgap (main). Owned file:
`docs/jira/v26.11.2/dflss/08-ax-pfmea.md` (merge, never clobber).
No git commands. Every cite re-read on disk this session before
landing.

## Per-item before/after

1. **PY-1 cite cluster** (was sovereign.rs:898-928 / keying :928 /
   court :245 / unblock :905-908 / "fix-01 receipt pending"):
   - `canonical_key` body is sovereign.rs:898-949; claim keying
     `fnv1a(&canonical_key(path))` is :945 (read this session).
   - `fa9_alias_race_denied` is fa9_commit_gating.rs:261 (was :245).
   - FIX-07 in-file disclosure is sovereign.rs:900-902, mirrored at
     lane-fix07-ffi-surface.md:57-64 (disclose-in-both-directions).
   - fix-01 receipt NOW ON DISK: lanes/lane-fix01-fa9-alias-rekey.md
     (ALIVE) — "receipt pending" superseded.
2. **PY-5 rewritten** from REGISTERED-DESIGN/PARTIALLY-BROKEN to
   LANDED-CODE (execution-reentrancy leg REGISTERED), citing
   lane-rc01-ref-store-reentrancy.md (reserve fetch_update guard at
   ref_store.rs:321-330; stamp->reserve :302; slot copy after
   reservation :306-309; flip 101->0) + lane-rc02 (width-arm compile
   witnesses w1-w5, 2 anti-vacuity flips) + RC-02's pub(crate)
   implcell SLA unblock disclosed in both receipts. Residual kept
   explicit: impl16/implcell are compile-witnessed, NOT
   execution-reentrancy-court-certified (host=impl64, rv32imc=impl32);
   that leg stays REGISTERED; interrupt-safe __atomic_* runtime still
   owed (red-11:96-103).
3. **PY-2 pins verified, not shifted**: BN3_PIN_* at
   scripts/verify_cscb_sram.py:67-68 (as dispatch predicted);
   parse_na_consts at :158 (doc previously said :154 — corrected);
   bn3_capacity_table :174-204; pin checks :196-203; charge
   :206-218; --mutate bn3-pin at :377-385. fix-02 receipts on disk
   (lane-fix02-bn3-sram-gate.md, fix02.alive) + RC-03 engine-side
   init-time bound cited.
4. **PY-3/PY-4/PY-6 refreshed from lane receipts on disk**:
   - PY-3 -> LANDED-CODE + WITNESSED-RUN (fix-05 receipt
     lanes/lane-fix05-pack-demux-poison.md ALIVE; m1/m3 flips exit
     101, sha256-exact restores). Re-cited truth: TrailingBytes :77,
     fired :379 (was :375), trailing_poison :270, ObjectSurplus :85,
     guard :454.
   - PY-4 -> LANDED-CODE + WITNESSED-RUN (fix-04 receipt
     lanes/lane-fix04-andon-quota.md ALIVE; both mutants flip).
     Truth cites: RefusalQuota struct :106, impl :117-156,
     DEFAULT_MAX_REFUSALS=4_096 :121 (was :124), wiring e.g. :266.
   - PY-6 -> fix-07 receipt on disk (lane-fix07-ffi-surface.md,
     ALIVE). Header re-read: cgap_i32_batch :193 (was :188),
     cgap_p28_invariant_check :214 (was :209).
5. **fix-09 manifest cross-refs corrected** (recorded in the doc's
   Divergences section per ERRATA-never-rewrite): chain link law
   `fn link` is crates/cgap-ffi/src/receipt_chain.rs:117-121 — the
   manifest's ":104-106" is the `receipts()` accessor, stale
   (manifest errata fix-09-wire-constant-manifest.md:118-123
   superseded by this read). trap.rs is crates/cgap-p28/src/trap.rs
   (ERR_P28_SPOLIATION = 0x5032_38FF at :36); cgap-ffi's pin is
   sovereign.rs:90.

## Commands / exits (this session)

All verification was reads + grep/sed over the shared tree; no
builds, no gates run, no shared-tree mutation. EXITS: every
grep/sed/awk read in this receipt exit 0 (files resolved at the
cited lines). Not ALIVE-eligible claims beyond reads: none made —
WITNESSED-RUN labels in PY rows are inherited from the cited lane
receipts, re-read on disk this session (stale-claims law honored:
each cited receipt's status line was read fresh, not recalled).

## Fences

Docs-only lane; no code/gates/courts edited, so no executable fence
can regress. Over-100-char lines: pre-existing file convention
(long table rows); this lane's rows follow it — flagged, not
reformatted (merge, never clobber).

## Standing

PARTIAL-ALIVE as a docs lane: every correction is a verified
on-disk read (commands+exit 0 above); no gate execution claimed.

## JSON terminus

```json
{"lane":"R-11","files_written":["/Users/sac/cgap/docs/jira/v26.11.2/dflss/08-ax-pfmea.md","/Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-r11-ax-pfmea-docfix.md"],"commands":["grep/sed reads: sovereign.rs 865-960 exit 0","grep -n verify_cscb_sram.py pins exit 0","grep -n receipt_chain.rs link law exit 0 (truth :117-121)","grep -n trap.rs/sovereign.rs SPOLIATION exit 0","grep -n fa9_commit_gating.rs:261 exit 0","grep -n pack_demux/ingress_ring/cgap.h cites exit 0","awk length check exit 0"],"courts_run":[],"standing":"PARTIAL (docs-only; all cites re-read on disk this session, exit 0)","divergences":["<=100-char line rule: file-wide long table rows pre-existing, kept (merge never clobber)","fix-09 manifest ERRATA :118-123 itself names :104-106 as link law — superseded by on-disk read (:117-121 fn link)"]}
```
