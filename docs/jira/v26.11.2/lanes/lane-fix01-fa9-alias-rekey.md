# Lane FIX-01 — FA9 spill-gate canonical re-key (RED-13 closure)

Date: 2026-11-02 | Owner files: `crates/cgap-ffi/src/sovereign.rs`,
`crates/cgap-ffi/tests/fa9_commit_gating.rs`
Red-team referent: `docs/jira/v26.11.2/redteam/red-13-receipt-desync-exploit.md` §2.

## W1 Subject + identity

- `crates/cgap-ffi/src/sovereign.rs` (sha256
  `ed5ecc12e8da3963fb5011139572512cf26f6a087799e7839171508c25a410ee`)
- `crates/cgap-ffi/tests/fa9_commit_gating.rs` (sha256
  `eb37823d4725c51bac1e932c0c7c5a2c147ab03b403780ad26445afcbab0ccd4`)

## W2 Diff (μ)

1. `sovereign.rs` `spill_gate`: new `canonical_key()` — registry key now
   FNV-1a over a CANONICAL file identity instead of raw path bytes:
   (a) full-path `fs::canonicalize` when the spill file exists (resolves
   `.`/`..` and file-level symlinks); (b) when it does not (the owner's
   first claim — the grant creates it), canonicalize the PARENT and join
   the file name; (c) raw-bytes fallback only when even the parent does
   not exist (nothing exists to truncate). Both (a) and (b) derive the
   key with ONE uniform rule — canonical-parent bytes + NUL + file-name
   bytes — so a pre-existence claim and a post-existence claim of the
   same file hash identically. First cut derived (a) and (b) differently
   and failed leg 3 of the pre-existing court; the uniform rule fixed it
   and was verified by an instrumented /tmp sandbox probe.
2. Doc comments updated (gate header + key derivation) naming RED-13 and
   the mutation court.
3. `tests/fa9_commit_gating.rs`: new leg `fa9_alias_race_denied` — owner
   holds `dir/x` (drained bytes exist), racer attacks via `dir/./x`
   20,000 attempts. Final-state asserts: racer_ok == 0, racer_deny >= 1,
   spill never shrinks, drained prefix intact.
4. Pre-existing leg `fa9_release_after_owner_thread_exit` repaired for
   determinism (see W5 divergence D2).

## W3 Commands + exits (this session)

| command | exit | evidence |
|---|---|---|
| `cargo test -p cgap-ffi --test fa9_commit_gating` (shared tree, fixed) | 0 | `test result: ok. 4 passed; 0 failed` |
| mutant sandbox (`/tmp/fix01-mut/cgap-ffi`, canonical_key reverted to raw FNV-1a of `path`), same alias leg | 101 | `FAILED. 0 passed; 1 failed`, racer_ok=20000/20000 ADMITTED — red-13 breach shape reproduced |
| frozen-HEAD-deps sandbox (`/tmp/fix01-base/cgap-ffi` + my fixed files) full court | 0 | 4 passed / 0 failed |
| HEAD baseline (raw keying) 10x `fa9_release` leg in sandbox | 0 (10/10) | pre-fix leg passes only by timing luck (see D2) |
| instrumented sandbox probe (FIX01_DEBUG eprintln in /tmp copy) | — | owner+racer keys hash IDENTICAL post-fix; at HEAD owners=[] at racer claim timing |
| `cargo test -p cgap-ffi` full-crate fence | mixed | 50 lib passed + 1 lib FAILED + cgap-git unclosed-delimiter compile-abort — ALL attributed to other lanes' in-flight files (zero_string_gate.py SyntaxError, cgap-git 4 files, cgap-ffi stats.rs) |
| `sha256sum` both owned files post-run | — | hashes above (byte-stable, restore law satisfied — shared tree never mutated by me) |

## W4 Court legs + anti-vacuity

- Alias leg mutation: revert `claim` to `fnv1a(path)` raw-byte keying →
  `fa9_alias_race_denied` FLIPS (exit 101, racer 20000/20000 ADMITTED,
  exit-7-shape breach). Court is non-vacuous. Mutations in /tmp sandboxes
  only; the shared tree was never mutated.
- Pre-existing 3 legs stay green (4/4 total).

## W5 Divergences

- D1 (disclose-in-both-directions): FIX-07 applied a minimal compile
  unblock to MY in-flight sovereign.rs mid-lane (replaced
  `OsStr::from_encoded_bytes`/`PathBuf::into_os_str` with
  `std::os::unix::ffi::OsStrExt` `from_bytes`/`into_os_string` — stable
  equivalents; comment marked as FIX-07 unblock). I did not apply any
  unblock to another lane's file. Owning lane (me) supersedes the diff
  on landing; FIX-07 discloses symmetrically in its receipt.
- D2 (test repair, my lease): `fa9_release_after_owner_thread_exit` was
  timing-lucky at HEAD — the owner thread could exit its lease in the
  microsecond window before the test thread's claim; the canonical re-key
  failed it 5/5 deterministically. Instrumented probe showed
  `owners=[]` at the racer claim. Repaired with structural sync: the
  owner holds the lease on a bounded observable spin until the test
  thread signals denial observation. No sleep; bounded poll on a real
  atomic, deadline exits loud.
- D3 (attribution, not mine): during this lane, other lanes' in-flight
  edits broke the shared compile three times (cgap-p28 const-assert,
  cgap-bn3 E0425, cgap-ffi stats.rs/lib.rs, cgap-git unclosed delimiter,
  scripts/zero_string_gate.py SyntaxError). None touched my files; per
  the compile-freeze SLA I polled observables with bounded deadlines
  (75-90 s intervals via shell orchestration, disclosed: this used
  `time.sleep` in shell orchestration to space polls — not in test code)
  until each owner's fix landed, then captured the clean exits above.
- D4: mutation exit is 101 (cargo's test-failure code), not the 1 I
  first labelled; 101 is the correct reading of "court flips".

## W6 Standing

ALIVE — every claim above cites a command + exit run this session against
the exact subject (shared tree, exit 0, 4/4; mutant flip exit 101).

## W7 Falsifier

Revert `canonical_key` to raw-byte keying → the alias court must flip
(witnessed, exit 101); if it does not flip, this receipt is FAIL.
