# Lane DOD-10 receipt — docs/ARCHITECTURE.md

Date: 2026-11-02. Contract: single owned file `docs/ARCHITECTURE.md`
(created; did not exist — `ls` ENOENT witnessed pre-write). No git
commands. Standing vocabulary per repo CLAUDE.md.

## W1 Subject

`docs/ARCHITECTURE.md` — 218 lines, sha256
`b3e64f1cf5b6ae7995bcc640cb4e0d05b2b7dde0cda83ad75d2d29a92cb34239`.
H1 matches filename; 6 H2 sections; 0 lines over 100 chars; written in
3 chunks <50 lines; token-swept on disk (python line/section sweep,
exit 0).

## W2 Diff

New file, index-not-duplicate topology doc: (1) tier-by-tier crate DAG
from `cargo metadata --no-deps` run this session (exit 0); (2)
six-stage memory-hierarchy dataflow (pack_demux -> ingress_ring ->
penstock 6-row taxonomy -> BLAKE3 receipt chain -> Arrow/Parquet ->
OCEL 2.0 Spoliation-verifying projection); (3) host-side operational
contract (SPSC sharding/affinity as host obligation, TARGET-carry,
CONTENTION-SUSPECT consumption law, exit-code ABI 0/1/2/3, status
registers); (4) perf register pointers (never restates numbers);
(5) deep-doc pointer index; divergences section.

## W3 Commands (this session)

- `cargo metadata --no-deps` (via python dep extraction) -> exit 0,
  DAG derived from output.
- `ls docs/ARCHITECTURE.md` -> ENOENT (pre-write existence check).
- Cite-verification greps/seds (all exit 0):
  penstock.rs:505-513 six-row table read;
  receipt_chain.rs:1-26 read; ingress_ring.rs:1-25 + RefusalQuota
  :114/:131 read; pack_demux.rs:1-30, DemuxError :58 read;
  ocel2 lib.rs:1-40 read; sovereign.rs:71-90 consts read;
  bitslice.rs:687-699 seed pair read; mmap_writer.rs RingSink :721
  read; bench-protocol §8 (:177) read; fleet-ops + fix-14 read;
  dflss/17-c4 read in full.
- Post-write sweep: 218 lines, 0 over-length, sections intact, exit 0.

## W4 Verification ladder

Doc lane: no build/test gate applies. Verification = every file:line
cite re-read on disk this session; perf figures appear ONLY as
register-row pointers (rows 5/8/28/10/29 of `11-measured-claims.md`)
with their receipt cites, never as local restatements.

## W5 Courts

None runnable against a prose doc; anti-vacuity is the cite law: every
claim names file:line or register row. No court skipped — nothing in
scope for one. Loud statement per skip law: [SKIP] no court applies to
a markdown index doc (no executable surface).

## W6 Standing

ALIVE — file on disk (sha256 above), all cites re-read from disk this
session, commands with exits listed. Performance figures carried only
as register citations; the affinity design note is [TARGET] (stated
in-doc, provenance divergence recorded).

## W7 Divergences (also recorded in-doc)

1. Dispatch cited lever IDs OPT-05/OPT-06; grep over docs/ + crates/
   finds NO on-disk referent (only OPT-03,
   `bench-validate/opt-03-channel-curve.md`). Doc yields to tree:
   zero-copy cited directly to `pack_demux.rs:20-24`; affinity
   contract carried as [TARGET] with stated provenance.
2. Dispatch's "dflss/17" resolved to
   `docs/jira/v26.11.2/dflss/17-c4-architecture.md` (orphan redirect
   per DF-2 at that file's :3-6).
3. `06-bench-protocol.md` has NO §10 (sections end at §8 + JSON
   footer); the loud-skip exit-3 class lives at `07-court-corpus.md:13`
   and `09-overnight-mutation-set.md:19` — first draft cited the wrong
   doc, corrected by Edit before landing (not an errata; pre-receipt
   fix).
