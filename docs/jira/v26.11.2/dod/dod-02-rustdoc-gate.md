# DOD-02 — rustdoc zero-warning gate

Lane: DOD-02. Date: 2026-11-02. Contract: read-only gate; only this file
written. No git commands. Build lease: /tmp/dod02-target (deleted at lane
end).

## W1 — Verdict

**FAIL at 7 crates** — acceptable DoD receipt: every error typed with
file:line. Honest count; zero-defect not required. The dispatch's prior
API-429-aborted run said "three crates"; re-derived this session: **7
crates, 31 located errors** (the "3" was partial-log hearsay — recorded as
divergence).

## W2 — Gate command + exit

```
RUSTDOCFLAGS="-D warnings -D rustdoc::broken_intra_doc_links" \
  CARGO_TARGET_DIR=/tmp/dod02-target \
  cargo doc --workspace --no-deps --document-private-items
EXIT=101
```

Log: /tmp/dod02-doc.log (lane-ephemeral; contents frozen in W3).

## W3 — Per-crate error census (31 located errors, 7 failing crates)

Classification key: PIL = private-item link (private-item doc lints),
BIDL = broken intra-doc link, REL = redundant explicit link target,
AMB = ambiguous (function-vs-primitive).

### cgap-arrow (11)
| file:line | class | error |
|---|---|---|
| blake3.rs:161:7 | PIL | `hash_chunk` -> private `hash_chunk_window` |
| blake3.rs:222:10 | PIL | `hash` -> private `hash_chunk_window` |
| slice.rs:5:51 | REL | redundant explicit link target |
| slice.rs:7:59 | AMB | `slice` function-vs-primitive ambiguity |
| slice.rs:15:26 | AMB | same ambiguity class |
| slice.rs:46:52 | REL | redundant explicit link target |
| slice.rs:48:32 | AMB | same ambiguity class |
| slab.rs:26:42 | BIDL | `cgap_core::frame::CgapFrameHeader` unresolved (cgap-arrow does not depend on cgap-core) |
| slab.rs:27:11 | BIDL | `cgap_gate::penstock::ingress` unresolved (no cgap-gate dep) |
| slab.rs:28:7 | BIDL | `cgap_storage::atomic_commit` unresolved (no cgap-storage dep) |
| foreign.rs:12:11 | BIDL | `ForeignArray` unresolved |

### cgap-bn3 (2)
| file:line | class | error |
|---|---|---|
| arena.rs:32:8 | PIL | `arena` -> private `header_index` |
| unify.rs:13:23 | PIL | `unify` -> private `SplitStore` |

### cgap-bont (4)
| file:line | class | error |
|---|---|---|
| primitives.rs:23:36 | BIDL | unresolved link `2` (numbered-list text parsed as link) |
| shacl_compiler.rs:29:71 | BIDL | unresolved link `8` |
| shacl_compiler.rs:376:69 | BIDL | unresolved link `8` |
| shacl_compiler.rs:826:35 | PIL | `malformed` -> private `KNOWN_PROP_LOCALS` |

### cgap-git (6)
| file:line | class | error |
|---|---|---|
| dag.rs:82:7 | BIDL | `wave9_powercut_sweep_dag_wire` unresolved |
| admission_gate.rs:12:46 | PIL | module doc -> private `pkt_line::PktError` |
| admission_gate.rs:132:50 | PIL | `Pkt` -> private `PktError` |
| ref_store.rs:17:19 | BIDL | `dependencies` unresolved |
| retract.rs:22:44 | BIDL | `F` unresolved |
| retract.rs:127:10 | BIDL | `f` unresolved |

### cgap-p28 (1)
| file:line | class | error |
|---|---|---|
| asc450.rs:356:19 | BIDL | `Lease` unresolved |

### cgap-parquet (5 workspace / 6 per-crate)
| file:line | class | error |
|---|---|---|
| byteslice.rs:20:7 | PIL | `byteslice` -> private `eq_mask_word` |
| byteslice.rs:84:7 | PIL | `scan_eq_byteslice` -> private `eq_mask_word` |
| scan.rs:321:44 | PIL | `rg_prunes` -> private `read_statistics` |
| scan.rs:329:12 | PIL | `rg_prunes` -> private `PhysType::from_id` |
| footer.rs:62:24 | BIDL | `MAX_ROW_GROUPS` unresolved |
| mmap_writer.rs:603:53 | BIDL | `FileSink` unresolved — per-crate re-run ONLY; divergence W6 |

