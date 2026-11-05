//! Breach-regression court (lane R-14 / FIX-17, v26.11.2 tollgate).
//!
//! Encodes the four witnessed hard breaches as regression legs — the court
//! that must stay green exactly when the four fixes (FIX-01/02/04/05) hold:
//!
//! - Leg 1 (FIX-01, RED-13): path-alias lease race. Owner holds `dir/x`;
//!   racer attacks via `dir/./x`. Post-rekey the racer is DENIED
//!   0x4347_0006. Asserted here by running FIX-01's own flip-bearing court
//!   leg (`fa9_alias_race_denied`) as a typed subprocess (cargo, distinct
//!   target lease so the parent's target lock is never contended).
//! - Leg 2 (FIX-02/RC-03, RED-12): full-capacity bn3 init returns the
//!   TYPED `CapacityExceeded` with exact red-12 numbers (no silent
//!   49.75x-ceiling SRAM overrun).
//! - Leg 3 (FIX-05, RED-09 F1/F2): post-trailer poison is a typed
//!   `TrailingBytes` refusal; surplus-object header lies are refused with
//!   objects_seen pinned at declared.
//! - Leg 4 (FIX-04, RED-14): a 1,000,000-frame class-101 refusal flood
//!   escalates the typed `RefusalQuota`, and a real admit+drain still
//!   succeeds after the flood (liveness, no sleeps).
//! - Leg 5 (FIX-02, RED-12): the SRAM gate itself refuses bn3 — typed
//!   subprocess assertion of `verify_cscb_sram.py --subject bn3` (exit 1
//!   + `BN3_ENCLAVE_REFUSED`), plus the gate's own `--mutate bn3-pin`
//!     anti-vacuity court (exit 0 = flip witnessed).
//!
//! Anti-vacuity law: every leg names its mutation (see the per-leg doc
//! comments and the lane receipt). Mutations were applied to /tmp sandbox
//! copies with sha256 fail-closed restore + mtime bump; the shared tree
//! was never mutated. A court that cannot flip is FAIL; skips are loud.
//!
//! Chicago discipline: real collaborators (real threads, real files, real
//! subprocesses, real byte-built packs), final-state asserts, no mocks,
//! no sleeps (flood + bounded subprocess exit codes, not polling).

use std::path::PathBuf;
use std::process::Command;

/// Repo root: this test lives at <root>/crates/cgap-git/tests/.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/cgap-git")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

/// Subprocess legs run against the workspace root this test was built in
/// (the sandbox copy inherits this file unchanged, so sandbox mutation
/// runs exercise the sandboxed collaborators automatically).
fn run_cargo_leg(test_target: &str) -> Result<String, String> {
    let root = repo_root();
    let status = Command::new(env!("CARGO"))
        .args([
            "test",
            "-p",
            "cgap-ffi",
            "--test",
            "fa9_commit_gating",
            "fa9_alias_race_denied",
        ])
        .current_dir(&root)
        .env("CARGO_TARGET_DIR", "/tmp/r14-leg1-target")
        .env_remove("RUSTC_WRAPPER")
        .output()
        .map_err(|e| format!("spawn cargo: {e}"))?;
    let out = format!(
        "{}{}",
        String::from_utf8_lossy(&status.stdout),
        String::from_utf8_lossy(&status.stderr)
    );
    if !status.status.success() {
        return Err(format!(
            "exit={:?} output tail: {}",
            status.status.code(),
            tail(&out)
        ));
    }
    if !out.contains(test_target) {
        return Err(format!("leg output missing '{test_target}'"));
    }
    Ok(out)
}

fn tail(s: &str) -> String {
    let n = s.len();
    s.chars().skip(n.saturating_sub(600)).collect()
}

/// LEG 1 — FIX-01 (RED-13) path-alias lease race.
/// ANTI-VACUITY MUTATION: the sufficient revert is the TWO-LAYER M4 in a
/// /tmp sandbox copy of sovereign.rs — canonical_key back to raw FNV-1a
/// of the raw path bytes AND identity_key to None. A single-layer revert
/// (canonical_key alone) PASSES: the redundant identity_key defense
/// holds the line. Witnessed: R-14 D5 and adv-14 mutations
/// M-L1-TEXTONLY / M-L1-INOONLY both exit 0 (no flip); only the two-layer
/// revert flips this leg's subprocess court (racer ADMITTED, exit 101).
#[test]
fn breach1_path_alias_lease_race_denied() {
    run_cargo_leg("test fa9_alias_race_denied ... ok")
        .unwrap_or_else(|e| panic!("LEG1 FLIP/BREACH: alias race admitted — {e}"));
}

