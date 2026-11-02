# P3-13 API-VERIFY-ALL Verdict Table

Lane: P3-13-api-verify (court-auditor, independent). 2026-11-02T03:29Z.
Repo /Users/sac/cgap. UA cgap-publish-receipt. Lease: this file + f3-13
umbrella witness append (done). No git writes. Raw evidence: /tmp/p313_api_*.json,
/tmp/p313_idx_*.txt (retained).

## Headline

- 5 original kernel crates (cgap, cgap-arrow, cgap-blake3, cgap-git,
  cgap-parquet): ALL PASS — num_versions == 2 exact, max_version 26.11.1,
  yanked false, downloads >= 9, published_by seanchatmangpt, present in
  sparse index with both versions.
- 17 new crates: ALL FAIL — HTTP 404 on BOTH the API and the corrected
  sparse index. None of the 17 exist on crates.io at any version. The
  task premise "17 new expect num_versions == 1" is contradicted by
  registry truth at poll time 03:21-03:24Z.
- Negative probe: sparse index p313-definitely-not-a-real-crate -> 404;
  API same name -> 400 (invalid-name class). Probe flips; check not vacuous.

## Originals (API + sparse index, sampled 03:21-03:24Z 2026-11-02)

| crate | API | num_versions | max_version | yanked | downloads | sparse idx | verdict |
|---|---|---|---|---|---|---|---|
| cgap | 200 | 2 | 26.11.1 | False | 9 | 200 (26.10.29+26.11.1) | PASS |
| cgap-arrow | 200 | 2 | 26.11.1 | False | 13 | 200 (26.10.29+26.11.1) | PASS |
| cgap-blake3 | 200 | 2 | 26.11.1 | False | 13 | 200 (26.10.29+26.11.1) | PASS |
| cgap-git | 200 | 2 | 26.11.1 | False | 13 | 200 (26.10.29+26.11.1) | PASS |
| cgap-parquet | 200 | 2 | 26.11.1 | False | 13 | 200 (26.10.29+26.11.1) | PASS |

## New crates (expect num_versions == 1) — ALL ABSENT

API and corrected sparse index (4-char prefix) returned 404 for every name
below, at 03:22-03:24Z. Whichever 17 of these 18 names constitute the
coordinator's set, every one FAILs identically (404, not num_versions 1):

| cgap-bont | 404 | - | - | - | - | 404 | FAIL (absent) |
| cgap-bosym | 404 | - | - | - | - | 404 | FAIL (absent) |
| cgap-core | 404 | - | - | - | - | 404 | FAIL (absent) |
| cgap-bn3 | 404 | - | - | - | - | 404 | FAIL (absent) |
| cgap-crypto | 404 | - | - | - | - | 404 | FAIL (absent) |
| cgap-gate | 404 | - | - | - | - | 404 | FAIL (absent) |
| cgap-p28 | 404 | - | - | - | - | 404 | FAIL (absent) |
| cgap-storage | 404 | - | - | - | - | 404 | FAIL (absent) |
| cgap-gpu | 404 | - | - | - | - | 404 | FAIL (absent) |
| cgap-audit | 404 | - | - | - | - | 404 | FAIL (absent) |
| cgap-ffi | 404 | - | - | - | - | 404 | FAIL (absent) |
| cgap-cli | 404 | - | - | - | - | 404 | FAIL (absent) |
| cgap-integration | 404 | - | - | - | - | 404 | FAIL (absent) |
| wasm-footprint | 404 | - | - | - | - | 404 | FAIL (absent) |
| cgap-cloud | 404 | - | - | - | - | 404 | FAIL (absent) |
| cgap-fibo | 404 | - | - | - | - | 404 | FAIL (absent) |
| cgap-court | 404 | - | - | - | - | 404 | FAIL (absent) |
| cgap-beam | 404 | - | - | - | - | 404 | FAIL (absent) |

`bench` on crates.io is an unrelated 2017 crate (max 1.1.0) — NOT ours;
our crates/bench is publish=false. cgap-court and cgap-beam carry
publish=false in-repo (p3-recon-manifests.md) and are absent from the
registry, which is consistent — but they are counted FAIL here for the
purpose of "26.11.1 live for all 22".

## Commands + exits (this lane, this session, all UA cgap-publish-receipt)

- curl API x24 candidates (crates.io/api/v1/crates/<name>): five 200,
  eighteen 404, plus bench 200 (unrelated). Exit 0.
