# Lane RC-11 — chains-ingress contention guard (red-10 exclusivity law)

Repo /Users/sac/cgap @ main. One file written in the shared tree:
`crates/bench/benches/chains-ingress.rs` (contract-owned). Plus four
disclosed SLA minimal-unblocks to OTHER lanes' in-flight files (below).
No git commands run.

## W1 Subject + law

`11-measured-claims.md` rows 5/6/7/8 carry QUIET-HOST-ONLY scoping; red-10
(`redteam/red-10-timing-jitter-exploit.md`) proved the bench could not
DETECT a noisy host. Landed: a two-channel contention guard that runs
before and during measurement and, when tripped, writes a loud typed
`CONTENTION-SUSPECT:` marker line to stderr. Divan cannot suppress
per-sample output, so the marker line is the contract — CI/consumers MUST
grep stderr for `CONTENTION-SUSPECT` and refuse to admit a marked run
against rows 5/6/7/8 (documented in the bench file header).

## W2 Design (two detectors, one contract)

- PRE-RUN calibration (`contention_guard`): 3 passes of a full-line
  streaming walk over a real 32 MiB buffer (the bandwidth channel red-10
  W5/W6 identified as dominant). Trips on absolute ns/64B-line > 12.0
  (quiet witnesses 1.63-1.68; ~7x headroom) OR calibration pass
  max/min ratio > 2.0 (quiet witnesses 1.00-1.03).
- IN-RUN (`record_pass`/`guard_finish`): every `ingress_chain` pass
  records integer ns/frame into per-const slots; after `divan::main`,
  worst-leg max/min is judged against ceiling 2.0. Quiet witnesses
  1.40/1.20 (per-leg; cross-leg mixing would inflate — dropped).
- Evolutions (each forced by a never-firing witness, per anti-vacuity):
  1-touch-per-line walk measured 0.77 ns/line quiet AND under a 4-hog
  load → FAIL as a guard; replaced with full-line reads. Min-of-3
  pre-run-only calibration stayed quiet (3.89 ns/line) while the bench
  itself degraded +126% median under bw8+cpu8 → added the in-run channel.

## W3 Receipt runs (all this session, /tmp/rc11-target build root)

Quiet (guard green, no marker), EXIT=0, row 5 reproduces (fastest
10.9-11.33 ms @1M = 10.4-10.8 ns/frame, within the red-10 4.5% band):
`cargo bench --bench chains-ingress -- ingress_chain` →
- calibration: `quiet (min 1.630 / max 1.645 ns/64B-line, jitter ratio 1.01 ...)`
- in-run: `leg 0 (65536) pass ratio 1.40`, `leg 1 (1048576) 1.20`,
  `in-run quiet (worst leg ratio 1.40 <= 2)`
Final quiet run `/tmp/rc11-q8.err`; the earlier flip's quiet run
`/tmp/rc11-load5.err`-era runs `/tmp/rc11-q5.err`/`q7` corroborate
(1.40/1.60/1.40).

Flip (bw8: 8x /tmp/rc11_hog 512MiB streamers + cpu8: 8x `yes`, killed by
pidfile after; `remaining=0` verified), EXIT=0:
- calibration: `CONTENTION-SUSPECT: ... min 2.728 / max 5.817 ns/64B-line,
  jitter ratio 2.13 (ceilings: 12 ns, ratio 2)`
- in-run: `leg 0 pass ratio 59.93`, `leg 1 pass ratio 7.38`,
  `CONTENTION-SUSPECT: in-run host-noise: worst leg ratio 59.93 > 2`
- Bench degradation corroborates the marker: @1M fastest 17.54 ms
  (+57% vs quiet) / median 30.3 ms (+155%) / slowest 124.1 ms (+827%) —
  red-10's numbers reproduce. Output: `/tmp/rc11-load7.err`.

## W4 Standing

ALIVE. Every claim above names a command + exit run this session.

## W5 Anti-vacuity

The induced-load leg IS the flip: guard was iterated three times BECAUSE
it failed to fire (0.77-ns strided walk; min-of-3 pre-run only), and the
final design flips under bw8+cpu8 while three separate quiet runs never
do. A guard that cannot flip is FAIL; this one flips with 2.1-30x margin.

## W6 SLA minimal-unblocks (disclose-in-both-directions law)

Four cross-lane unblocks while other lanes were mid-flight; each is a
minimal mechanical fix, disclosed here for the owning lanes' receipts:
1. `crates/cgap-p28/src/attestation.rs` — added missing
   `use cgap_crypto::chatman::keyed_digest;` (compile E0425/E0433).
   SUPERSEDED: the owner landed their own import at what is now :1402; I
   removed my duplicate. Owner lane: RC-04-class (keyed attestation MAC).
2. `crates/cgap-p28/src/attestation.rs` — const assert `INPUT_LEN == 42`
   → `== 43` (MAC_DOMAIN "cgap-p28/attestation-mac/v1" is 27 bytes,
   27+16=43; E0080 compile abort). STILL PRESENT.
3. `crates/cgap-bn3/src/noalloc.rs` — removed duplicate
   `required_bytes: na_engine_bytes(),` field init (E0062, ~5 min frozen;
   fix02 lane, alive-marker `lanes/fix02.alive` present). STILL PRESENT.
4. `crates/cgap-ffi/src/stats.rs:106` — `pub quota_total_refusals: v,`
   → `u64` (offset 64..72 per adjacent doc comment; E0425 x3). STILL
   PRESENT. Owner lane: FIX-08-class (ffi receipt-desync).
Owning lanes supersede these diffs on landing.

## W7 Fences

- Fences my diff could break: bench membrane law (divan pin, checksum
  assertions, PARETO docs — untouched; bench groups not restructured),
  the existing preflight/chain drift courts in the same file (ran green
  every run), and gate0/root-workspace builds (no shared-file edits other
  than the disclosed unblocks, all compile-restoring — cgap-p28/bn3/ffi
  compiled green in every subsequent run, witnessed by EXIT=0 bench runs
  4-8 above).
- /tmp scratch (`rc11_hog`, `rc11_hogs.pids`, err files) is lane-local;
  build root `/tmp/rc11-target` lease deleted at lane end.
