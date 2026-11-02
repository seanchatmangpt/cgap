# Lane GIT-06: Deterministic Tree Synthesis (receipt)

- **Date**: 2026-11-02
- **Lane**: GIT-06, branch main, repo /Users/sac/cgap
- **Files written (complete list)**:
  - `crates/cgap-git/src/tree_synth.rs` (315 lines, core-only, zero-dep, additive)
  - `crates/cgap-git/tests/tree_synth_court.rs` (182 lines)
  - `docs/jira/v26.11.2/lanes/lane-git-06-tree-synth.md` (this file)
- **Out of lane ownership, untouched**: `crates/cgap-git/src/lib.rs`
  (module is mounted by the court via `#[path = "../src/tree_synth.rs"]`),
  `crates/cgap-git/Cargo.toml` (no dep added; cgap-blake3 already present
  as a dev-dependency from lane p1-lineage).

## Framing (documented divergence — not faked parity)

Entry frame: `"100644 blob " (12B) | <64-char lowercase BLAKE3 hex> (64B)
| '\t' | <path> | '\0'`. Git canonical uses a 40-hex SHA-1; this frame
uses 64-hex BLAKE3 — a stated divergence for cgap-internal synthetic
trees, not Git object interchange.

## API (additive only)

- `Namespace` {Facts, Rules, Derived, Receipts} -> `DIR_FACTS=/facts/`,
  `DIR_RULES=/rules/`, `DIR_DERIVED=/derived/`, `DIR_RECEIPTS=/receipts/`.
- `Coord {s, p, o: u64}`; path = `<dir><s>-<p>-<o>` decimal.
- `synthesize(coords, out, hasher) -> Result<usize, TreeSynthError>` —
  sorts `coords` IN PLACE by canonical path order (insertion sort, zero
  heap), frames entries into fixed buffer. Empty set -> `Ok(0)`.
- `frame_len`, `entry_at`, `entry_path`, `verify_canonical_order`.
- Errors: `TooManyEntries`, `BufferOverflow{needed,capacity}`,
  `NonCanonicalOrder{at}`, `MalformedEntry{at}`.
- Digest input: 24-byte LE [s,p,o]; digest function injected
  (`Fn(&[u8]) -> [u8;32]`) because `[dependencies]` must stay empty for
  the published zero-dep kernel. The court binds the REAL
  `cgap_blake3::hash` — a real collaborator, not a mock; the court also
  recomputes one digest independently against official-register output.

## Comparator

Strict lexicographic byte order over full encoded path bytes
(`/derived/` < `/facts/` < `/receipts/` < `/rules/`); decimal components
compare as digit strings ("3" < "30", fenced in the projection court).

## Courts (all run this session, exit 0, 6/6)

1. `court_byte_stability_two_passes` — MUTATION: pass 2 fed a scrambled
   input order; identical framed bytes + per-entry digest equality;
   additionally recomputes /derived/4-5-6's digest against the real
   `cgap_blake3::hash` output (anti-mock-laundering fence).
2. `court_swapped_order_flips_to_typed_err` — ANTI-VACUITY FLIP TWIN:
   swap entries 0/1 in the framed buffer (rebuilt e1++e0++rest, frames
   are variable-length) -> `Err(NonCanonicalOrder{at:1})` exactly. A
   court that cannot flip is FAIL; this one flips typed.
3. `court_namespace_projection` — every coordinate under its virtual
   dir, exact path bytes, digit-string ordering fenced ("3" < "30").
4. `court_empty_set` — Ok(0), verifier green on empty frame.
5. `court_too_many_entries_typed_err` — MAX_ENTRIES+1 -> typed refusal.
6. `court_buffer_overflow_typed_err` — full-1 buffer -> typed
   BufferOverflow with needed/capacity asserted.

## Commands + exits (this session)

```
cd /Users/sac/cgap/crates/cgap-git && cargo test --test tree_synth_court
  -> test result: ok. 6 passed; 0 failed ... EXIT=0
cd /Users/sac/cgap/crates/cgap-git && cargo test --test tree_synth_court
  (post-fix reruns) -> EXIT=0 each
```

## Standing

**ALIVE** for the lane surface: `cargo test --test tree_synth_court`
exit 0, 6/6 courts, run this session against the exact on-disk subject.
Not claimed: whole-crate green (`cargo test -p cgap-git`) — currently
compile-ABORTS on OTHER lanes' untracked in-flight test files
(`ingress_ring_court.rs`, `ref_store_court.rs`, `pkt_line_court.rs`;
compile-abort is not a failure count and those files are outside GIT-06
ownership). No perf numbers claimed. Seeds untouched.

## Must-not-regress fences (stated + run)

- Fence: my diff touches no file referenced by lib.rs; `cargo test -p
  cgap-git --lib` unaffected (verified: my court compiles the module
  via #[path] independently). Run: full-crate test attempted; failures
  are other lanes' pre-existing compile aborts, witness output above.
- Fence: no lib.rs/Cargo.toml edits, so make gate0 / boundary/zero-string
  gates have no new surface from this lane. Not re-run (no diff to their
  subjects).
- Mutations ran against in-test copies of the framed buffer; the shared
  tree was never mutated, so no sha256-restore cycle was required.
