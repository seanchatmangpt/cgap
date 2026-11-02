# GAP-6 Identity Receipt (F2 schema-drift close-out)

- **Lane**: GAP-6 (lane-runner), repo `/Users/sac/cgap`, branch `main`,
  ONE canonical checkout; no git write commands run; 2026-11-01.
- **Lease**: appends to `bench-bn3-receipt.md` + `bench-seal-receipt.md`
  (identity sections only), NEW `f2-15-bench-flips.md`, this receipt.
  Nothing else written; bench subjects consumed read-only.
- **Finding closed**: F2-15 witnessed (a) `bench-bn3-receipt.md` and
  `bench-seal-receipt.md` cite no subject sha256 (identity riding on
  lease statements — R_missing_identity-lite), and (b) the 4/4 flip
  verdict existed only as the audit's final message, no marker file
  (`f3-20-terminus.md` recorded "No `f2-15` marker file exists").

## Subject sha256s written into the receipts

| File | sha256 | Size | Written into |
|---|---|---|---|
| `crates/bench/benches/bn3.rs` | `5bc657174eaff206dac209dca2088f7728b7e0410a2fcdc66296caf2d5014745` | 14,646 B | `bench-bn3-receipt.md` "Identity (GAP-6 backfill)" |
| `crates/bench/benches/seal.rs` | `0bc7e0b738d609c0b18356ed4647d98e83b863d2162f4bf5d056d8d2963ef3f4` | 19,320 B | `bench-seal-receipt.md` "Identity (GAP-6 backfill)" |

Both subject hashes computed with `shasum -a 256`, then re-verified
unchanged AFTER both appends (post-write re-read; the identity claims
hold at use time, not only at capture time).

## sha256s of the files this lane produced (post-write, on-disk)

| File | sha256 (post-write) |
|---|---|
| `bench-bn3-receipt.md` (appended) | `8bfb4e1eb8fddc176b2b86952ff7a8ad632461647122927745357b4b31e7340b` |
| `bench-seal-receipt.md` (appended) | `199b1366d5e12d6289d1efd45257bf580d0fc0d2c66e07e42c52bec1e2170bb1` |
| `f2-15-bench-flips.md` (new) | `84c5cb8afd94602e88947eaa62380a511602021eae4a5e7b87a014c7d478f7d1` |

`gap-6-identity.md` (this file) carries no self-hash — a file cannot
contain its own digest; its identity is established by the parent
coordinator at integration.

## Commands + exits (this session, all real)

| Command | Exit |
|---|---|
| `shasum -a 256 crates/bench/benches/bn3.rs crates/bench/benches/seal.rs` | 0 (hashes above) |
| `wc -c` on both bench subjects | 0 (14,646 / 19,320 B) |
| `grep -rl "F2-15" docs/jira/v26.11.1/` | 0 (sole hit `f3-20-terminus.md`) |
| Edit-append x2 + Write x2 (lease files) | success (tool receipts) |
| Post-write re-run of `shasum`/`wc` on subjects + outputs | 0 (table above; appends read back byte-clean, no write corruption) |

## Provenance boundary (no overclaiming)

- The F2-15 flip numbers (ingress 21x; parquet retained-alloc channel
  semantics note; bn3 118.9 -> 232.2 ns/pair 1.95x; seal forced BLAKE3
  recompute +12%, controls <1%) are the F2-15 court-auditor's recorded
  verdicts as relayed by the GAP-6 dispatch. GAP-6 ran no bench, no flip,
  no restore; "sha-exact restore" is the audit's claim, cited not
  re-verified. Those rows' standing is the audit's; this lane's own
  standing below covers only the identity patch.
- Per-receipt corroboration that exists on disk independently of the
  audit message: `bench-ingress-receipt.md` (M-F2-11-1 flip, exit 101)
  and `bench-parquet-receipt.md` (court 2 sandbox flip, exit 101).

## Standing

**ALIVE** (lane scope: receipt identity patch) — every sha256 above was
computed and written this session with commands+exits shown; on-disk
bytes re-verified post-write. The transcribed F2-15 verdicts remain
F2-15's evidence; nothing here re-measures them.
