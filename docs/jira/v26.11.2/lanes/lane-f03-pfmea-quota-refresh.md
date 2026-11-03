# Lane F-03 — FM-08/PY-4 quota-repair receipt refresh

Date: 2026-11-02 · Standing: **ALIVE** (all commands + exits this session)

## 1. Identity

- Repo `/Users/sac/cgap` (branch main). Lane F-03, contract: three
  dflss docs only, merge-not-clobber, no git commands.
- Written: `docs/jira/v26.11.2/dflss/04-pfmea-risk-matrix.md`
  (FM-08 section + register row),
  `docs/jira/v26.11.2/dflss/08-ax-pfmea.md` (PY-4 row),
  `docs/jira/v26.11.2/dflss/06-control-plan.md` (§5.1 quota note +
  Exit-101 SPC quota row). This receipt. No other files.

## 2. Independent verification (all this session, lease root
/tmp/f03-target)

- `cargo test -p cgap-git --test andon_liveness_court --test
  quota_alloc_purity_court` (cwd crates/cgap-git) → exit 0:
  andon 5 passed / 0 failed, purity 1 passed / 0 failed.
- /tmp path-dep probe (`/tmp/f03-probe`, since deleted) → exit 0,
  `F03-PROBE-PASS`: public-API re-run of the ADV-12 spoiler-1
  shape — 4,097 FrameTooLarge refusals → ONE admitted frame →
  `tripped()` STAYS true; 1023 admits still tripped, 1024th
  clears (exact `CLEAN_ADMISSIONS_TO_CLEAR` boundary);
  `acknowledge()` clears; `window_refusals` pinned at `u32::MAX`
  past saturation (no debug panic); ring-level twin of the same
  shape. Spoiler-2 saturation arm included.
- API check: `tripped()`, `note_refusal()`, `note_admission()`,
  `acknowledge()` and the saturation seam (`window_refusals` pub
  field) are public — no sandbox needed.

## 3. Row diffs (before → after)

- 04 FM-08 section: STATUS REGISTERED-DESIGN ("no on-disk
  subject") → REPAIRED (spoiler-1 level-trigger + 1024 hysteresis
  + ack; spoiler-2 saturating_add; M1-M4 flips cited) with
  independent F-03 re-verify paragraph. RESIDUAL kept explicit
  and unchanged in kind: quota is OBSERVATIONAL-ONLY, throttle
  obligation stays with the host (was IMMUNE, still stated).
- 04 §3.1 register row: `REGISTERED-DESIGN` →
  `PARTIAL BREACH/REGISTERED -> REPAIRED-RECEIPTED` citing
  lane-r-adv12fix + F-03 re-verify.
- 08 PY-4 row: `LANDED-CODE + WITNESSED-RUN` →
  `LANDED-CODE-REPAIRED + WITNESSED-RUN` with level-triggered
  semantics, saturation, 4/4 repair mutants, F-03 re-verify;
  fix-04-era edge-triggered premise marked superseded.
- 06 §5.1 note: "no trip-gate fix receipt exists; hardening
  remains a convention-level recommendation" → signal repaired
  (receipts cited), host throttle obligation explicitly retained.
- 06 Exit-101 SPC row: witness + status columns refreshed to the
  level-triggered + saturation semantics with both receipts.
  DESIGN-INTENT of the diagram itself unchanged (still no exit
  mapping on disk — verified in the same grep family as ADV-12).

## 4. Cite verification

- adv-12 (`redteam/corrosion/adv-12-control-plan-evasion.md`),
  repair receipt (`lanes/lane-r-adv12fix-quota-repair.md`), and
  lane-fix04 (superseded premise) all read in full this session;
  repair receipts' court legs re-executed by me (§2).
- R-ADV09FIX's CONTENTION-SUSPECT rows in 06-control-plan: NOT
  touched (grep confirms only quota rows edited).

## 5. Fences

- Owned docs only; no code changed, so no compile fence. The
  quotas' own courts (andon + purity) re-run green post-read —
  the rows now cite the same counts I measured.
- Line-length law: new prose wrapped ≤100; new table rows are
  single-line per pre-existing house style in these files.

## 6. Standing

ALIVE for the FM-08/PY-4/06-quota refresh: every new claim cites
a command run this session (§2 exits above). Probe and build-root
leases deleted at lane end (verified: /tmp has no f03 entries).
