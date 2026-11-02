//! Lane A17 finisher — integration courts over the PUBLIC API seam
//! (`crates/cgap-p28/tests/`, previously absent).
//!
//! What this target adds beyond the in-crate unit courts (src/attestation.rs,
//! src/trap.rs): a consumer's view of the crate — public exports only, the
//! REAL pinned instrument (`LICENSE-P28-SIL.md`) read from disk (the unit
//! wire sweep uses a synthetic instrument; this one does not), sweep counts
//! WITNESSED in stdout (A17 dispatch: "count witnessed in output"), and the
//! §7.2 strip form witnessed as a REAL process death through the public halt
//! edge. No mocks; every assertion is final state.
//!
//! Spec: docs/jira/v26.11.1/lanes/A17-p28-derivative.md (§2 layout, §5
//! ordered validation, §7.1 sweep, §7.2 stripping, §7.3 controls).

use core::mem::{align_of, offset_of, size_of};
use std::process::Command;

use cgap_p28::attestation::{
    instrument_header_span, origin_authority_digest, P28LicenseAttestation, Spoliation,
    ATTESTATION_LEN, ATTESTATION_VERSION, FLAG_DERIVATIVE, FLAG_LINEAGE_MASK, FLAG_RESERVED_MASK,
    FLAG_ROOT, FLAG_TRANSPILED, LICENSE_MAGIC, SAFE_HARBOR_CEILING_MAX_CENTS,
};
use cgap_p28::trap::{spoliation_halt, ERR_P28_SPOLIATION, STAGED_DISCRIMINATOR};

/// The REAL pinned instrument, exactly as on disk (§3: digest is computed
/// over instrument bytes; the URN is de-referenced out-of-band).
fn instrument_file_bytes() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../LICENSE-P28-SIL.md"
    ))
    .expect("pinned instrument LICENSE-P28-SIL.md is readable from the repo root")
}

/// §3 canonical span of the real pinned instrument.
fn instrument_span() -> Vec<u8> {
    instrument_header_span(&instrument_file_bytes())
        .map(|s| s.to_vec())
        .expect("canonical instrument has a `---` line")
}

/// Valid Root record over the real instrument span (§2 field values).
fn root_for(span: &[u8]) -> P28LicenseAttestation {
    P28LicenseAttestation {
        license_magic: LICENSE_MAGIC,
        version: ATTESTATION_VERSION,
        attestation_flags: FLAG_ROOT,
        safe_harbor_ceiling_cents: SAFE_HARBOR_CEILING_MAX_CENTS,
        origin_authority_digest: origin_authority_digest(span),
        upstream_receipt: [0u8; 32],
    }
}

/// Valid Derivative record bound to its parent's wire bytes (§5.7).
fn derivative_from(parent: &P28LicenseAttestation) -> P28LicenseAttestation {
    P28LicenseAttestation {
        license_magic: LICENSE_MAGIC,
        version: ATTESTATION_VERSION,
        attestation_flags: FLAG_DERIVATIVE,
        safe_harbor_ceiling_cents: SAFE_HARBOR_CEILING_MAX_CENTS,
        origin_authority_digest: parent.origin_authority_digest,
        upstream_receipt: cgap_blake3::hash(&parent.to_bytes()),
    }
}

/// D-A17-1 re-derived survivor law (§7.1): with the ceiling pinned at MAX,
/// the only tolerated single-bit mutations are CLEAR-flips of the SET bits
/// of the ceiling field (bytes 8..16) — clearing a set bit lowers the value,
/// so it stays <= MAX (§5.8). Hand-count pin: popcount(5_000_000_000) = 11.
fn predicted_ceiling_survivors(good: &[u8; 80]) -> Vec<(usize, u8)> {
    let mut v = Vec::new();
    for i in 8..16 {
        for j in 0..8u8 {
            if good[i] & (1 << j) != 0 {
                v.push((i, j));
            }
        }
    }
    v
}

/// The real §5 verdict a consumer runs: wire bytes -> parse -> full ordered
/// validation against the real pinned instrument span.
fn real_verdict(
    wire: &[u8; 80],
    parent: Option<&P28LicenseAttestation>,
    span: &[u8],
) -> bool {
    P28LicenseAttestation::from_bytes(wire)
        .and_then(|r| r.validate(span, parent))
        .is_ok()
}

