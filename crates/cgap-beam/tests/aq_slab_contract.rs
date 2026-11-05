//! L19 BEAM contract court (aq-nif-boundary-spec.md first surface).
//!
//! One Rust integration test drives the REAL dylib in a REAL BEAM via
//! escript: mmap open (+ refusals), zero-copy egress projection, windows,
//! stamp ingress, lease bitmask lifecycle with GC-dtor law, the CrcMismatch
//! negative leg (file pwrite corruption of a mapped frame), the alloc-spy
//! zero-alloc egress census, and the SHIM-COURT anti-vacuity mutation
//! (AQ_MUTATE_CRC_SKIP=1): running the same court against the mutated
//! validator MUST exit 1 — a green run is VACUOUS. No mocks.

use std::path::PathBuf;
use std::process::Command;

/// Locate (building if needed) the NIF dylib; mirrors bridge_contract.rs.
fn nif_dylib_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target_root = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            manifest_dir
                .join("../../target")
                .canonicalize()
                .unwrap_or(manifest_dir.join("../../target"))
        });
    for profile in ["debug", "release"] {
        for ext in ["dylib", "so"] {
            let p = target_root.join(profile).join(format!("libcgap_beam.{ext}"));
            if p.exists() {
                return p;
            }
        }
    }
    panic!("build the dylib first: cargo build -p cgap-beam");
}

/// Write the escript to a temp file.
fn write_escript(src: &str) -> PathBuf {
    // escript derives the module name from the file basename; load_nif refuses
    // a mismatch against the dylib's entry name (cgap_beam) — bridge_contract
    // precedent. Write as cgap_beam.erl.
    let p = std::env::temp_dir().join("cgap_beam");
    std::fs::write(&p, src).expect("write escript");
    p
}

fn run_court(escript: &PathBuf, dylib: &PathBuf, mutate: &str) -> i32 {
    let mut cmd = Command::new("escript");
    cmd.arg(escript).arg(dylib);
    if mutate == "crc" {
        cmd.env("AQ_MUTATE_CRC_SKIP", "1");
    }
    if mutate == "window-oob" {
        cmd.env("AQ_MUTATE_WINDOW_OOB_SKIP", "1");
    }
    let out = cmd.output().expect("spawn escript");
    if !out.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&out.stdout));
    }
    if !out.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&out.stderr));
    }
    out.status.code().unwrap_or(1)
}

const ESCRIPT: &str = r##"
-module(cgap_beam).
-export([main/1, aq_slab_open/1, aq_slab_frame/2, aq_slab_window/4, aq_slab_stamp/2, aq_slab_lease/2, aq_slab_sync/1, cgap_new/2, cgap_new_fast/2, cgap_new_i64/2, cgap_len/1, cgap_get/2, cgap_get_i64/2, cgap_free/1, cgap_stats/1, cgap_version/0, cgap_envelope_new/1, cgap_envelope_len/1, cgap_batch_binary/1, pointer_of/1, cgap_binary_data_pointer/1, cgap_dtor_stats/0, cgap_alloc_stats/0, cgap_flush_to_parquet/1, cgap_seal_to_packfile/1, cgap_receipt_query/1, cgap_open/1, cgap_scan/3]).

aq_slab_open(_) -> erlang:nif_error(nif_not_loaded).
aq_slab_frame(_, _) -> erlang:nif_error(nif_not_loaded).
aq_slab_window(_, _, _, _) -> erlang:nif_error(nif_not_loaded).
aq_slab_stamp(_, _) -> erlang:nif_error(nif_not_loaded).
aq_slab_lease(_, _) -> erlang:nif_error(nif_not_loaded).
aq_slab_sync(_) -> erlang:nif_error(nif_not_loaded).
cgap_alloc_stats() -> erlang:nif_error(nif_not_loaded).
cgap_new(_, _) -> erlang:nif_error(nif_not_loaded).
cgap_new_fast(_, _) -> erlang:nif_error(nif_not_loaded).
cgap_new_i64(_, _) -> erlang:nif_error(nif_not_loaded).
cgap_len(_) -> erlang:nif_error(nif_not_loaded).
cgap_stats(_) -> erlang:nif_error(nif_not_loaded).
cgap_get(_, _) -> erlang:nif_error(nif_not_loaded).
cgap_get_i64(_, _) -> erlang:nif_error(nif_not_loaded).
cgap_free(_) -> erlang:nif_error(nif_not_loaded).
cgap_version() -> erlang:nif_error(nif_not_loaded).
cgap_envelope_new(_) -> erlang:nif_error(nif_not_loaded).
cgap_envelope_len(_) -> erlang:nif_error(nif_not_loaded).
cgap_batch_binary(_) -> erlang:nif_error(nif_not_loaded).
pointer_of(_) -> erlang:nif_error(nif_not_loaded).
cgap_binary_data_pointer(_) -> erlang:nif_error(nif_not_loaded).
cgap_dtor_stats() -> erlang:nif_error(nif_not_loaded).
cgap_open(_) -> erlang:nif_error(nif_not_loaded).
cgap_scan(_, _, _) -> erlang:nif_error(nif_not_loaded).
cgap_flush_to_parquet(_) -> erlang:nif_error(nif_not_loaded).
cgap_seal_to_packfile(_) -> erlang:nif_error(nif_not_loaded).
cgap_receipt_query(_) -> erlang:nif_error(nif_not_loaded).

