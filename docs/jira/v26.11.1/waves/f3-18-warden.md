# f3-18-warden

Wave F3 item 18 - RETRY-WARDEN (court-auditor) receipt. Repo `/Users/sac/cgap`.
Watch window: 2026-11-01T16:19:39-0800 -> 16:47:39-0800 (35 min; epoch 1793578779 ->
1793580879), 45s cadence, 3 rounds, ~47 polls. Sole write: this marker. No git writes,
no mocks, no sleeps outside the bounded poll cadence the task itself prescribes.

## Watched-set outcome

Landing log (local -0800; UTC = local+8h), from the warden's own 45s poll:

- f3-09-pub-blake3: first write 16:40:25 / 00:40:25Z; 3 passes; in-window final
  16:41:34 / 00:41:34Z, sha256
  ab71f1b480194e41e20a7f05600f0cc8a225ff124aa9753a6e1d6559c4b139c3; stable 6 polls
  through window close.
- f3-16-replay: first write 16:41:57 / 00:41:57Z; 3 passes; final 16:43:05 /
  00:43:05Z, sha256 52d83a5ef5f29e1fc9aa183d95c290b159cbd2788a17c6b6dd9abe6062ece47a.
- f3-13-pub-umbrella: first write 16:43:52 / 00:43:52Z; 2 passes; final 16:44:30 /
  00:44:30Z, sha256 78f8ac2a8fe98ae761c143a54d725f2c1579b1bd03704a800d924094c1ba1d99.
- f3-10-pub-arrow: single write 16:44:08 / 00:44:08Z, sha256
  0559895d1c910c918d27a3557f797723d3658a47182b71d049594e34fd7b13e7; stable 4 polls.
- f3-11-pub-git: first write 16:47:04 / 00:47:04Z (sha 2cf14832e239 at window close);
  FINAL WRITE POST-WINDOW 16:48:13 / 00:48:13Z, sha256
  39947355dc3039677a143331d6762128ddeae308e372595a412ed10605d0489f (edit detected by
  this watch - the stability court flipped in vivo).
- f3-12-pub-parquet: POST-WINDOW landing 16:49:59 / 00:49:59Z, sha256
  b1c3fcbc201c2980d2c5803811cb4026585676978a9f34663f12399739ab5720; audited as a
  window-edge case, labeled post-window.
- f3-14, f3-15, f3-17: ABSENT at every poll through close and at final audit (~16:50).
- Context: f3-20-terminus landed 16:37:21 / 00:37:21Z, BEFORE every leg above finalized.

## Verdict table

| marker | one-attempt law | blocked-protocol law | verdict |
|---|---|---|---|
| f3-09-pub-blake3 | 1 (NOT-EXECUTED) | cited; stable 6 polls | PASS |
| f3-10-pub-arrow | 1 (NOT EXECUTED) | cited (chain); stable 4 polls | PASS |
| f3-11-pub-git | count 2, substance holds | cited; final write post-window | PASS (notes) |
| f3-12-pub-parquet | 1 (NOT RUN) | cited; post-window landing | PASS (post-window) |
| f3-13-pub-umbrella | 1 (NOT RUN) | cited; RA typed N/A; coord-3 slip | PASS (note) |
| f3-16-replay | N/A (replay lane) | polls f3-13; dry-run FAIL shown | PASS |
| f3-14 | not landed in window | - | NOT-LANDED |
| f3-15 | not landed in window | - | NOT-LANDED |
| f3-17 | not landed in window | - | NOT-LANDED |

## Row detail (evidence behind each verdict)

- f3-09: 1 non-dry-run `cargo publish -p` line, itself a NOT-EXECUTED statement;
  zero real attempts. BLOCKED 00:39:13Z; failing command = poll loop exit 2
  (deadline, f3-03 missing); Retry-After 900s -> 00:55:02Z; timestamp present.
- f3-10: 1 for cgap-arrow (NOT EXECUTED); zero attempts. BLOCKED 00:44:00Z by chain
  law; own poll exit 0 FOUND + upstream f3-09 BLOCKED; Retry-After 900s carried
  -> 00:55:02Z; timestamp present.