/// Parent-side child spawn for the strip court (§7.4 subprocess mechanics).
fn spawn_strip_child(
    exe: &std::path::Path,
    artifact: &std::path::Path,
) -> (bool, bool, bool) {
    let out = Command::new(exe)
        .args(["--exact", "strip_halt_child", "--nocapture"])
        .env("CGAP_P28_STRIP_CHILD", "1")
        .env("CGAP_P28_ARTIFACT", artifact)
        .output()
        .expect("spawn strip-halt child");
    let sigabrt = {
        #[cfg(unix)]
        {
            std::os::unix::process::ExitStatusExt::signal(&out.status) == Some(6)
        }
        #[cfg(not(unix))]
        {
            false
        }
    };
    (
        out.status.success(),
        sigabrt,
        String::from_utf8_lossy(&out.stdout).contains("staged=0x503238ff"),
    )
}

/// Mission 1 — the §2/§2.1 layout is frozen, witnessed from the PUBLIC seam.
/// Anti-vacuity: any offset/size/flag/constant edit flips this court and the
/// compile-time pins in src/attestation.rs simultaneously.
#[test]
fn layout_frozen_integration() {
    assert_eq!(size_of::<P28LicenseAttestation>(), 80);
    assert_eq!(align_of::<P28LicenseAttestation>(), 8);
    assert_eq!(offset_of!(P28LicenseAttestation, license_magic), 0);
    assert_eq!(offset_of!(P28LicenseAttestation, version), 4);
    assert_eq!(offset_of!(P28LicenseAttestation, attestation_flags), 6);
    assert_eq!(
        offset_of!(P28LicenseAttestation, safe_harbor_ceiling_cents),
        8
    );
    assert_eq!(
        offset_of!(P28LicenseAttestation, origin_authority_digest),
        16
    );
    assert_eq!(offset_of!(P28LicenseAttestation, upstream_receipt), 48);
    // §2.2 flag-bit freeze + the authority pins this lane owns.
    assert_eq!(FLAG_ROOT, 0x0001);
    assert_eq!(FLAG_DERIVATIVE, 0x0002);
    assert_eq!(FLAG_TRANSPILED, 0x0004);
    assert_eq!(FLAG_LINEAGE_MASK | FLAG_RESERVED_MASK, u16::MAX);
    assert_eq!(LICENSE_MAGIC, *b"P28S");
    assert_eq!(ATTESTATION_VERSION, 1);
    assert_eq!(SAFE_HARBOR_CEILING_MAX_CENTS, 5_000_000_000);
}

/// Mission 3 — ceiling boundary court (§5.8), one place, real instrument
/// bytes: at exactly 5_000_000_000 the record ADMITS; at 5_000_000_001 it
/// is refused. Both the full §5 path and the intrinsic (C-ABI) path.
/// (The asc450 meter boundary is lane A18's module and spec, not this one.)
#[test]
fn ceiling_boundary_court() {
    let span = instrument_span();
    let mut root = root_for(&span);

    // Exactly at the ceiling: admitted (full §5 path, real instrument).
    assert!(
        root.validate(&span, None).is_ok(),
        "a record at exactly 5_000_000_000 cents must admit"
    );
    assert!(
        root.validate_intrinsic().is_ok(),
        "a record at exactly the ceiling passes the intrinsic path"
    );

    // One cent over: refused, full path.
    root.safe_harbor_ceiling_cents = SAFE_HARBOR_CEILING_MAX_CENTS + 1;
    assert!(
        root.validate(&span, None).is_err(),
        "5_000_000_001 cents must be refused (§5.8)"
    );

    // Same boundary in WIRE form: the refused value serialized, parsed back,
    // and re-validated — the boundary lives in the bytes, not the struct.
    let wire = root.to_bytes();
    let reparsed =
        P28LicenseAttestation::from_bytes(&wire).expect("80 bytes always parse");
    assert!(
        reparsed.validate(&span, None).is_err(),
        "wire-form 5_000_000_001 must be refused"
    );

    // And through the intrinsic path the C-ABI entry uses.
    assert!(reparsed.validate_intrinsic().is_err());
}

