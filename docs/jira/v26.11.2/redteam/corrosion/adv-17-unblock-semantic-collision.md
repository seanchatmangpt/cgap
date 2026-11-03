# ADV-17 — SLA-unblock semantic collision & multi-owner deadlock probe

Repo /Users/sac/cgap @ main, 2026-11-02. READ-ONLY over crates/ (compile
probes in /tmp); the only written file is this receipt. No git commands.
Inputs read first: lane-r17-unblock-supersession.md, lane-fix07-ffi-surface.md,
lane-fix01-fa9-alias-rekey.md, lane-rc11-contention-guard.md,
lane-rc04-attestation-mac.md.

## Attack 1 — FIX-07's OsStr unblock x FIX-01's canonical_key: SPOILER-CONFIRMED (live-probed)

Surface: `crates/cgap-ffi/src/sovereign.rs:898-935` (canonical_key, containing
FIX-07's disclosed OsStrExt unblock at :900-903). R-17 verified supersession
mechanically (compiles + 4/4 court); the SEMANTIC gap R-17's compile-level
sweep cannot see is confirmed by a real /tmp probe (exit 0, output below):

- Owner claims `/d/x` while the file is ABSENT -> branch (b): key = canonical
  parent + NUL + "x" (0x1d19b59755bde5f1).
- `/d/x` later becomes a symlink to `/d/y`: canonicalize(`/d/x`) now RESOLVES
  (branch a) -> key flips to canonical parent + NUL + "y"
  (0x1d19b49755bde43e) — a DIFFERENT registry key for the SAME claimed path.
- Exploit: owner claims `/d/x` absent (Kx registered). Attacker plants a
  dangling symlink `/d/x -> /d/y` in the spill dir (same local threat model as
  red-13: needs only dir-write access), creates `/d/y`, then claims `/d/y`.
  canonicalize(`/d/y`) succeeds -> Ky != Kx -> `claim()` ADMITS the racer
  while the owner's grant `File::create("/d/x")` FOLLOWS THE SYMLINK and
  truncates `/d/y` — one physical file, two registry keys, zero denial.
  Probe witnessed the write-through: after File::create(/d/x), y len = 0.
- Control leg (anti-vacuity): textual alias `/d/./y` vs `/d/y` hashes
  IDENTICAL (0x1d19b49755bde43e) — the probe proves the gate's existing
  defense works for what it covers, so the flip is the symlink case, not a
  broken probe.
- Court coverage: `fa9_alias_race_denied` (tests/fa9_commit_gating.rs) covers
  only textual aliases (`dir/./x`) against an EXISTING drained file. The
  absent -> created-via-different-name transition (and the symlink flip) is
  UNCOVERED.

Probe: /tmp/adv17-probe/probe.rs replicates canonical_key VERBATIM
(sovereign.rs:898-935), rustc-compiled, run twice (run2 output above).
KEY1(KEY1b stable) != KEY1c — the same claimed path /d/x hashes to two
different keys depending on existence-order of its symlink target.

Residual honesty: the key must also FLIP for the owner mid-lease: after the
attacker completes the symlink+target setup, any owner RE-CLAIM of /d/x now
hashes Ky — if the owner thread re-claims its own path after the flip it gets
a key its OWNERS row doesn't match... (claim() allows same-tid re-entry only
by exact hash match, :952) — see "multi-owner deadlock" note below.

### Multi-owner deadlock corollary

claim() :949-955: denial requires `ph == h && t != tid`; re-entry requires
exact (h, tid). If the key for a held path FLIPS mid-lease (symlink planted
behind the holder), the owner's next claim hashes to a NEW key; the OLD key
stays registered against its tid until thread exit. Two consequences:
(a) the owner can double-register stale keys in OWNERS (bounded growth only
by thread lifetime); (b) a racer claiming via the NEW key of a path the
owner holds under the OLD key is ADMITTED (the escape above). Both stem from
registry state being keyed by a MUTABLE property of the filesystem, not an
immutable inode identity (st_dev/st_ino would be stable across the flip).

