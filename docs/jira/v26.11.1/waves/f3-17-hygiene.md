# F3-17 Hygiene Receipt

Lane F3-17-git-hygiene (court-auditor, read-only), 2026-11-01. Repo: /Users/sac/cgap.
Window: publish window F3, open 16:21:56 local, close 16:50:57 local (29m01s).

## Window monitoring

- Start HEAD triple: cfd085a9 / 2c4131ab / 38f9d942
  (full SHA cfd085a9fd230696b9dcdb437aa3e6b0aea2e60e, `git rev-parse HEAD` rc=0).
- End HEAD triple: identical to start (re-read rc=0 at 16:51 local).
- Bounded poll: 30 checks at 60s cadence, log /tmp/f3-17-poll.log, exit 0,
  POLL-DONE 16:50:57. Every iteration: head= identical, staged=0.

## Results

- HEAD unchanged start to end: TRUE.
- Staged-events count: 0. `git status --porcelain | grep -c '^[MAD]'` = 0 at
  open (rc=1, zero matches), all 30 poll iterations, and window end (rc=1).
  The dirty second-column set grew only with untracked wave markers; nothing
  was ever staged.
- Token-material hits: 0. `grep -rn -E "sk-[A-Za-z0-9]|CARGO_REGISTRY_TOKEN *="`
  over docs/jira/v26.11.1/waves/f3-*.md: rc=2 at open (no f3-* files yet —
  loud subject-missing, not a pass), rc=1 (zero hits) at five interim sweeps
  and at window end over 16 markers.
- Base64-run sweep (40+ char runs, eyeball-classified): 5 hits, all hex
  digests, zero credential-shaped material — 4x HEAD commit SHA (f3-12:4,
  f3-16:4, f3-20:4, f3-20:16) and 1x 64-hex SHA-256 graph digest (f3-02:10,
  "graph hash in sync closure"). Receipt identity fields, not secrets.

## Anomalies

None. No staged entries, no commits, no token-shaped secret in any f3-*
receipt. Start-of-window rc=2 (no f3-* files existed) is disclosed; the
binding window-end sweep covered all 16 markers that landed during the window.

## Verdict

ALIVE — clean window.

## Window 2 (2026-11-01, R15 re-dispatch hygiene window)

Window: publish window F3 re-dispatch, open 17:22:55 local, close 17:51:57 local
(29m02s). Sole write: this section (marker append). No git writes, no mocks, no
sleeps outside the bounded poll cadence the task itself prescribes.

### Window monitoring

- Start HEAD triple: cfd085a9 / 2c4131ab / 38f9d942 (full SHA
  cfd085a9fd230696b9dcdb437aa3e6b0aea2e60e, `git rev-parse HEAD` rc=0 at open).
- End HEAD triple: identical (re-read rc=0 at 17:52 local; `git log -3` rc=0).
- Bounded poll: 30 checks at 60s cadence, log /tmp/f3-17-window2-poll.log,
  script exit 0, POLL-DONE 17:51:57 (29m02s). Every iteration: head= identical,
  staged=0, br=main. Zero VIOLATION lines (`grep -c VIOLATION` = 0, rc=1).
- Staged-events count: 0. `git status --porcelain | grep -c '^[MADR]'` = 0
  (rc=1, zero matches) at open, all 30 iterations (awk first-column count 0),
  and window end (rc=1). Dirty set grew only via untracked markers:
  untracked 69 -> 75 across the window.
- Marker-set growth disclosed: waves/ r*.md was EMPTY at open (ls rc=1, loud
  no-match); r10-gate3.md, r12-bench-tmp.md, r13-quad64.md, r14-pin-final.md
  and gate-1-workspace.md landed mid-window. Binding window-end sweep covered
  the final 28-marker set (20 f3-*, 4 r*, 4 gate-*; ls | wc -l = 28, rc=0).
  f3-19 never landed (naming gap; no hygiene impact).

### Window-end token sweep (all 28 markers)

- `sk-[A-Za-z0-9_-]+`: 1 hit over waves/ (zero over f3-*/gate-*, rc=1; rc=0 via
  r*/gate-1) — r10-gate3.md:108 "disk-tier": English word, not key material.
  Extra-file disclosure: dflss/risk-controls.md:1,345 match via the word
  "risk-controls" (substring sk-); same classification.
