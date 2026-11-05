//! Lane F2-6 (SEAM-OABI) — the `.oabi` slot-lookup boundary verb: typed
//! UNSUPPORTED(scope) decision record + the surface-freeze court.
//!
//! DECISION RECORD (dispatch branch b — no third option). The F2-6 dispatch
//! offered (a) a `#[no_mangle] pub extern "C"` `.oabi` slot-lookup export
//! wired to `cgap_bont::oabi::OabiPackV1`, or (b) typed UNSUPPORTED(scope).
//! Branch (a) is out-of-scope on the A19 surface definition, on four
//! independent grounds, each cited:
//!
//! 1. `docs/jira/v26.11.1/lanes/A19-libcgap-ffi-xip.md:47` — "**Decision**:
//!    the entire cgap capability crosses the boundary as four functions."
//!    A `.oabi` slot lookup (lexeme/slot resolution as a standalone boundary
//!    call) is a fifth verb: it is neither admit, eval, commit, nor verify.
//! 2. `A19:57-60` — "Why exactly four": "a fifth verb is either one of these
//!    in costume or it belongs in a lane, not the ABI." `.oabi` consumption
//!    in the spec reaches the boundary ONLY inside eval (`A19:95`
//!    `cgap_eval_bn3`; §6.3:323 "execute one `.oabi` rule ... eval"; §6.2:318
//!    "flip an OABI version field -> ErrOabiMismatch" — an EVAL-producer
//!    fault, producers A12+A13 per the frozen register
//!    (`sovereign.rs:74-76`), never a lookup verb.
//! 3. `A19:157-166` — the owner table assigns the four BODIES to owner
//!    lanes (A14+A16 / A12+A13 / A02+A03 / A17); A19 owns signatures,
//!    header, status enum, XIP script, courts — no slot-lookup body is
//!    assigned to any lane, and include/cgap.h is the Wave-1 ABI freeze of
//!    exactly the four prototypes (header file-top: "lane A19 Wave 1 ABI
//!    freeze").
//! 4. Structural: wiring `OabiPackV1` needs a cgap-bont dep edge in
//!    `crates/cgap-ffi/Cargo.toml` — outside this lane's lease (same
//!    structural blocker the A19 finish receipt recorded at DoD 5,
//!    `waves/a19-finish-receipt.md:158`: "Cargo.toml is outside this lane's
//!    lease — the bodies cannot land in-lane"). Re-implementing the parser
//!    in-lane is refused by the O(N^2) drift law (one calculus, one
//!    kernel; cite don't redefine A06 §6).
//!
//! Cross-lane context: A10 T5 (`A10-cloud-oabi.md:313-321`) expects "the
//! libcgap boundary stub defined for lane A19's in-repo tests"; no such
//! stub exists in the A19 spec, and A10 §3.2:131 defers the surface to
//! A19 ("cited, not redefined"). With this record, T5's blocker remains
//! BLOCKED-on-A19 (standing flip path: A10 §10, post-hoc when the owner
//! lanes land a boundary surface that names slot resolution).
//!
//! This module therefore carries the typed record below PLUS the
//! enforcement court: the boundary surface must carry NO `.oabi` verb, and
//! the court FLIPS if one ever appears (a refusal with no court is prose;
//! see testing-chicago-style). Host + `no_std` dual: the record is
//! core-only; the court is `#[cfg(test)]`.
//!
//! @lane F2-6 SEAM-OABI, 2026-11-01, repo /Users/sac/cgap, branch main.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Typed decision record
// ---------------------------------------------------------------------------

/// Typed UNSUPPORTED(scope) record for the `.oabi` slot-lookup boundary
/// verb (F2-6 dispatch branch b).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OabiLookupUnsupported {
    /// The scope statement: why the verb is out-of-scope, as spec citations.
    pub scope: &'static str,
}

/// The decision, as a typed value: any future consumer (court, doc
/// generator, coordinator roll-up) reads this instead of re-deriving it.
pub const OABI_LOOKUP_UNSUPPORTED: OabiLookupUnsupported = OabiLookupUnsupported {
    scope: "A19-libcgap-ffi-xip.md:47 (boundary = exactly four functions); \
            :57-60 (a fifth verb belongs in a lane, not the ABI); \
            :157-166 (no lane owns a slot-lookup body); \
            a19-finish-receipt.md:158 (Cargo.toml dep edges outside lane lease)",
};

