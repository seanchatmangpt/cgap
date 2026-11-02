# DXD-08 — Interop Scratch Janitor — Design

Status: DESIGN (no code lands in this doc)
Scope: what the Go interop harness (courts/interop-chicago) leaks into
scratch space, and the fail-closed grammar that reclaims it.
Companion to DX-3 (`scripts/lease_janitor.py`), which this design EXTENDS
rather than forks.

## 1. Inventory (measured 2026-11-02, fresh `ls`/`du`)

### 1a. mmap temp files (IPC-4 / shm court)
Source: `courts/interop-chicago/shm/shm.go:91` —
`os.CreateTemp("", "cgap-shm-court-*.seg")`, mmap'd MAP_SHARED, truncated
to HeaderBytes + n*FactBytes. `Segment.Close()` (shm.go:128-135) unmaps
and calls `os.Remove(s.path)`, so on the happy path nothing leaks.
Leak only on crash/signal-kill mid-court: the .seg file survives with no
owner. Measured today: 0 live `cgap-shm-court-*.seg` in `/tmp` or
`$TMPDIR` — Close() is holding. The grammar still covers the pattern
because crash-leak is a when-not-if.

### 1b. Go profile files (mem.out / cpu.out)
Searched `courts/` and `target/` for `mem.out`, `cpu.out`, `*.profile`:
ZERO exist. No Go bench profile run is currently resident. Design records
the grammar rule anyway (cheap insurance); no reclaim expected.

### 1c. gap0*-target build-root leases
Already covered by the DX-3 grammar (`*-target`).
Measured today:
- `/tmp/gap08-target`   70 MB   (mtime age 1.3 min — ACTIVE)
- `/tmp/gap07-sb`        4.9 MB (dir, age 20.8 min — ACTIVE)
- `/tmp/gap09-run-*.log` ~80 KB total (20 files, unclassified today)
- `/tmp/cscb-sram-target` 2.2 MB (age 56.9 min — ACTIVE at 60-min rule)

### 1d. cscb_corpus*.bin fixtures
- `/tmp/cscb_corpus.bin` 24,000,000 B (23 MB), mtime age 98.4 min.
- Generator: `scripts/generate_cscb_corpus.py:41`
  `--output` default IS `/tmp/cscb_corpus.bin`.
- REGENERABLE, NOT version-controlled. `git ls-files` has no .bin and no
  cscb_corpus path; `.gitignore` never mentions it. Deterministic
  generator => safe to delete when orphaned. The janitor must still treat
  the canonical path as guarded (below) so `just clean-interop` never
  races a court mid-run holding the corpus open.

### 1e. Repo-adjacent scratch
- `/Users/sac/cgap/target` 2.4 GB — protected by repo law
  (`.gitignore`: `target/` is a lease, never an asset); the janitor
  does NOT touch the repo's `target/`. Out of scope by construction.
- `$TMPDIR` (`/Users/sac/.cache/tmp`, 704 MB total):
  - `cgap_stats_skip_*` dirs (42 live, 8-24 KB each) — parquet test
    fixtures from stats-skip tests; leak when tests fail mid-run.
  - `cgap_flush_*.parquet` (10+ files, 163 B each) — flush leftovers.
  - `cgap_a08_t5_*` dirs — a08 scratch crate scaffolds.
  - `tmpdmev63fl.o` / `tmpv9on6hgw.o` — toolchain `.o` temp files.
  These are all `cgap_*`-prefixed and never appear under version control.

## 2. Why REUSE (extend lease_janitor.py), not fork

