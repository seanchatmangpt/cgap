# Code Wave v26.11.4 — Closing Court Receipt

Standing: Exit code 0

Date: 2026-11-04. Lane: CLOSING COURT (verification only — no git
transitions performed by this lane). Poll discipline: 60s intervals,
60-minute cap reached; repos settled to near-clean before ladders ran
(cgap 162→10 dirty, arcus 41→6, ggen 350→0 across the window).

## Per-repo state

| Repo | HEAD at ladder run | Dirty at ladder run | Dirty at receipt time |
|---|---|---|---|
| cgap | b8a7333a | 16 | 10 |
| arcus | f6d0a5e | 5 | 6 |
| ggen | d256aa383 | 2 | 0 (clean) |

Heads continued advancing during the ladder window (lane docs/receipt
commits still landing); ladder evidence is as-of the heads above.

## Lane roster — landed (commit census, 2026-11-04)

- cgap: 54 commits today. Wave-relevant heads include: b804e695 (BR-5
  bench-court vacuity), b8a7333a (blake3 NEON x2), 878d66b6 (P7
  status-band drift-pin), a7520de6 (lane4 flip-ladder court, 7 green
  legs), a0e02750+39b53add (cgap-receipt crate + anti-vacuity court,
  5/5 mutations flipped), e251f258 (L7 push-state sweep), 33017460
  (aoq_bridge host crypto/BUSY/seq-LE/events/OCEL), 83207531 (session
  M1 keys + ML-DSA-65, AAD ruling Option (b)), b5c6d5d0 (publish
  workflow ZP-16 tier DAG), 53f4fe7f (cbmqs-audit + RFC-0089 cert
  gate), 8ccb43d0 (AOQ chain hot-path fold), eb841d71 (D1/D2/D3
  decision receipts), 9ef5653b (census gate ZP-18/18b), e2f509da
  (profile_guard compose courts).
- arcus: 32 commits today. Includes: f6d0a5e (ZP-19 budget=0 perf),
  1eaf6ff (lane4 statutory export formatter + court), 885f13e (ZP-12
  sidecar supervisor), 4f49914+fed6db3 (W2-B/W2-D AOQ1 Go twin +
  flight admission), 81e8d1c (PL6 ML-KEM handshake twin), 2df1fea
  (stale-session poisoning), 2665ba1 (PL-X4 pinner lease), 5d2f1f4
  (ZP-13 blake3 ABB), e3d671d (goldens re-hash, 7 surfaces), 1a71050
  (goldens fail-closed verify), 583a330 (zd-122 origin created, main
  pushed), 7bc5463 (CLAUDE.md git transport), e88a481 (security
  workflow), 9661bbf (M2a confirm tag + v3 Finish law + ChaCha20).
- ggen: 39 commits today. Includes: 377442907/9e6a77fb1/c40898b1a/
  8fca6032b/cce1e8aa0 (S1–S5 build-time cuts), 3004a5909 (CI
  concurrency + paths), fa7c47faa (publish workflow ZP-16 + lean4
  portability ZB-65), 187bb9cb2 (PRE-FIX pins =26.11.4), c724602a8
  (ZB-75 PRD), c1ed78abe (ZP-11..25 standing flips), 032d6c958 (F2
  Tera whitespace census gate), fb23d4c91+be0d85402 (pnpm/npm vuln
  remediation), d256aa383 (marketplace bundle ruling), a963f094d
  (NV10 vuln closing receipt).
- Named lanes observed in commit subjects: F-lanes (F2 Tera census,
  F2/F3/R1 settle in ede3fe4d), LANE-1 (receipt engine), lane3/lane4
  (arcus bench + statutory export), BR-2/3/5/6, BM-3/7/9, L7,
  S1–S5, W2-B/W2-D, PL6/PL-X2/X4/X5, ZP-11..25, ZD-121/124/125..128,
  ZB-65/75, NV10, KD-1..3.

## Final ladders (this court, witnessed this session)

