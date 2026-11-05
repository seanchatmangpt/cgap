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
