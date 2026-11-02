# PUB-W1 publish witness (crates.io, lane A5 publishes)

Poll: crates.io API v1, UA `cgap-publish-witness`, 60s interval, 90-min deadline (bounded poll — no fixed-sleep coordination; deadline-checked).

## Baseline (2026-11-02T07:24:55Z)

| crate | max_version | num_versions |
|---|---|---|
| cgap-blake3 | 26.11.1 | 2 |
| cgap-arrow | 26.11.1 | 2 |
| cgap-git | 26.11.1 | 2 |
| cgap-parquet | 26.11.1 | 2 |

## Landed

| crate | first seen | max_version | num_versions before -> after | exactly-once |
|---|---|---|---|---|
| cgap-blake3 | 2026-11-02T07:25:57Z | 26.11.2 | 2 -> 3 | PASS (+1) |
| cgap-arrow | 2026-11-02T07:26:58Z | 26.11.2 | 2 -> 3 | PASS (+1) |
| cgap-git | 2026-11-02T07:26:58Z | 26.11.2 | 2 -> 3 | PASS (+1) |
| cgap-parquet | 2026-11-02T07:26:58Z | 26.11.2 | 2 -> 3 | PASS (+1) |

No crate incremented by >1; no CRITICAL immutability finding. Loop exited ALL_DONE at poll 3 (~2 min elapsed of 90-min budget).

## JSON footer

```json
{"lane":"PUB-W1","subject":"crates.io:cgap-{blake3,arrow,git,parquet}@26.11.2","date":"2026-11-02","method":"crates.io API v1 poll, UA cgap-publish-witness, 60s interval, 90min deadline","baseline":"all max_version=26.11.1 num_versions=2 @ 2026-11-02T07:24:55Z","result":{"cgap-blake3":{"first_seen":"2026-11-02T07:25:57Z","num_versions":[2,3],"exactly_once":"PASS"},"cgap-arrow":{"first_seen":"2026-11-02T07:26:58Z","num_versions":[2,3],"exactly_once":"PASS"},"cgap-git":{"first_seen":"2026-11-02T07:26:58Z","num_versions":[2,3],"exactly_once":"PASS"},"cgap-parquet":{"first_seen":"2026-11-02T07:26:58Z","num_versions":[2,3],"exactly_once":"PASS"}},"critical_findings":[],"standing":"ALIVE"}
```