main([NifPath0]) ->
    %% OTP 28 load_nif appends .so unconditionally on this platform: copy the
    %% dylib to a .so-named file first (real file, same bytes).
    %% Bridge law: stage the dylib as <stem>.so, load the stem path — the VM
    %% appends .so to a suffix-less path only.
    NifPathSo = filename:join([tmp_base(), "libcgap_beam_court.so"]),
    NifPath = filename:join([tmp_base(), "libcgap_beam_court"]),
    {ok, _} = file:copy(NifPath0, NifPathSo),
    ok = case erlang:load_nif(NifPath, 0) of
             ok -> ok;
             {error, R} -> erlang:error({load_nif_failed, R})
         end,
    Res = try
              case os:getenv("AQ_MUTATE_WINDOW_OOB_SKIP") of
                  "1" -> t_window_oob_mutant();
                  _ -> contracts()
              end
          of
              ok -> 0
          catch Class:Reason:Stack ->
              io:format("COURT_FAIL ~p:~p~n~p~n", [Class, Reason, Stack]),
              1
          end,
    halt(Res).

contracts() ->
    t_open_refusals(),
    t_stamp_frame_window(),
    t_crc_negative_leg(),
    t_window_lease_respect(),
    t_lease_lifecycle(),
    t_alloc_spy_egress(),
    ok.

%% Segment builder: a real wire-shm-layout file (bookkeeping + zero slots).
make_segment(Dir, Name, Capacity) ->
    Path = filename:join(Dir, Name),
    Book = <<0:64/big-unsigned-integer, Capacity:32/big-unsigned-integer,
             0:((4096 - 12) * 8)/unit:1>>,
    Slots = <<0:((4096 * Capacity) * 8)/unit:1>>,
    ok = file:write_file(Path, <<Book/binary, Slots/binary>>),
    Path.

t_open_refusals() ->
    Dir = mktempd(),
    %% capacity 65: over the u64 free-mask law -> typed open refusal.
    Wide = make_segment(Dir, "wide.wire", 65),
    error = aq_slab_open(unicode:characters_to_binary(Wide)),
    %% capacity-field vs file-size mismatch (ring.rs:57-71 law).
    P2 = filename:join(Dir, "mismatch.wire"),
    ok = file:write_file(P2, <<0:((4096 + 2 * 4096) * 8)/unit:1>>),
    error = aq_slab_open(unicode:characters_to_binary(P2)),
    %% missing file
    error = aq_slab_open(unicode:characters_to_binary(filename:join(Dir, "nope.wire"))),
    %% non-binary arg
    error = aq_slab_open(42),
    io:format("ok open_refusals~n").

t_stamp_frame_window() ->
    Dir = mktempd(),
    Cap = 4,
    Path = make_segment(Dir, "seg.wire", Cap),
    Slab = aq_slab_open(unicode:characters_to_binary(Path)),
    true = is_reference(Slab),
    %% stamp 3 frames (writer side, one bounded BEAM->slab copy).
    ok = aq_slab_stamp(Slab, <<"hello">>),
    ok = aq_slab_stamp(Slab, <<"the entire continuity of memory">>),
    ok = aq_slab_stamp(Slab, <<0, 255, 1, 254>>),
    %% zero-copy egress: bytes out == bytes in, incl. wrap addressing.
    <<"hello">> = aq_slab_frame(Slab, 0),
    <<"the entire continuity of memory">> = aq_slab_frame(Slab, 1),
    <<0, 255, 1, 254>> = aq_slab_frame(Slab, 2),
    error = aq_slab_frame(Slab, 3),         %% unstamped slot: BadMagic
    error = aq_slab_frame(Slab, 8),         %% 8 mod 4 = 0 but stored ord 0 != 8
    error = aq_slab_frame(not_a_slab, 0),
    error = aq_slab_frame(Slab, -1),
    %% windows: O(1) slices of the projected frame, bounds vs plen.
    <<"hello">> = aq_slab_window(Slab, 0, 0, 5),
    <<"ello">> = aq_slab_window(Slab, 0, 1, 4),
    <<>> = aq_slab_window(Slab, 0, 5, 0),
    error = aq_slab_window(Slab, 0, 4, 2),  %% off+len > plen
    error = aq_slab_window(Slab, 0, 7, 0),
    io:format("ok stamp_frame_window~n").

