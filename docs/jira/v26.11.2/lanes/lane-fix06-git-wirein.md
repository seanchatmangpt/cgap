# LANE FIX-06 — cgap-git module wire-in (coordinator integration)

Date: 2026-11-02. Lane: FIX-06. Branch: main. Standing: **ALIVE**.

## Scope

Contract: write ONLY `crates/cgap-git/src/lib.rs` (additive `pub mod`
lines) + this receipt. No git commands. 15 GIT-lane modules landed via
`#[path]` courts; lib.rs did not declare them. The crate is
`publish = true`; additive pub API by design.

## Wired (15 modules → 14 mounted, 1 WIRE-PENDING)

| module | status |
|---|---|
| pack_demux, pkt_line, ingress_ring, delta_resolver, egress_ring, commit_synth, admission_gate, patent_court, retract, octopus_merge, ref_store, mcp_driver, a2a_swarm, tree_synth | **MOUNTED** as `pub mod` (lib.rs:145-160 post-format) |
| object_store | **WIRE-PENDING(lane-git-05)** — see below |

## WIRE-PENDING: object_store

`src/object_store.rs` hard-codes `cgap_blake3::hash` (object_store.rs:97,
:354). `cgap-blake3` is dev-dep-only (`crates/cgap-git/Cargo.toml:22`),
so lib mounting is E0433. Options are both outside this lane's file
ownership: (a) add a real `[dependencies]` entry (conflicts with the
zero-external-dep kernel law, .claude/rules/kernels-no-std.md) or
(b) an in-crate digest. Its `#[path]` court
(`tests/object_store_court.rs`) compiles the module independently and is
green (6 passed, both runs). The module line is commented out in lib.rs
with the `// WIRE-PENDING(lane-git-05)` marker.

## Commands + exits (this session, exact subject)

```
cargo build -p cgap-git            → exit 0 (after WIRE-PENDING edit)
CARGO_TARGET_DIR=/tmp/fix06-target cargo test -p cgap-git → EXIT1=0 (29 suites ok)
CARGO_TARGET_DIR=/tmp/fix06-target cargo test -p cgap-git → EXIT2=0 (29 suites ok, pass counts identical)
```

29 result lines per run include the octopus mutation court's intentional
child re-exec (expected exit-101 panic, `GIT12_ANDON_CHILD=1`); its
output interleaves into the log as a "FAILED ... 0 passed; 1 failed"
line. This is the anti-vacuity witness, not a suite failure — parent
binaries all ok, cargo exit 0 both runs. (First pass I mis-read this
interleave as a real failure; corrected here, no tree change.)

## Fences run / not regressed

- `cargo build --workspace` not run (out-of-lane crates moving in
  concurrent lanes; coordinator runs it at integration).
- Build was green BEFORE any FIX-06 edit? No — the pre-FIX-06 lib.rs did
  not declare the 15 modules; the crate built green with 5 modules
  (deflate, pack_write, idx_read, ladder, inflate, dag). My diff is
  additive pub mod lines; the only non-additive element is the commented
  object_store line, which is itself new.
- `tests/soc_full_chip_court.rs` (GIT-16) is a tests/ file, no lib.rs
  change needed — ran green (1 passed) both runs.

## Divergences

- Dispatch expected all 15 modules mountable; 14/15 mounted. object_store
  blocked on a Cargo.toml dep decision owned by the coordinator/GIT-05
  lane. LOUD, marker in lib.rs, court stays green via #[path].
- Temporary `// TEMP-DIAG(fix06)` probe of octopus_merge (comment-out +
  rerun) was applied and fully reverted; final tree has `pub mod
  octopus_merge;` mounted and the suite exit 0 on top of it.
