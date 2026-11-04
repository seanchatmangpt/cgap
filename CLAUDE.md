# cgap

Sovereign `#![no_std]` columnar/deductive substrate; 25-member root
workspace at 26.11.3 by `cargo metadata` (23 `crates/` members + 2
ontology packs); registry census (zd-119, 2026-11-04): cgap-aoq is
NAME-FREE (first-publish candidate), wasm-footprint max 26.11.1, all
other censused publish=true crates at 26.11.2, 26.11.3 free everywhere
(`bench` there is a name
collision, not ours; `cgap-verification` is on disk, not a root member;
`cgap-integration`/`cgap-court`/`cgap-beam` carry `publish = false`).
cgap-datafusion was removed
(`docs/jira/v26.11.1/waves/cgap-datafusion-removal.md:6`) — do not
re-scaffold. Campaign: v26.11.2 starDF (`docs/jira/v26.11.2/
00-ard-prd.md`; runbook `OVERNIGHT-LOOP.md`, wave plan `05-wave-plan.md`).

## Commands

```bash
cargo build --workspace                 # root workspace must stay green
cargo test -p cgap-court                # court_verdict harness
cd crates/cli && cargo build && cargo test   # standalone ggen CLI (route proofs + mutation_kills RSS courts)
just doctor                             # 10 checks; YELLOW = informational (exit 1 by design)
python3 scripts/boundary_gate.py        # campaign boundary gate (fail-closed)
python3 scripts/zero_string_gate.py     # zero-string gate (--self-check = non-vacuity proof)
just doctor/status via justfile         # side-bins run with --manifest-path crates/cli/Cargo.toml
cd crates/cli && rm Cargo.toml && ggen sync run && python3 manifest_patch.py   # CLI regen (byte-identical replay)
make gate0                            # ggen sync x2 byte-identical replay + freestanding header compile
                                      #   FM-PACK-008 fix: rm ggen.lock && ggen sync run (twice, cmp)
cargo bench -p bench --bench chains-ingress -- <stage_leg>   # exact leg name; divan omits
                                      #   all-zero alloc cols — absent means 0 only if courts corroborate
```

Gate exit codes: 0 pass / 1 gate fail / 2 subject missing / 3 loud skip.
A skip is loud, never a fake pass. A court that cannot flip is FAIL; a
mutation-free green gate run is VACUOUS (exit nonzero).

## Publish law (crates.io)

- ONE publish attempt per crate; sequenced post-window, never
  concurrent with lever lanes (`docs/jira/v26.11.2/OVERNIGHT-LOOP.md:146-149`).
- Registry slide (index reject or timeout): back off 1 hour. No retry
  storms.
- Registry census (zd-119 live census, 2026-11-04, 23 crates probed):
  cgap-aoq is NAME-FREE (first-publish candidate); wasm-footprint's
  registry max is 26.11.1 (it skipped 26.11.2); every other censused
  publish=true crate sits at 26.11.2; 26.11.3 is free everywhere —
  zero registry collisions (receipt:
  `docs/jira/v26.11.3/receipts/zd-119-cgap-preflight.md`). Historical
  2026-11-02 re-derivation table: `docs/jira/v26.11.2/
  01-release-gates.md:325ff` (prior "19/22 published" was UNVERIFIED
  registry state).
- `publish = false` (explicit, on disk, re-derived 2026-11-04): bench
  (registry name collision), beam, court (structural
  [[test]]-only), wire-shm (YD-3, 2026-11-04). gate/p28/crypto/
  branchless carry `publish = true` explicitly (branchless flipped to
  true by lane DOD-02, 2026-11-02: first-publish candidate,
  `lanes/lane-dod-02-branchless-pub.md`).
  25/25 members declare an explicit flag (21 true, 4 false) as of
  2026-11-04 (`cargo metadata --no-deps` + per-manifest grep; prior
  23/23 = 20 true / 3 false as of 2026-11-02, superseded by cgap-aoq
  and the two ontology packs joining the true set and wire-shm joining
  the false set). `cgap-mcp` (YD-3)/
  `cgap-ocel2`/`cgap-fleet-ops`/`cgap-verification`/`csct`/`cli`
  are on disk but NOT root members.