/// The frozen slot surface (A19 §1 table, A10 receipt: "exports exactly six
/// C-ABI symbols"), each witnessed by the court below via `nm -gU`.
pub const FROZEN_SLOT_SURFACE: [&str; 6] = [
    "cgap_i32_batch",
    "cgap_schema_addr",
    "cgap_array_addr",
    "cgap_batch_rows",
    "cgap_version",
    "cgap_digest_of",
];

// ---------------------------------------------------------------------------
// Surface-freeze court (Chicago: real collaborators, assert final state).
//
//   leg 1 (green, real): nested lease build of THIS crate -> `nm -gU` the
//     FRESH cdylib -> detector green AND all six frozen symbols present
//     (the presence pin is what makes the nm leg load-bearing).
//   leg 2 (negative control): a gutted (empty) nm output must refuse the
//     presence pin — a detector that cannot refuse cannot vouch for green.
//   leg 3 (flip, real): cc-compiled /tmp fixture dylib exporting
//     `cgap_oabi_lookup` -> detector REFUSES naming §2; the pristine twin
//     (green probe, no oabi verb) passes. Canonical tree never touched.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod surface_court {
    use super::{FROZEN_SLOT_SURFACE, OABI_LOOKUP_UNSUPPORTED};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root canonicalize")
    }

    /// Bounded wait on a real observable (child exit status) — the one
    /// audited poll: deadline-checked, kill on breach. SLEEP-CLASS: POLL-STEP
    /// (bounded poll, never a coordination sleep).
    const POLL_STEP: Duration = Duration::from_millis(200);
    const BUILD_DEADLINE: Duration = Duration::from_secs(600);

    fn run_bounded(
        program: &str,
        args: &[&str],
        envs: &[(&str, String)],
    ) -> Result<(Option<std::process::ExitStatus>, String, String), String> {
        let mut cmd = Command::new(program);
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
        for (k, v) in envs {
            cmd.env(k, v);
        }
        let mut child = cmd.spawn().map_err(|e| format!("spawn {program}: {e}"))?;
        let out = child.stdout.take().expect("stdout piped");
        let err = child.stderr.take().expect("stderr piped");
        let rd_out = std::thread::spawn(move || {
            let mut s = String::new();
            let _ = std::io::Read::read_to_string(&mut { out }, &mut s);
            s
        });
        let rd_err = std::thread::spawn(move || {
            let mut s = String::new();
            let _ = std::io::Read::read_to_string(&mut { err }, &mut s);
            s
        });
        let started = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(st)) => break Some(st),
                Ok(None) => {
                    if started.elapsed() >= BUILD_DEADLINE {
                        let _ = child.kill();
                        let _ = child.wait();
                        break None;
                    }
                    std::thread::sleep(POLL_STEP); // SLEEP-CLASS: POLL-STEP
                }
                Err(e) => return Err(format!("try_wait: {e}")),
            }
        };
        Ok((
            status,
            rd_out.join().expect("stdout reader"),
            rd_err.join().expect("stderr reader"),
        ))
    }

    /// The detector under test: refuse any exported symbol whose normalized
    /// (leading-underscore-trimmed, case-folded) name contains "oabi".
    /// This is the §2 fifth-verb law as executable code.
    fn oabi_verb_refusal(nm_out: &str) -> Result<(), String> {
        let mut hits: Vec<String> = Vec::new();
        for line in nm_out.lines() {
            let Some(tok) = line.split_whitespace().last() else {
                continue;
            };
            let norm = tok.trim_start_matches('_').to_ascii_lowercase();
            if norm.contains("oabi") {
                hits.push(norm);
            }
        }
        if hits.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "A19 §2 fifth-verb law violated: exported symbol(s) {hits:?} — \
                 a .oabi boundary verb is out-of-scope (A19:47/:57-60); it belongs in a lane, not the ABI"
            ))
        }
    }

    /// Build the CURRENT crate in a fresh lease target dir (build roots are
    /// leases, not assets) and return the fresh cdylib path. No deadlock:
    /// the lease dir never collides with the parent cargo's primary target
    /// dir (wave9 pattern). Lease deleted by the caller.
    fn lease_build_cdylib(dir: &PathBuf) -> Result<PathBuf, String> {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let td = dir.join("target").display().to_string();
        let mp = repo_root().join("crates/cgap-ffi/Cargo.toml");
        let mp = mp.display().to_string();
        let envs = vec![
            ("CARGO_TARGET_DIR", td),
            ("CARGO_NET_OFFLINE", "true".to_string()),
        ];
        let (status, out, err) = run_bounded(
            "cargo",
            &[
                "build",
                "-p",
                "cgap-ffi",
                "--lib",
                "--manifest-path",
                mp.as_str(),
            ],
            &envs,
        )?;
        let st = status.ok_or_else(|| "lease build exceeded deadline".to_string())?;
        if !st.success() {
            return Err(format!(
                "lease cargo build failed: {}\n{}",
                tail(&out),
                tail(&err)
            ));
        }
        find_lease_cdylib(&dir.join("target")).ok_or_else(|| {
            format!(
                "lease build produced no cdylib under {}: {}",
                dir.join("target").display(),
                tail(&out)
            )
        })
    }

    fn find_lease_cdylib(target_root: &Path) -> Option<PathBuf> {
        let names = ["libcgap_ffi.dylib", "libcgap_ffi.so", "cgap_ffi.dll"];
        for name in names {
            let p = target_root.join("debug").join(name);
            if p.is_file() {
                return Some(p);
            }
        }
        None
    }

    fn tail(text: &str) -> String {
        let n = text.chars().count();
        if n <= 800 {
            text.to_string()
        } else {
            text.chars().skip(n - 800).collect()
        }
    }

    /// Presence pin: every frozen symbol must appear in the nm output
    /// (normalized: trim the Mach-O leading underscore). Returns the list
    /// of MISSING names; empty = all present.
    fn missing_frozen(nm_out: &str) -> Vec<String> {
        let norm_lines: Vec<String> = nm_out
            .lines()
            .filter_map(|l| l.split_whitespace().last())
            .map(|t| t.trim_start_matches('_').to_string())
            .collect();
        FROZEN_SLOT_SURFACE
            .iter()
            .filter(|n| !norm_lines.iter().any(|s| s == **n))
            .map(|n| n.to_string())
            .collect()
    }

    /// Compile a C fixture dylib in the sandbox dir via real `cc`.
    fn cc_fixture(dir: &PathBuf, c_src: &str, out_name: &str) -> Result<PathBuf, String> {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let c_path = dir.join("fixture.c");
        std::fs::write(&c_path, c_src).map_err(|e| e.to_string())?;
        let out = dir.join(out_name);
        let out_str = out.display().to_string();
        let c_str = c_path.display().to_string();
        let (status, so, se) = run_bounded(
            "cc",
            &["-shared", "-fPIC", "-o", out_str.as_str(), c_str.as_str()],
            &[],
        )?;
        let st = status.ok_or_else(|| "cc exceeded deadline".to_string())?;
        if !st.success() {
            return Err(format!("cc failed: {}\n{}", tail(&so), tail(&se)));
        }
        Ok(out)
    }

    fn nm_gu(path: &Path) -> Result<String, String> {
        let (status, out, err) =
            run_bounded("nm", &["-gU", path.display().to_string().as_str()], &[])?;
        let st = status.ok_or_else(|| "nm exceeded deadline".to_string())?;
        if !st.success() {
            return Err(format!(
                "nm -gU failed on {}: {}",
                path.display(),
                tail(&err)
            ));
        }
        Ok(out)
    }

    // -- Leg 1: green on the REAL, FRESH cdylib ------------------------------

    #[test]
    fn surface_freeze_green_on_fresh_cdylib() {
        let dir = std::env::temp_dir().join(format!("cgap-f26-lease-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let verdict = (|| -> Result<(), String> {
            let dylib = lease_build_cdylib(&dir)?;
            let text = nm_gu(&dylib)?;
            oabi_verb_refusal(&text).map_err(|e| format!("FROZEN SURFACE BROKEN: {e}"))?;
            let missing = missing_frozen(&text);
            if !missing.is_empty() {
                return Err(format!(
                    "nm leg is not testing the real surface — frozen symbol(s) {missing:?} absent from `nm -gU`"
                ));
            }
            Ok(())
        })();
        let cleaned = fs::remove_dir_all(&dir).is_ok();
        assert!(
            cleaned,
            "lease root {} not deleted — build roots are leases, not assets",
            dir.display()
        );
        verdict.unwrap_or_else(|e| panic!("surface-freeze court: {e}"));
        println!(
            "SURFACE-FREEZE GREEN: fresh cdylib exports the six frozen slot symbols, zero .oabi verbs ({:?} typed UNSUPPORTED(scope) stands)",
            OABI_LOOKUP_UNSUPPORTED.scope.get(0..24)
        );
    }

    // -- Leg 2: negative control (gutted output must refuse) -----------------

    #[test]
    fn gutted_surface_output_refused() {
        // (a) The presence pin must refuse a gutted (empty) nm output:
        // a pin that passes on nothing cannot vouch for the green leg.
        let empty: Vec<String> = missing_frozen("");
        assert_eq!(
            empty.len(),
            FROZEN_SLOT_SURFACE.len(),
            "presence pin accepted an EMPTY surface — cannot flip, VACUOUS"
        );
        // (b) The detector must be a no-op only on a verb-free surface; on
        // an empty surface it is green by construction (correct: nothing
        // exported = nothing violated). Its flip sensor is leg 3.
        assert!(
            oabi_verb_refusal("").is_ok(),
            "empty surface must not fabricate a verb"
        );
    }

    // -- Leg 3: the real flip — cc fixture with the forbidden verb -----------

    /// Pristine fixture: one benign export, no oabi verb.
    const FIXTURE_CLEAN: &str = "int cgap_f26_green_probe(void) { return 1; }\n";
    /// Mutant fixture: the §2 fifth verb, as a real exported symbol.
    const FIXTURE_OABI: &str =
        "int cgap_oabi_lookup(const unsigned char* b, unsigned long n) { return b && n ? 1 : 0; }\n";

    #[test]
    fn fifth_verb_fixture_flips_the_detector() {
        let dir = std::env::temp_dir().join(format!("cgap-f26-flip-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        // Pristine twin first: the detector must PASS it (else the court
        // flips on anything and proves nothing).
        let clean = cc_fixture(&dir.join("clean"), FIXTURE_CLEAN, "libclean.dylib")
            .unwrap_or_else(|e| panic!("clean fixture: {e}"));
        let clean_out = nm_gu(&clean).unwrap_or_else(|e| panic!("nm clean: {e}"));
        oabi_verb_refusal(&clean_out)
            .unwrap_or_else(|e| panic!("VACUOUS COURT: pristine fixture refused: {e}"));
        assert!(
            missing_frozen(&clean_out).len() == FROZEN_SLOT_SURFACE.len(),
            "premise: the fixture is NOT the real surface (it carries none of the frozen six)"
        );
        // The mutant: real dylib, real export, detector must REFUSE it,
        // and the refusal must NAME the law (§2).
        let mutant = cc_fixture(&dir.join("mutant"), FIXTURE_OABI, "libmutant.dylib")
            .unwrap_or_else(|e| panic!("mutant fixture: {e}"));
        let mutant_out = nm_gu(&mutant).unwrap_or_else(|e| panic!("nm mutant: {e}"));
        let refusal = oabi_verb_refusal(&mutant_out).expect_err(
            "VACUOUS GATE: the cgap_oabi_lookup export was NOT refused — the detector cannot flip",
        );
        assert!(
            refusal.contains("A19 §2") && refusal.contains("fifth-verb"),
            "refusal must name the law (A19 §2 fifth-verb), got: {refusal}"
        );
        println!("FIFTH-VERB FLIP: real cc-compiled export cgap_oabi_lookup refused, naming A19:47/:57-60");
        let _ = fs::remove_dir_all(&dir);
    }

    // -- Typed-record pin -----------------------------------------------------

    #[test]
    fn unsupported_record_is_typed_and_cited() {
        // The record must stay TYPED and SPEC-CITED: gutting the citations
        // (or replacing the law with prose) flips here.
        let s = OABI_LOOKUP_UNSUPPORTED.scope;
        assert!(s.contains("A19"), "scope must cite A19");
        assert!(
            s.contains(":47"),
            "scope must cite the four-function decision line"
        );
        assert!(
            s.contains("57-60"),
            "scope must cite the fifth-verb law lines"
        );
        assert!(
            s.contains("157-166"),
            "scope must cite the owner-table lines"
        );
        assert!(
            s.contains("a19-finish-receipt"),
            "scope must cite the DoD-5 structural blocker"
        );
    }
}