## Attack 2 — RC-11's INPUT_LEN 42->43: NO SPOILER (derived, not pinned)

attestation.rs:1447-1448: `const INPUT_LEN: usize = MAC_DOMAIN.len() + 16;`
with `const _: () = assert!(INPUT_LEN == 43);`. DERIVED from MAC_DOMAIN
(:1383, b"cgap-p28/attestation-mac/v1" = 27 B), not hard-pinned; the const
assert pins the total. Grep across cgap-ffi/src and include/cgap.h for any
consumer pinning 42/INPUT_LEN wire-side: ZERO matches (the only "42" in
cgap.h is a coincidental substring of 0x43535442 at :224). The MAC layer is
not exported in cgap.h at all, so there is no wire-side desync surface. The
RC-11 unblock (42->43) is semantically consistent with the landed owner code.

Minor finding (doc drift, not a desync): attestation.rs:1446 comment still
reads "MAC_DOMAIN(26) || key_id(16) = 42 B" — stale arithmetic from before
RC-11's W6.2 fix; actual 27+16=43. Cosmetic; flag for the RC-04 owner.

## Attack 3 — stats.rs Cell64 single-writer: HOLDS VACUOUSLY (no recorder wired)

stats.rs:168-198 defines Cell64 + `unsafe impl Sync` under a documented
kernel single-writer law; :221-233 explicitly says ring integration is
"documented, NOT wired by this lane". Grep of every cgap-ffi source outside
stats.rs (lib.rs, sovereign.rs, oabi_lookup.rs, receipt_chain.rs, xip_tables.rs):
zero call sites for record_admitted/record_refused_class/record_quota/
record_receipt / stats::global(). Single-writer law holds VACUOUSLY today —
no concurrent recorder exists to violate it. Recorded per dispatch: the law
is untested by any real caller; the first ring-owner wiring is the load-
bearing event that turns vacuous-holding into a real (untested) invariant.

## Standing + verdicts

- Attack 1: SPOILER-CONFIRMED, ALIVE (probe compiled+run exit 0 this session,
  /tmp/adv17-probe/run2 output above). Neither FIX-01 nor R-17 at fault for
  coverage they never claimed; this is a NEW finding on the landed surface.
- Attack 2: NO SPOILER — derived const + zero wire pins (grep exit run this
  session). Doc-drift flag only.
- Attack 3: NO SPOILER — vacuous-hold recorded with the wiring gap named.
- Lane standing: ALIVE (every verdict above cites a command+exit this session).

## Fences (must-not-regress surfaces this receipt could implicate)

- fa9_commit_gating court (FIX-01's): unrun by this lane (READ-ONLY lane; the
  probe is a verbatim /tmp replica, not a mutation of the shared tree — no
  restore law triggered, shared tree bytes untouched).
- R-17's verdicts re-read from disk this session; no shared file modified, so
  no regression surface introduced by this lane.

## JSON terminus

```json
{"lane":"ADV-17","files_written":["docs/jira/v26.11.2/redteam/corrosion/adv-17-unblock-semantic-collision.md"],"commands":["python3 assemble probe.rs -> written 62 lines","rustc -O probe.rs -> exit 0","./probe /tmp/adv17-probe/run2 -> exit 0 (KEY1 != KEY1c symlink flip witnessed; control alias hashes identical)","grep MAC_DOMAIN/42 pins cgap-ffi+cgap.h -> zero matches","grep stats call sites outside stats.rs -> zero matches"],"courts_run":["adv17 canonical_key divergence probe (control leg = anti-vacuity)"],"standing":"ALIVE","divergences":["attestation.rs:1446 stale comment says 26/42, actual 27/43 (doc drift, RC-04 owner)","stats.rs single-writer law holds vacuously — no recorder wired"]}
```