%% L23 lease-respect law: a window over a leased slot's frame refuses
%% (typed SlabSlotLeased -> error atom); after the lease dtor clears the
%% bit (bounded poll, no sleep), the same window succeeds.
t_window_lease_respect() ->
    Dir = mktempd(),
    Path = make_segment(Dir, "seg.wire", 4),
    Slab = aq_slab_open(unicode:characters_to_binary(Path)),
    ok = aq_slab_stamp(Slab, <<"leased-frame-payload">>),
    <<"leased">> = aq_slab_window(Slab, 0, 0, 6),
    Parent = self(),
    Child = spawn(fun() ->
        L = aq_slab_lease(Slab, 0),
        true = is_reference(L),
        Parent ! {lease_held, self()},
        receive
            release -> ok
        after 5000 -> ok
        end
    end),
    receive
        {lease_held, Child} -> ok
    after 5000 -> erlang:error({window_lease_child_timeout, lease_held})
    end,
    %% While the lease is alive in the child: window must refuse.
    error = aq_slab_window(Slab, 0, 0, 6),
    Child ! release,
    Ref = erlang:monitor(process, Child),
    receive
        {'DOWN', Ref, process, Child, _} -> ok
    after 5000 -> erlang:error({window_lease_child_timeout, down})
    end,
    %% Bounded poll until the dtor cleared the bit (GC is asynchronous).
    wait_window_ok(Slab, 500),
    <<"leased">> = aq_slab_window(Slab, 0, 0, 6),
    io:format("ok window_lease_respect~n").

wait_window_ok(_Slab, 0) ->
    erlang:error({lease_dtor_did_not_fire, window_bounded_poll_exhausted});
wait_window_ok(Slab, N) ->
    garbage_collect(),
    case aq_slab_window(Slab, 0, 0, 6) of
        <<"leased">> -> ok;
        error -> wait_window_ok(Slab, N - 1)
    end.

%% L23 SHIM-COURT mutant leg: under AQ_MUTATE_WINDOW_OOB_SKIP=1 the same
%% over-reading window (3, 7) past plen=5 must NOT be refused — either it
%% returns a binary (flip witnessed) or the VM aborts on the illegal
%% enif_make_sub_binary (also a flip, nonzero exit). Baseline runs refuse.
t_window_oob_mutant() ->
    Dir = mktempd(),
    Path = make_segment(Dir, "seg.wire", 2),
    Slab = aq_slab_open(unicode:characters_to_binary(Path)),
    ok = aq_slab_stamp(Slab, <<"hello">>),
    <<"hello">> = aq_slab_window(Slab, 0, 0, 5),  %% in-bounds sanity
    case aq_slab_window(Slab, 0, 3, 7) of
        error -> erlang:error({window_oob_mutation_did_not_flip, oob_window_still_refused});
        _Bin -> io:format("ok window_oob_mutant (over-read window accepted)~n")
    end.

t_crc_negative_leg() ->
    Dir = mktempd(),
    Path0 = make_segment(Dir, "seg.wire", 2),
    Path = unicode:characters_to_binary(Path0),
    Slab = aq_slab_open(Path),
    ok = aq_slab_stamp(Slab, <<"tamper-target">>),
    <<"tamper-target">> = aq_slab_frame(Slab, 0),
    %% Corrupt the payload THROUGH THE FILE (pwrite): macOS unified buffer
    %% cache makes the byte visible to the mapping.
    {ok, F} = file:open(Path0, [read, write, binary]),
    ok = file:pwrite(F, 4096 + 16, <<$X>>),
    ok = file:close(F),
    %% Typed in-memory CrcMismatch refusal -> atom error; never a binary over
    %% bad bytes.
    error = aq_slab_frame(Slab, 0),
    error = aq_slab_window(Slab, 0, 0, 1),
    io:format("ok crc_negative_leg~n").

