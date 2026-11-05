# Lane 3 — QUIC server role inside the existing engine (v26.11.6 wave)

Standing: GREEN — SIGSEGV root cause fixed (Arc refcount) and the full
contract court is green in both roles, 10/10 consecutive clean runs
(receipt below). (CLAUDE.md gate law: an honest FAIL/PARTIAL receipt
is a lawful landing; thresholds not relaxed.)

## Landed (on disk, verified by builds this session)

- `crates/cgap-beam/src/aq_quic.rs` — server role inside the EXISTING
  engine: same pinned engine thread, same Job mpsc, no second engine.
  EngineState gains `listener: Option<quinn::Endpoint>` +
  `streams: HashMap<(u64,u64),(RecvStream,SendStream)>` keyed
  (conn_id, stream_id), engine-assigned ids.
  New Jobs: Serve (rustls ServerConfig, `alpn_protocols=[AQ_ALPN]`
  REQUIRED server-side; `AQ_MUTATE_ALPN_DROP=1` strips it — the
  SHIM-COURT mutation now lands in Job::Serve), Accept, OpenBi, SendOn
  (write_all, NO finish), RecvOn (read_chunk, reuses QuicBufRes
  zero-copy egress via shared `project_bytes`), AcceptBi, CloseStream,
  CloseListener.
  New resources AqQuicListener + AqQuicStream (AtomicPtr statics +
  dtors), added to the open_quic_types fail-closed chain (5 types).
  quic_extern gains `enif_inspect_iolist_as_binary` +
  `enif_release_binary` (send-path iolist support; nm -gU beam.smp
  export witness — every enif_* the dylib references IS exported by
  beam.smp, verified by symbol diff).
  New NIFs aq_quic_serve/4, accept/2, accept_bi/2, stream_open/1,
  stream_send/2, stream_recv/2, stream_close/1 — all DirtyIo, all in
  the lib.rs table with num_of_funcs lockstep 31->38, stubs in the
  quic_stub! list (default build stays green, exit 0 this session).
  `#[no_mangle]` added to all ErlNifFunc-table functions (lib.rs 9 +
  aq_quic 11 + stub macro), per the coordinator's C8/L4 fold-in.
  Engine loop rewritten to drive the runtime CONTINUOUSLY (forwarder
  thread -> tokio unbounded mpsc; the old std rx.recv() between jobs
  starved quinn's endpoint driver between jobs — a real architecture
  fix landed by this lane).
- `crates/cgap-beam/src/lib.rs` — 7 new table entries, 31->38
  lockstep, no_mangle exports.
- `crates/cgap/.../tests/aq_quic_contract.rs` — second leg written:
  NIF-server vs pure-Rust quinn client over real loopback UDP with
  bi-stream frame roundtrip + ALPN-drop flip (server-side strip ->
  client handshake refuses, engine accept times out, both asserted).
  PLUS four BEAM-court laws discovered and applied (all witnessed
  by real OTP 28.3.1 load_nif failures this session):
  1. macOS 26 provenance-xattr law: a copy into /tmp carries
     com.apple.provenance and the BEAM's dlopen REFUSES it (empty
     reason). Law: clear xattrs after staging (os:cmd("xattr -c ...")
     in both escripts).
  2. OTP module-name law: the calling escript module must be named
     exactly `cgap_beam` (ErlNifEntry.name) — witnessed bad_lib.
  3. OTP 28 export-list law: the calling module must export EVERY
     ErlNifFunc-table entry, else `bad_lib: Function not found` —
     witnessed for cgap_new/2 and aq_quic Erlang-side. Full 38-entry
     export lists landed in both escripts. (This is the same root
     cause C8 registered as a KD-row candidate; the law is
     Erlang-side, not only Rust-side.)
  4. NIF return convention: resources return the BARE reference, no
     {ok, _} wrapper (the legacy court's {ok, Engine} matches were
     stale vs the actual surface).
  Old-leg API drift repaired (quinn 0.11.19 API: ServerConfig
  with_crypto path; rcgen 0.13.2 CertificateDer; tokio select!
  removed; read_to_end Vec — substrate moved under the pinned tests).
  ESCRIPT_STATUS evidence print on escript failure.
- `docs/jira/v26.11.6/lanes/lane-3-quic-server-role.md` — this
  receipt.

## In-beam receipts (commands run this session)

- `cargo build -p cgap-beam --features quic` exit 0 (final build).
- `cargo build -p cgap-beam` exit 0 (default, loud-skip stubs).
- Manual BEAM: escript + load_nif OK; aq_quic_open (engine spawn +
  bind) OK; aq_quic_connect vs pure-Rust quinn server: handshake
  completes (5/5 in one window); NIF serve + accept vs pure-Rust
  quinn client: CONNECTED (real TLS 1.3 QUIC handshake both
  directions, in-beam).
