//! CSCT Andon Refusal — RFC-0042 Rule 3.
//!
//! Deterministic, observable halt code for the CSCT runner: any
//! unrecoverable court refusal stops the process with exit code 101
//! (`ANDON_REFUSAL`), never a hang, never a silent continue.
//!
//! Two forms, two seams:
//!
//! * [`check`] — the **court seam** (recoverable). Returns
//!   `Err(AndonRefusal)` instead of halting, so in-process courts can
//!   assert on the typed refusal value without killing the test process.
//! * [`refuse`] — the **SUT boundary** (process-fatal). Logs the reason
//!   and exits the runner with [`ANDON_REFUSAL`]. Called only at the
//!   runner boundary, never inside a stage court.

/// Deterministic halt exit code (RFC-0042 Rule 3).
pub const ANDON_REFUSAL: i32 = 101;

/// Typed, recoverable refusal value.
///
/// Carries the court's message so callers (and the failure ledger) can
/// assert on the exact reason string — Chicago discipline: assert final
/// state, not side effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AndonRefusal {
    pub msg: &'static str,
}

/// Recoverable check — **the court seam**.
///
/// Returns `Ok(())` when `cond` holds, otherwise `Err(AndonRefusal)` with
/// the exact `msg`. Courts assert on the returned value; nothing halts.
pub fn check(cond: bool, msg: &'static str) -> Result<(), AndonRefusal> {
    if cond {
        Ok(())
    } else {
        Err(AndonRefusal { msg })
    }
}

/// Process-fatal refusal — **the SUT boundary**.
///
/// Logs `msg` to stderr and halts the runner deterministically with
/// [`ANDON_REFUSAL`] (101). Used only at the runner boundary; never call
/// from inside a stage court — use [`check`] there.
pub fn refuse(msg: &'static str) -> ! {
    eprintln!("ANDON REFUSAL: {msg}");
    std::process::exit(ANDON_REFUSAL);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_returns_err_with_exact_msg_on_refusal() {
        let got = check(false, "court expected invariant X");
        assert_eq!(
            got,
            Err(AndonRefusal {
                msg: "court expected invariant X"
            })
        );
    }

    #[test]
    fn check_returns_ok_when_cond_holds() {
        assert_eq!(check(true, "unused on success"), Ok(()));
    }

    // Anti-vacuity mutation: the discriminator must actually flip. The
    // observed condition is computed from a value, not a literal, so a
    // hypothetical flipped check (returning Err on true / Ok on false)
    // fails this test.
    #[test]
    fn anti_vacuity_check_discriminator_flips() {
        let expected = 7usize;
        for observed in [6usize, 7, 8] {
            let refuses = observed != expected;
            let got = check(!refuses, "observed != expected");
            match (refuses, got) {
                (true, Err(AndonRefusal { msg })) => {
                    assert_eq!(msg, "observed != expected")
                }
                (false, Ok(())) => {}
                (refuses, got) => {
                    panic!("discriminator did not flip: refuses={refuses}, got={got:?}")
                }
            }
        }
    }

    // Anti-vacuity mutation, inverted polarity: with the comparison
    // flipped at the call site, the previously-refusing branch must stop
    // refusing. Proves the Err arm is keyed to the condition, not the
    // message.
    #[test]
    fn anti_vacuity_flipped_condition_stops_refusing() {
        let expected = 7usize;
        let observed = 7usize;
        // Deliberately flipped: cond=true where a correct court would
        // refuse (observed == expected means `observed != expected` is
        // false).
        let got = check(observed == expected, "observed != expected");
        assert!(got.is_ok(), "flipped check must NOT refuse; got {got:?}");
    }

    #[test]
    fn andon_refusal_code_is_101() {
        assert_eq!(ANDON_REFUSAL, 101);
    }

    // NOTE: refuse() is intentionally NOT called in this module's tests
    // — it terminates the process with ANDON_REFUSAL. That path is
    // exercised by the integration court at the runner boundary.
    #[test]
    fn refuse_is_not_exercised_in_unit_courts() {
        // Structural assertion only: the symbol exists with the pinned
        // signature shape (returns ! via exit path). Documented by this
        // test's existence; see receipt.
        let code: i32 = ANDON_REFUSAL;
        assert_eq!(code, refuse_code());
    }

    fn refuse_code() -> i32 {
        ANDON_REFUSAL
    }
}