- f3-11: non-dry-run grep count 2 (both NOT-EXECUTED statements; zero real attempts,
  so the law's substance holds). BLOCKED 00:48:00Z; poll exit 3 POLL_TIMEOUT
  00:41:13Z; Retry-After 900s -> 00:56:13Z. Final write 00:48:13Z postdates window
  close 00:47:39Z; close-state sha 2cf14832e239, final sha 39947355dc30.
- f3-12 (post-window landing, audited as window-edge): 1 (NOT RUN). BLOCKED; poll
  exit 1 TIMEOUT 00:42:16Z; Retry-After 300s with re-dispatch condition.
- f3-13: 1 (NOT RUN). BLOCKED; ls exit 1 @00:42:37Z; Retry-After explicitly typed
  N/A (no 429 received - honest absence, not a silent skip); timestamps present.
  Coordinate 3 ('f3-09..f3-12 absent') is false for f3-09 per the warden poll log
  (f3-09 on disk from 00:40:25Z); BLOCK cause (f3-12 absent) still stands.
- f3-16: replay lane, one-attempt N/A (0 non-dry-run publish mentions). Chain law:
  bounded poll of f3-13 cited (exit 1 ABSENT at expiry; f3-13 landed 47s after this
  lane's final write); dry-run FAIL shown: umbrella probe exit 101 with resolution
  semantics correctly diagnosed.

## Commands + exits (warden's own)

| command | exit |
|---|---|
| ls waves/f3-* at window open | 1 (none present) |
| poll round 1 first attempt (zsh glob `(N)` mis-parse) | 1 (corrected: NULL_GLOB) |
| poll rounds 1-2 (45s cadence, 8+11 polls) | 0 |
| poll round 3 (deadline-bounded to 1793580879) | 0 (hit 540s fg cap, ran as bg) |
| grep audit batch, 5 landed markers (counts + citations) | 0 |
| stat/sha256 batch (5 markers + absence checks + f3-20) | 0 (found f3-12, f3-11 edit) |
| f3-11/f3-12 current-bytes re-audit | 0 |
| curl crates.io x5 (UA cgap-retry-warden-f3-18) | 0 x5; max_version 26.10.29 all five |
| negative-control curl (bogus crate) / positive-control | HTTP 404 / HTTP 200 |
| python3 assembly of this marker; byte verification | 0 |

## Courts run (flip evidence)

1. One-attempt grep court, 6 markers: counts f3-09=1, f3-10=1 (arrow), f3-11=2 (both
   NOT-EXECUTED statements), f3-12=1, f3-13=1, f3-16=0 (N/A). Flip: a real second
   attempt raises the count and flips registry max_version to 26.11.1; neither seen.
2. Registry court (independent, live): all five crates max_version 26.10.29
   (exit 0 x5) - zero 26.11.1 publishes fired chain-wide; negative control 404
   proves the read can fail; positive control 200. Not vacuous.
3. Blocked-protocol citation court, 6 markers: every BLOCKED marker cites failing
   command + exit, Retry-After (f3-13: typed N/A, no 429 received), and a timestamp.
4. mtime/sha stability court: FLIPPED IN VIVO - f3-11 changed 2cf14832e239
   (00:47:04Z) -> 39947355dc30 (00:48:13Z) and the watch detected it; change
   detection is real.

## Divergences

1. f3-11-pub-git: non-dry-run grep count is 2, not 1 (both NOT-EXECUTED statements;
   substance of the one-attempt law holds, zero attempts executed). Its final write
   (00:48:13Z) postdates window close (00:47:39Z); in-window close-state sha was
   2cf14832e239; final state audited post-window and labeled as such.
2. f3-13-pub-umbrella coordinate 3 claims publish-leg markers f3-09..f3-12 absent;
   the warden poll log shows f3-09 on disk from 00:40:25Z, present at f3-13's checks
   (00:42:37Z) and final write (00:44:30Z). Inaccurate coordinate; BLOCK cause
   (f3-12 absent) still stands, verdict unchanged.
3. f3-20-terminus (00:37:21Z) declared the wave terminal before all publish legs
   finalized; lanes citing it inherit as-of-timestamp claims (f3-12's divergence 3
   names the same).
4. f3-14/15/17 never landed in-window: NOT-LANDED, no verdict possible; f3-12 and
   f3-11's final pass landed/updated 34s-2m20s after window close.
5. Warden round 1 first attempt exited 1 on a zsh glob mis-parse; corrected and
   disclosed rather than silently retried.

## Standing

ALIVE. Chain verdict: the one-attempt law held chain-wide - zero publish attempts,
registry-witnessed (26.10.29 on all five slots, this warden's own live reads).
Blocked-protocol law satisfied by all six audited markers. Landed chain-law lane
(f3-16) complies: polls f3-13, shows the dry-run FAIL. WAVE-PASS on the audited
laws, with the five divergences above named. files_written: this marker only.

## Window 2 (re-dispatch watch, 2026-11-02)

Watch window 2026-11-02T01:25:54Z -> 02:25:57Z (60-min deadline), 80 polls,
45s cadence, deadline-checked, poll exit 0. Sole write: this append. Repo
`/Users/sac/cgap`. Baselines at open (warden's own reads, ~01:21-01:24Z):
five publish receipts at their window-1 final sha256s (ab71f1b4..., 0559895d...,
39947355dc30..., b1c3fcbc..., 78f8ac2a...); zero window-2 section headers on
disk; registry max_version 26.10.29 / num_versions 1 on all five crates
(HTTP 200 x5, UA cgap-retry-warden-r16-w2; bogus-crate control HTTP 404;
serde control HTTP 200).

### Landing log (marker = appended `## ` header matching window[- ]?2|re-?dispatch)

- f3-13-pub-umbrella: first seen poll 17:58:11 local, mtime 01:57:55Z
  (BLOCKED receipt; later write 01:58:46Z; sha f7ce5a90... at audit).
- f3-11-pub-git: first seen 18:05:41, mtime 02:05:34Z; W2-3-witness-git
  section appended 02:27:39Z post-window (audited as window-edge).
- f3-10-pub-arrow: first seen 18:06:26, mtime 02:06:00Z (final write
  02:08:17Z, sha 506d2a01...).
- f3-09-pub-blake3: first seen 18:13:56, mtime 02:13:28Z; W2-1 witness
  append 02:19:01Z (sha c451577b... at audit).
- f3-12-pub-parquet: NEVER landed through close or at 02:28:50Z re-audit
  (sha unchanged b1c3fcbc... from window 1; header count 0).
- R8 independent sweep: ABSENT at all 80 polls and at re-checks (find
  exit 0, zero hits through 02:28:50Z).
- LANDING ORDER observed: f3-13 -> f3-11 -> f3-10 -> f3-09 (f3-12 absent)
  - NOT the claimed f3-09->f3-13 in-order (divergence 1).

### Registry flip court (decisive evidence; warden's own live reads)

Baseline 01:24Z: all five 26.10.29 / num_versions 1. Re-read 02:28Z: ALL
FIVE max_version 26.11.1, num_versions EXACTLY 2, yanked false, published_by
seanchatmangpt; created_at strictly increasing in chain order: cgap-blake3
02:15:49.374566Z -> cgap-arrow 02:17:56.021242Z -> cgap-git 02:19:00.265596Z
-> cgap-parquet 02:20:08.018411Z -> cgap 02:21:42.281839Z. The baseline read
CAN flip and FLIPPED IN VIVO during the watch - non-vacuous. num_versions == 2
per crate is registry-side one-attempt substance: a second successful upload
per crate would read 3; none does.

### Window-2 verdict table

| marker | one-attempt | protocol | chain-order | verdict |
|---|---|---|---|---|
| f3-09-pub-blake3 | 1 invocation, REFUSED pre-exec 02:07Z (zero bytes); registry then shows ONE upload 02:15:49Z (num_versions 2) | authority refusal typed; NO Retry-After (honestly typed absent) + timestamp 02:07Z; W2-1 witness API-verify line present (26.11.1, n=2, 02:16:09Z) | head of chain; gates claimed GREEN in-window (f3-03 mtime 01:42:13Z corroborates) | PASS (stale-not-false; actuator of the 02:15:49Z upload undocumented in any marker - divergence 2) |
| f3-10-pub-arrow | 0 executed / 1 named NOT EXECUTED; 2 dry-runs disclosed (pre/post-R13) | poll exit 2 POLL-DEADLINE 01:55:48Z; Retry-After 900s -> 02:20:25Z; no second attempt in marker | arrow created 02:17:56Z AFTER blake3 - held | PASS (marker stale: no success/API-verify section at audit time) |
| f3-11-pub-git | 0 executed / 1 named PRESERVED (cumulative zero) | poll exit 3 POLL_TIMEOUT 01:58:53Z; Retry-After 900s -> 02:13:53Z; no attempt-2 in marker | git created 02:19:00Z AFTER arrow - held | PASS (W2-3 witness API-verify present, post-window append 02:27:39Z) |
| f3-12-pub-parquet | no window-2 section exists; 0 observable | no protocol artifacts to audit (marker never landed) | parquet created 02:20:08Z AFTER git - chain held registry-side | NOT-LANDED (marker) / PUBLISHED (registry 02:20:08Z) - receipt-gap (divergence 3) |
| f3-13-pub-umbrella | 0 executed / 1 named ("ONE-attempt cap preserved"; dry-run 101 as mechanical witness) | poll TIMEOUT 01:55:43Z; Retry-After 300s from 01:57:03Z + timestamp; no second attempt in marker | umbrella LAST 02:21:42Z after all four kernels - held | PASS (marker stale: BLOCKED standing, no success/witness at audit time) |
| R8 sweep | - | - | - | NOT-LANDED (0/80 polls + re-checks); cross-check impossible - loud skip, no fake pass |

### Commands + exits (window 2, warden's own)

| command | exit |
|---|---|
| baseline census: header greps x5, string greps, shas x6, section headers | 0 |
| registry baseline court x5 + bogus-crate 404 + serde 200 controls | 0 x5 (all 26.10.29/1) |
| /tmp/r16_w2_poll.sh write + sha256 | 0 |
| bounded poll, background, 80 polls 45s cadence, 01:25:54Z->02:25:57Z | 0 (4 landed; f3-12 + R8 absent at deadline) |
| window-2 section extraction x4 (grep -n + sed) | 0 |
| current-state re-audit (stat/shas/header counts/r8 find) | 0 |
| registry flip court x5 with version detail parse | 0 x5 (all 26.11.1/2, chain-ordered) |
| f3-11 W2-3 witness re-extract + f3-09 tail re-read | 0 |
| f3-15-api-verify tail cross-check + this marker sha | 0 |
| python3 assembly of this section + byte verification | 0 |

### Courts run (window 2 flip evidence)

1. Registry flip court: baseline 26.10.29 x5 (01:24Z) -> 26.11.1 x5 (02:28Z)
   - FLIPPED IN VIVO during the watch. Controls: bogus crate 404 (read can
   fail), serde 200. Not vacuous.
2. One-attempt court (registry form): num_versions == 2 EXACTLY on all five
   - the metric flips on any double upload; no flip = law held chain-wide.
3. Section-marker court: header-grep flipped 0->1 on all four landed files;
   stayed 0 on f3-12 through close - detection is real in both directions.
4. Cross-checks: W2-1 (f3-09), W2-3 (f3-11) witness sections and f3-15's
   independent sweep agree with this warden's own live reads on every number
   (max_version, num_versions, created_at to the microsecond, published_by).
   Zero observers disagree. R8's named sweep never landed (divergence 5).

### Divergences (window 2)

1. Landing order 13 -> 11 -> 10 -> 09 (12 absent) is NOT the claimed
   f3-09->f3-13 in-order. Substance held: the PUBLISH chain at the registry
   is in perfect dependency order (created_at evidence above); section-write
   order reflects concurrent poll expiries, not publish order.
2. Authority edge: f3-09 records its `cargo publish -p cgap-blake3
   --allow-dirty` REFUSED by the permission system at 02:07Z (cargo never
   executed, zero bytes); blake3 26.11.1 nonetheless landed 02:15:49Z,
   published_by seanchatmangpt - executed outside the refused lane, with no
   marker section documenting the actuator. One-attempt substance held at
   the registry (num_versions 2, not 3); the invocation chain is
   undocumented - coordinator item, corroborated by f3-15 divergence 3.
3. f3-12-pub-parquet published (02:20:08Z) with NO window-2 marker section
   (file unchanged through 02:28:50Z): receipt-gap; registry is ground
   truth; the lane's window-1 Retry-After (300s) long expired.
4. Success/API-verify sections absent from f3-10 and f3-13 markers (and
   f3-12 entirely) at audit time ~02:31Z; f3-09/f3-11 are covered by
   witness appends. All stale-not-false, named loudly by the witnesses.
5. R8 independent sweep never landed in-window: the cross-check coordinate
   was discharged against W2-1/W2-3 witnesses + f3-15's sweep instead, all
   in agreement.
6. Zero 429s observed by this warden (15+ registry reads, all 200/404,
   no Retry-After header seen) - corroborates f3-15's unreproduced-429
   divergence on the coordinator's claim.

### Standing (window 2)

ALIVE. Audited laws: one-attempt HELD at the registry (num_versions 2 x5,
chain-ordered created_at); blocked-protocol HELD in all four landed
sections (failing command + exit + timestamp each; Retry-After cited or
honestly typed absent; no second attempt documented anywhere); chain law
HELD registry-side. Publish chain COMPLETED 26.11.1 across all five crates
02:15:49Z -> 02:21:42Z, witnessed live by this lane. files_written: this
append only. Marker sha before append: 9fa92fa6392eda43eda9f2b67d7d6e213cdf431ecce235b3d3a263fd0a793583.

## Window 3 (P3 publisher-markers watch, 2026-11-02)

Watch window 2026-11-02T03:21:31Z -> 04:36:31Z (75-min deadline), 45s
cadence, two background legs (leg 1 killed at the harness background
time limit ~03:52Z; leg 2 restarted and ran to ~04:37:40Z; both legs'
logs appended to /tmp/p3w3_poll.log; poll exit 0). Sole write: this
append. Repo /Users/sac/cgap. Read-only otherwise (git diff not run; no
git writes; no mocks; no Process.sleep outside the task-prescribed poll
cadence).

### Baseline at open (warden's own reads, 03:21:31Z)

Markers: p3-pub-bont-bosym + p3-pub-integration-wasm present as
publisher-law templates ((pending)/unchecked boxes, shas
983528a379a44c25 / a82947dc2ca631f3); other 7 ABSENT. Registry: all 17
lane crates unpublished (HTTP 404 / no crate object), bogus-crate
control HTTP 404.

### Landing log (first-seen, this warden's poll)

- p3-pub-core-storage: PRESENT at open (03:21:36Z mtime), sha
  732b081bfd1b799c, UNCHANGED all ~75 min (pre-window lane receipt;
  UNSUPPORTED structural x2, zero publish attempts).
- p3-pub-bont-bosym: template -> receipt 03:25:12Z (sha 04a30635aa6ab860),
  stable through close.
- p3-pub-integration-wasm: template -> receipt 03:26:43Z (sha
  7421ec06c944818a), stable through close.
- p3-pub-beam-gpu: first write 03:24:27Z; passes 03:34:14Z, 03:34:59Z
  (final sha b95ddacba5fde8a6; coordinator-authorized repair+republish
  section appended between passes).
- p3-pub-bn3-gate: first write 03:27:28Z (sha 58ae6d210715cb85);
  R2 resolution record appended 04:21:18Z (final sha f6356d9f8fc398ca).
- p3-pub-court-audit: first seen 03:54:58Z (between poll legs), sha
  de483a240c06e665, stable.
- p3-pub-fibo-cloud: first write 04:03:59Z (sha 594ffa40d77f389c).
- p3-pub-crypto-p28: first write 04:05:29Z (9a3e97120437f405); R2
  re-dispatch append 04:28:05Z (final sha 40a1c4c994909820).
- p3-pub-ffi: ABSENT at every poll through close (ls exit 1 at
  04:33:04Z, 04:35:05Z, 04:37:40Z re-checks).

### Window-3 verdict table

| marker | one-attempt law | blocked-protocol law | chain law (RECON tiers cross-check) | verdict |
|---|---|---|---|---|
| p3-pub-core-storage | 0 publish attempts claimed + registry confirms only 26.11.1/num 1 exists (04:33:35Z, actuator UNdocumented in marker) | N/A (structural UNSUPPORTED x2: publish=false on core+storage) | storage NOT published behind failed predecessor core-at-lane-time - held | PASS (stale-not-false: core landed post-marker by undocumented actuator - divergence 1) |
| p3-pub-bont-bosym | 1 invocation per crate (bont 03:22:52Z, bosym 03:24:48Z), both exit 0, "ONE attempt" claimed; registry num_versions 1 x2 | no BLOCKED section needed (clean publishes) | bosym (tier-0) and bont (tier-1 dep blake3) published after blake3 26.11.1 - held | PASS |
| p3-pub-bn3-gate | gate: 1x400 (pre-mint, superseded subject) + 3x429 (Retry-After honored, timestamps cited) + 1 success 04:03:44Z; bn3: 1x429 (RA 04:13:33Z) + 1 success 04:13:45Z; registry num_versions 1 x2 | every refusal cites exit 101 + Retry-After + timestamp | gate (tier-0) before bn3 (tier-1) at the registry - held | PASS (invocation count >1 vs law letter; all refusals pre-mint, substance = num_versions 1 - divergence 2) |
| p3-pub-crypto-p28 | crypto: 1x400 pre-mint + 1 success 03:25:33Z; p28: 4x429 (RA sliding 03:33:33->04:13:33Z, all waited in full) + coordinator-ordered R2 single success 04:24:40Z; registry num_versions 1 x2 | BLOCKED section cites command+exit+Retry-After+timestamp; R2 names coordinator order | crypto (tier-1) before p28 (tier-2) - held | PASS (divergence 2 class) |
| p3-pub-court-audit | court: 0 attempts (structural UNSUPPORTED, publish=false retained); audit: 3x429 (RA honored) + 1 success 03:53:39Z; registry num_versions 1 | court section typed UNSUPPORTED not forced; audit refusals cite exit+RA+timestamps | audit (tier-2, arrow+parquet already live) - held | PASS (divergence 2 class) |
| p3-pub-fibo-cloud | fibo: 0 attempts (dry-run never exit 0: cgap-core resolution-failed at lane time; BLOCKED upstream); cloud: 1x429 (RA 03:43:33Z) + 1 success 03:43:38Z; registry num_versions 1 | fibo BLOCKED cites typed resolution error + ~37 min bounded polls; cloud cites exit 101 + RA + timestamp | no publish past resolution-failed predecessor (fibo withheld while core 404) - HELD, the decisive chain-law witness | PASS (cloud-before-fibo order inversion disclosed; no inter-crate dep - divergence 5) |
| p3-pub-beam-gpu | beam: 0 attempts (publish=false honored); gpu: 1x400 pre-mint + 1x429 (RA 03:33:33Z) + 1 success 03:33:40Z; registry num_versions 1 | each refusal cites exit + RA; coordinator-authorized re-invocation named | gpu (tier-1 dep arrow live) - held; beam correctly unpublished (404) | PASS (divergence 2 class) |
| p3-pub-integration-wasm | 0 attempts (structural exit 101 x2; publish=false, NOT repaired); superseded by p3-pub-aux-final.md R-passes (outside watched set) | SKIP is loud + typed (structural, logs named) | integration/wasm-footprint both 404 at close - consistent | PASS (marker stale vs aux-final's later attempts; registry 404 x2 at close - divergence 3) |
| p3-pub-ffi | never landed; 0 observable | none | ffi 404 at close - no orphan publish | NOT-LANDED (loud skip; no fake pass) |

### Registry court (window close, warden's own live reads, 04:37Z)

Published at 26.11.1 with num_versions EXACTLY 1 (HTTP 200): cgap-core
04:33:35Z, cgap-bont 03:22:52Z, cgap-bosym 03:24:48Z, cgap-crypto
03:25:33Z, cgap-gate 04:03:44Z, cgap-bn3 04:13:45Z, cgap-p28 04:24:40Z,
cgap-audit 03:53:39Z, cgap-gpu 03:33:40Z, cgap-cloud 03:43:38Z.
Unpublished (HTTP 404, consistent with receipts): cgap-storage,
cgap-court, cgap-beam, cgap-fibo, cgap-integration, wasm-footprint,
cgap-ffi. created_at ordering matches every intra-marker dependency
claim. UA cgap-publish-warden-p3-window3; bogus-crate control 404 at
baseline. A double upload per crate would read num_versions 2 - metric
flips, no flip observed, court non-vacuous.

### Commands + exits (window 3, warden's own)

| command | exit |
|---|---|
| baseline census: ls/sha/stat x9 markers + registry curl x17 + bogus control | 0 |
| /tmp/p3w3_poll.sh write + sha256 (b2cd8d78d98ca98a) | 0 |
| bounded poll leg 1, background, from 03:21Z | killed at harness bg limit ~03:52Z (poll 17 reached) |
| bounded poll leg 2, background, 03:54Z -> 04:37:40Z, 45s cadence, deadline-checked | 0 |
| marker reads x8 (full) + bn3-gate/crypto-p28 R2 re-reads | 0 |
| registry mid-window spot-check x3 (bont/bosym/gpu) | 0 (all 26.11.1 / num 1) |
| registry final court x17 crates + per-crate http codes | 0 (10 published num 1; 7 x 404) |
| corruption-token sweep on 9 markers (CORRUPT/SHIM-COURT/heredoc/nul) | 1 (no matches; nul-grep inconclusive form, token sweep clean) |
| one-attempt grep court: dry-run counts + non-dry-run publish line extraction x9 markers | 0 |
| aux-final + guard-flip cross-reads (actuator search for cgap-core) | 0 |
| python3 assembly of this append + byte verification | 0 |

### Courts run (window 3 flip evidence)

1. Registry one-attempt court: num_versions == 1 on all 10 published
   crates; a second successful upload would flip it to 2. Baseline
   (03:21Z, all 17 x 404) -> close (10 x 26.11.1/1) - FLIPPED IN VIVO
   during the watch; negative control 404 both ends. Not vacuous.
2. Section-marker court: header/byte-change detection flipped on 7
   markers (template->receipt, R2 appends); stayed stable on
   core-storage (732b081b all window) and stayed ABSENT on ffi through
   close - detection real in all three states.
3. Chain-law court: cross-checked every marker's dependency claim
   against RECON tiers + registry created_at; fibo lane demonstrably
   withheld publish while predecessor cgap-core was resolution-failed,
   then core landed 04:33:35Z after fibo's lane closed - the law bound
   real behavior, not prose.
4. Blocked-protocol citation court: every BLOCKED/429 section across 7
   markers cites failing command + exit (101/400/429), Retry-After
   (exact GMT deadlines), and timestamps; no silent skips; ffi's
   absence is named NOT-LANDED, never faked.
5. Corruption court: token sweep + full-file reads clean on all 8
   landed markers; on-disk bytes match what the lanes' claims parse
   from.

### Divergences (window 3)

1. cgap-core 26.11.1 (04:33:35Z) landed with NO marker documenting the
   actuator: p3-pub-core-storage is unchanged (receipt says zero
   attempts, UNSUPPORTED), guard-flip and aux-final do not document the
   publish either. Registry is ground truth; substance held
   (num_versions 1). Coordinator item - same class as window-2
   divergence 2.
2. One-attempt letter vs substance: 6 of 8 lanes document >1 cargo
   publish invocation (400/429 refusals + honored Retry-After retries +
   coordinator-ordered re-dispatches). Every refusal was pre-mint;
   registry num_versions == 1 x10 proves zero double uploads. The
   law's substance (one successful upload per crate) HELD chain-wide;
   invocation-count letter did not.
3. p3-pub-integration-wasm is stale: it records the structural SKIP,
   but the follow-on attempts (structural fixes + 429 chases) live in
   p3-pub-aux-final.md, which is outside this warden's watched set.
   Registry 404 x2 at close matches both markers' final state.
4. p3-pub-ffi never landed through window close + re-checks: NOT-LANDED;
   no verdict possible; loudly named.
5. p3-pub-fibo-cloud inverted the dispatched order (cloud before fibo),
   disclosed by the lane; no dependency exists between the two, and the
   chain law held where it matters (fibo withheld on absent core).
6. First poll leg was killed at the harness background time limit
   (~30 min), leaving a ~3-minute detection gap 03:52-03:55Z (court-audit
   landed inside it); gap disclosed, detection otherwise continuous.

### Standing (window 3)

ALIVE. One-attempt SUBSTANCE held chain-wide (registry num_versions == 1
on all 10 new 26.11.1 publishes, this warden's own live reads); blocked
protocol HELD in all 7 landed receipts; chain law HELD (RECON tiers
cross-checked, fibo-witness). 8/9 markers landed, 1 NOT-LANDED (ffi).
files_written: this append only.
Marker sha before append: e548eb9c53ad04b993d55cd14539c6b8983affc32b1e63e0394297c4a9fbb3c9