# Open items — cgap v26.11.6

Standing: PARTIAL

Open items register for the cgap side of the v26.11.6 milestone. Census date
2026-11-06, lane R-CGCLOSE.

## OI-1 — Untracked v26.11.6 artifacts need a landing decision

`crates/bench/src/bin/cgap_e2e_consumer.rs`,
`reports/e2e_cross_subsystem_receipt.json`,
`scripts/courts/court_e2e_arcus_cgap_smalt.sh` are untracked but constitute a
completed, ALIVE cross-subsystem court (arcus -> cgap -> smalt, 1M frames,
both conservation asserts passed, anti-vacuity pass 2 included). Disposition
options: (a) land as-is with the receipt, (b) land after wiring the consumer
into `crates/bench/Cargo.toml` as a `[[bin]]` target (currently only buildable
as a loose bin — `cargo build -p bench --bin cgap_e2e_consumer` unbuilt at
census; verify before landing), (c) re-home under a cross-repo court directory
if arcus/smalt want joint ownership. Close: files are tracked at HEAD with a
receipt or an explicit park note.

## OI-2 — arcus KD-1 cgap side: `cgap_import_arrow` surface unimplemented

Registered absence, recounted grep-zero 2026-11-06 (arcus
`docs/architecture/cdata-abi.md` REGISTERED-ABSENCE note; arcus
`docs/rfc/parked-register.md` P-10). cgap carries NO `import_arrow` surface
anywhere under `crates/` — only a DuckDB docs URL in
`docs/jira/v26.11.1/waves/research-duckdb-arrow.md:192`. Close: cgap lands
the header signature, arcus switches import path, arcus KD-1 flips CLOSED
with a post-close receipt.

## OI-3 — cgap KD-1 (own register) not yet flippable

cgap's own register row KD-1 (version-refusal status-word collision, OE-4)
still cannot flip CLOSED: the L2/L3 court legs were not re-pinned to the new
word and no sidecar wire-probe witness receipt exists
(`docs/jira/v26.11.4/KNOWN_DIVERGENCES.md` FINDING KD-RC-1). Close: mint the
witness receipt (sidecar wire probe asserting `0x4347A009` on version skew +
re-pinned L2/L3 legs), then flip.

## OI-4 — e2e receipt conservation asymmetry worth a look (LOW)

`reports/e2e_cross_subsystem_receipt.json`: `admitted == pruned + cleared`
holds (355,824 == 355,468 + 356), but the producer refused 644,176/1,000,000
frames (64.4%) — the SPSC membrane refused at saturation, lawful per regime 3,
but the accepted-ratio is undocumented in the court script. LOW severity;
close: court script prints/documents the expected admission ratio band or
asserts it.
