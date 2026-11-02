# Lane TC-05: posh_wire composition-test merge (cull-03 note 1)

Date: 2026-11-02. Repo: /Users/sac/cgap (branch main). Owned files
only: `crates/cgap-git/tests/posh_wire_court.rs`,
`crates/cgap-git/tests/ingress_ring_court.rs`, and this receipt. No
git commands run. Shared tree diff limited to the two owned test
files.

## W1 Subject

cull-03 (`docs/jira/v26.11.2/audit/cull-03-git-court-census.md:12-13`)
verdict OVERLAP-PARTIAL for posh_wire_court.rs: move
`scatter_gather_zero_reassembly_court` (formerly posh_wire_court.rs
:143-197) into `ingress_ring_court.rs` as a composition court; keep
the magic/stride flip courts with posh_wire OR merge them into
posh_refusal_matrix (lane's call, disclosed in W3).

## W2 Diff

- `ingress_ring_court.rs` (411 lines final): appended a
  self-contained POSH header-law section — WireChunkHeader
  repr(C, align(64)), 64 bytes, size/align const-asserts, WIRE_MAGIC
  `ARM\0`, fact_byte, gather_validate, TX — plus the moved court,
  now driving the REAL `cgap_git::ingress_ring::IngressRing<4096>`
  through fully qualified lib paths (no collision with the file's
  own `#[path]`-included `mod ingress_ring`).
- `posh_wire_court.rs` (209 lines final): excised the moved test and
  its now-orphaned `scatter_push` helper; magic-flip doc comment now
  cites the new home. All remaining helpers are still used by the
  two flip courts (no dead code; warning-free build witnessed).

## W3 Court call, disclosed

KEPT both flip courts in posh_wire_court (not merged into
posh_refusal_matrix). Ground: cull-03:13 — the magic-flip demux
cross-check (now posh_wire_court.rs:225-239 region, the REAL
`Demux` feed of XRM\0 vs PACK) is the only wire-side assert of the
SubjectMissing law; a merge into posh_refusal_matrix would orphan
that corroboration. posh_refusal_matrix_court.rs untouched.

## W4 Test-name enumeration (`-- --list`, this session)

BEFORE (captured pre-edit, exit 0):

- posh_wire_court (3): scatter_gather_zero_reassembly_court,
  magic_flip_refusal_court, stride_flip_refusal_court
- ingress_ring_court (6 debug-visible):
  wrap_around_byte_exact_court, overflow_refusal_court,
  overflow_mutation_flip_twin, penstock_sieve_court,
  sieve_mutation_flip_twin, tiny_ring_wrap_court
  (sieve_throughput_release_report is release-only; not listed under
  debug profile — unchanged by this lane)

AFTER (captured post-edit, exit 0):

- ingress_ring_court (7): wrap_around_byte_exact_court,
  overflow_refusal_court, overflow_mutation_flip_twin,
  penstock_sieve_court, sieve_mutation_flip_twin,
  tiny_ring_wrap_court, scatter_gather_zero_reassembly_court
- posh_wire_court (2): magic_flip_refusal_court,
  stride_flip_refusal_court

Set comparison: the union of test names is IDENTICAL before and
after; exactly one test moved (same name, new binary). Zero coverage
lost.

## W5 Commands and exits (this session)

- BEFORE list:
  `CARGO_TARGET_DIR=/tmp/tc05-target cargo test -p cgap-git --test posh_wire_court --test ingress_ring_court -- --list` — exit 0.
- Post-edit run:
  `CARGO_TARGET_DIR=/tmp/tc05-target cargo test -p cgap-git --test posh_wire_court --test ingress_ring_court` — exit 0:
  ingress_ring_court 7 passed / 0 failed; posh_wire_court 2 passed /
  0 failed. First post-edit run surfaced one dead_code warning
  (`scatter_push` orphaned by the move) — fixed by removing the
  helper; re-run is warning-free.
- Fence gate:
  `CARGO_TARGET_DIR=/tmp/tc05-target cargo test -p cgap-git --test posh_refusal_matrix_court` — exit 0, 4 passed / 0 failed (not
  touched by this lane; must-not-regress fence).
- AFTER list: same command as BEFORE list — exit 0.

## W5b Flip battery for the moved court, from its new home

Sandbox: /tmp/tc05-sandbox/ws (crate copy with a minimal standalone
manifest; the sandbox is disposable, shared tree untouched — shared
`ingress_ring_court.rs` sha256 `06d2679d...ad4359b` identical before
and after all mutation work).

- BASELINE: `cargo test -p cgap-git --test ingress_ring_court
  scatter_gather` (sandbox) — pass.
- MUTANT (SHIM-COURT): patch the moved court's expected-magic
  literal in gather_validate: `assert_eq!(hdr.magic, WIRE_MAGIC)` ->
  `assert_eq!(hdr.magic, *b"XRM\0")` (sandbox copy only;
  mutant sha256 `7decedb6...69e4824`) — `scatter_gather_zero_reassembly_court` FAILED, 0 passed / 1 failed. The flip is real: the
  court's clean-consume BEFORE witness fails when the header law is
  corrupted.
- RESTORE: byte-exact pristine restore (sha256 matches shared-tree
  file `06d2679d...ad4359b`) — first rerun FAILED: the mtime-stale
  trap witnessed live (copy2 preserved mtimes; cargo reused the
  mutant binary despite byte-exact restore). `touch` per
  CLAUDE.md mtime-bump law, rerun — pass, 1 passed / 0 failed.
  Note: a first attempted mutation (`WIRE_MAGIC` constant swap) did
  NOT flip — both producer and checker read the same constant, a
  self-consistent mutation; recorded as a tautology lesson, the
  flipping mutation targets the checker's literal, not the shared
  constant.

## W6 Standing

ALIVE for the moved court and both flip courts, this session, exact
subject, commands and exits in W5/W5b. posh_refusal_matrix fence
green. The moved composition court flips from its new home (W5b).

## W7 Fences

- posh_refusal_matrix_court (adjacent POSH court family) — run,
  green (W5).
- posh_wire_court's demux SubjectMissing corroboration — retained
  in place, 2/2 green (W5).
- Shared compile not touched (test files only); root workspace lib
  untouched.
- Standing caveat: `sieve_throughput_release_report` is
  `#[cfg(not(debug_assertions))]` — not run this session (debug
  profile only); its release-profile law is UNVERIFIED this session
  and was not modified by this lane.

## See Also

docs/jira/v26.11.2/audit/cull-03-git-court-census.md ·
docs/jira/v26.11.2/lanes/ (lane family receipts)