t_lease_lifecycle() ->
    Dir = mktempd(),
    Path = make_segment(Dir, "seg.wire", 4),
    Slab = aq_slab_open(unicode:characters_to_binary(Path)),
    %% Fill slots 0..1 so next_ordinal = 2. Ordering below is by receive
    %% events with timeouts (runtime-owned time), never sleeps.
    ok = aq_slab_stamp(Slab, <<"first">>),
    ok = aq_slab_stamp(Slab, <<"second">>),
    <<"first">> = aq_slab_frame(Slab, 0),
    %% Child holds the leases so they can actually die (stack-bound vars stay
    %% alive in the caller until its function returns; GC cannot free them).
    Parent = self(),
    Child = spawn(fun() ->
        L1 = aq_slab_lease(Slab, 2),
        true = is_reference(L1),
        %% Deterministic refusal inside the holder: slot 2 is the next write
        %% target and its bit is set -> SlabSlotLeased.
        error = aq_slab_stamp(Slab, <<"third">>),
        %% Double lease -> error (bit already set).
        error = aq_slab_lease(Slab, 2),
        L2 = aq_slab_lease(Slab, 3),
        true = is_reference(L2),
        Parent ! {leases_done, self()}
    end),
    Ref = erlang:monitor(process, Child),
    receive
        {leases_done, Child} -> ok
    after 5000 -> erlang:error({lease_child_timeouts, leases_done})
    end,
    receive
        {'DOWN', Ref, process, Child, _Reason} -> ok
    after 5000 -> erlang:error({lease_child_timeouts, down})
    end,
    %% Bounded poll on the stamp observable until the lease dtors cleared
    %% the bits (GC of the child's terms fires them asynchronously).
    wait_stamp_ok(Slab, <<"third">>, 500),
    <<"third">> = aq_slab_frame(Slab, 2),
    io:format("ok lease_lifecycle~n").

wait_stamp_ok(_Slab, _Payload, 0) ->
    erlang:error({lease_dtor_did_not_fire, bounded_poll_exhausted});
wait_stamp_ok(Slab, Payload, N) ->
    garbage_collect(),
    case aq_slab_stamp(Slab, Payload) of
        ok -> ok;
        error -> wait_stamp_ok(Slab, Payload, N - 1)
    end.

t_alloc_spy_egress() ->
    Dir = mktempd(),
    Path = make_segment(Dir, "seg.wire", 4),
    Slab = aq_slab_open(unicode:characters_to_binary(Path)),
    ok = aq_slab_stamp(Slab, <<"census-frame">>),
    {A0, B0} = cgap_alloc_stats(),
    K = 1000,
    ReadF = fun F(0, Acc) -> Acc;
               F(N, Acc) ->
                  Bin = aq_slab_frame(Slab, 0),
                  Acc2 = Acc + byte_size(Bin),
                  F(N - 1, Acc2)
           end,
    Total = ReadF(K, 0),
    Expected = K * byte_size(<<"census-frame">>),
    Expected = Total,
    {A1, B1} = cgap_alloc_stats(),
    %% S12 egress law: 0 counted allocs, 0 counted bytes over K projections.
    {A0, B0} = {A1, B1},
    %% Window census too.
    WF = fun G(0, Acc) -> Acc;
            G(N, Acc) ->
                Win = aq_slab_window(Slab, 0, 2, 4),
                Acc2 = Acc + byte_size(Win),
                G(N - 1, Acc2)
        end,
    WTotal = WF(K, 0),
    WExpected = K * 4,
    WExpected = WTotal,
    {A1, B1} = cgap_alloc_stats(),
    io:format("ok alloc_spy_egress (K=~p frames+windows, delta 0,0)~n", [K]).

%% mktempd/0: a fresh temp dir per leg (real files on disk, no mocks).
mktempd() ->
    D = filename:join([tmp_base(), "aq_court_", integer_to_list(erlang:unique_integer([positive]))]),
    ok = filelib:ensure_dir(filename:join(D, "x")),
    D.

tmp_base() ->
    case init:get_argument(tmpdir) of
        {ok, [[T]]} -> T;
        _ -> "/tmp"
    end.

"##;
#[test]
fn aq_slab_be_court() {
    let dylib = nif_dylib_path();
    let escript = write_escript(ESCRIPT);
    // Leg 1: the court against the real validator must pass.
    let rc = run_court(&escript, &dylib, "");
    assert_eq!(rc, 0, "court against the real validator must pass (exit 0), got {rc}");
    // Leg 2 (SHIM-COURT anti-vacuity): the SAME court under the crc-skip
    // mutation must FLIP — exit nonzero, failing in the crc leg.
    let rc_m = run_court(&escript, &dylib, "crc");
    assert_ne!(rc_m, 0, "court under AQ_MUTATE_CRC_SKIP=1 must flip (exit nonzero), got {rc_m}");
    // Leg 3 (L23 window SHIM-COURT): under AQ_MUTATE_WINDOW_OOB_SKIP=1 the
    // mutant leg must PASS (exit 0) — the over-reading window (3,7) past
    // plen=5 is ACCEPTED by the mutated boundary, proving the baseline `error`
    // refusal is enforced by exactly this bounds check. A mutation leg that
    // exits nonzero is indistinguishable from an unapplied mutation = vacuous.
    let rc_w = run_court(&escript, &dylib, "window-oob");
    assert_eq!(
        rc_w, 0,
        "window OOB mutation leg must PASS under AQ_MUTATE_WINDOW_OOB_SKIP=1 (flip witnessed: over-read accepted), got {rc_w}"
    );
    let _ = std::fs::remove_file(&escript);
}
