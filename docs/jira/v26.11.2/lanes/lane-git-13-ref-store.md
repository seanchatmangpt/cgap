# Lane GIT-13 Receipt — ref_store (lease-sealed atomic ref CAS + journal rollback)

Status: ALIVE (all commands + exits run this session, listed below).

## Files written (owned, disjoint)

- `crates/cgap-git/src/ref_store.rs` (NEW, 461 lines)
- `crates/cgap-git/tests/ref_store_court.rs` (NEW, 175 lines)
- this receipt

## Reality fence: 0x0001_FF80 has NO in-tree referent

Dispatch named a "128-byte hardware seal register at 0x0001_FF80". Search
(grep over cgap-ffi/sovereign.rs, cgap-gate/penstock.rs, docs for FF80 /
seal-register addresses): NO referent exists. Honest implementation: the
seal register is a caller-provided fixed 128-byte slot
(`SealRegister`, `SEAL_SLOT_LEN = 128`) with an atomic monotonic counter
(`core::sync::atomic::AtomicU64`, SeqCst fetch_add); `SEAL_REGISTER_ADDR
= 0x0001_FF80` survives only as a named const whose provenance comment
says no hardware map exists in-tree. No hardware lore was invented. The
monotonic counter mirrors cgap-crypto `lease.rs` (80-byte PhiLease, §5
tick semantics, exhaustion) in semantics only — cgap-git is a published
zero-dep kernel (empty [dependencies]), so the API is mirrored, not
imported via #[path]; divergence D1 below.

## API surface (additive; lib.rs untouched — court uses #[path])

- `RefStore` — fixed-capacity (16) loose+packed ref table, no heap:
  `create`, `cas(name, expected_lease, new_oid, seal)`,
  `get`, `fingerprint()`, `namespace_ok` (refs/heads/*, refs/agents/*).
- `SealRegister` — 128B slot + AtomicU64 counter + FNV ordering chain;
  `stamp(offset, tag)` increments atomically, refuses CounterExhausted
  (typed, never wraps), refuses slot-range overflow (TableFull class).
- `Journal` — sequential-write + commit-marker model (in-memory crash-
  consistency simulation, honestly labeled; A05 owns the real WAL): write
  (uncommitted), commit(seq), replay (in-order apply, TornWrite rollback
  at the first missing marker, per-record lease precheck StaleLease).
- `Refusal { kind, msg }` — AndonRefusal-class (RFC-0042 Rule 3 shape,
  same family as csct/src/andon.rs, cgap-git/src/ingress_ring.rs;
  exit-101-class, deterministic in-process `Err`).

## Courts (tests/ref_store_court.rs, 5/5 green)

- C1 cas_success_path — create → CAS with fresh lease; final state:
  oid advanced, ref lease bound to stamped counter, register counter 1,
  slot tag nonzero, chain advanced; chained second CAS to counter 2.
- C2 stale_parent_lease — out-of-order CAS with stale parent lease ⇒
  typed StaleLease refusal, `fingerprint()` byte-unchanged, register
  counter unchanged (refused update stamps nothing).
- C3 torn_write_rollback — journal record written without commit marker ⇒
  replay refuses TornWrite, table fingerprint equals pre-journal bytes;
  marker committed ⇒ replay applies exactly 1 record.
- C4 monotonic_strict_increment — 4 updates strictly increment 1..4,
  ceiling=4 exhausts to typed CounterExhausted refusal, no wrap.
- C5 namespace/capacity — refs/tags refused (BadNamespace), unknown ref
  (UnknownRef), refs/agents legal, fill to REF_TABLE_CAP then TableFull.

## Anti-vacuity mutation battery (sandbox /tmp/lane-git-13-sandbox)

Shared tree never mutated. Base sha256 of ref_store.rs:
`e1fafeaeb8b41dc7...` (full: e1fafeaeb8b41dc782736d952b7932b05cd1568993f05f013dd88273e76ea96c).
Each mutation: mutate sandbox copy → run `cargo test --test ref_store_court`
→ expect FAIL → sha256-exact restore + mtime bump → re-verify.

| Mutation | Targets | Result |
|---|---|---|
| M1: stale-lease check `if cur.lease != expected_lease` → `if false` | C2 | FLIPPED (c2 FAILED, 4 passed 1 failed, exit nonzero) |
| M2: replay `if !rec.committed` → `if false` | C3 | FLIPPED (c3 FAILED, 4 passed 1 failed) |
| M3: `counter.fetch_add(1,..)+1` → `Ok(cur + 1)` (no increment) | C1/C4 | FLIPPED (c1, c2, c4 FAILED, 2 passed 3 failed) |

All three restores sha256-exact (YES/YES/YES) with mtime bumps; post-battery
sandbox re-run: 5 passed / 0 failed. Shared-tree ref_store.rs was never
touched by mutations (all in /tmp sandbox).

## Commands + exits (this session)

| Command | Exit | Result |
|---|---|---|
| `cargo test -p cgap-git --test ref_store_court` (shared tree) | 0 | 5 passed / 0 failed |
| `cargo test -p cgap-git --lib` | 0 | 61 passed / 0 failed |
| `cargo test -p cgap-git --test pkt_line_court` | 0 | 10 passed / 0 failed (after SLA unblock) |
| pre-existing targets fsck_court(2), git_interop(3), large_offset(6), lineage_workflow(1), packfile_fuzz(2), perf_baseline(1 ignored) | 0 | all green |
| sandbox baseline + M1/M2/M3 runs + post-restore re-run | as tabled | 3/3 mutations flip; post-restore 5/5 green |

Full `cargo test -p cgap-git` (all targets) currently compile-ABORTS on two
other lanes' in-flight untracked test files — `ingress_ring_court.rs` and
`retract_court.rs` (E0753/E0768/E0433). Census law: compile-abort is not a
failure count; those files are not mine and were broken before my lane
touched anything. My court target and all pre-existing committed targets
are green. `lib.rs` was not edited (contract).

## Compile-freeze SLA disclosure (disclose-in-both)

SLA minimal unblock applied to ANOTHER lane's in-flight file:
`crates/cgap-git/src/pkt_line.rs` lines 216/222/228 — `*out = FLUSH;`
(and DELIM, RESPONSE_END) failed to compile (`&mut [u8;64]` vs
`[u8;4]`). Fix: `out[..4].copy_from_slice(&FLUSH);` (same for DELIM,
RESPONSE_END). Semantics preserved (pkt_line_court 10/10 green after).
Owning lane must name this in its receipt; owning lane's diff supersedes
on landing.

## Divergences

- D1: no #[path] reuse of cgap-crypto lease.rs — published zero-dep kernel
  keeps empty [dependencies]; PhiLease tick semantics mirrored, not
  imported. If the coordinator wants real reuse, that is a cross-crate
  decision (would add a path dev-dependency for courts only).
- D2: journal is an in-memory commit-marker model, not durable WAL (A05).
- D3: SEAL_REGISTER_ADDR has no hardware referent (see reality fence).

## Fences this diff could break

- `cargo test -p cgap-git --lib` (61 tests) — re-run green post-change.
- pkt_line_court target — was compile-broken before my session; now green
  via disclosed unblock; re-run green.
- Shared `lib.rs`, Cargo.toml, and every other module: untouched.

## Standing

ALIVE for the lane subject (ref_store + courts + mutation battery, exits
above). Full-crate test gate is PARTIAL (blocked by other lanes' in-flight
compile aborts, cited above).