/// LEG 5 — FIX-02 (RED-12) SRAM gate refusal for bn3, typed subprocess.
/// The gate MUST exit 1 with the loud typed refusal; the gate's own
/// `--mutate bn3-pin` court MUST exit 0 (flip witnessed in-gate).
/// ANTI-VACUITY MUTATION (sandbox M5): neutralize the bn3 charge in a
/// /tmp copy of the gate (huge budget) — this leg's exit-1 assertion
/// FLIPS (gate admits, exit 0) — witnessed in the lane receipt.
#[test]
fn breach5_sram_gate_refuses_bn3_typed() {
    let root = repo_root();
    let gate = root.join("scripts").join("verify_cscb_sram.py");
    assert!(gate.exists(), "SRAM gate missing at {}", gate.display());
    let py_out = Command::new("python3")
        .arg(&gate)
        .arg("--subject")
        .arg("bn3")
        .current_dir(&root)
        .output()
        .expect("spawn python3 gate");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&py_out.stdout),
        String::from_utf8_lossy(&py_out.stderr)
    );
    assert_eq!(
        py_out.status.code(),
        Some(1),
        "gate must refuse bn3 with exit 1 (typed refusal), got {:?}; out: {}",
        py_out.status.code(),
        tail(&text)
    );
    assert!(
        text.contains("BN3_ENCLAVE_REFUSED"),
        "typed refusal marker missing; out: {}",
        tail(&text)
    );

    // In-gate anti-vacuity court: the bn3-pin mutation MUST flip (exit 0).
    let mut_out = Command::new("python3")
        .arg(&gate)
        .arg("--mutate")
        .arg("bn3-pin")
        .current_dir(&root)
        .output()
        .expect("spawn python3 gate --mutate bn3-pin");
    let mut_text = format!(
        "{}{}",
        String::from_utf8_lossy(&mut_out.stdout),
        String::from_utf8_lossy(&mut_out.stderr)
    );
    assert_eq!(
        mut_out.status.code(),
        Some(0),
        "gate's own bn3-pin anti-vacuity court must flip (exit 0); out: {}",
        tail(&mut_text)
    );
    assert!(
        mut_text.contains("DIVERGED"),
        "expected DIVERGED verdict from --mutate bn3-pin; out: {}",
        tail(&mut_text)
    );
}

/// LEG 2 — FIX-02/RC-03 (RED-12) bn3 capacity: full-capacity init at the
/// 131,072 B enclave ceiling returns the TYPED `CapacityExceeded` with
/// the exact red-12 host-size_of numbers. Final-state asserts; nothing is
/// constructed on Err (construction would blow the default test stack).
/// ANTI-VACUITY MUTATION: relax `NaMatches::init_within` to always Ok in
/// a /tmp sandbox copy of cgap-bn3 (the pre-typed-refusal shape) — this
/// leg FLIPS — witnessed in the lane receipt.
#[test]
fn breach2_bn3_full_capacity_init_refuses_typed() {
    // NaMatches (~2.16 MB) / NaEngine (~4.3 MB) Result slots exceed the
    // default 2 MiB test stack: run the refusal body on a big-stack thread.
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            use cgap_bn3::noalloc::{
                na_enclave_charge_bytes, na_init_capacity_check, CapacityExceeded, NaEngine,
                NaMatches, NA_SRAM_CEILING_BYTES,
            };
            assert_eq!(NA_SRAM_CEILING_BYTES, 131_072);
            let m = NaMatches::init_enclave();
            assert_eq!(
                m,
                Err(CapacityExceeded {
                    required_bytes: 2_162_696,
                    ceiling_bytes: NA_SRAM_CEILING_BYTES
                })
            );
            let Err(err_e) = NaEngine::init_enclave() else {
                panic!("VACUOUS: NaEngine::init_enclave admitted at the ceiling");
            };
            assert_eq!(
                err_e,
                CapacityExceeded {
                    required_bytes: 4_358_288,
                    ceiling_bytes: NA_SRAM_CEILING_BYTES
                }
            );
            assert_eq!(
                na_init_capacity_check(NA_SRAM_CEILING_BYTES),
                Err(CapacityExceeded {
                    required_bytes: 6_520_984,
                    ceiling_bytes: NA_SRAM_CEILING_BYTES
                })
            );
            assert_eq!(na_enclave_charge_bytes(), 6_520_984);
        })
        .expect("spawn 64 MiB stack")
        .join()
        .expect("leg2 refusal body ok");
}