- cgap: `cargo build --workspace` rc=0 (1.91s finish, warm). Full
  `cargo test --workspace --no-fail-fast` (correct flag placement
  before `--`): 277 test binaries all `ok`, **1789 passed / 0 failed /
  12 ignored**. Full log: /tmp/cgap_test_full.log (3548 lines).
- arcus: `go build ./...` clean (no output); `go vet ./...` clean;
  `go test ./... -count=1 -timeout 300s`: 4 packages ok (adapter,
  cdata, spog, + cmd no-test-files), 1 package FAIL —
  `internal/arcus/flight` `TestUdsTransitProbeRealSidecar` (broken
  pipe on sanity emit under full-suite load; the test self-discloses
  drain-window retries). **Re-run in isolation: PASS** (rc=0, full
  probe statistics emitted: dial-only p50 8834ns, trip-256B p50
  49667ns, residual-socket p50 39522ns). Classified: load-contention
  flake, not a regression.
- ggen: `cargo check -p ggen-engine` rc=0 (1m07s). Pre-fix gate
  simulation (exact gate script from .github/workflows/publish.yml
  :63-73 run locally): abb req = 26.11.4, scip req = 26.11.4 →
  **BLOCKED(PRE-FIX), rc=1** — fail-closed, as designed for a gate
  whose expected literal (26.11.3) is behind the landed pin bump
  (187bb9cb2 moved pins to =26.11.4 without updating the gate's
  expected literal). The gate refused; the refusal is the drift
  signal recorded below.

Courts/mutations: witnessed this court = ladders only. Mutation
counts carried from landed commit receipts (lane-reported, not
re-run here): 5/5 (cgap-receipt anti-vacuity), 7 green legs (lane4
ffi flip-ladder). All other lane-reported court counts: UNVERIFIED
by this court, cited from commit subjects.

## Drift items — closed vs open

Closed this wave:
- Goldens re-hash: CLOSED (arcus e3d671d re-hashed adapter.sha256
  for 7 re-manufactured surfaces; 1a71050 made verify fail-closed).
- Arcus remote: EXECUTED (583a330 — origin created, main pushed;
  supersedes the no-remote operator-pending memo e82b4ad).
- AAD ruling: RULED Option (b) and landed (cgap 83207531 var-AAD
  promotion).
- D1/D2/D3: RULED this wave (cgap eb841d71 decision receipts at
  docs/jira/v26.11.4/receipts/decision-d1/d2/d3-*.md).

