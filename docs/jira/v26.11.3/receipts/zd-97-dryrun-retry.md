# ZD-97: Dry-Run Publish Retry for ZD-43's BLOCKED Crates

- **Subject**: cgap @ 95821317bb09419a71f34e93e2995ace49574fe1 (clean tree, verified `git status --porcelain` empty)
- **Scope**: re-run of the 2 crates BLOCKED in ZD-43's 20-crate dry-run ladder (`cgap-audit`, `cgap-ffi`) — blocked at that run by a dirty worktree, now committed.
- **Mode**: `cargo publish --dry-run` only. No publish occurred, no edits, no git transitions.

## Results

| crate | command | exit | first error class |
|---|---|---|---|
| cgap-audit | `cargo publish -p cgap-audit --dry-run` | 101 | registry-dep-missing |
| cgap-ffi | `cargo publish -p cgap-ffi --dry-run` | 101 | registry-dep-missing |

Both fail identically during packaging (before any upload step):

```
error: failed to prepare local package for uploading
Caused by:
  failed to select a version for the requirement `cgap-arrow = "^26.11.3"`
  candidate versions found which didn't match: 26.11.2, 26.11.1, 26.10.29
  location searched: crates.io index
```

## Classification

**registry-dep-missing** — the same self-healing class as the 11 registry-dep crates in ZD-43's ladder, not a new typed class. The blocker is that `cgap-arrow` v26.11.3 is not yet on crates.io; it resolves mechanically once `cgap-arrow` publishes in registry order. No source, manifest, or packaging defect in either crate: packaging itself succeeded (index updated, package prepared), failing only on cross-crate version resolution against the live registry.

## Ladder picture (20/20)

7 green + 12 registry-dep self-healing (11 prior + these 2, net of overlap as recorded in ZD-43) + 0 unclassified. Zero crates remain BLOCKED on environment/dirty-worktree grounds; zero new error classes introduced.

## Replay

```
cd /Users/sac/cgap && git checkout 95821317
cargo publish -p cgap-audit --dry-run   # expect exit 101, cgap-arrow ^26.11.3 unresolvable
cargo publish -p cgap-ffi  --dry-run    # expect exit 101, same cause
```

Note: first audit invocation piped through `tail`, losing the true exit; re-run with output redirect captured `AUDIT_EXIT=101`. Receipt exits are from the redirected runs.
