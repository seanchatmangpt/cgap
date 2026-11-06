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

STATUS: CLOSED (2026-11-06). `cgap_import_arrow` and `cgap_read_arrow_ipc`
landed in `cgap-arrow` / `cgap-ffi` with prototypes in `include/cgap.h`.
Receipt: `docs/jira/v26.11.6/kd1-surface-receipt.md`.

## OI-3 — cgap KD-1 (own register) not yet flippable

STATUS: CLOSED (2026-11-06). Witness receipt minted at
`docs/jira/v26.11.6/kd1-witness-receipt.md` citing `aoq_flip_ladder_court.rs`
L2 leg asserting `0x4347A009`. Divergence flipped to CLOSED in
`docs/jira/v26.11.4/KNOWN_DIVERGENCES.md`.

## OI-4 — e2e receipt conservation asymmetry worth a look (LOW)

`reports/e2e_cross_subsystem_receipt.json`: `admitted == pruned + cleared`
holds (355,824 == 355,468 + 356), but the producer refused 644,176/1,000,000
frames (64.4%) — the SPSC membrane refused at saturation, lawful per regime 3,
but the accepted-ratio is undocumented in the court script. LOW severity;
close: court script prints/documents the expected admission ratio band or
asserts it.
