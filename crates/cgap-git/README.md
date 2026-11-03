# cgap-git — zero-dep no_std Git substrate kernel (pack v2, rings, DAG, courts)

Zero-external-dep, `#![no_std]`, `#![forbid(unsafe_code)]` Git kernel
for the cgap sovereign columnar substrate: 23 src modules spanning the
packfile v2 codec (deflate/inflate, pack_write/idx_read, pkt_line,
pack), the DAG ladder (dag, ladder), the wire-in/wire-out rings
(ingress_ring, egress_ring), pack demultiplexing and delta resolution
(pack_demux, delta_resolver), the object surface (object_store,
ref_store, tree_synth, commit_synth), and the court/protocol membranes
(admission_gate, patent_court, retract, octopus_merge, mcp_driver,
a2a_swarm). Dual-register design: the git-compat register
(`write_pack_v2`, git-identical parsers, `git_object_oid`, SHA-1) and
the receipt register (`commit_blocks`, content-addressed block
receipts) — one real snapshot round-trips through both registers with
cross-register OID-set binding, and any one-byte mutation moves the
encoded root address.

## Status

WORKING-26.11.2 — kernel source at workspace version 26.11.2 (publish
law: one attempt per crate, sequenced; prior PUBLISHED-26.11.1 record
archived under readme-archive/). Census:
`docs/jira/v26.11.2/waves/crate-reality/cgap-git.md` — full suite
exit 0 this session at 26.11.2.

## Capabilities (receipt-cited)

- Suite 89/0 (61 inline + 28 integration, 2 loud ignored):
  `cargo test -p cgap-git` exit 0 —
  `waves/crate-reality/cgap-git.md`.
- Dual-register round-trip court
  `facade_court_both_registers_round_trip` (pack.rs): both
  content-addressing registers on one snapshot — same census.
- Three mutation flips, all targeting real courts, sha256 byte-exact
  restores (`waves/a03-finish-receipt.md`): SHIM-OFFSET (d9 flipped +
  3 wire-leg collateral, 56/60), SHIM-STABILITY (d1 flipped, 56/60),
  SHIM-PARENT (exactly d3 + d8 flipped, 58/60, isolated).
- Ingress ring: measured 4 ns/frame vs the 10 ns/frame TARGET
  (`lanes/lane-git-03-ingress-ring-receipt.md:66` — number is a
  TARGET-class claim, cited as measured-once, not a fence).
- Octopus merge: 1.873 ms debug-profile merge, under the 33.3 ms
  release-frame budget (`lanes/lane-git-12-octopus-merge.md:60`;
  debug-profile, not a release measurement).
- Module lanes: pack demux (git-01), pkt_line (git-02), ingress ring
  (git-03), delta resolver (git-04), object store (git-05), tree
  synth (git-06), egress ring (git-07), commit synth (git-08),
  admission gate (git-09), patent court (git-10), retract (git-11),
  octopus merge (git-12), ref store (git-13), mcp driver (git-14),
  a2a swarm (git-15), SoC court (git-16) — receipts under
  `docs/jira/v26.11.2/lanes/`.

## Honest failures and caveats

- Two ignored tests (`ladder_timing_stress`, `perf_baseline`) are
  loud perf/timing skips, not passes — census
  `waves/crate-reality/cgap-git.md`.
- SHIM-OFFSET flipped 3 wire-leg tests beyond its target (56/60):
  collateral is disclosed, not hidden — `a03-finish-receipt.md`.
- The 4 ns/frame ingress figure was measured once in the lane run;
  the standing target remains 10 ns/frame. No fresh re-derivation at
  26.11.2 — treat as receipt-cited, not re-witnessed
  (`lane-git-03-ingress-ring-receipt.md:66`).
- Octopus merge figure is debug-profile
  (`lane-git-12-octopus-merge.md:60`); release-profile cost is
  unmeasured here.

## docs.rs build matrix

Stable toolchain, edition 2021, no `.cargo/config.toml` rustflags —
docs.rs x86_64-linux default-features `--docs` builds clean with no
target features required. This crate: `#![no_std]` body, optional
`std` feature; SHA-1, CRC32, and DEFLATE implemented in-crate, so no
cross-platform backend selection affects the docs build
(audit `pub-06-readme-plan.md` §4).

## Install

```bash
cargo add cgap-git@26.11.2
```

## Lane receipt

`docs/jira/v26.11.2/lanes/lane-dod-08-readmes.md` (this rewrite);
census `docs/jira/v26.11.2/waves/crate-reality/cgap-git.md`;
historical: `docs/jira/v26.11.1/waves/a03-finish-receipt.md`,
`waves/a03-wave5-audit-receipt.md`, `waves/p3-api-verify-all.md`,
and lane receipts `lanes/lane-git-01` .. `lane-git-16`.