1. Shared invariants: fail-closed grammar, active-vs-orphan mtime rule,
   dry-run-by-default, --clean deletes orphaned only. A fork re-derives
   these and drifts (the loop's known drift class).
2. The janitor's court is the grammar itself. Two grammars = two courts.
3. `just leases` already wires the script; wiring is per-flag, not
   per-script.
4. Extension cost is small: add patterns + a FILES list (DX-3 only ever
   considered directories; interop debris is mostly FILES) and a guard
   set. Interface stays `--clean/--active-min/--tmp` + new flags.

Design decision: add to DX-3's GRAMMAR a second table, FILE_GRAMMAR
(matched against basenames of non-directories), plus GUARDS. No new
script, no second copy of the deletion loop.

## 3. Safe-deletion grammar for interop debris

Directory grammar (added to GRAMMAR):
    cgap_stats_skip_*      parquet stats-skip test dirs (files inside)
    cgap_a08_t5_*          a08 scratch crate scaffolds
File grammar (FILE_GRAMMAR):
    cgap-shm-court-*.seg   mmap IPC segments (crash leaks)
    cgap_flush_*.parquet   flush spill files
    cscb_corpus*.bin       corpus fixtures (GUARD first, see section 5)
    cscb-sram-target*      sram lane scratch (file or dir)
    gap0*-target           gap lane build-root leases (dirs, existing
                           *-target rule already matches; recorded for
                           audit completeness)
    gap0*.log/.json/.md/.err/.txt/.rs
                           gap lane run logs, baselines, mutants — file
                           debris like gap09-run-*.log (20 files seen)
    mem.out, cpu.out       Go profile files (none exist today)
    go*.prof, *.prof       pprof profiles
    tmp*.o / tmp*.o?       toolchain .o temp files
A path matching NOTHING is refused, never deleted (fail-closed, same as
DX-3's "unclassified dirs skipped" counter).

## 4. Active vs orphan rule (unchanged semantics)

ACTIVE = mtime within ACTIVE_MIN (default 60). ORPHANED = older.
`--clean` deletes ORPHANED only. Measured consequences today: gap08-target
(1.3 min) and cscb_corpus.bin (98.4 min) — the corpus is orphaned now,
but section 5's guard decides before age does.
Directories sized by the existing `dir_size` walk; files by `lstat`.
Scan roots: BOTH `/tmp` and `$TMPDIR` (DX-3 hardcodes TMP = "/tmp" only).
New flag `--roots` defaults to `os.path.join(tmp_default, "")`-style:
`["/tmp", tempfile.gettempdir()]`, deduped.

## 5. Guard: version-controlled vectors + repo target/

Guards are checked BEFORE grammar + age; a guard hit is never a delete
candidate, printed as GUARD, not counted as reclaimable.
1. REPO_ROOT/target and REPO_ROOT/target-*/ — repo law: build roots are
   leases but belong to the repo owner's cargo, not the janitor.
   GUARD rule: refuse anything under the repo root's `target*`.
2. The canonical corpus fixture is NOT version-controlled (section 1d)
   — it is /tmp-only and regenerable. The guard therefore protects the
   LIVE path from mid-run deletion, not bytes from loss:
   - `cscb_corpus*.bin` whose mtime is ACTIVE is GUARDED (never delete
     a corpus a court may hold open; os.CreateTemp-style rename is not
     used by the generator, so the open file would unlink orphan).
   - Orphaned corpus files delete normally (regenerable via
     `scripts/generate_cscb_corpus.py --validate`).
3. Never delete anything whose resolved path is NOT under a scan root
   (defense against basename grammar matching a symlinked name).
4. Never follow symlinks; delete the symlink only if the SYMLINK name
   matches the grammar (rmtree/Open must not be handed a link target).

## 6. Courts (verification)

### 6a. Idempotence court
Run `--clean` twice; second pass must reclaim 0 bytes and exit 0.
Asserts the deletion set is exactly the orphaned-and-matched set, and
that no GUARD/REFUSED classification changes between passes.

### 6b. Negative-control court (survivors)
Seed scratch with non-matching names (e.g. `keep_me.txt`,
`unrelated-dir/`) plus matching-orphaned and matching-active fixtures.
After --clean: non-matching paths survive byte-identical; matching
orphaned gone; matching active present. Exit 1 if any refused name
disappears. This is the fail-closed proof: deletion is by grammar, not
by convenience.

### 6c. Guard court (added with extension)
Corpus fixture + a repo-target-shaped path survive --clean even when
orphan-aged; grammar-matched orphan under scan root is deleted.

## 7. Wire-in (design only — do not write these files)

### justfile
    # DXD-08: interop scratch janitor (extends DX-3 grammar; dry-run
    # default, --clean deletes orphaned only, fail-closed).
    clean-interop:
        python3 scripts/lease_janitor.py --interop

Where `--interop` = full-grammar scan over BOTH roots (the extension),
vs plain `just leases` = dirs-only over /tmp (today's behavior).
`just clean-interop --clean` is not valid just syntax; instead:
    clean-interop ARG=''):
        python3 scripts/lease_janitor.py --interop {{ARG}}

### CLAUDE.md "Agent fan-outs" bullet (add after the lease bullet)
    - Interop debris (cgap-shm-court-*.seg, cgap_flush_*.parquet,
      cgap_stats_skip_*, gap0* run logs, cscb_corpus*.bin when
      orphaned) is reclaimed via `just clean-interop`; the canonical
      corpus is regenerated by
      `python3 scripts/generate_cscb_corpus.py --validate`, never
      hand-copied.

## 8. Implementation sequence

1. Extend `scripts/lease_janitor.py`: FILE_GRAMMAR, GUARDS, --roots,
   --interop, print GUARD rows; dirs-only restriction relaxed to files
   only under the --interop flag (default behavior byte-identical).
2. Add the two courts to the script's `--selftest` (stdlib only), or a
   sibling `scripts/lease_janitor_court.py` if a court needs its own
   sandbox root; wire into `just courts` ordering AFTER unit courts.
3. Add `clean-interop` justfile recipe.
4. Add the CLAUDE.md fan-out bullet.
5. Land a receipt under docs/jira/v26.11.2/receipts/ per house law.

## 9. Risks

- `$TMPDIR` on macOS is per-session; scanning it from an agent session
  sees only this session's debris. Acceptable: /tmp is the real dump.
- `tmp*.o` is broad-ish; scoped to scan roots and orphan-age it is safe,
  but keep it LAST in FILE_GRAMMAR and revisit if a court ever flags a
  false positive.
- Agents corrupt long-file writes: implementer writes the script edit in
  <50-line chunks and verifies on-disk bytes (CLAUDE.md law).