/// One sweep pass over `limit` single-bit mutations, classifying each by
/// `verdict`. Returns (executed, survivors). The count law and the survivor
/// law live at the call sites; this function only observes.
fn run_sweep(
    good: &[u8; 80],
    parent: Option<&P28LicenseAttestation>,
    span: &[u8],
    limit: usize,
    verdict: &dyn Fn(&[u8; 80], Option<&P28LicenseAttestation>, &[u8]) -> bool,
) -> (usize, Vec<(usize, u8)>) {
    let mut executed = 0usize;
    let mut survivors = Vec::new();
    for k in 0..limit {
        let (i, j) = (k / 8, (k % 8) as u8);
        assert!(i < ATTESTATION_LEN, "bit index {k} within the 640-bit record");
        let mut mutated = *good;
        mutated[i] ^= 1 << j;
        executed += 1;
        if verdict(&mutated, parent, span) {
            survivors.push((i, j));
        }
    }
    (executed, survivors)
}

/// Mission 2 — the exhaustive 640-bit single-bit flip sweep at the
/// integration seam, over the REAL pinned instrument (not synthetic bytes),
/// Root and Derivative records, through the full public §5 path.
///
/// Law (§7.1 + re-derivation D-A17-1): every one of the 640 single-bit
/// mutations must be refused by validation, EXCEPT the exactly-predicted
/// tolerated class — CLEAR-flips of the 11 set bits of the ceiling field at
/// MAX (clearing a set bit lowers 5_000_000_000, staying <= MAX; no other
/// single-bit mutation stays lawful). Positive control admits first, so the
/// sweep cannot pass against a reject-everything validator.
#[test]
fn bit_flip_sweep_640_integration() {
    let span = instrument_span();
    let root = root_for(&span);
    let deriv = derivative_from(&root);
    let mut total_executed = 0usize;
    let mut total_kills = 0usize;
    for (name, rec, parent) in [
        ("root", root, None),
        (
            "derivative",
            deriv,
            Some(&root) as Option<&P28LicenseAttestation>,
        ),
    ] {
        let good = rec.to_bytes();
        assert!(
            real_verdict(&good, parent, &span),
            "{name}: positive control must admit before the sweep"
        );
        let mut predicted = predicted_ceiling_survivors(&good);
        assert_eq!(
            predicted.len(),
            11,
            "{name}: popcount(5_000_000_000) == 11 (hand-count pin)"
        );
        predicted.sort_unstable();
        let (executed, mut survivors) = run_sweep(&good, parent, &span, 640, &real_verdict);
        assert_eq!(
            executed, 640,
            "{name}: sweep must execute all 640 flips (anti-vacuity count law)"
        );
        survivors.sort_unstable();
        assert_eq!(
            survivors, predicted,
            "{name}: survivors must be exactly the D-A17-1 ceiling set-bit class"
        );
        let kills = executed - survivors.len();
        total_executed += executed;
        total_kills += kills;
        println!(
            "{name}: executed {executed} single-bit flips, {kills} refused, \
             {} tolerated (D-A17-1 ceiling set-bit class)",
            survivors.len()
        );
    }
    assert_eq!(total_executed, 1280);
    assert_eq!(total_kills, 2 * (640 - 11));
    println!(
        "bit_flip_sweep_640_integration totals: \
         {total_executed} mutations executed, {total_kills} refused"
    );
}

/// Non-vacuity witness for the sweep laws themselves (§7.1 anti-vacuity,
/// in-process shim-court analog): the two laws — count == 640 and survivors
/// == the predicted class — must be able to FAIL. Injected-fault probes
/// (explicit mutations for the court's own non-vacuity, not test doubles):
///
/// Probe A — neutered trap: a validator that admits everything leaves all
/// 640 flips halt-free; the harness observes 640 survivors != 11, so the
/// real court's survivor law fires and the sweep exits nonzero.
///
/// Probe B — truncated sweep: a harness that skips mutations is caught by
/// the count law (100 != 640).
///
/// Probe C — reject-everything validator: 0 survivors != 11, caught by the
/// survivor law (and by the positive control before any sweep).
#[test]
fn sweep_vacuity_probes() {
    let span = instrument_span();
    let root = root_for(&span);
    let good = root.to_bytes();
    let mut predicted = predicted_ceiling_survivors(&good);
    predicted.sort_unstable();

    let (exec_a, surv_a) = run_sweep(&good, None, &span, 640, &|_, _, _| true);
    assert_eq!(exec_a, 640);
    assert_eq!(surv_a.len(), 640, "neutered trap: every flip is halt-free");
    assert_ne!(
        surv_a.len(),
        predicted.len(),
        "survivor law distinguishes a halt-free sweep (the vacuous case exits nonzero)"
    );

    let (exec_b, _surv_b) = run_sweep(&good, None, &span, 100, &real_verdict);
    assert_eq!(exec_b, 100);
    assert_ne!(exec_b, 640, "count law distinguishes a truncated sweep");

    let (exec_c, surv_c) = run_sweep(&good, None, &span, 640, &|_, _, _| false);
    assert_eq!(exec_c, 640);
    assert!(surv_c.is_empty(), "reject-all validator: no flip admitted");
    assert_ne!(
        surv_c.len(),
        predicted.len(),
        "survivor law distinguishes a reject-everything validator"
    );
}

