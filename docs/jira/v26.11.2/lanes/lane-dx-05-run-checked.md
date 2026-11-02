# Lane DX-5: run_checked.py — pipe-safe command runner

Date: 2026-11-02. Standing: **ALIVE** (all commands below run this
session, exits witnessed in transcript).

## Defect class

`$?` after a shell pipe is the LAST pipe member's rc. Witnessed 4+
times this session: G11's tail-piped gate exit was `tail`'s 0; G15's
grep-piped exits were grep's; EXP lanes had to recapture un-piped.

## Tool

`/Users/sac/cgap/scripts/run_checked.py` (NEW)

```
python3 scripts/run_checked.py --out /tmp/x.log --label NAME \
    --timeout N -- <cmd...>
```

- Runs the command via `subprocess.Popen(shell=False, list form)` —
  the rc read is the command's own, never a pipe member's.
- Tees combined stdout+stderr: live stream to this stdout, full
  transcript (header + captured output + typed tail) to `--out`.
- Prints typed tail: `exit=<real rc>` (plus `run_checked: TIMEOUT ...`
  when the deadline fires), then exits with that same rc.
- `--timeout` default 1800s; on expiry SIGKILL + typed TIMEOUT +
  exit 124 (GNU convention). Signal kills map to 128+N.
- Spawn failure (ENOENT etc.) -> typed SPAWN-ERROR + exit 127.

## Courts (all run, all flipped — none vacuous)

### Court 1: bug demonstration + fix

Old piped style (the bug):

```
$ sh -c 'exit 1' | tail -1; echo $?
bug_piped_rc=0
```

A command that exits 1 reports 0 through the pipe. The fix:

```
$ python3 scripts/run_checked.py --out /tmp/dx5-c1.log --label court1 \
    -- sh -c 'exit 1'
run_checked: label=court1 timeout=1800s cmd=['sh', '-c', 'exit 1']
exit=1
fix_wrapper_rc=1
```

Anti-vacuity mutation: the subject command itself exits 1; a wrapper
that reported 0 (the bug class) would flip court 1 to FAIL. Observed
flip: piped style reports 0, wrapper reports 1.

### Court 2: real cargo command

```
$ python3 scripts/run_checked.py --out /tmp/dx5-c2.log --label court2 \
    -- cargo metadata --no-deps --format-version 1   (> /dev/null)
cargo_wrapper_rc=0
tail -1 /tmp/dx5-c2.log -> exit=0
```

Log file captured the full 69,616-byte JSON transcript; wrapper exit
matches the typed line.

### Court 3: timeout path

```
$ python3 scripts/run_checked.py --out /tmp/dx5-c3.log --label court3 \
    --timeout 2 -- sleep 30   (> /dev/null)
court3_wrapper_rc=124
tail -2 /tmp/dx5-c3.log:
  run_checked: TIMEOUT after 2s
  exit=124
```

Bounded subject (`sleep 30` under `--timeout 2`); typed TIMEOUT line +
nonzero exit 124, wrapper rc identical.

## Files written

- `/Users/sac/cgap/scripts/run_checked.py` (NEW — only file edit)

## Proposed CLAUDE.md rule (MV-8 owns the file — coordinator to land)

```markdown
- Receipt exit codes: never take `$?` after a pipe — it is the last
  pipe member's rc (tail/grep report 0 for a failed upstream command).
  Wrap lane/gate commands:
  `python3 scripts/run_checked.py --out /tmp/<lane>.log --label <name> \
   -- <cmd...>`; the typed `exit=` line and the tool's own exit are the
  command's real rc. Cite that line in receipts.
```

## Fences (must-not-regress surfaces)

- No existing gate, court, or script imports `run_checked.py` — additive
  file, zero callers this diff. No fence can regress from this lane.
- Verify: `grep -r run_checked scripts/ crates/ | grep -v run_checked.py`
  returns nothing (new file only).

## Divergences

- None from dispatch. One post-court refinement: timeout rc normalized
  from the raw `-9`/shell-247 artifact to GNU `124` (signal deaths to
  `128+N`) so the shell-visible rc is stable and positive; dispatch
  required "typed TIMEOUT + nonzero exit" — satisfied.

## Standing

ALIVE — courts 1a/1b/1c, 2, 3 all executed this session with the exits
shown above; log artifacts at `/tmp/dx5-c{1,2,3}.log`.
