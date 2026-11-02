# W3-R2: boundary --deps/--strings repair receipt

Repo: /Users/sac/cgap @ main (6801affa). Writes: `scripts/boundary_gate.py`
(deps arm only) + this receipt. Read: w2-08-gate-reruns.md first.

## Deps-arm law repair (W2-20 BLOCKED item 2a)

Old law: any dep spec with a `version` key = EXTERNAL. Proven WRONG against
shipped reality: crates.io serves cgap-storage 26.11.1 with `cgap-blake3
^26.11.1` as a NORMAL registry req and cgap-p28 26.11.1 with
`cgap-blake3` + `cgap-crypto ^26.11.1` normal (crates.io API v3
`/dependencies`, fetched this session); those in-repo declarations are the
cargo publish pattern `{ path = "...", version }`. New law: EXTERNAL iff
`git`/`registry` source, registry-only `version` (no path), or a `path`
that resolves OUTSIDE the repo. In-repo paths = workspace-internal.

Result: `python3 scripts/boundary_gate.py --deps --strings --target-dir
/tmp/w3-r2-target` — deps arm GREEN: "DEPS GATE: E=0 runtime externals
across 15 covered crates (§6.4)"; all 9 W2-08 hits reclassified
workspace-internal. Overall exit 1 remains (strings arm, below).

## Strings-arm A07 scan (2b) — diagnosis, no emitter edit

The scanned=0 crates were NOT manifest-sensitive. The canonical emitter
(`zero_string_gate.py cmd_scan`) refuses typed
`crate-no-artifacts`/`execution-crate-no-artifacts` when a member crate has
zero built debug artifacts. After `cargo build --workspace` (see SLA note)
all 7 W2-08 scanned=0 crates scan >0 (bosym=1, cloud=1, fibo=2, git=1,
gpu=1, integration=1, storage=2). Boundary's fail-closed consumption was
CORRECT behavior both times.

## Remaining failure classification

- bench scanned=0 → emitter GateRefused → boundary consumption refused
  (exit 1): **SESSION-INTRODUCED (cross-lane transient)** — cgap-bn3 lib
  does not compile (9 rustc errors incl. unclosed delimiter, in-flight
  lever work by another lane), and bench depends on cgap-bn3, so bench
  artifacts cannot be built this session. bn3 itself scans 2 from stale
  artifacts. Resolution owner: bn3 lane + a follow-up `cargo build -p
  bench`; the gate script needs no change.
- The ffi E0063 (FrameView `escalate`/`prefilter` missing) was unblocked
  under the compile-freeze SLA: 7-line init with documented zero-defaults
  in crates/cgap-ffi/src/sovereign.rs (~line 426). DISCLOSE-IN-BOTH:
  this fix is for the bn3/ffi lever lane's benefit; name it in that
  lane's receipt too.

## Anti-vacuity (repaired arm still kills)

Sandbox /tmp/w3r2-sandbox (manifest-only copy, shared tree untouched):
- BASELINE `--deps` → exit 0, E=0 across 15 crates.
- MUT-1 `serde = "1.0"` injected into cgap-blake3 → exit 1,
  "EXTERNAL: serde", "DEPS GATE FAILED: 1 runtime external(s)".
- MUT-2 outside-repo path dep (`path = "/tmp/w3r2-outside"`) → exit 1,
  "EXTERNAL: serde,outside". Both flips witnessed; restore N/A (pure
  /tmp sandbox, deleted).

## Cleanup

`shutil.rmtree` exit-clean: /tmp/w3r2-sandbox, /tmp/w3r2-outside,
/tmp/w3-r2-target (target lease, per lease law).
