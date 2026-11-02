# W1-16: release_check.py version pin repair

- Lane: W1-16, repo /Users/sac/cgap, branch main.
- Subject: scripts/release_check.py line 24 — `REQUIRED_VERSION`.
- Before: `REQUIRED_VERSION = "26.10.29"` (26.10.29-era pin).
- After: `REQUIRED_VERSION = "26.11.2"` (campaign version).
- Sole write per dispatch; Cargo.toml untouched (coordinator-owned).
- Other stale pins found, NOT edited (report-only):
  - Line 5 (docstring): "exactly 26.10.29" — stale prose pin.
- Other non-version findings from the run (pre-existing, report-only):
  - cgap-integration pins a literal 26.11.1 version instead of
    `version.workspace = true`.
  - cgap-verification, cgap-cli, wasm-footprint on disk under crates/ but
    not workspace members.
  - Many crates missing license/repository metadata per the gate's
    zero-dep rules; several path-only deps and cargo package exit 101s.

## Run receipt

```
python3 scripts/release_check.py   # exit 1
```

Verbatim tail: `RELEASE-CHECK: FAIL root [workspace.package].version is
'26.11.1', expected '26.11.2'; cgap-integration: package.version is pinned
literal '26.11.1' ...` (full output: /tmp/w1-16-relcheck.out).

Exit 1 is the gate working: workspace is still 26.11.1, so the repaired
pin correctly refuses. Full output at /tmp/w1-16-relcheck.out.

## Standing

- UNVERIFIED-proposal: bumping workspace version to 26.11.2 in root
  Cargo.toml is a coordinator-owned decision, not made here.
- Pin repair: ALIVE (command + exit 1 above, run this session).
