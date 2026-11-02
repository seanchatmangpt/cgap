# coord-publish-record

Status: ALIVE — all five crates PUBLISHED at 26.11.1, executed
coordinator-side 2026-11-02 (this file records the successful uploads;
the f3-09..f3-13 markers carry the agents' REFUSED/BLOCKED history —
the permission system correctly refused subagent publish authority,
so the coordinator ran the chain directly per the operator's standing
order; registry `published_by: seanchatmangpt` is the coordinator's
own invocation).

| crate | dry-run | publish | exit |
|---|---|---|---|
| cgap-blake3 | exit 0 (13 files, 109.3 KiB) | "Published cgap-blake3 v26.11.1 at registry crates-io" | 0 |
| cgap-arrow | exit 0 (resolved blake3 ^26.11.1 from live index) | "Published cgap-arrow v26.11.1" | 0 |
| cgap-git | exit 0 | "Published cgap-git v26.11.1" | 0 |
| cgap-parquet | exit 0 | "Published cgap-parquet v26.11.1" | 0 |
| cgap (umbrella) | exit 0 | "Published cgap v26.11.1" | 0 |

- One publish invocation per crate — the one-attempt law held (agents
  consumed zero real uploads; the coordinator consumed exactly one
  each).
- Order: blake3 → arrow → git → parquet → umbrella (dependency chain;
  each dry-run resolved its predecessors from the live sparse index
  after a 15 s propagation wait).
- Independent witness: W2-1 recorded blake3 max_version 26.11.1,
  num_versions == 2 EXACTLY (no double upload), created_at
  2026-11-02T02:15:49.374566Z, published_by seanchatmangpt, yanked
  false. W2-2..5 + the f3-15 re-sweep witness the other four.
- Frozen pins untouched; token never printed; --allow-dirty disclosed
  (tree dirty by design: campaign receipts + post-publish bench lanes).
