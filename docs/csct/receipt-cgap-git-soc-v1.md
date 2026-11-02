# CSCT receipt: cgap-git SoC full-chip v1

- emitted_by: crates/cgap-git/tests/soc_full_chip_court.rs (lane GIT-16)
- standing: ALIVE (court green this run)


## Stages (all green)

| stage | module | result |
|---|---|---|
| corpus | in-court splitmix64, seed 0xA20C0FFEE5A7E000 | OK, 1000000 facts |
| pack window serialization | in-court builder (pack_write::sha1 law) | OK, 733 windows |
| ingress SWAR relay | ingress_ring.rs 32768B ring, push_sieved | OK, 34023456 bytes relayed |
| pack demux | pack_demux.rs streaming Demux | OK, objects_seen verified per window |
| closure | object_store.rs BLAKE3 ledger (cap 256 x 128B) | OK, 128 distinct / 999872 deduped |
| delta resolver | delta_resolver.rs apply_delta | OK, 1 target |
| tree synth | tree_synth.rs synthesize + verify_canonical_order | OK, 11812 B |
| egress projection | egress_ring.rs real ring push/copy_into/advance_tail + receipt frame | OK, 11912 B |
| receipt seal | real cgap-blake3::hash | OK, root 3d1c80d411301988b5bd18ef88e71d4b7fa7d52833505f46684388510c3a5711 |


## Scale (honest)

- facts streamed: 1000000 (1,000,000)
- windows: 733 (732 x 1365 + 1 x 820)
- retained closure objects: 128 (store cap 256)
- heap allocations in window: 0 (positive control observed 1)
- stack canary 8192B 0xDEADBEEFCAFEBABE: byte-unchanged


## Mutation courts (all flipped)

- A: pack magic corrupt at ingress boundary -> DemuxError::SubjectMissing (typed)
- B: body-byte flip -> court-side trailer law refusal
- control: unmutated pack accepted


## Loud typed exclusions

- commit_synth.rs / admission_gate.rs / patent_court.rs / retract.rs / ref_store.rs
  landed after this pipeline froze -> NOT integrated (loud, in-scope next wave)
- bn3 closure not reachable via #[path] (foreign crate; closure = object_store.rs BLAKE3 ledger)


## Transcript root

`3d1c80d411301988b5bd18ef88e71d4b7fa7d52833505f46684388510c3a5711`
