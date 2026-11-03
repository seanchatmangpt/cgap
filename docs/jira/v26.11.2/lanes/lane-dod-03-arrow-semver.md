# lane-dod-03-arrow-semver

DOD-03 (2026-11-02): PUB-04 BREAKING remedy option (b) — `nested-types`
into DEFAULT features for one cycle, removal deferred to next major.

## Diff

- `crates/cgap-arrow/Cargo.toml`: `default = ["nested-types"]` (was `[]`);
  feature comment rewritten as deprecation-for-removal note (next major
  returns `default = []`). Note: a concurrent lane added
  `categories`/`keywords`/`[package.metadata.docs.rs]` to this same
  Cargo.toml mid-flight; both diffs coexist, owner-of-record is the
  publish lane for those keys.
- `crates/cgap-arrow/README.md`: new "Deprecation notice: nested-types
  feature (26.11.2)" section — default-on one cycle, migrate off the
  `Timestamp*` re-exports before the next major.
- `crates/cgap-arrow/src/lib.rs`: UNCHANGED — the
  `#[cfg(all(not(target_os = "none"), feature = "nested-types"))]` gates at
  lib.rs:80-81 and :110-111 are already correct; no edit needed.

## Commands (all this session, /tmp/dod03-* lease targets, deleted at close)

| command | exit | result |
|---|---|---|
| `cargo build -p cgap-arrow` (default features) | 0 | Finished dev profile |
| `cargo test -p cgap-arrow` | 0 | 139 unit + 3+9+1+36+1+4+6 doc/integ = 199 passed, 0 failed |
| strings-grep rmeta `TimestampMilli` (main tree, default-on) | 0 | 2 matches in built rmeta |
| strings-grep rmeta `TimestampMilli` (sandbox default-[]) | 0 | 2 matches — COURT DID NOT FLIP, discarded |

## Anti-vacuity court (the one that counts)

rmeta string-grep is NOT a valid flip court for this surface:
`generated_layout.rs:334` carries an unconditional `"TimestampMilli"` label
string, so the rmeta contains the name even with the feature off (witnessed:
sandbox default-[] rmeta still had 2 matches). Discarded as vacuous; replaced
with a use-site probe:

- `/tmp/dod03-probe-on`  -> `cargo build` against the real tree
  (default-on): **exit 0** — `cgap_arrow::TimestampMilli` resolves.
- `/tmp/dod03-sandbox` (sha-independent copy, `default = []` mutated):
  probe `cargo build`: **exit 101**, `error[E0425]: cannot find type
  'TimestampMilli' in crate 'cgap_arrow'` — flip confirmed.

Sandbox mutation was /tmp-only; shared tree untouched by the court. Probe
targets and sandbox deleted at lane close.

## Standing

ALIVE for the remedy itself (build 0, tests 199/0, use-site flip court
101-vs-0). The publish-gate context (frozen publishes, semver ledger
decision) is PUB-04's lane; DOD-03 only lands the feature-default change.

## Fences checked

- cgap-arrow host suite (139 unit) green — no regression from the
  default-feature change; dev-dep courts unchanged.
- No shared lib.rs/Cargo.toml-of-record edits beyond cgap-arrow's own
  manifest.

## Divergences

1. Dispatch named lib.rs as a writable file; it needed no edit — the
   cfg gates were already correct. Recorded as no-op, not a miss.
2. First anti-vacuity court (rmeta string-grep) failed to flip due to the
   unconditional `generated_layout.rs:334` label; replaced, per
   "a court that cannot flip is FAIL."
3. Mid-flight Cargo.toml collision disclosed: another lane added
   categories/keywords/docsrs metadata while DOD-03 held the
   default-features diff. Both landed; per disclose-in-both-directions,
   that lane should cite this receipt.
