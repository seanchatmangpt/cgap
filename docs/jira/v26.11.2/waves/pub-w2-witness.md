# PUB-W2 Publish Witness

Lane: PUB-W2 (gate-runner — publish witness). Subject: crates.io registry
state for cgap, cgap-core, cgap-storage, polled after the v26.11.2 publish
wave.

## Method

Bounded poll (60s interval, 90-min deadline) against
`https://crates.io/api/v1/crates/<crate>` with a crates.io-required
User-Agent header (`cgap-lane-pub-w2 (chatman@avec.dev)`); bare curl without
UA returns an empty body. Exit criteria: `max_version == 26.11.2` and
`num_versions` exactly +1 per crate (umbrella 2→3, others 1→2).
`num_versions` > expected would be CRITICAL; none observed.

## Ladder

| Attempt | cgap (max, num) | cgap-core (max, num) | cgap-storage (max, num) | Verdict |
|---|---|---|---|---|
| 1 (baseline) | 26.11.1, 2 | 26.11.1, 1 | 26.11.1, 1 | not yet published |
| 2–3 | 26.11.1, 2 | 26.11.1, 1 | 26.11.1, 1 | pending |
| 4 | 26.11.2, 3 | 26.11.1, 1 | 26.11.1, 1 | umbrella landed; partial |
| 5 | 26.11.2, 3 | 26.11.2, 2 | 26.11.2, 2 | PUBLISH_WITNESSED |

Post-exit independent re-read (fresh curl, this session): cgap 26.11.2 ×3,
cgap-core 26.11.2 ×2, cgap-storage 26.11.2 ×2 — matches expected exactly,
no over-publish.

```json
{
  "lane": "PUB-W2",
  "verdict": "PUBLISH_WITNESSED",
  "witnessed_at_utc": "2026-11-02T07:29:26Z",
  "poll_attempts": 5,
  "critical_overpublish": false,
  "crates": {
    "cgap":         { "max_version": "26.11.2", "num_versions": 3, "expected": 3 },
    "cgap-core":    { "max_version": "26.11.2", "num_versions": 2, "expected": 2 },
    "cgap-storage": { "max_version": "26.11.2", "num_versions": 2, "expected": 2 }
  },
  "evidence": "/private/tmp/claude-501/-Users-sac-cgap/272848c3-308c-4869-a37d-e625a85ab211/tasks/bxj78i4c2.output (exit 0)"
}
```
