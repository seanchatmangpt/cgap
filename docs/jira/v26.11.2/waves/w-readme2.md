# Lane W-README2 — README reconciliation receipt (2026-11-01)

Archive-before-overwrite of 9 moonshot/aux crate READMEs into
`docs/jira/v26.11.1/readme-archive/` (`<crate>-pre-v26.11.2.md`, sha256
fail-closed verified on copy), then rewritten to 26.11.2 state with
test counts grounded in `waves/crate-reality/` receipts (this wave,
2026-11-01).

## Bounded poll (observable, no sleeps)

Dispatch named 9 crates whose version observable is
`version.workspace = true`; the workspace `[workspace.package] version`
is the poll target. First check: root `Cargo.toml:42` `version = "26.11.2"`
already live, and all 9 `crate-reality` receipts present — poll condition
satisfied on first observation, zero wait iterations, zero sleeps.

## Archives (sha256, byte-exact restore-verified)

| crate | file | sha256 | bytes |
|---|---|---|---|
| cgap-core | readme-archive/cgap-core-pre-v26.11.2.md | e49afc1b8fc9b7beb9c1b2b61adcdb1d957e0aaac99f3cc1259f1f8a81f149a7 | 1730 |
| cgap-storage | readme-archive/cgap-storage-pre-v26.11.2.md | 77aa283b85dd3f48ccc6f7886f65755ed036ff21a9e8d4cac956c1bac3246184 | 1378 |
| cgap-bont | readme-archive/cgap-bont-pre-v26.11.2.md | 3843d217e18c4435c2cad37e6d29f0dd9afaa09a6199d2a5e7e400e519306d92 | 1288 |
| cgap-bosym | readme-archive/cgap-bosym-pre-v26.11.2.md | 0bc654ac145bcaf154306de8c34203aa31f3788e705a207120fc01b097dffc78 | 1322 |
| cgap-bn3 | readme-archive/cgap-bn3-pre-v26.11.2.md | fdd9a80c15f753cd62e137628280ed84ce8d75eafc28b4abca1d5eee9eaccaa6 | 1549 |
| cgap-gate | readme-archive/cgap-gate-pre-v26.11.2.md | 077567fc4e84476e2fbb789009fef03fa9bab12686fa3affd487aadbb6dfb0d4 | 2042 |
| cgap-crypto | readme-archive/cgap-crypto-pre-v26.11.2.md | 65f5c28e29519de5789448086a8873c5543edb271ca521f7bbfa2d66fdb6cd48 | 1549 |
| cgap-p28 | readme-archive/cgap-p28-pre-v26.11.2.md | 6c6a3aa06e3a90ed948e10dcfd8ea981ad71891964a4f610f5783e341dafea2c | 1563 |
| cgap-ffi | readme-archive/cgap-ffi-pre-v26.11.2.md | 236b6d2ff880e1aadcde59d6355d0e73f4e10f01719bb4ace080b189d6b339e0 | 2198 |

## Rewrites (version refs -> 26.11.2, counts from crate-reality receipts)

| crate | gate claim now in README | source receipt |
|---|---|---|
| cgap-core | 63 collected, 62/1 (manifest_court pre-existing fail) | crate-reality/cgap-core.md |
| cgap-storage | 24 passed / 0 failed, 6 suites | crate-reality/cgap-storage.md |
| cgap-bont | 35 passed / 0 failed, exit 0 | crate-reality/cgap-bont.md |
| cgap-bosym | 14 passed / 0 failed, exit 0 | crate-reality/cgap-bosym.md |
| cgap-bn3 | 126 passed / 0 failed / 0 skipped | crate-reality/cgap-bn3.md |
| cgap-gate | BROKEN-MID-EDIT: 22 compile errors, 0 tests ran | crate-reality/cgap-gate.md |
| cgap-crypto | 25 passed / 0 failed | crate-reality/cgap-crypto.md |
| cgap-p28 | PARTIAL: unit target compile-abort (71 err); integration courts 8/0; census 46 | crate-reality/cgap-p28.md |
| cgap-ffi | 56 passed / 0 failed; nm census 11 T symbols | crate-reality/cgap-ffi.md |

Verification: all 9 rewritten files on disk — H1 exact, 26.11.2 refs
present (2-6 per file), UTF-8 clean, no corruption markers.

## Divergences from dispatch

1. Dispatch test counts vs receipt counts: core dispatch said 75, receipt
   census says 63 collected (62 pass / 1 pre-existing manifest_court
   fail) — README states the receipt number. gate dispatch said 15,
   receipt says 12 `#[test]` blocked at compile — README states BROKEN-MID-EDIT
   with the receipt's census. p28 dispatch 46 matches the 46-test census;
   README records the unit-target compile-abort honestly.
2. `readme-archive/INDEX.md` not updated: not in this lane's file
   ownership; left for the coordinator/index owner.
3. Publish-status lines retained from the v26.11.1 wave witnesses,
   explicitly labeled as-of-date; no new crates.io reads were run by
   this lane, so no new publish claims were made.
4. Poll satisfied at first observation (workspace already 26.11.2 +
   receipts present); no 60s/90min wait iterations needed.

## Commands (this session)

| command | exit |
|---|---|
| version/readme survey over 9 crates | 0 |
| python archive loop (copy + sha256 verify round-trip) | 0 |
| on-disk verification sweep (H1/refs/UTF-8) | 0 |

No git commands run. Build roots: none allocated (no builds needed —
counts grounded in existing receipts). Canonical tree untouched outside
the 10 owned files (9 READMEs + this receipt).