## CLI (ggen six-pack generated)

`crates/cli` is a STANDALONE ggen crate (own `[workspace]`, NOT a root
member). Consumer seams: `crates/cli/src/custom_handlers.rs` (the
cnv:CustomBehavior hole, filled once) and `crates/cli/manifest_patch.py`
(post-sync kernel deps + `[patch.crates-io]` to `~/clap-noun-verb`
26.9.1). Side-bins `doctor`/`status`/`verify_v26` are hand-maintained.

## Layout (pointers only — read the tree for detail)

- `crates/cgap-{arrow,blake3,git,parquet}` — published zero-dep kernels
- `crates/{cgap-core,cgap-storage,cgap-bont,cgap-bosym,cgap-bn3,
  cgap-gate,cgap-crypto,cgap-p28}` — moonshot lane crates
- `crates/{cgap-ffi,cgap-beam,cgap-gpu,audit,integration,cgap-court}` — membranes, court harness
- `crates/cgap-verification` — on disk, NOT in the 25-member `cargo metadata` list
- `crates/wire-shm` — root member, `publish = false` (YD-3),
  ORPHANED-DATUM-FROZEN: no production-data-path consumer;
  `cgap-mcp` consumes it as a runtime path dep (bridge surface),
  `cgap-aoq` as a dev-dep (AB-11 court surface); arcus p02/p09 name
  its bytes as compat datum (UNBUILT). Escalation ladder: parked
  raise = cached `next_ordinal` + direct-slice write (cgap-mcp
  `wire.rs` byte-identity court is the drift fence); sanctioned
  bounded change = 3-line payload-offset shift iff arcus p09
  selects the 64B header; delete-first applies only if cgap-mcp
  retires AND no arcus build lands in one release cycle.
- `crates/cli` — standalone ggen CLI (above); `ontology-packs/{fibo,cloud}` — O-ABI packs
- `docs/jira/v26.11.1/{lanes,waves}/` — A01-A20 specs + 152 receipts
  (`docs/jira/v26.11.2/waves/v2-07-census.md:9`)
- `docs/jira/v26.11.2/` — starDF campaign (00..05, OVERNIGHT-LOOP.md, waves/)
- `scripts/boundary_gate.py`, `scripts/zero_string_gate.py` — fail-closed gates

## Generated artifacts (never hand-edit; regenerate via the graph)

| File | Regenerate |
|---|---|
| `crates/cli/src/{main,lib,generated_cli,generated_behavior,generated_boundaries}.rs`, `tests/zero_code_cli.rs`, `docs/command-reference.md`, `ggen.lock` | `cd crates/cli && rm Cargo.toml && ggen sync run && python3 manifest_patch.py` |
<!-- row above intentionally exceeds 100 chars: verbatim command, do not wrap -->
| `crates/cli/Cargo.toml` | crate-pack output, then patched by `manifest_patch.py` |
| `ontology/cgap.ttl` projections | repo-root `ggen sync run` (`gen-templates/` consumer extras) |

## Conventions

- Standing vocabulary: ALIVE requires a command + exit run this session
  against the exact subject; UNVERIFIED never rounds up; perf numbers
  are TARGETs until measured with a stated method.
- Two seeds never conflate: SplitMix64 multiplier `0xBF58_476D_1CE4_E5B9`
  vs corpus seed `0xA20C_0FFE_E5A7_E000` — both in
  `crates/cgap-parquet/src/bitslice.rs:687-699`; swapping them breaks
  byte-identical replay.
- Frozen byte-pins, never sweep: license URN
  `urn:avec:license:p28-sil:v2.1.0-sovereign`; v26.10.31 SHACL IRIs
  (`avec:` -> `https://avec.chatman.dev/`).
- `git commit -F <file>`, never inline `-m`; no rebase, no force-push.
- Chicago courts: real collaborators, assert final state, no mocks;
  every court names its anti-vacuity mutation; mutate /tmp sandbox
  copies with sha256 fail-closed restore, never the shared tree.