- `CARGO_REGISTRY_TOKEN *=` (assignment-shaped): 0 hits (rc=1). Prose mentions
  only, no value: f3-05-auth.md:12,50 ("-> unset"), f3-17-hygiene.md:21
  (window-1 pattern citation).
- Beyond-spec extra keypats (AKIA/ghp_/github_pat_/xox*/PRIVATE KEY): 0 hits
  (rc=1) over the full marker set.
- 40+ char base64-ish runs: 29 total, all classified, UNKNOWN = 0:
  - 7x 40-hex HEAD commit SHA cfd085a9fd23... (git cat-file -t = commit, rc=0):
    f3-publish-ledger.md:4,126; f3-20-terminus.md:4,16; f3-17-hygiene.md:9;
    f3-16-replay.md:4; f3-12-pub-parquet.md:4.
  - 6x 64-hex warden digests f3-18-warden.md:14,17,19,21,24,27 — shasum -a 256
    recomputed byte-exact against the six on-disk markers (rc=0, all match).
  - 1x crates.io checksum f3-14-consumer.md:59 d874cf83... — byte-exact vs
    shasum -a 256 of cached cgap-26.10.29.crate (rc=0); public registry digest.
  - 1x graph hash f3-02-regen.md:10 a36a4dab... — subject named in-line ("graph
    hash in sync closure"); not independently recomputable; subject-named.
  - 2x new this window, both verified: r12-bench-tmp.md:8,35 99adbfba... =
    pre-delete sha256 of crates/cgap-blake3/tests/bench_tmp.rs, reproduced via
    `git show HEAD:...bench_tmp.rs | shasum -a 256` (rc=0); r10-gate3.md:41
    8184d1e9... = sha256 of tests/court_verdict.rs, recomputed byte-exact (rc=0).
  - 11x file paths/URLs (non-entropy).

### Verdict

ALIVE — clean window 2. Zero staged events, HEAD unchanged start to end, zero
credential-shaped material, zero unclassified high-entropy runs.
Court non-vacuity: the poll's two violation legs were flip-proven on /tmp
sandbox repos (/tmp/r15_flip, kept as evidence): staged-entry injection ->
exit 8 ("VIOLATION: staged>0 iter=1"); HEAD-move injection (818a0fbe ->
014ecc07 between iters) -> exit 7 ("VIOLATION: head-moved iter=2"). Shared
tree never mutated; sole shared-tree write is this marker append.

## Window 3 (P3-14-hygiene, court-auditor watch) — 2026-11-02T03:30Z

- Start HEAD `cfd085a9fd230696b9dcdb437aa3e6b0aea2e60e` (tree ee07c838...,
  parent 2c4131ab...); end HEAD identical. Bounded poll 60 s, 12 iterations
  @5 s: head_moves=0, staged_events=0 (`git diff --cached --name-only` count 0
  throughout; grep exit 1 = zero matches).
- Token sweep over all 91 files in waves/ (`grep -rInE
  "sk-|CARGO_REGISTRY_TOKEN=|AKIA|ghp_|github_pat_|xox[pbsa]-|BEGIN .*PRIVATE KEY"`,
  rc=0): 9 lines, ALL word-fragment false positives ("mask-", "disk-tier",
  "risk-controls", self-referential pattern text in this file) — zero
  credential-shaped material. 40+ char runs: 40 unique 64-hex digests, each
  carrying receipt context (sha256/cksum/digest claim in lane receipts) plus
  HEAD cfd085a9... — all KNOWN; UNKNOWN=0. Remaining runs: paths, test/symbol
  names, wire-hex fixtures — non-entropy classes.
- Flip evidence (courts): (1) token court — planted `sk-ant-do-not-ship-...`
  in /tmp/p314-fixture copy; tightened key-material pattern
  `sk-[A-Za-z0-9_-]{8,}` fired rc=0 on the planted line (word-fragment
  false-positive class disclosed, same as in-tree). (2) HEAD-move court —
  /tmp/p314-headfix-HEAD fixture mutated mid-watch: silent at baseline,
  FIRES exit 3 ("FLIP: HEAD-move detected"). Fixtures removed after both
  courts (ls /tmp/p314* = no matches). Shared tree never mutated by courts;
  sole shared-tree write is this append.
- Anomalies: none. Verdict: ALIVE — clean window 3.
