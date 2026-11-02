# M3 — Lease-Ceiling Audit (65,376 vs 65,536)

Lane: M3 (Measure/audit), repo /Users/sac/cgap, branch main.
Ownership: this receipt file only. No source edits, no git writes, no
mocks. All mutation courts ran in /tmp sandboxes.

## Verdict

Both are true at different layers; the receipts are correct.

- `crates/cgap-ffi/src/sovereign.rs:738` and `:782`: the lease grants a
  `StaticPageBuffer::<65_536>` — that constant is 65,536 **BYTES** of
  block-buffer capacity, not lanes. (The lane-analysis citation of
  "sovereign.rs:737" is off by one; the buffer sites are 738 and 782.)
- The lane ceiling is DERIVED, not stored: the sealed block
  (page header + hybrid body + 4-byte row-group span + 96-byte footer
  trailer) must fit 65,536 bytes. For the bench probe corpus
  (`i % 256`, bits 8, no 3-runs) that leaves 65,376 lanes — exactly the
  receipts' number.

## Measurement (code, not paper)

Sandbox probe (`/tmp/m3-probe`, against a /tmp copy of the real
cgap-parquet at sha256 `724c728f…97aa`, mutated nowhere in the shared
tree): bisect write_slab + seal over `StaticPageBuffer::<65_536>` with
the probe corpus.

- max lanes accepted: **65,376** (sealed block = 65,531 bytes)
- k = 65,377: `write_slab` succeeds; `seal` refuses
  `RunOverrun { run_len: 65,571, budget: 65,536 }`

This reproduces the a3/p2 receipts' claim exactly: 65,376 accepted,
first refusal at 65,377. The refusal class is `seal`'s RunOverrun
(mapped to CGAP_ERR_LEASE_EXHAUSTED at sovereign.rs:758-760), not
`write_slab`'s buffer check.

## Anti-vacuity court (FLIPPED)

Named mutation: hybrid_len literal-group width 8 -> 16 (both pad
sites, /tmp/m3-parquet/src/mmap_writer.rs:411 et al.). Result: probe
boundary moved 65,376 -> 8 lanes (sealed 150 bytes at k=8). The probe
detects size-law mutations; not vacuous.

Two earlier mutation attempts (TRAILER_FIXED 96->104, ATTESTATION_LEN
80->88) were structurally refused by the const-assert at
footer.rs:100-103 — noted as a defense of the trailer law, not the
court.

## Restore

Sandbox `mmap_writer.rs` restored sha256-exact to
`724c728f815f5a7e6c8fdc32c2c87d859acdc52108146be6f50acb304c6497aa`
(diff vs shared tree: identical); post-restore probe re-run
reproduces 65,376 / 65,531 bytes.

## Commands + exits (this session)

1. grep receipt constants — exit 0
2. grep sovereign.rs:738/782 StaticPageBuffer::<65_536> — exit 0
3. cargo bench -p bench --bench chains-sovereign (shared tree) — exit 1:
   **pre-existing compile break** `crates/cgap-parquet/src/scan.rs:553`
   `missing field 'bounds' in RowGroupEntry` (another lane's in-flight
   edit; shared tree dirty per git status). Shared-tree bench
   reproduction BLOCKED by that, not by this lane.
4. /tmp sandbox probe, pre-mutation — exit 0, 65,376 lanes
5. mutation court — exit 0, boundary 8 lanes (FLIP confirmed)
6. restore + post-restore re-run — exit 0, 65,376 lanes again

## Errata rows (coordinator-owned; old receipts untouched)

| Receipt | Claim | Verdict | Errata text |
|---|---|---|---|
| waves/a3-ring-sink.md:37-38,51 | 65,376 accepted / refusal at 65,377 (0x43470004) | CONFIRMED (measured) | none needed |
| waves/p2-receipts-ring-receipt-chain.md:46-47 | 70,000-lane call refuses 0x43470004 "at the 64 KiB lease ceiling" | CONFIRMED | clarify: ceiling is 65,376 lanes for the probe corpus; 64 KiB is the byte budget (page header + hybrid body + span + 96-B trailer must fit 65,536 B); boundary is data-dependent |
| lane analysis citing "sovereign.rs:737: StaticPageBuffer is 65,536" | reads 65,536 as a LANE ceiling | ERROR (bytes-vs-lanes conflation) | "sovereign.rs:738/782 pin 65,536 BYTES of lease buffer, not lanes; the observed 65,376-lane ceiling is derived from the 64 KiB byte budget minus framing overhead for the probe corpus" |

## Standing

ALIVE (measured this session, receipts re-read from disk). Ceiling
65,376 lanes / 65,536 bytes: CONFIRMED. Divergences: shared-tree
compile break at scan.rs:553 (pre-existing, disclosed above).
