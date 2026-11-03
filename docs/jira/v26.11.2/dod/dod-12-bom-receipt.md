# DOD-12 — conformance kit BOM + cryptographic bill of materials

Lane DOD-12 / /Users/sac/cgap / branch main / 2026-11-02.
Owned files: `docs/jira/v26.11.2/dod/SHA256SUMS.manifest` (generated),
this receipt. No git commands run.

## W1 Subject

`SHA256SUMS.manifest` — 126-entry sha256 BOM covering: all 85 cargo-test
court files the exporter inventories, 23 member Cargo.tomls
(cargo metadata, 23 packages) + root Cargo.toml + Cargo.lock + 3
standalone Cargo.tomls (cli, cgap-verification, csct) + 26 crates/ Cargo.tomls
(dedup via set; crates/ dir walk), include/cgap.h, LICENSE.md (NOTE:
`LICENSE` does not exist; LICENSE.md is the repo license file),
LICENSE-P28-SIL.md, README.md, CHANGELOG.md, 11-measured-claims.md,
4 bench-validate/*.md (docs/jira/v26.11.2/bench-validate/ — the
contract's bare `bench-validate/` resolved here).

## W2 Diff

One new generated file + receipt. No shared-file edits.

## W3 Commands + exits (this session)

| leg | command | exit |
|---|---|---|
| determinism | `python3 scripts/conformance_kit_export.py --out /tmp/dod12-manifest.json` x2 + `cmp` | 0,0,BYTE-IDENTICAL |
| inventory | `--list` | 95 courts = 85 cargo-test + 5 corpus-doc + 5 gate-mode |
| gen | `python3 /tmp/dod12_gen.py` | 0; hashed=126 missing=0 |
| verify | `shasum -a 256 -c SHA256SUMS.manifest` | 126/126 OK (grep -v OK empty) |
| self-hash | `shasum -a 256 SHA256SUMS.manifest` | see W7 BOM digest |
| seed pins | sed -n 687,699p bitslice.rs + grep x2 | both constants verbatim |

## W4 Anti-vacuity

Flip mutation: flipped include/cgap.h digest line to 64 zeros in a /tmp
copy, ran `shasum -c` from repo root -> `include/cgap.h: FAILED` —
detection witnessed. Same run also flagged
`docs/jira/v26.11.2/11-measured-claims.md: FAILED` against the gen-1
digests: a REAL concurrent-lane edit landed between gen-1 and the flip;
manifest was regenerated (gen-2) and re-verified 126/126 OK. The court
flips on synthetic mutation AND on real drift; cannot pass vacuously.

## W5 Seed pins (SplitMix64 replay)

- File sha256: 755fb69773a5712977438a7f2b42a3d1db020ef5bb03ad4accc85ff859f094a6
- Byte window bitslice.rs:687-699 sha256:
  b31820a6fa3b1ef7f90efab25bba446fed58c0a7ee01d38f03454f7d9980bab5
- grep witnesses: `0xBF58_476D_1CE4_E5B9` x6 hits,
  `0xA20C_0FFE_E5A7_E000` x2 hits — both verbatim.

## W6 Standing + divergences

- ALIVE for the BOM and its verification/flip legs (commands+exits above).
- Live inventory count is 95 courts (X-05's receipt counted 91; the tree
  gained 4 cargo-test courts since — 81 -> 85 cargo-test). Recorded as
  live count per contract; XC-05 receipt stands as-of-its-run-time.
- CONCURRENT-CHURN (witnessed twice): 11-measured-claims.md then
  README.md were edited by other lanes between BOM generations; gen-3
  re-verified 126/126 OK. The BOM digest is a snapshot of its run time;
  consumers must re-run `shasum -c` at use time.
- Self-hash excluded by construction (the manifest cannot contain its
  own digest).

## W7 JSON terminus

{"lane":"DOD-12","files_written":["docs/jira/v26.11.2/dod/SHA256SUMS.manifest","docs/jira/v26.11.2/dod/dod-12-bom-receipt.md"],
"commands":["exporter --out x2 exit 0 + cmp BYTE-IDENTICAL","--list 95","gen exit 0 hashed=126 missing=0","shasum -c 126/126 OK","flip leg: cgap.h FAILED detected","seed greps 6+2 hits verbatim"],
"courts_run":["shasum -c flip court (cgap.h zero-digest -> FAILED)"],
"standing":"ALIVE",
"bom_sha256":"14ba88fda68298bc8fa249332eb24cd5c64a6e2be4d9db153a4adce874f6536a",
"file_count":126,
"divergences":["LICENSE absent (LICENSE.md used)","bench-validate lives under docs/jira/v26.11.2/","live court count 95 vs X-05's 91","11-measured-claims.md concurrently edited; digest is run-time snapshot"]

## ERRATA (FS-03 seal-time re-run, 2026-11-02)

DOD-12's BOM digest `14ba88fd…6536a` was a RUN-TIME SNAPSHOT, not the
seal-time value. FS-03 re-ran the full pipeline over the FINAL tree:

- exporter `--out /tmp/fs03-m{1,2}.json` x2 + cmp: BYTE-IDENTICAL
  (95 courts, 85 cargo-test — same inventory as DOD-12's run).
- Manifest regenerated over the final tree: `hashed=130 missing=0`
  (126 -> 130: wave lanes landed 4 new files post-snapshot — new
  cargo-test courts and member Cargo.tomls). Verified `shasum -c`:
  130/130 OK, 0 non-OK.
- Flip court re-witnessed: zeroed `include/cgap.h` digest in a /tmp
  copy -> `shasum -c` rc=1, `include/cgap.h: FAILED`. Shared tree
  untouched.
- Seed pins re-verified verbatim: bitslice.rs 6+2 grep hits
  (0xBF58_476D_1CE4_E5B9 x6, 0xA20C_0FFE_E5A7_E000 x2).

SEAL-TIME BOM digest (authoritative, supersedes 14ba88fd…):
`31debbdfc885204fce70d4b6b8d734db62580a30e0221dbec5f84826e38947c5`

Drift explanation: the DOD-12 digest was taken while the v26.11.2 wave
was still landing files; lanes added 4 manifest-covered files between
the DOD-12 snapshot and the FS-03 seal (130 vs 126 entries). This
receipt is corrected here per errata law — the original W7 JSON above
is preserved verbatim as-of-its-run-time.

RE-VERIFY-AT-USE-TIME LAW (restated): the BOM digest is a snapshot of
its generation instant; any consumer MUST re-run
`shasum -a 256 -c docs/jira/v26.11.2/dod/SHA256SUMS.manifest` against
the tree at use time and never trust a recalled digest. Anything that
edits a covered file after seal invalidates the BOM until regenerated.
