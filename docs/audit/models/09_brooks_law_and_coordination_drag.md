# AU9 — Brooks' Law & Coordination Drag (Phase 2, 2026-11-02)

## 1. Grounding: implied team sizes from AU8
AU8 (docs/audit/models/08_parametric_software_models.md): COCOMO II baseline 400–740 PM;
verification uplift ×1.5–2.0 → planning envelope 600–1,480 PM. TDEV = 3.67·PM^0.28 →
20–23 calendar months. Implied steady-state team size n = PM / TDEV:
- COCOMO baseline: 400/23 ≈ 17 … 740/20 ≈ 37 → **n ≈ 18–37**
- Uplift envelope: 600/23 ≈ 26 … 1,480/20 = 74 → **n ≈ 26–74**
- ASSUMPTION: uniform staffing, no ramp; effort spread evenly across the calendar.
- Combined planning band: **n ≈ 18–74 engineers**; the AU9 grid (5/10/20/40/65)
  brackets the COCOMO-implied band and reaches into the uplift band.

## 2. Brooks channels: c = n(n−1)/2
| n | channels |
|---|---|
| 5 | 10 |
| 10 | 45 |
| 20 | 190 |
| 40 | 780 |
| 65 | 2,080 |
At the AU8 band endpoints: n=18 → 153; n=37 → 666; n=74 → 2,701 channels.

## 3. Non-coding drag taxonomy — every fraction is ASSUMPTION (industry-standard ranges)
| Channel | Eng-time fraction (ASSUMPTION) | Basis |
|---|---|---|
| Architecture RFCs / written design review | 0.05–0.10 | RFC-heavy orgs report 5–10% of eng time in written review |
| Review committees / gate meetings | 0.03–0.08 | standing cadence, 1–4 h/wk plus prep |
| Sprint syncs / standups / planning | 0.05–0.10 | daily standup + planning/review/retro ≈ 5–10% |
| Cross-team status reporting | 0.03–0.08 | status decks, escalation, OKR rollups |
| Interview / hiring load | 0.03–0.07 | ~1–2 on-sites/wk per 20-eng team at 40–70% growth |
| Onboarding ramp (charged to teammates) | 0.05–0.15 | 2–6 months to full productivity |
| Sum (low / high) | **0.24 / 0.58** | independent sums, no double-count adjustment — ASSUMPTION |
Effective coding fraction = 1 − drag = **0.76 (lean) … 0.42 (heavy)**.
Second-order Brooks cost (ramp drag on the schedule itself, "nine women" term) is NOT
modeled numerically here; the fractions above capture only recurring non-coding load.

## 4. Effective coding capacity, 20–23-month program (AU8 effort 600–1,480 PM)
Capacity = n × TDEV × coding fraction, at mid TDEV ≈ 21 mo (nominal program PM = n × 21).
| n | channels | est. coding fraction (lean–heavy drag) | effective coding PM over ~21 mo |
|---|---|---|---|
| 5 | 10 | 0.76–0.42 | 80–44 of 105 nominal PM |
| 10 | 45 | 0.76–0.42 | 160–88 of 210 |
| 20 | 190 | 0.76–0.42 | 319–176 of 420 |
| 40 | 780 | 0.76–0.42 | 638–353 of 840 |
| 65 | 2,080 | 0.76–0.42 | 1,037–573 of 1,365 |
Read: AU8's high scenario (1,480 PM of rigor-class work) is unreachable by any row's
heavy-drag capacity except near n=65 lean; the drag assumption, not headcount, is the
binding constraint. At n=65 the 2,080-channel mesh is itself a drag multiplier not
separately priced — these two effects compound, and the table prices only the first.

## 5. OBSERVATION (labeled observation, not an estimate): this repo's structural alternative
The tables above model the conventional org. This repository is a live counterexample
architecture, recorded here as observation only:
- **Single canonical checkout, coordinator-owned git.** All lanes work in one tree;
  a lane never runs `git commit`/`push` itself. Merge conflicts, branch algebra, and
  integration risk are concentrated in ONE coordinator instead of distributed across
  n(n−1)/2 pairwise channels.
- **~151 wave receipts** under docs/jira/v26.11.2/waves/ over ~2 days of dispatch
  (peak concurrency 7 per docs/jira/v26.11.2/05-wave-plan.md; 130+ lane dispatches
  across the campaign), i.e. coordination is by file-ownership partition, not by
  person-to-person channel. Disjoint file ownership per lane eliminates the
  between-lane channel entirely for the common case.
- **Mechanized coordination substrate: the C-wave hook/census fleet.** PreToolUse and
  PostToolUse hooks in .claude/settings.json (ban_generic_agents.py, guard-generated.sh,
  guard-secrets.sh, publish-guard.sh, cwd-guard.sh, fmt-edited.sh,
  mtime-restore-guard.sh, flip-tests.sh) enforce the rules that a conventional org
  prices as "review committees" and "status reporting" in §3 — automatically, at
  ~zero marginal eng-time cost. Census receipts (v2-07-census.md, m2-nostd-census.md,
  w3-t2-asm-census.md, w5-asm-census-repair.md, w6-b6-census-court.md) mechanize
  state reporting that §3 charges 0.03–0.08 of eng time.
- **Where the cost actually goes:** coordination cost is near-zero BETWEEN lanes
  (disjoint ownership + hooks replace channels) but NONZERO AT the coordinator:
  the single git owner is a serial bottleneck (bandwidth, context, queueing), and
  hook/census authoring is a fixed cost paid once, not per pair. This inverts the
  Brooks curve: it does not eliminate coordination cost, it relocates it from
  O(n²) pairwise to O(n) hub-and-spoke, at the price of coordinator saturation —
  the structural risk the wave plan manages by serializing waves (peak 7 lanes).
- SCOPE CAVEAT: lanes here are AI agents (sublinear per-lane cost, no onboarding
  ramp, no hiring load), so §3's human drag fractions do NOT transfer to this
  architecture; the observation is architectural, not a headcount claim.
