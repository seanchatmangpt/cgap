//! KD-7 court — `cgap_p28_invariant_check` header decision (v26.11.14 lane E2).
//!
//! Decision (re-derived from disk): the symbol is EXPOSED. include/cgap.h
//! carries exactly one prototype `uint32_t cgap_p28_invariant_check(const
//! uint8_t* att);` (FIX-07 slot surface). The ledger name
//! `_cgap_p28_invariant_check` (leading underscore) never existed as an
//! export; it must not appear in the header.
//!
//! Anti-vacuity mutations (on in-memory copies of the header text; the
//! shared tree is never written): M1 drop the prototype, M2 duplicate it,
//! M3 mutate the parameter type. Each must flip the verdict to Err.

use std::path::PathBuf;

const PROTO: &str = "uint32_t cgap_p28_invariant_check(const uint8_t* att);";

// Real collaborator: the cgap-p28 export itself (rides the link as a dep).
use cgap_p28::trap::cgap_p28_invariant_check;

fn header() -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../include/cgap.h");
    std::fs::read_to_string(p).expect("read include/cgap.h")
}

fn verdict(h: &str) -> Result<(), String> {
    let n = h.matches("cgap_p28_invariant_check").count();
    if h.contains("_cgap_p28_invariant_check") {
        return Err("underscore-prefixed phantom name present".into());
    }
    let protos = h.lines().filter(|l| l.trim() == PROTO).count();
    if protos != 1 {
        return Err(format!("expected exactly 1 prototype, found {protos}"));
    }
    if n < 1 {
        return Err("symbol absent".into());
    }
    Ok(())
}

#[test]
fn kd7_header_declares_p28_invariant_check_once() {
    verdict(&header()).expect("checked-in header must declare the symbol once");
}

#[test]
fn kd7_symbol_rides_the_link() {
    let f: unsafe extern "C" fn(*const u8) -> u32 = cgap_p28_invariant_check;
    assert!(f as usize != 0);
}

#[test]
fn kd7_mutations_all_flip() {
    let h = header();
    let m1 = h.replace(PROTO, "");
    let m2 = h.replace(PROTO, &format!("{PROTO}\n{PROTO}"));
    let m3 = h.replace(PROTO, "uint32_t cgap_p28_invariant_check(const uint8_t* x);");
    let m4 = format!("{h}\n_cgap_p28_invariant_check\n");
    for (n, m) in [("M1-drop", m1), ("M2-dup", m2), ("M3-param", m3), ("M4-phantom", m4)] {
        assert!(verdict(&m).is_err(), "VACUOUS GATE: {n} not detected");
    }
}