/// Child entry for the §7.2 strip court: the production consumer path —
/// read the artifact file, parse, validate. A stripped/truncated header is
/// spoliation; the only lawful outcome is the real halt edge (§6.2), never
/// an advisory errno. Reaching the end admits the record (clean exit).
#[test]
fn strip_halt_child() {
    if std::env::var("CGAP_P28_STRIP_CHILD").as_deref() != Ok("1") {
        return; // normal suite run: inert entry point
    }
    let path =
        std::env::var("CGAP_P28_ARTIFACT").expect("CGAP_P28_ARTIFACT set in child mode");
    let bytes = std::fs::read(&path).expect("artifact file readable");
    let rec = match P28LicenseAttestation::from_bytes(&bytes) {
        Ok(r) => r,
        Err(Spoliation) => {
            // Header stripped entirely: stage the discriminator, print it
            // (post-mortem witness), take the real halt (§6.1/§6.2).
            STAGED_DISCRIMINATOR.store(
                ERR_P28_SPOLIATION,
                core::sync::atomic::Ordering::SeqCst,
            );
            println!(
                "staged={:#010x}",
                STAGED_DISCRIMINATOR.load(core::sync::atomic::Ordering::SeqCst)
            );
            spoliation_halt();
        }
    };
    let span = instrument_span();
    rec.validate(&span, None)
        .expect("valid record must be admitted by the full §5 path");
}

/// Mission 4 — strip the header entirely and watch spoliation fire, as a
/// REAL process death (§7.4: subprocess mechanics, final state asserted).
#[test]
fn strip_header_entirely_halts() {
    let span = instrument_span();
    let root = root_for(&span);
    let dir = std::env::temp_dir().join(format!("cgap-p28-a17-strip-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create court temp dir");
    let exe = std::env::current_exe().expect("test binary path");

    // Positive control: the full 80-byte record admits -> clean exit 0,
    // no SIGABRT, no staged discriminator.
    let full = dir.join("full.art");
    std::fs::write(&full, root.to_bytes()).expect("write full artifact");
    let (ok, sig, staged) = spawn_strip_child(&exe, &full);
    assert!(ok, "full record must admit (exit 0), got clean=false");
    assert!(!sig, "admission must not abort");
    assert!(!staged, "admission must not stage a spoliation discriminator");

    // §7.2: header stripped entirely (0-byte artifact) -> real death.
    let empty = dir.join("empty.art");
    std::fs::write(&empty, b"").expect("write empty artifact");
    let (ok, sig, staged) = spawn_strip_child(&exe, &empty);
    assert!(!ok, "stripped header must halt; child exited cleanly");
    #[cfg(unix)]
    assert!(sig, "stripped header must die by SIGABRT");
    assert!(staged, "death must stage 0x503238ff (§7.4)");

    // Truncated header (79 of 80 bytes) -> the same fail-closed death.
    let wire = root.to_bytes();
    let trunc = dir.join("trunc.art");
    std::fs::write(&trunc, &wire[..79]).expect("write truncated artifact");
    let (ok, sig, staged) = spawn_strip_child(&exe, &trunc);
    assert!(!ok, "truncated header must halt; child exited cleanly");
    #[cfg(unix)]
    assert!(sig, "truncated header must die by SIGABRT");
    assert!(staged, "death must stage 0x503238ff (§7.4)");

    let _ = std::fs::remove_dir_all(&dir);
}