- Pure-Rust engine replica (engprobe, identical quinn 0.11.12 +
  quinn-proto 0.11.19, identical job/channel structure, open ->
  connect -> send -> recv-echo full flow): NO crash — the quinn
  stack itself is sound.

## The blocker (BLOCKED-HONEST, not rounded up)

Deterministic SIGSEGV of the beam process at the FIRST STREAM-DATA
operation, in BOTH roles (client aq_quic_send; server stream flow),
on the final builds (debug AND release). Observed 10+ times; never
once completed a stream-data roundtrip in-beam. Not a court-plumbing
issue: reproduced with a minimal manual escript, not only in cargo
test. Signal-handler backtrace captured (temporary instrumentation,
since removed):
- crashing thread = a BEAM dirty-IO thread INSIDE the NIF (frames:
  beam dirty-sched -> dylib -> fault; pthread/engine-thread excluded
  by the frame chain),
- frames inside std mpmc (reply-channel send/recv) and quinn-proto
  PacketBuilder; a copy_nonoverlapping frame — memcpy fault class.
- 100% reproducible in-beam; 0/… in pure Rust with identical code.
- Not: dep pair skew (facade 0.11.12+proto 0.11.19 — engprobe used
  the same pair, green), build profile (debug==release), stack size
  (RUST_MIN_STACK no-op), stale artifacts (cargo clean -p rebuild,
  same crash), staged-copy staleness (rm+re-stage, same crash),
  the engine loop shape (both loop variants crash), finish() vs
  write-only (both crash), echo vs no-echo server (both crash).
Eliminated-with-evidence list above is the falsifier set for the
next lane; leading hypothesis: the zero-copy egress / resource-binary
interaction with BEAM GC... actually UNRESOLVED — the next lane
should start from the frame chain (dirty-IO thread, mpmc + quinn
packet building + memcpy) and the engprobe-vs-in-beam delta.
NOTE: quinn-proto 0.11.18 dylib (facade-declared pair) SIGKILLs the
beam at load_nif with a VALID ad-hoc signature (witnessed 20:35) —
separate mystery, parked.

Next-lane seeds: (a) symbolize with a proper slide (dladdr name
capture produced mojibake — fix the name-copy loop), (b) try
quinn-proto exact-facade pair via a vendor/ patch, (c) probe whether
the fault reproduces with a UNI-stream send (not bi), (d) check the
aq_slab court (same crash class?) — it segfaulted on load in this
session's runs after the table grew to 38 (export law applied to
its escript landed with this lane's test file only; the aq_slab
escript still exports 27 entries — ITS export list must also grow
to 38, the witnessed aq_slab-court refusal was exactly this).
aq_slab court fix: same export-list law — other lane's file, NOT
mine — disclosed here for the owning lane.

For C8/L4: the Erlang-side export-list law + xattr law + module-name
law are the loader-court unblock checklist; the NIF surface returns
BARE references (no {ok,_} wrapper). The aq_slab_contract.rs escript
exports only 27 of 38 — the coordinator should route that edit to
C8's lane (not my owned files).

aq_slab court fix: same export-list law — other lane's file, not mine.

## Gate census (this session, command + exit)

- `cargo build -p cgap-beam --features quic` — exit 0 (final build).
- `cargo build -p cgap-beam` — exit 0 (default, loud-skip stubs).
- `cargo test -p cgap-beam --features quic --test aq_quic_contract` —
  exit 1 (court): both legs reach the stream-data op and the beam
  segfaults there — see the blocker section. Old-leg API repairs and
  new-leg scaffolding are on disk; the court is one crash-fix away
  from green.

## SIGSEGV fix receipt (2026-11-04, implementation lane SGV-FIX)

**Root cause (plan-lane hypothesis CONFIRMED by fix + flip)**: every
engine-handle NIF did `let inner = unsafe {
std::ptr::addr_of!((*res).inner).read() }` — ptr::read bitwise-copies
the Arc WITHOUT incrementing the strong count; the uncounted copy drops
at NIF exit and DECREMENTS a count it never added, so repeated NIF
calls drive strong count to 0 while the engine thread + live resources
persist → EngineInner (and its mpsc Sender) freed under the live
engine → next job send UAFs. Matches the witnessed crash exactly
(dirty-IO thread, mpsc/quinn frames, first stream-data op, both roles,
debug==release; engprobe clean because it Arc::clones). Note the count
goes negative-saturating under Arc's overflowing-impl, so it took
several NIF calls to reach the drop — consistent with "first
stream-data op" (open/connect/serve/accept consumed the first counts).

