# lane-dod-08-readmes

PUB-06 remediation lane (2026-11-02). Repo /Users/sac/cgap, branch
main. No git write commands run; shared tree touched only in the three
owned files + this receipt.

## Files written

- `crates/cgap-git/README.md` (rewrite, 88 lines): honest identity
  naming the real 23-module surface (pack codec, DAG/ladder,
  ingress/egress rings, pack_demux, delta_resolver, object_store,
  ref_store, tree_synth/commit_synth, admission_gate, patent_court,
  retract, octopus_merge, mcp_driver, a2a_swarm, pkt_line, pack);
  mandatory section order per `audit/pub-06-readme-plan.md` §3
  (Status / Capabilities receipt-cited / Honest failures / docs.rs
  matrix / Install / Lane receipt). Numbers: 89/0 (crate-reality
  census), mutation flips (a03 receipt), 4 ns/frame ingress
  (lane-git-03:66, marked measured-once vs 10 ns TARGET), 1.873 ms
  octopus (lane-git-12:60, debug-profile flagged).
- `crates/cgap/README.md` (re-census, 181 lines): version 26.11.2,
  crate count re-derived from `cargo metadata` this session = 23
  workspace packages (new member cgap-branchless row added,
  publish=false per pub-01 census); membership correction cited to
  `audit/pub-01-manifest-census.md` (21 crates, 17 true/4 false,
  default-publishable set empty); test counts re-witnessed from
  `waves/crate-reality/*.md` (arrow 134, blake3 27/1, git 89/2,
  parquet 222, bont 35, bosym 14, crypto 25=1+18+6, core 24, storage
  5 target-readout, bn3 119, gate BROKEN-MID-EDIT 22 errors 0 ran,
  p28 8 target-readout, ffi 56, audit 37, court 14 courts, beam real
  escript); rows without a 26.11.2 census carry 26.11.1 counts with
  explicit as-of markers (cloud, fibo, integration). Added
  Honest-failures (gate compile-abort, stardf VACUOUS-DEFAULT,
  wasm-footprint 1/5 court fail) and docs.rs matrix sections per
  pub-06 §4; chains-ingress PARETO now carries the 10.1x receipted /
  9.6x fresh dual citation (tps-01:24).
- `crates/bench/README.md` (append, 3 lines): freshness qualifier on
  the 102.7 ms chains-ingress row completed — DRIFT-not-refutation
  wording, fastest 10.70 vs receipted 10.20/10.30, <15 ns/frame
  target met, cite-both instruction
  (`audit/tps-01-reality-ledger.md:24`).

## Commands (this session)

| command | exit |
|---|---|
| `cargo metadata --no-deps` -> python count of packages | 0 (23 packages) |
| `grep -n "64-byte frame\|5.70"` over the three files | 1 (zero matches) |
| `awk 'length > 100'` sweep over the three files | 0 matches after table-row tightening |

## Verification

- Re-read all three files after edit; on-disk content matches intent
  (no corruption; all writes in <=50-line chunks or python-assembled).
- Pub-06 §5 post-pass satisfied: <=100-char lines, no emoji, every
  number carries a receipt or as-of citation.

## Fences

`pub-10-claims-preflight.md` README perf-sentence table is the fence
for this diff: the bench 102.7 ms row now carries the exact
qualifier pub-10 required; no unqualified "5.70 ns", bare seals/s, or
"64-byte frame" introduced (grep rc=1, zero matches). No manifests,
no lib.rs, no generated artifacts touched.

## Divergences and disclosures

- The bench README freshness qualifier lines 32-33 were ALREADY
  present when I applied my edit (a terse two-line form). I completed
  the existing qualifier rather than appending a duplicate. If another
  lane landed those lines mid-flight, that lane's receipt supersedes
  the terse wording; my completion is a superset (adds DRIFT wording,
  10.70/10.20 figures, cite-both). Disclosed here per the
  disclose-in-both-directions law; no other lane's file touched.
- pub-01 flags cgap umbrella manifest `readme` field MISSING (PUB-01
  owns manifest edits — not touched here, reported per pub-06 §6).

## Standing

ALIVE for the README surface: every claim in the three files is
re-read from a receipt on disk this session; commands above ran this
session with exits shown. Test counts are census-cited, not re-run by
this lane (re-running 20 suites is PUB-06 non-goal scope; the
censuses are same-week same-session artifacts). Perf numbers remain
receipt-cited / as-of-date, never rounded up.
