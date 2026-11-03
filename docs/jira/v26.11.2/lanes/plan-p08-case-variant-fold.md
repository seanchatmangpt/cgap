# P-08 — Case-Variant ASCII Fold, Design A (PLAN)

PLAN (session-delivered, transcribed 2026-11-02; not yet executed).

Register row: RW-09-O1. Finder:
`redteam/corrosion/adv-10-fmea-risk-laundering.md:73-78` (alias
escape channel on case-INsensitive filesystems whose realpath
does not fold case).

## 1. Design A — ASCII fold in canonical_key

- `crates/cgap-ffi/src/sovereign.rs:931-968` `canonical_key` has
  three resolution arms: (1) full `canonicalize()` of the path;
  (2) canonicalized parent + raw file name; (3) raw-text
  FNV-1a fallback when neither the path nor its parent resolves.
- Design A: apply an ASCII-only case fold to the NAME component
  in arms 1 and 2 (the bytes after the final NUL join
  separator); arm 3 folds the WHOLE byte string (there is no
  parsed name there — the fallback hashes the raw path).
  Fold-at-claim replaces raw-case hashing, so two claimed paths
  differing only in ASCII case collide into one canonical key.

## 2. Rejected alternatives (with grounds)

- Full Unicode fold: case folding is 1:many in Unicode
  (e.g. sharp-s -> "ss"); a 1:many fold changes key cardinality
  and breaks the collision-bound argument. OUT.
- Locale-sensitive folds: locale is process-environment state,
  not a file identity — two threads with different locales would
  derive different keys for the same bytes. OUT.
- Design B (per-mount probe gate: stat a case-variant probe file
  per mount at grant time to detect ci-mounts and refuse them):
  rejected — the probe crosses the FFI contract (a grant-time
  filesystem side effect on paths the caller never claimed) and
  REFUSES REAL MOUNTS that are legitimately in use.

## 3. New denial leg

- `fa9_case_variant_denied`: semantics-conditional — fires in
  the fa9 commit-gating matrix
  (`crates/cgap-ffi/tests/fa9_commit_gating.rs`) when a claimed
  path ASCII-folds onto an already-claimed path. The claim is
  denied at the same gate point as the existing alias legs; the
  fa9 leg count N -> N+1.

## 4. Portability matrix

- APFS case-insensitive (this host): CLOSED — already
  protected (adv-10:73-78); fold adds key-layer defense.
- ext4 / other case-sensitive: CLOSED — fold closes the alias
  at the hash layer, previously open.
- vfat / NTFS-3g: CLOSED FOR ASCII — driver non-ASCII quirks
  residual, disclosed.
- NFS: RESIDUAL — case-conflict latency windows remain.
- non-ASCII names: RESIDUAL — no Unicode fold (per §2).

## 5. Blast radius / courts

- Files: `sovereign.rs` (fold + deny leg), a new court test file;
  every consumer hashing raw paths must re-derive; the adv-10
  alias hash fixture (13527984850778196721) CHANGES — its replay
  fixture updates in the same diff.
- Court: flip-witnessed on a real case-sensitive mount (VERIFY
  volume case-sensitivity at court time; APFS-ci host requires a
  case-sensitive shadow volume or loud skip `[SKIP] no
  case-sensitive volume`). Mutation = revert the fold to raw-case
  hashing in a /tmp sandbox copy -> the deny leg must go quiet ->
  court FAILs. sha256-exact restore + mtime bump.

## 6. Standing

UNVERIFIED (plan transcribed, not executed). Cites re-read on
disk 2026-11-02: sovereign.rs:931-968 (three arms, raw fallback),
fa9 test file present.