Open (named owners):
1. ggen pre-fix-gate literal stale — publish.yml expects 26.11.3,
   pins are =26.11.4 (witnessed by this court's gate sim, rc=1).
   Owner: operator/lane (one-line gate literal bump, must travel
   with any future pin bump).
2. GitHub environments (ggen publish-ggen; cgap publish-tier0/1/2)
   required reviewers + crates.io trusted-publisher registration —
   operator, still open (zp-16 receipt operator-gated list).
3. arcus flight full-suite contention flake — passes isolated,
   fails under parallel package load (witnessed this court both
   ways). Owner: arcus lane (bounded-poll/drain hardening or
   serial-classify), not safety-critical (self-disclosing probes).
4. lean4 end-to-end lake build failure (Fortune5Policy module
   prefix; lakefile declares only lean_lib Lean4RustPipeline) —
   pre-existing, byte-identical on committed tree (zp-16 receipt
   correction). Owner: ggen lane/operator.

## Exit code

0 — cgap build+tests fully green; arcus build/vet green, tests green
modulo one witnessed contention flake (passes isolated); ggen engine
check green, pre-fix gate correctly BLOCKED (fail-closed) with the
staleness registered as open drift item 1.

## Addendum (late tail, 2026-11-04)

Events after this receipt's fill. Every SHA below ancestry-verified
this session via `git branch -r --contains` → `origin/main` unless
noted. D-numbers are coordinator lane IDs, not cgap receipt files.

| Event | SHA | Artifact (one line) | Ancestry |
|---|---|---|---|
| D91 settle | 601e0274 | fix(ocel2): roundtrip_court hex32 — dead helper deleted; lane-reported settle 1803 passed / 0 failed (+14 vs this receipt's F10 1789) | origin/main |
| D77 pin re-mints | ef43dbee | chore(freeze): status_maps + cscb-targets blake3 pins re-minted for the v3 era | origin/main |
| D50/D53/D56 ocel2/mcp hygiene | fa8f45b0 | chore(ocel2,mcp): dead exports removed; ocel2 README added | origin/main |
| D50/D53/D56 v3 fixtures | 9f668ce2 | fix(ocel2): conformance fixtures to v3 wire law (1104B Finish + sealed AppData) | origin/main |
| D27 crypto legs | b257a70f | test(crypto): verify_tag_var_aad negative legs + mldsa primitive property tests | origin/main |
| D51 dma court | 7b21f537 | E-h finding 5: cgap-local drift courts for dma_ring + isomorphism | origin/main |
| D52 de-pub | 16b20c91 | refactor(branchless): dead exports de-published (E-h audit) | origin/main |
| D83 CLAUDE | b69ccabd | docs: publish-flag drift fixed in CLAUDE.md (E-y row) | origin/main |
| D80-class artifact batch | f30e19f4 | chore: tracked build artifacts removed (wrapper_host.o, `__pycache__` .pyc) | origin/main |
| E-o integrity | (court, no commit) | chain-is-integrity: session inline court + chain_courts c7/c9 tamper legs green (wire-law-v3-landing.md:78) | n/a |
| E-t purity coherence | (court, no commit) | purity_gate source scan green across stress suites (wire-law-v3-landing.md:83) | n/a |

### Standing re-derivation (supersedes the PARTIAL-ish posture above)

The receipt's PARTIAL-ish fences have resolved: the lint fence was
ggen-side, not cgap; D28's fence landed (note: the coordinator-cited
SHA `a86d167` is an lsp-max commit — D28's work was dual-landed;
in cgap the CLAUDE-drift fence is `b69ccabd` (D83), verified in
cgap's object store; the fence closure is witnessed by the courts
below).

Re-derived this session against HEAD 601e0274:

- `cargo build --workspace` rc=0 (0.12s warm).
- Full `cargo test --workspace --no-fail-fast` in the canonical
  checkout: 145 binaries `ok`; the ONLY failures (2) are in the
  UNTRACKED in-flight lane file
  `crates/cgap-aoq/tests/alloc_density_census.rs:179` — not committed
  code, lane-owned, untouched by this lane.
- Committed-HEAD cross-check via `git archive HEAD` → /tmp scratch
  (worktree-free): 280 binaries, **1821 passed / 1 failed / 12
  ignored**. The 1 failure is
  `court_zero_fixed_shared_write_tmp_dirs_in_tree`
  (crates/cgap-court/tests/shared_fixtures_court.rs:74) — a METHOD
  artifact, not a code defect: that court derives repo root as
  `CARGO_MANIFEST_DIR.parent×3`, which only resolves correctly in the
  canonical `~/cgap` layout; in scratch it reads `/tmp/crates` and
  panics. It passes in the canonical checkout (first ladder above).
- ancestry: all nine commit rows above contained by origin/main.

Verdict: cgap's own fences are closed; standing promotes to **ALIVE**
on the committed tree, with three residuals named honestly:

1. `crates/cgap-aoq/tests/alloc_density_census.rs` — untracked
   in-flight lane file, failing (2 tests); must land green or be
   withdrawn by its owning lane before the tree is truly clean.
2. `shared_fixtures_court` repo_root derivation is layout-fragile
   (`manifest.parent×3` assumes the canonical checkout path) — a
   latent fence defect for any non-canonical run; harmless at the
   canonical root.
3. cgap's own `fmt-clippy` CI job is explicitly NON-BLOCKING
   (ci.yml:109) pending a dedicated fmt pass — the one cgap-side
   quality fence still open. The rust-quality.yml timeout + Docker
   31m items are lsp-max's, and the clippy wider-scope is ggen's;
   neither is cgap-side.