- mtime-bump restore law: byte-exact restore with pre-mutation mtimes
  does NOT invalidate cargo's fingerprint — the mutated binary
  survives (`docs/jira/v26.11.1/waves/a02-finish-receipt.md:170-184`);
  bump mtimes or `cargo clean` between mutation and restore.
- KD register law: `docs/jira/v26.11.4/KNOWN_DIVERGENCES.md` is the
  register of truth. A divergence closes only via its close condition;
  a court that documents a row is not a fix. Repo-scoped refs are bare
  `KD-n`; cross-repo references must qualify (`stardf KD-12`), never
  cite a foreign register with a bare id. Fix sites carry the token
  `// KD-COVERS: KD-n` so coverage tooling can find them. The flip
  tool (`kd_flip.py`-class) is deferred until the register holds >= 5
  open rows or the first flip event occurs.
- cwd resets to repo root between Bash calls — absolute paths only.
- No Process.sleep: bounded polls on observables; the runtime owns time.
- llvm tools: `~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/` (NOT toolchain `bin/`); that toolchain's `llc` lacks a riscv backend — `/opt/homebrew/opt/llvm/bin/` has one.
- Footprint truth is LINKED ELF, not audit-profile rlib: audit rlibs (lto=false, cgu=16) run ~2.8-3.2x CGU sprawl; release rlibs hold bitcode-only `.rcgu.o` llvm-size refuses. Linked truth = /tmp no_std caller crate (`#![no_main]` + panic_handler + black_box-ed inputs; z/lto=fat/cgu=1/panic=abort, target riscv32imc). Constant inputs fold to vacuum ELFs — volatile/black_box the prologue. rustc `-C lto` conflicts with cargo's `-C embed-bitcode=no` on lib targets.
- CGAP status codes never conflate: 0x4347_0004=CGAP_ERR_LEASE_EXHAUSTED (`sovereign.rs:80`), 0x5032_38FF=CGAP_ERR_P28_SPOLIATION (`trap.rs:36`), 0x4347_0006=CONCURRENT_WRITER_DENIED (`sovereign.rs:87`). RingSink lease ceiling is 65,376 LANES derived from a 65,536-BYTE StaticPageBuffer (bytes≠lanes; data-dependent).

## Agents

Only `.claude/agents/` agents (lane-runner, court-auditor, gate-runner)
plus read-only Explore/Plan — the PreToolUse hook refuses the built-in
generics. Subagent prompts carry the absolute repo path + contract block.

## Agent fan-outs

- Never `git add -A` — stage explicit paths; `target-*/` build roots are
  lease debris (`.gitignore` is the backstop, not the mechanism;
  `python3 scripts/lease_janitor.py --clean` / `just leases` reclaims
  orphaned leases, fail-closed on the naming grammar).
- No stop/resume mid-wave (kills in-flight work); one `resumeFromRunId`
  pass at completion. Post-run truth is `journal.jsonl`.
- Agents corrupt long-file writes (~1 in 5 over ~100 lines): write in
  <50-line chunks or assemble via python; verify on-disk bytes.

## Bash (zsh, macOS, BSD)

- `echo ===` breaks zsh ("= not found" glob); quote separators.
- BSD xargs has no `-a`; use `while read -r; do ...; done < file`.
- `$?` after a pipe is the last command's rc — check
  `git rev-parse origin/main`, don't trust `git push | tail`.
- Exit codes: capture un-piped, or via
  `python3 scripts/run_checked.py --out /tmp/x.log -- <cmd>`
  (tees output, prints/propagates the command's OWN rc; timeout→124,
  spawn-fail→127) — witnessed hit 4× on 2026-11-02
  (`docs/jira/v26.11.2/lanes/lane-dx-05-run-checked.md`).
- `sed -i ''` (empty arg) on macOS.
- Slow push: check payload (`git diff-tree --stat <sha> | tail -1`),
  background with a long window, never kill mid-pack.