/// LEG 3 — FIX-05 (RED-09 F1/F2) pack_demux poison + surplus guard.
/// Real byte-built packs via cgap_git::pack_write (stored framing), no
/// mocks. ANTI-VACUITY MUTATION: revert the `Phase::Done` arm to `{}` in
/// a /tmp sandbox copy of pack_demux.rs (pre-FIX-05 shape) — both asserts
/// below FLIP (feed returns Ok, poison silently absorbed) — witnessed in
/// the lane receipt.
#[test]
fn breach3_pack_demux_poison_is_typed_refusal() {
    use cgap_git::pack_demux::{Demux, DemuxError};
    use cgap_git::pack_write::{encode_object_header, sha1};

    // Stored-framing pack builder (mirrors pack_demux_court.rs).
    fn build_pack(objs: &[(u8, Vec<u8>)]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"PACK");
        out.extend_from_slice(&2u32.to_be_bytes());
        out.extend_from_slice(&(objs.len() as u32).to_be_bytes());
        for (t, body) in objs {
            out.extend_from_slice(&encode_object_header(*t, body.len() as u64));
            out.extend_from_slice(body);
        }
        let sum = sha1(&out);
        out.extend_from_slice(&sum);
        out
    }

    // 3a: clean-pack baseline (zero poison) — positive control.
    let pack = build_pack(&[(3u8, b"body".to_vec())]);
    let mut clean = Demux::new();
    clean.feed(&pack).expect("clean pack feed");
    assert!(clean.is_done());
    assert_eq!(clean.finish(), Ok(()));
    assert_eq!(clean.trailing_poison(), 0);

    // 3b: post-trailer poison -> typed TrailingBytes(4), trailer intact.
    let mut poisoned = pack.clone();
    poisoned.extend_from_slice(b"\xDE\xAD\xBE\xEF");
    let mut d = Demux::new();
    let err = d
        .feed(&poisoned)
        .expect_err("VACUOUS: post-trailer poison silently absorbed (FIX-05 reverted?)");
    assert_eq!(err, DemuxError::TrailingBytes(4));
    assert_eq!(d.trailing_poison(), 4);
    assert!(d.is_done());
    assert_eq!(d.finish(), Ok(()));
    assert_eq!(d.objects_seen(), 1);
    assert_eq!(
        d.captured_trailer().expect("trailer"),
        sha1(&pack[..pack.len() - 20])
    );

    // 3c: surplus-object header lie (declare 1, supply 2) -> 26
    // post-trailer bytes (6 surplus framing + 20 stale checksum) refused
    // typed; objects_seen pinned at declared.
    let surplus_body = vec![b's'; 24];
    let mut lie = build_pack(&[(3u8, b"body".to_vec()), (3u8, surplus_body)]);
    lie[8..12].copy_from_slice(&1u32.to_be_bytes());
    let mut dd = Demux::new();
    let err2 = dd
        .feed(&lie)
        .expect_err("VACUOUS: surplus object absorbed into a clean finish");
    assert_eq!(err2, DemuxError::TrailingBytes(26));
    assert_eq!(dd.objects_seen(), dd.declared_objects().unwrap());
}

/// LEG 4 — FIX-04 (RED-14) + ADV-12 repair: andon liveness under a
/// 1,000,000-refusal flood. Post-ADV-12 the escalated state is
/// LEVEL-TRIGGERED: one admitted frame does NOT clear `tripped` (sticky);
/// it clears after `CLEAN_ADMISSIONS_TO_CLEAR` (1024) consecutive clean
/// admissions. ANTI-VACUITY MUTATION: delete the `note_refusal`
/// escalation (`self.tripped = true`) in a /tmp sandbox copy of
/// ingress_ring.rs — the `tripped` assert below FLIPS — witnessed in the
/// lane receipt.
#[test]
fn breach4_andon_flood_then_admit_drain_succeeds() {
    use cgap_git::ingress_ring::IngressRing;
    let mut ring: IngressRing<1024> = IngressRing::new();
    let garbage = [0u8; 40_000];
    for _ in 0..1_000_000 {
        match ring.push(&garbage) {
            Err(e) => assert_eq!(e.andon_exit_class(), 101),
            Ok(()) => panic!("oversized frame admitted"),
        }
    }
    // Final state: flood admitted nothing; quota escalated; lifetime
    // audit counter exact.
    assert_eq!(ring.pending(), 0);
    assert!(
        ring.refusal_quota().tripped(),
        "VACUOUS: 1M flood did not escalate the quota (FIX-04 reverted?)"
    );
    assert_eq!(ring.refusal_quota().total_refusals, 1_000_000);

    // Post-flood liveness: a real frame is ADMITTED and drained
    // byte-exact even while the andon signal is sticky (ADV-12: one
    // admitted frame must NOT silently erase the escalation).
    let legit = [0xABu8; 256];
    ring.push(&legit).expect("admission after flood");
    assert_eq!(ring.pending(), 256);
    assert!(
        ring.refusal_quota().tripped(),
        "ADV-12 contract: one admission must NOT clear the sticky escalation"
    );
    let mut out = [0u8; 256];
    let n = ring.pop(&mut out).expect("drain after flood");
    assert_eq!(n, 256);
    assert!(out.iter().all(|&b| b == 0xAB));
    assert_eq!(ring.refusal_quota().total_refusals, 1_000_000);

    // Recoverability: 1024 consecutive clean admissions clear the
    // escalation (level-triggered auto-clear); the lifetime audit counter
    // survives the recovery.
    for _ in 0..1024 {
        let frame = [0x5Au8; 64];
        ring.push(&frame).expect("clean admission during recovery");
        let mut buf = [0u8; 64];
        let got = ring.pop(&mut buf).expect("clean drain during recovery");
        assert_eq!(got, 64);
    }
    assert!(
        !ring.refusal_quota().tripped(),
        "1024 consecutive clean admissions must clear the escalation"
    );
    assert_eq!(ring.refusal_quota().total_refusals, 1_000_000);
}