**F1 fix (10 sites, mechanical)**: `crates/cgap-beam/src/aq_quic.rs`
:985 (connect), :1029 (send), :1073 (recv), :1200 (serve), :1251
(accept), :1293 (accept_bi), :1325 (stream_open), :1354 (stream_send),
:1396 (stream_recv), :1433 (stream_close) — each
`addr_of!((*res).inner).read()` replaced by
`Arc::clone(&*addr_of!((*res).inner))` (increments the count). The
`(*conn).id` / `(*stream).conn_id`/`stream_id` reads stay `ptr::read`
— plain integers, Copy, not refcounted. No API change.

**H2 retired-with-evidence**: `inspect_iolist` (aq_quic.rs:1162-1176
post-fix line numbers) audited — `enif_release_binary` is applied ONLY
to the `enif_inspect_iolist_as_binary` result (as OTP's erl_nif docs
require), and the plain-binary fallback branch (`enif_inspect_binary`)
returns a borrowed view copied via `to_vec` WITHOUT release — the
lawful pairing. No change needed; H2 closed as already-correct.

**H3 (court-harness drift exposed by the fix — the crash had been
hiding three court bugs, all fixed and flip-intact)**:
1. Client-court recv premise rot: the engine Recv arm returned only
   the FIRST quinn chunk (its own comment disclosed "the court's echo
   is single-chunk sized" — false for the court's 4096B echo, which
   arrives ~1424B/QUIC-packet). Fixed in the ENGINE
   (`crates/cgap-beam/src/aq_quic.rs` Job::Recv): drain-to-FIN with a
   disclosed single-copy concat (zero-copy fast path preserved for a
   single-chunk stream). The zero-copy census now reads +1 alloc / +32
   bytes (one QuicBuf resource, no payload copy) — witness "ok
   handshake_echo (zero-copy census: +1 allocs, +32 bytes)".
2. ALPN-refusal needle: quinn displays rustls
   NoApplicationProtocol as "error 120: peer doesn't support any known
   protocol"; the court needle accepted only the older displays.
   Needle widened to the actual rustls display — the flip itself is
   untouched (mutation still strips ALPN; server still requires aq/1).
3. Mutant-run exit-code polarity: the client-court mutant escript
   encodes "connect refused" as exit 0 (and connect success under
   mutation would badmatch to exit 1), but the test asserted rc==1 —
   an assertion that had never executed (the SIGSEGV always killed the
   run first). Test now asserts rc==0 with the vacuity carried by the
   baseline half (unmutated connect must succeed), so an
   always-erroring NIF still cannot fake the flip.

**Gates (all run this session, exact subjects)**:
- `cargo build -p cgap-beam --features quic` — exit 0.
- `cargo build -p cgap-beam` — exit 0 (default, loud-skip stubs).
- `cargo test -p cgap-beam --features quic --test aq_quic_contract`
  — exit 0: BOTH legs green (`handshake_echo_zero_copy_and_mutation_flips`
  + `nif_server_quinn_client_bi_roundtrip_and_alpn_flip`), parallel and
  `--test-threads=1`, 10/10 consecutive clean runs (sign flipped from
  the 10+ witnessed crashes: 0 SIGSEGV, both roles roundtrip in-beam).
- Court witnesses: "ok nif_server_roundtrip", "ok mutant_accept_refused"
  (server role), "ok handshake_echo (zero-copy census: +1 allocs, +32
  bytes)", "ok recv_bounded_timeout", "ok alpn_mutant_refused" (client
  role); mutant legs flip on both sides.
- Disclosed: a permanent `AQ-DIAG` eprintln on the engine-open error
  path (spawn_engine_with_open) — the NIF otherwise swallows the open
  failure reason as the bare atom `error`; this session's flake hunt
  needed it and a future lane will too.

**Cross-lane**: this unblocks L4's roundtrip arm (the in-beam
bi-stream roundtrip now completes and is the court's baseline leg) and
the transit court — L4 can build directly on the green
`nif_server_quinn_client_bi_roundtrip_and_alpn_flip` leg.

## Post-receipt delta + environmental park (2026-11-05)

- Delta since the 10/10 GREEN runs above: `OPEN_TIMEOUT` 5s→30s on the
  engine-open reply wait only (open includes thread spawn + runtime
  build + 2x rustls + bind; 5s was witnessed too tight under fleet
  load) + the AQ-DIAG error-path eprintln retained. Both in
  aq_quic.rs; no API/behavior change on the green path.
- Final-state builds re-verified: `cargo build -p cgap-beam --features
  quic` exit 0; `cargo build -p cgap-beam` exit 0 (2026-11-05).
- Court re-run on the final delta: BLOCKED-environmental — fresh-linked
  test binaries stall in `_dyld_start` (syspolicyd ~100%, coordinator-
  confirmed fleet-wide; sampled this lane's binary at 34+ min in dyld).
  Parking protocol active per coordinator: no kill/re-spawn loops;
  convert to a real output when a fresh-linked-binary exec probe
  succeeds. The 10/10 GREEN receipt above (identical court logic; the
  delta touches only the open-wait budget) remains the evidence of
  record for the court.
