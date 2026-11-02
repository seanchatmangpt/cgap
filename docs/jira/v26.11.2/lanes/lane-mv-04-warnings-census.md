# lane-mv-04-warnings-census

Lane MV-4. Repo `/Users/sac/cgap`, branch main. No git commands run.

## Subject

Clean-target warnings census of `cargo build --workspace --all-targets`
after FIX-07. Gate: zero errors (met). Stretch (zero warnings) not
reachable from this lane's file set: every remaining warning is on an
in-flight or out-of-scope surface.

## FIX-07 verification (read this session)

- `crates/bench/benches/chains-parquet.rs:467` — `let block_len =
  sealed.bytes().len();` (named binding, consumer intact). Landed.
- `crates/bench/benches/chains-seal.rs:490` — `let prev_root` (no
  `mut`); `:491` — `let mut lease` (E0596 repair intact). Landed.

## Build (cold lease)

`CARGO_TARGET_DIR=/tmp/mv4-target cargo build --workspace
--all-targets` — lease verified COLD before the run (`ls /tmp/mv4-target`
→ "No such file or directory"), so no incremental cache could hide
warnings.

| Command | Exit |
|---|---|
| python3 rmtree /tmp/mv4-target (pre-clean) | 0 |
| `cargo build --workspace --all-targets` (CARGO_TARGET_DIR=/tmp/mv4-target) | **0** |

## Anti-vacuity

A cold cache plus 99 `^warning` lines in `/tmp/mv4-build.log` (864-line
log) — the census is non-vacuous by construction: a cached/vacuous run
would show zero findings on a warm target dir; freshness is witnessed
by the pre-run absence of the lease dir. The stated mutation ("a
warning in a writable bench file must be fixed or disclosed") had zero
instances to flip — no bench-file warning exists to mutate.

## Warnings census (full, exit 0, 0 errors)

Summary: `cgap-git` lib ×10, cgap-git courts ×72, `bench` (manifest) ×3
unused deps (`cgap-ffi`, `cgap-git`, `cgap-storage`).
**Zero warnings name any `crates/bench/benches/*.rs` file** (span-file
grouping over the build log, all 19 distinct span files enumerated
below).

cgap-git lib (in-flight FIX-06 wire-in surface — left per dispatch):

- `src/egress_ring.rs` ×23 spans (`#![no_std]`-at-non-root ×2,
  deprecated `fetch_update`, dead items: `PKT_BUF_LEN`, `err_message`,
  `is_flush`, `encode_pkt`, `HEX`, `encode_flush`, `encode_delim`,
  `encode_response_end`, `encode_err`)
- `src/pkt_line.rs` ×8 spans (dead: `SEAL_REGISTER_ADDR`, `is_empty`,
  unused Result ×3)
- `src/ref_store.rs` ×1, `src/mcp_driver.rs` ×1, `src/delta_resolver.rs`
  ×4, `src/object_store.rs` ×5, `src/tree_synth.rs` ×3,
  `src/commit_synth.rs` ×3, `src/pack_demux.rs` ×2,
  `src/ingress_ring.rs` ×2, `src/admission_gate.rs` ×2

cgap-git court tests (GIT-16 in-flight surface — left per dispatch):

- `tests/soc_full_chip_court.rs` ×8 spans / 46 warnings,
  `tests/pkt_line_court.rs` ×3, `tests/delta_resolver_court.rs` ×4,
  `tests/patent_court_git.rs` ×1

bench manifest (outside this lane's benches-only writable set;
pre-existing per lane-fix07 receipt):

- `crates/bench/Cargo.toml` — unused deps `cgap-ffi`, `cgap-git`,
  `cgap-storage` (3 warnings)

## Fixes applied

None. No warning names a file in this lane's writable set; every
finding is attributed above to FIX-06 (cgap-git lib), GIT-16 (cgap-git
courts), or the bench manifest (not in the dispatch's file set).
Per-file disclosure is the census itself; LOUD-PENDING for the cgap-git
surfaces.

## Standing: ALIVE

Zero errors witnessed this session on a verified-cold lease. Lease
`/tmp/mv4-target` deleted at lane end (python3 rmtree below, before
receipt land). Divergence from dispatch: none — the in-flight
conflict-check instruction self-corrected in the dispatch to
mtime-vs-start comparison; not needed, no error occurred in any file.
