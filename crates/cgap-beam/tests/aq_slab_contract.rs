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
    let p = std::env::temp_dir().join(format!("aq_slab_court_{}.erl", std::process::id()));
    std::fs::write(&p, src).expect("write escript");
    p
}

fn run_court(escript: &PathBuf, dylib: &PathBuf, mutate: bool) -> i32 {
    let mut cmd = Command::new("escript");
    cmd.arg(escript).arg(dylib);
    if mutate {
        cmd.env("AQ_MUTATE_CRC_SKIP", "1");
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
-module(aq_slab_court).
-export([main/1]).

aq_slab_open(_) -> erlang:nif_error(nif_not_loaded).
aq_slab_frame(_, _) -> erlang:nif_error(nif_not_loaded).
aq_slab_window(_, _, _, _) -> erlang:nif_error(nif_not_loaded).
aq_slab_stamp(_, _) -> erlang:nif_error(nif_not_loaded).
aq_slab_lease(_, _) -> erlang:nif_error(nif_not_loaded).
cgap_alloc_stats() -> erlang:nif_error(nif_not_loaded).

main([NifPath]) ->
    ok = case erlang:load_nif(NifPath, 0) of
             ok -> ok;
             {error, R} -> erlang:error({load_nif_failed, R})
         end,
    Res = try contracts() of
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
    error = aq_slab_open(Wide),
    %% capacity-field vs file-size mismatch (ring.rs:57-71 law).
    Path = filename:join(Dir, "mismatch.wire"),
    ok = file:write_file(Path, <<0:((4096 + 2 * 4096) * 8)/unit:1>>),
    error = aq_slab_open(Path),
    %% missing file
    error = aq_slab_open(filename:join(Dir, "nope.wire")),
    %% non-binary arg
    error = aq_slab_open(42),
    io:format("ok open_refusals~n").

t_stamp_frame_window() ->
    Dir = mktempd(),
    Cap = 4,
    Path = make_segment(Dir, "seg.wire", Cap),
    Slab = aq_slab_open(Path),
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

t_crc_negative_leg() ->
    Dir = mktempd(),
    Path = make_segment(Dir, "seg.wire", 2),
    Slab = aq_slab_open(Path),
    ok = aq_slab_stamp(Slab, <<"tamper-target">>),
    <<"tamper-target">> = aq_slab_frame(Slab, 0),
    %% Corrupt the payload THROUGH THE FILE (pwrite): macOS unified buffer
    %% cache makes the byte visible to the mapping.
    {ok, 16} = file:pwrite(Path, 4096 + 16, <<$X>>),
    %% Typed in-memory CrcMismatch refusal -> atom error; never a binary over
    %% bad bytes.
    error = aq_slab_frame(Slab, 0),
    error = aq_slab_window(Slab, 0, 0, 1),
    io:format("ok crc_negative_leg~n").

t_lease_lifecycle() ->
t_lease_lifecycle() ->
    Dir = mktempd(),
    Path = make_segment(Dir, "seg.wire", 4),
    Slab = aq_slab_open(Path),
    %% Fill slots 0..1 so next_ordinal = 2; lease slot 2 (the next write
    %% target) and prove stamp refuses it while the bit is set.
    ok = aq_slab_stamp(Slab, <<"first">>),
    ok = aq_slab_stamp(Slab, <<"second">>),
    <<"first">> = aq_slab_frame(Slab, 0),
    Lease = aq_slab_lease(Slab, 2),
    true = is_reference(Lease),
    error = aq_slab_stamp(Slab, <<"third">>),
    %% Double lease -> error (bit already set).
    error = aq_slab_lease(Slab, 2),
    %% Lease slot 3 too; then drop BOTH lease refs and poll (bounded, on the
    %% stamp observable, no sleeps) until the dtor cleared the bits.
    Lease2 = aLease2_placeholder

    Lease2 = aq_slab_lease(Slab, 3),
    true = is_reference(Lease2),
    error = aq_slab_stamp(Slab, <<"third">>),
    Leases = [Lease, Lease2],
    Leases2 = [Leases, Lease2],
    _ = Leases2,
    %% Bounded poll on the stamp observable until the lease dtors clear the
    %% bits (no sleeps).
    wait_stamp_ok(Slab, <<"third">>, 200),
    <<"third">> = aq_slab_frame(Slab, 2),
    io:format("ok lease_lifecycle~n").

wait_stamp_ok(_Slab, _Payload, 0) ->
    erlang:error({lease_dtor_did_not_fire, bounded_poll_exhausted});
wait_stamp_ok(Slab, Payload, N) ->
    case aq_slab_stamp(Slab, Payload) of
        ok -> ok;
        _ -> garbage_collect(), wait_stamp_ok(Slab, Payload, N - 1)
    end.


t_alloc_spy_egress() ->
    Dir = mktempd(),
    Path = make_segment(Dir, "seg.wire", 4),
    Slab = aq_slab_open(Path),
    ok = aq_slab_stamp(Slab, <<"census-frame">>),
    {A0, B0} = cgap_alloc_stats(),
    K = 1000,
    Read = fun Read(0, Acc) -> Acc;
               Read(N, Acc) ->
                  Bin = aq_slab_frame(Slab, 0),
                  Acc2 = Acc + byte_size(Bin),
                  Read(N - 1, Acc2)
           end,
    Total = Read(K, 0),
    K * byte_size(<<"census-frame">>) = Total,
    {A1, B1} = cgap_alloc_stats(),
    %% S12 egress law: 0 counted allocs, 0 counted bytes over K projections.
    {A0, B0} = {A1, B1},
    %% Window census too.
    W = fun W(0, Acc) -> Acc;
            W(N, Acc) ->
                Win = aq_slab_window(Slab, 0, 2, 4),
                Acc2 = Acc + byte_size(Win),
                W(N - 1, Acc2)
        end,
    WTotal = W(K, 0),
    K * 4 = WTotal,
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

main2() ->
    ok.

"##

#[test]
fn aq_slab_be_court() {
    let dylib = nif_dylib_path();
    let escript = write_escript(ESCRIPT);
    // Leg 1: the court against the real validator must pass.
    let rc = run_court(&escript, &dylib, false);
    assert_eq!(rc, 0, "court against the real validator must pass (exit 0), got {rc}");
    // Leg 2 (SHIM-COURT anti-vacuity): the SAME court under the crc-skip
    // mutation must FLIP — exit nonzero, failing in the crc leg.
    let rc_m = run_court(&escript, &dylib, true);
    assert_ne!(rc_m, 0, "court under AQ_MUTATE_CRC_SKIP=1 must flip (exit nonzero), got {rc_m}");
    let _ = std::fs::remove_file(&escript);
}