### cgap-storage (2)
| file:line | class | error |
|---|---|---|
| dred.rs:52:10 | BIDL | `f` unresolved |
| closure.rs:103:35 | BIDL | `j` unresolved |

## W4 — Auxiliary checks

### W4a — crate-level `#![doc]`-equivalent (`//!`) headers, publish-intended crates

Publish-intended set = crates WITHOUT `publish = false` in Cargo.toml,
excluding registry-absent/non-member `cgap-verification` (13 crates).
**All 13 carry a `//!` crate-level doc header** — zero lacking. The 10
declared `publish = false` crates (bench, beam, court, ffi, gate, core,
storage, ocel2, fleet-ops, audit) are out of scope for this check; 10 of
those also carry headers (spot-read, not censused).

### W4b — `#![no_std]` headers

Present (10): cgap, cgap-arrow, cgap-blake3, cgap-branchless, cgap-crypto,
cgap-gate, cgap-git, cgap-p28, cgap-parquet, wasm-footprint.
Absent (15): audit, cgap-beam, cgap-bn3, cgap-bont, cgap-bosym,
cgap-core, cgap-ffi, cgap-fleet-ops, cgap-gpu, cgap-ocel2, cgap-storage,
cgap-verification, cli, csct, integration.

### W4c — raw unescaped `<` angles in `///` lines (grep census)

Python census over `crates/*/src/**/*.rs`: strip code spans, match
`<` + letter. **4 hits**, none inside code fences:
- crates/cgap-parquet/src/mmap_writer.rs:1603 — `<N arms.`
- crates/cgap-core/src/isomorphism.rs:40 — `<t` / `<i` (two hits)
- crates/csct/src/bin/csct_runner.rs:37 — `<accepted`

None of these four produced a rustdoc error in W3 (different crates /
lines), so they are latent, not currently failing.

### W4d — cfg-annotation rendering spot-check (5 crates, per-crate runs)

`cargo doc -p <crate> --no-deps --document-private-items`,
RUSTDOCFLAGS="-D warnings", /tmp/dod02-target:

| crate | exit |
|---|---|
| cgap-core | 0 |
| cgap-bn3 | 101 |
| cgap-p28 | 101 |
| cgap-parquet | 101 |
| cgap-git | 101 |

cgap-core renders clean (its `#[cfg]`-gated items render under
`--document-private-items` with zero warnings); the four failures
reproduce exactly the W3 error sets for those crates (no new error
classes).

## W5 — Fences (must-not-regress)

- Root `cargo build --workspace` — NOT rerun by this lane (read-only
  doc gate; no source edits made, so no build-regression surface
  exists; the doc gate does not touch rlib artifacts).
- No prior court/gate diffs touched: zero source files written; only
  this receipt under docs/jira/v26.11.2/dod/.
- Seed constants, byte-pins, published kernels: untouched.

## W6 — Divergences

1. **Dispatch said "three crates"; gate re-derived 7.** The 429-killed
   prior run's "3" was captured from a partial log before abort; this
   session's full run shows 7 failing crates. Full census in W3.
2. **Per-crate vs workspace parquet delta**: `-p cgap-parquet` run
   surfaced `mmap_writer.rs:603:53 unresolved link FileSink` which the
   workspace run's log does not contain. Cause UNVERIFIED (likely
   rustdoc cross-crate resolution nuance vs. cache state at
   --workspace time). Flagged, NOT fixed (read-only lane).
3. W4a scope: `//!` header census run only over the 13 publish-intended
   crates; the 10 publish=false crates spot-read only.

## W7 — Standing

Standing: **PARTIAL** — gate ALIVE and honestly FAILING (exit 101,
command + exit witnessed this session); receipt landed. Errors typed
with file:line per contract; no fixes applied (read-only lane; fixes
route to a repair lane).

## W8 — Replay

```
RUSTDOCFLAGS="-D warnings -D rustdoc::broken_intra_doc_links" \
  CARGO_TARGET_DIR=/tmp/<fresh-lease> \
  cargo doc --workspace --no-deps --document-private-items
# expect: EXIT=101, 7 "could not document" crates, 31 located errors (W3)
```
