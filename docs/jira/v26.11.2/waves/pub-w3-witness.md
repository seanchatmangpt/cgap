# PUB-W3 Publish Witness

Lane: PUB-W3 (gate-runner, publish witness). Date: 2026-11-01.
Method: bounded poll of `https://crates.io/api/v1/crates/<crate>` (60s
interval, 90 min deadline, UA header required) plus one final confirm
read after the operator freeze. No publish actuation was performed by
this lane; the witness observes registry state only.

## Freeze citation

OPERATOR FREEZE: "do not publish anything yet" — publish actuator
stopped, recorded in `docs/jira/v26.11.2/waves/publish-halt.md`.
This lane entered HOLD after the final write; no further
publish-related polling.

## Ladder (crates.io API, live reads)

| Crate       | Baseline 23:25:31Z | Flipped 23:30:17Z | Final confirm (post-freeze) | Verdict |
|-------------|--------------------|-------------------|-----------------------------|---------|
| cgap-bont   | 26.11.1 / num=1    | 26.11.2 / num=2   | 26.11.2 / num=2             | LANDED  |
| cgap-bosym  | 26.11.1 / num=1    | 26.11.2 / num=2   | 26.11.2 / num=2             | LANDED  |
| cgap-bn3    | 26.11.1 / num=1    | 26.11.2 / num=2   | 26.11.2 / num=2             | LANDED  |
| cgap-gate   | 26.11.1 / num=1    | 26.11.2 / num=2   | 26.11.2 / num=2             | LANDED  |

All four crates: max_version 26.11.2, max_stable_version 26.11.2,
num_versions exactly 1 -> 2. Transition observed between samples
23:29:08 and 23:30:17 (UTC-local clock), consistent across two
independent read passes (background poller + final confirm).

## Standing

PUB-W3 witness: ALIVE (commands + exits this session against the exact
registry subjects). Publish actuator itself: HELD by operator freeze —
any crates beyond this set remain UNPUBLISHED and are not witnessed.

## JSON footer

```json
{
  "lane": "PUB-W3",
  "date": "2026-11-01",
  "freeze_citation": "docs/jira/v26.11.2/waves/publish-halt.md",
  "expected": {"max_version": "26.11.2", "num_versions": 2},
  "observed_final": {
    "cgap-bont":  {"max_version": "26.11.2", "max_stable_version": "26.11.2", "num_versions": 2},
    "cgap-bosym": {"max_version": "26.11.2", "max_stable_version": "26.11.2", "num_versions": 2},
    "cgap-bn3":   {"max_version": "26.11.2", "max_stable_version": "26.11.2", "num_versions": 2},
    "cgap-gate":  {"max_version": "26.11.2", "max_stable_version": "26.11.2", "num_versions": 2}
  },
  "baseline": {
    "cgap-bont":  {"max_version": "26.11.1", "num_versions": 1},
    "cgap-bosym": {"max_version": "26.11.1", "num_versions": 1},
    "cgap-bn3":   {"max_version": "26.11.1", "num_versions": 1},
    "cgap-gate":  {"max_version": "26.11.1", "num_versions": 1}
  },
  "flip_window_utc_local": ["23:29:08", "23:30:17"],
  "method": "crates.io API poll, 60s interval, 90min deadline, UA header",
  "verdict": "LANDED",
  "publish_actuator": "HELD (operator freeze)",
  "standing": "ALIVE"
}
```
