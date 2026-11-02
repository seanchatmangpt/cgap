# repair-12 — enterprise script drain-path witness (W16 item 4 / lane F13)

Date: 2026-11-02 · Branch: main · Mode: additive

## Witness design

`scripts/generate_enterprise_cgap.py --drain <out>` adds a bounded
multi-commit rotation witness (`rotation_drain_witness`, bound 2048
commits, no sleep). FFI surface audit (witnessed 2026-11-02, sovereign.rs
`#[no_mangle]` sweep): **no drain export exists** — exports are
`cgap_version`, `cgap_verify_p28`, `cgap_admit_frame`, `cgap_eval_bn3`,
`cgap_commit_parquet`, `cgap_xip_start`. The drain is ring-internal on
rotation (`mmap_writer.rs drain_to_spill` via the `SpillSync` seam),
reachable only by committing until the 64 KiB ring wraps.

Type acceptance: cgap sealed-container layout (footer.rs §6.1) carries
magic `b"PAR2"` at L-8 and footer-length at L-4 — there is NO leading
PAR1 file magic, so the witness asserts the trailer (PAR2 + 0 < footer_len
<= size). The lane doc's "PAR1 magic bytes" premise was wrong; corrected
to the pinned A02 layout.

## Run outputs (exit 0)

- `[+] ROTATION WITNESSED at commit 415: drain_spill.parquet (65412 bytes)
  trailer magic=b'PAR2' footer_len=118 (type-accepted)`
- N = **415 commits** to ring wrap (64 KiB ring, 16-lane 8-bit slabs,
  ~157 B/block). fsck --strict exit 0. Payload label: `drained spill
  parquet (rotation witness)`.
- Default mode regression: unchanged (staging-latch envelope branch,
  exit 0).

## Honest verdict

**ALIVE**: the positive round-trip is witnessed — a real, non-empty,
structurally type-accepted cgap container reaches disk via the rotation
drain, bounded at 415 commits, in one process, no sleeps. Full
cgap-parquet parser acceptance remains UNVERIFIED from Python (no parser
binding); the structural trailer check is the strongest Python-side
witness available.

## Anti-vacuity court (flips, ran this session)

Mutated a /tmp copy of the drained container (trailer magic `PAR2` →
`XXXX`): witness check accepts=False (court flips); pristine copy
verified intact. Mutation + restore confined to /tmp; shared tree
untouched except the two owned files.

## Findings (recorded, not fixed — outside lane ownership)

1. **Single-slot lease funnels paths**: `COMMIT_LEASE` is ONE thread-local
   slot; the spill `path` is honored only at the first grant. Any later
   `cgap_commit_parquet(lanes, n, other_path)` in the same process
   commits/drians into the FIRST granted lease's file — the second path is
   never created. Witnessed: witness ordered after the seal produced
   `microkernel_seal.parquet` non-empty (rotated into) and
   `drain_spill.parquet` 0 bytes forever. Fix direction if a multi-target
   consumer appears: slot keyed by path, or an explicit lease-reset export.
2. `ffi_real_seal` prints "refused: status 0x0" when the target file is
   absent (CGAP_OK but no file) — misleading label, honest outcome;
   pre-existing branch, unchanged.
