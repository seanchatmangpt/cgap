# lane-dod-16-compendium

DOD-16: finish CULL-08/TC-08's deferred compendium body-row
correction (Errata 2's "original lines retained above unedited"
clause). 2026-11-02, branch main.

## W1 — Subject

`docs/REVENUE_GENERATION_COMPENDIUM.md` (:59, :237, :288) + errata
sweep. Divergence recorded: while this lane held read snapshots of
the three target rows, the file was concurrently corrected by lane
DOD-10 (per-file evidence: new Errata 3 signed DOD-10 now at the
errata head; three rows carry 7.71 receipted / 7.35 fresh / 8.13
median). Zero edits by DOD-16 — ownership per receipt law belongs
to DOD-10; DOD-16 owns the verification sweep and this receipt.
No SLA unblock in either direction (no compile surface touched).

## W2 — Authority

tps-01-reality-ledger.md:21-24,28-34 (5.70 FAILED; receipted 7.71,
fresh 7.35/8.13 median; 10.20 drifted to 10.70/11.80, <15 target
met, ratio 9.6x fresh) · pub-10-claims-preflight.md:20-23 (offender
rows :59/:237/:288) · w3-l1-ingress.md:63-72 (receipted figures).

## W3 — Commands / exits

| command | exit |
|---|---|
| grep -n -E "9-term refuted-figure alternation" on the compendium | 0 (errata hits only) |
| sed -n post-collision re-read of :57-61, :235-239, :286-290, :452-480 | 0 |
| sed -n '480,500p' + refuted-figure sweep | 0 |

Grep sweep result (the DOD-16 verification pass, run against the
post-DOD-10 file):

- `5.70` — hits at :469-470 only (Errata 2 text). Zero body
  occurrences. PASS.
- `5.79` — zero hits. PASS.
- `175.5M` — hits at :473, :481 only (Errata 2 + Errata 3 citation
  contexts). Body row now 136.1M derived from 7.35 fresh. PASS.
- `10.20 / 10.30 / 10.1x / 102.7 / 9.6x` — zero hits anywhere in
  the compendium: the drifted ingress-chain figures never appeared
  here, so no correction owed on that row (divergence: dispatch
  asked to sweep for them; nothing to sweep).
- `Sigma_PQC` — hits at :457-459 only (Errata 1). Errata 1 covers
  the sole body occurrence (was :103); no other Sigma_PQC row
  exists. Errata coverage COMPLETE.
- 88-byte checkpoint (:204) — feature sketch per dispatch; left
  unedited.

## W4 — Per-row before/after (as-landed by DOD-10, verified this session)

| line | before | after (verified on disk) |
|---|---|---|
| :59 | `(5.70 ns/frame Admission)` | `(7.71 ns/frame receipted Admission)` |
| :237 | `5.70 ns/frame (175.5M f/s/core)` | `7.71 receipted (7.35-8.13 re-derived)` |
| :288 | `5.79/5.70 fast, 5.87/6.40 med, 175.5M` | `7.71 rcpt / 7.35 fresh, 8.13 med, 136.1M` |

New Errata 3 (:476-486 region, signed DOD-10) records the body
correction and supersedes Errata 2's retain-original clause;
Errata 2 remains the re-derivation authority.

## W5 — Fences (must-not-regress)

- fence R1: unqualified-5.70 sweep (pub-10's gate analog for docs/
  prose) — run above, exit 0 with errata-only hits. An errata-text
  edit that reintroduced a bare 5.70 in a body row would flip this;
  mutation not re-injected post-DOD-10 landing (the pre-collision
  file DID contain the bare rows and the same grep returned them at
  :59/:237/:288 — flip witnessed this session pre-collision).
- fence R2: Sigma_PQC errata completeness (tc-08) — swept, complete.
- No compile surface touched; no cargo gate applies.

## W6 — Standing

ALIVE for the verification sweep (commands + exits above, run this
session against the exact on-disk subject). The body-row WRITE is
not DOD-16's — it landed as DOD-10's mid-flight diff (witnessed:
Edit-tool modified-since-read refusal, then on-disk re-read showing
corrected rows + Errata 3). Honest classification: PARTIAL for the
dispatch's write task (superseded, not abandoned), ALIVE for the
sweep task. No figure is asserted as measured here beyond what
tps-01 receipts; drift rows (10.20→10.70/11.80) cited from
tps-01:22, not re-run by this lane.

## W7 — Replay

Files: docs/REVENUE_GENERATION_COMPENDIUM.md (read-only for
DOD-16), docs/jira/v26.11.2/lanes/lane-dod-16-compendium.md (this
receipt). Coordinator note: DOD-16 and DOD-10 claimed the same
three lines; per disclose-in-both-directions law this receipt
discloses the collision, and DOD-10's diff is the landing of
record. Errata law honored: no landed receipt text edited; new
errata (3) documents the supersession.