- curl sparse index x21 (corrected https://index.crates.io/<2>/<2>/<name>):
  five 200, sixteen 404. Exit 0. First attempt used a broken 3-char prefix
  (cut -c3) — all 404; discarded and rerun; the corrected run is the
  evidence above (self-caught bug, disclosed).
- 404 negative probes: sparse-index fake name 404; API fake name 400. Exit 0.
- Bounded poll for nine p3-pub-* markers: 30s cadence, 45-min budget,
  launched 03:20Z; at 03:26Z resolved_markers=1 — IN PROGRESS at write
  time; final count to be appended below at poll exit.

## Divergences

1. MAJOR: the 17 new crates are not on crates.io. coord-publish-record.md
   covers only the five; the P3 publisher lanes' markers are still
   "(pending)" at sweep time. Registry contradiction = FAIL per contract.
2. Sparse-index URL bug in this lane's first probe (cut -c3): caught by
   this lane (5 known-200 crates returned 404), corrected, rerun.


---

## FINAL SWEEP 2026-11-02T04:24Z (poll expiry 04:20:58Z)

Bounded poll: two windows, total ~57 min (03:20Z-03:56Z killed by harness
background limit, disclosed; 03:58Z-04:20:58Z ran to its own deadline,
POLL2-TIMEOUT exit 1). At expiry: 10 p3-pub-*.md marker files on disk;
this lane's detector counted 6 resolved (ALIVE/BLOCKED/UNSUPPORTED/PARTIAL
in standing body); the other files carry final-section headings this
detector's regex missed — counts below supersede the counter.

Registry truth at 04:24:28Z (API + corrected sparse index, 4-char prefix):

- PASS x13 total: five originals (num_versions 2 exact, max 26.11.1,
  yanked false, downloads >= 9) + eight new crates at num_versions 1
  exact, max 26.11.1, yanked false, published_by seanchatmangpt:
  cgap-bont (03:22:52Z), cgap-bosym (03:24:48Z), cgap-bn3 (04:13:45Z),
  cgap-crypto (03:25:33Z), cgap-gate (04:03:44Z), cgap-gpu (03:33:40Z),
  cgap-audit (03:53:39Z), cgap-cloud (03:43:38Z).
- FAIL x10 (404 on BOTH API and sparse index, still absent):
  cgap-core, cgap-p28, cgap-storage, cgap-ffi, cgap-cli, cgap-integration,
  wasm-footprint, cgap-fibo, cgap-court, cgap-beam.
- Task expectation "17 new expect num_versions == 1" contradicted: 8 of
  17 live at sweep-final; 10 absent. Mid-poll movement witnessed: 6
  crates went live between 03:22Z and 03:58Z, 2 more by 04:24Z — the
  publisher wave was ACTIVELY UPLOADING during this lane's poll.
- Negative probe (retained): sparse-index fake name -> 404; API fake
  name -> 400. Probe flips; court not vacuous.

### Final verdict rows (26.11.1-required crates)

| crate | num_versions | max | yanked | sparse | verdict |
|---|---|---|---|---|---|
| cgap | 2 | 26.11.1 | false | 200 | PASS |
| cgap-arrow | 2 | 26.11.1 | false | 200 | PASS |
| cgap-blake3 | 2 | 26.11.1 | false | 200 | PASS |
| cgap-git | 2 | 26.11.1 | false | 200 | PASS |
| cgap-parquet | 2 | 26.11.1 | false | 200 | PASS |
| cgap-bont | 1 | 26.11.1 | false | 200 | PASS |
| cgap-bosym | 1 | 26.11.1 | false | 200 | PASS |
| cgap-bn3 | 1 | 26.11.1 | false | 200 | PASS |
| cgap-crypto | 1 | 26.11.1 | false | 200 | PASS |
| cgap-gate | 1 | 26.11.1 | false | 200 | PASS |
| cgap-gpu | 1 | 26.11.1 | false | 200 | PASS |
| cgap-audit | 1 | 26.11.1 | false | 200 | PASS |
| cgap-cloud | 1 | 26.11.1 | false | 200 | PASS |
| cgap-core | - | - | - | 404 | FAIL absent |
| cgap-p28 | - | - | - | 404 | FAIL absent |
| cgap-storage | - | - | - | 404 | FAIL absent |
| cgap-ffi | - | - | - | 404 | FAIL absent |
| cgap-cli | - | - | - | 404 | FAIL absent |
| cgap-integration | - | - | - | 404 | FAIL absent |
| wasm-footprint | - | - | - | 404 | FAIL absent |
| cgap-fibo | - | - | - | 404 | FAIL absent |
| cgap-court | - | - | - | 404 | FAIL absent |
| cgap-beam | - | - | - | 404 | FAIL absent |

Standing of this lane: PARTIAL — 13 of 23 candidate names PASS, 10 FAIL
(absent); the wave is mid-publish, re-verify the 10 after publisher
lanes land their remaining markers.
