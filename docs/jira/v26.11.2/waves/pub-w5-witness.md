# PUB-W5 Publish Witness — crates.io lane

Lane: PUB-W5 (gate-runner, publish witness)
Window: bounded poll, 60s interval, 120 min deadline (started 2026-11-02T07:25:18Z)

## Baseline receipt (2026-11-02T07:23Z, crates.io API v1, UA cgap-gate-runner/26.11.2)

| crate | live max | num_versions | status vs 26.11.2 |
|---|---|---|---|
| cgap-audit | 26.11.1 | 1 | pending (+1 needed) |
| cgap-fibo | 26.11.1 | 1 | pending (per lane note: possibly first publish — actually 26.11.1 already live, +1 needed) |
| cgap-cloud | 26.11.1 | 1 | pending (+1 needed) |
| cgap-integration | MISSING | 0 | first publish pending |
| wasm-footprint | 26.11.1 | 1 | pending (+1 needed; LAST in sequence) |

## Poll result

PENDING — poll running (background task b3v7q0s3p).

```json
{"lane":"PUB-W5","status":"PENDING","window":{"start":"2026-11-02T07:25:18Z","interval_s":60,"deadline_min":120}}
```

## ERRATA (W7 census, 2026-11-02)

W7 drift census (waves/w7-errata.md): this receipt carries ZERO
witnessed command+exit blocks — baseline table and poll status record
claims with no command line or exit code; the poll itself terminated
PENDING with no observed outcome recorded. Per the p4-terminus Table 4
metagate standard (waves/p4-terminus.md:65-82), VACUOUS-class as
written; all rows here are UNVERIFIED. Successor witness must embed
actual curl commands + exits per read, and a terminal observed state.
