# Lane I-12 — Stale Legal-Cite Refresh (2026-11-02)

Post-GAP-01/GAP-01b line-number drift repair across the P28-SIL legal
cite surface. Every new cite verified against disk this session.

## Commands

- `grep -n <clause fragments> LICENSE-P28-SIL.md` — exit 0; located all
  quoted clauses at their current lines (333-line instrument).
- `sed -n` verification reads: LICENSE-P28-SIL.md :105-108, :134-136,
  :303-314; 34-p28-sil-compliance.md :55-90, :88-105; LICENSE.md :1-10;
  Cargo.toml :2-3, :46. All exit 0.
- `python3` assert-guarded replacements (count==1 per sub) on the four
  owned files; `awk length>100` sweeps — all appended blocks ≤100-char
  lines. Exit 0.

## Refreshed-cite table (LICENSE-P28-SIL.md, 291 -> 333 lines)

| Item | Old | New |
|---|---|---|
| TC-08 ERRATA RIDER | :279-290 | :303-314 |
| §2.3 exact-instant | :110-112 | :130-132 |
| §3.3 adverse inference / Eq. / §3.4 estoppel | :139 / :141 / :147 | :163 / :165 / :171 |
| §4.1 ASC 450 / SOX integration | :155 / :166-167 | :179 / :188-191 |
| §5.3 privilege | :189-191 | :213-215 |
| §6.2 ring drops | :212 | :236 |
| §8.2 §1519 / §9 SCA-SBOM / CI fail-closed | :244 / :248-256 / :258-264 | :268 / :272-284 / :278-284 |
| §3.1 covenant | :118-120 | :142-144 |
| §7.2 arbitration | :228 | :252 |
| R10 UnitCost / L_assessed | :157-159 / :141 | :181-183 / :165 |
| §2.1 grant | :96 | :96 (unchanged) |
| §2.2 triggers | :105-108 | :105-108 (unchanged) |
| Assent chain Cargo.toml / LICENSE.md | :2-3,:46 / :7 | unchanged (re-verified) |

34-p28-sil-compliance.md self-mapping: instrument §2.4's
":60-71" -> :66-88 (heading :66, amendment note :79-88); ":75-88" ->
:90-105 (heading :90, ≥ pin :98-100). §2.4 itself: :134-136.

## Quote integrity

Every quoted clause re-read verbatim and matched at its new location —
zero quote divergence. One flag: FIX-11 R10's clause quote is a
pre-existing ASCII transliteration; the instrument (:165) carries a
leading nabla the transliteration omits. Retained verbatim, flagged in
the register's ERRATA, not rewritten (dispatch law: flag, don't touch).

## Files written

- `docs/jira/v26.11.2/redteam/fix-11-covenant-repair-register.md`
  (living register: cites refreshed in place + ERRATA section appended)
- `docs/jira/v26.11.2/lanes/lane-gap01-license-amendment.md`
  (append-only ERRATA; :279-290 -> :303-314 at its line 53)
- `docs/jira/v26.11.2/lanes/lane-gap01b-ttl-reconciliation.md`
  (append-only ERRATA; same cite correction at its line 115)
- `docs/jira/v26.10.31/34-p28-sil-compliance.md`
  (Amendment-of-Record block only: §2.4 stale-cross-cite errata note;
  the instrument itself was NOT edited — coordinator-sensitive)
- This receipt.

## Standing

ALIVE — all edits assert-guarded, on-disk, line-length-swept this
session. LICENSE-P28-SIL.md untouched (diff confirms). No git commands.
No fences to run: documentation-only lane, no code/gates/courts touched;
nearest fence is the pre-existing `git status` cleanliness of owned
files (three are untracked in-flight lane receipts — coordinator owns
commits).
