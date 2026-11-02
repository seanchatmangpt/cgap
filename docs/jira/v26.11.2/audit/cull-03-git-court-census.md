# cull-03-git-court-census

CULL-03 cgap-git court census (2026-11-02, read-only): 31 files,
10,608 LOC, each read per-test with line cites.

## Verdict summary

UNIQUE: 29 of 31 files. OVERLAP-PARTIAL: posh_wire_court.rs
(merge the scatter-gather composition test :143-197 into
ingress_ring_court; keep the magic/stride flip courts — the
magic-flip demux cross-check :225-239 is the only wire-side
assert of the SubjectMissing law). REDUNDANT-leaning single
test: pack_demux_court c7_parse_path_is_zero_alloc (:346-368)
— its dynamic claim is partially duplicated by c8 static
source scan + soc's whole-pipeline zero-alloc window; cull or
harden if the flake persists (flag 2). No fully REDUNDANT
files; no duplicate whole files.

Clean three-way split confirmed: fuzz_codec (object+zlib
codecs) / packfile_fuzz (pack framing + git agreement) /
pack_demux_court (hand-computed demux laws) do not overlap.
object_store_court vs soc stages: complementary (soc asserts
integration laws only; eytzinger/search/digest-stability laws
live solely in object_store_court).

## Intermittent courts

Flag 1 — octopus mutation_conflict_injection_flip (:296-373):
GAP-09 fixed the :309 index bug; GAP-08 still saw ~50% fails
at :366 — the failure mode is the self-re-exec + --test-threads-1
child run, not the merge logic. The typed-refusal half
(MergeError::DomainClash match :329-337) is asserted twice
in-file; the andon-101 exit arm is the flaky half.
Recommendation: split the 101-exit witness into its own
#[ignore]-gated test; keep the in-process DomainClash arm.

Flag 2 — pack_demux c7 zero-alloc (:346-368): SERIAL mutex
protects within this binary only; false NON-zero from foreign
allocations (parallel binary or spawned thread) is the failure
mode. Distinct value = isolating demux alone; soc + c8 cover
the law otherwise.

## See Also

audit/cull-01-gate-overlap-matrix.md ·
lanes/lane-gap-09-octopus.md · lanes/lane-dx-06-fixture-races.md
