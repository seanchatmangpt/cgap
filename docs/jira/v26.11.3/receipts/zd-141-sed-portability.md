# zd-141 — sed portability fix in verify_v26.10.31.sh classify branch

Standing: LANDED

Date: 2026-11-04. Status: LANDED (unpushed at receipt time; see git log).

## Defect (ZB-65, one live ubuntu failure)

`scripts/verify_v26.10.31.sh` line ~80 used BSD-only in-place edit
`sed -i '' 's/...'` in the PRE-EXISTING-FAIL classify branch. On
ubuntu-latest (GNU sed), `''` is misparsed as the script text, so the
branch fails when check-workspace FAILs and matches the baseline —
turning a disclosed pre-existing failure into a hard script error.

## Fix

Replaced the `-i` form with a portable rewrite (no `-i` at all):

```sh
sed 's/| `check-workspace` | FAIL |/| `check-workspace` | PRE-EXISTING-FAIL |/' "$TABLE" > "$TABLE.tmp" \
  && mv "$TABLE.tmp" "$TABLE"
```

Behavior preserved: same substitution, same file mutated, `$?` still
reflects sed failure (`&&` gates the mv; if sed fails, tmp is left and
mv never runs — the classify branch then proceeds to the FAIL path with
sed's nonzero rc visible). No `.bak` residue (mv consumes the tmp).

Also updated the now-stale transport note in
`.github/workflows/ci.yml` (was lines 86-88) from "pre-existing" to
FIXED.

## Verification (this session, macOS/BSD sed)

- `bash -n scripts/verify_v26.10.31.sh` → exit 0 (SYNTAX-OK).
- Minimal reproduction of the exact snippet on the input state
  (`| \`check-workspace\` | FAIL | 3 |`): post-edit file reads
  `| \`check-workspace\` | PRE-EXISTING-FAIL | 3 |` and the directory
  contains only `table.md` (tmp consumed by mv, no residue).
- GNU-sed failure mode not directly executable here (darwin host, BSD
  sed); the defect is the documented GNU parse of `sed -i ''` and the
  fix removes `-i` entirely, which is POSIX-identical on both seds.
  Full proof lands when the next ubuntu-latest CI run exercises the
  classify branch (still requires check-workspace FAIL + baseline
  match — rare path).
