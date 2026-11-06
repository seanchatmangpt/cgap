//! Tier-2 provenance types: the measured/pinned distinction is enforced by
//! the type system, so fabricating a PASS from a pinned constant is not
//! merely discouraged — it is unrepresentable.
//!
//! - `Measured` carries its producing command + ISO timestamp. The
//!   constructor validates both are non-empty (a "measured" row with an
//!   empty source stamp refuses at construction, not at audit time).
//! - `Pinned` carries a path:line reference to the pinned constant.
//! - `Verdict::pass` accepts ONLY `&[Measured]`: a `Pinned` value in the
//!   input list is a compile-time non-starter. The runtime path
//!   (`Verdict::pass_sources`) refuses `Pinned`/`Unmeasured` sources with
//!   `pinned_constant_cannot_pass`.
//!
//! Falsifiers live in the unit tests: a Pinned 12.4 fed to the runtime pass
//! path must come back `Refused(PinnedConstantCannotPass)`; an empty-command
//! "measured" value must refuse at construction.

/// Provenance of a measured value: the exact command that produced it and
/// the ISO-8601 timestamp of the run.
#[derive(Debug, Clone, PartialEq)]
pub struct Provenance {
    pub command: String,
    pub iso_ts: String,
}

/// A value that was actually measured this run, with its provenance.
/// Construction refuses an empty command or timestamp.
#[derive(Debug, Clone, PartialEq)]
pub struct Measured(f64, Provenance);

/// A pinned constant, referenced by its source location (path + 1-based
/// line). Never eligible for a `Verdict::Pass`.
#[derive(Debug, Clone, PartialEq)]
pub struct PinRef {
    pub path: String,
    pub line: u32,
}

/// A pinned constant value with its source reference.
#[derive(Debug, Clone, PartialEq)]
pub struct Pinned(f64, PinRef);

/// Why a verdict was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusalReason {
    /// A pinned constant (or an unmeasured placeholder) cannot produce a
    /// PASS — only a `Measured` value can.
    PinnedConstantCannotPass,
    /// A `Measured` was constructed with an empty command or timestamp.
    EmptyProvenance,
}

/// Source classification for the runtime pass path (`pass_sources`).
/// The type-enforced path (`pass`) takes `&[Measured]` directly and needs
/// no runtime classification at all.
#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    Measured(Measured),
    Pinned(Pinned),
    /// A metric this run did not measure at all.
    Unmeasured,
}

/// Tier-2 verdict.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    /// Passed, backed only by measured values.
    Pass,
    /// Refused, with the reason.
    Refused(RefusalReason),
    /// Not measured this run.
    Unmeasured,
}

impl Measured {
    /// Construct a measured value. Refuses at construction time if the
    /// command or ISO timestamp is empty (tamper fence: a "measured" row
    /// with no source stamp is not measured).
    pub fn new(value: f64, command: &str, iso_ts: &str) -> Result<Self, RefusalReason> {
        if command.is_empty() || iso_ts.is_empty() {
            return Err(RefusalReason::EmptyProvenance);
        }
        Ok(Measured(
            value,
            Provenance {
                command: command.to_string(),
                iso_ts: iso_ts.to_string(),
            },
        ))
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn provenance(&self) -> &Provenance {
        &self.1
    }

    /// Receipt tag: `measured(cmd,ts)`.
    pub fn source_tag(&self) -> String {
        format!("measured({},{})", self.1.command, self.1.iso_ts)
    }
}

impl Pinned {
    /// Construct a pinned constant with its source reference.
    pub fn new(value: f64, path: &str, line: u32) -> Self {
        Pinned(
            value,
            PinRef {
                path: path.to_string(),
                line,
            },
        )
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn pin_ref(&self) -> &PinRef {
        &self.1
    }

    /// Receipt tag: `const(path:line)`.
    pub fn source_tag(&self) -> String {
        format!("const({}:{})", self.1.path, self.1.line)
    }
}

impl Source {
    /// Receipt tag for any source classification.
    pub fn source_tag(&self) -> String {
        match self {
            Source::Measured(m) => m.source_tag(),
            Source::Pinned(p) => p.source_tag(),
            Source::Unmeasured => "unmeasured".to_string(),
        }
    }
}

impl Verdict {
    /// Type-enforced pass path: accepts ONLY measured values. A `Pinned`
    /// value in the input list is a compile-time non-starter — the caller
    /// cannot even express it. Any empty list refuses (nothing measured,
    /// nothing passes).
    pub fn pass(measurements: &[Measured]) -> Result<Verdict, RefusalReason> {
        if measurements.is_empty() {
            return Err(RefusalReason::PinnedConstantCannotPass);
        }
        Ok(Verdict::Pass)
    }

    /// Runtime path over classified sources: a `Pinned` or `Unmeasured`
    /// source in the list yields `Refused(PinnedConstantCannotPass)`.
    pub fn pass_sources(sources: &[Source]) -> Verdict {
        if sources.is_empty() {
            return Verdict::Unmeasured;
        }
        for s in sources {
            match s {
                Source::Measured(_) => {}
                Source::Pinned(_) | Source::Unmeasured => {
                    return Verdict::Refused(RefusalReason::PinnedConstantCannotPass)
                }
            }
        }
        Verdict::Pass
    }

    /// Receipt tag for the verdict itself.
    pub fn source_tag(&self) -> &str {
        match self {
            Verdict::Pass => "pass(measured)",
            Verdict::Refused(RefusalReason::PinnedConstantCannotPass) => {
                "refused(pinned_constant_cannot_pass)"
            }
            Verdict::Refused(RefusalReason::EmptyProvenance) => "refused(empty_provenance)",
            Verdict::Unmeasured => "unmeasured",
        }
    }
}

/// Tier-3 liveness check: a live measurement must MOVE under perturbation.
/// Asserts the relative change between `baseline` and `perturbed` is at
/// least `ratio` (fraction of the baseline magnitude). A pinned constant
/// perturbs by exactly 0 and fails this check.
///
/// # Panics
/// When the relative change is below `ratio` (the value looks pinned).
pub fn assert_live(baseline: f64, perturbed: f64, ratio: f64) {
    let denom = baseline.abs();
    let rel_change = if denom > 0.0 {
        (perturbed - baseline).abs() / denom
    } else {
        (perturbed - baseline).abs()
    };
    assert!(
        rel_change >= ratio,
        "value looks PINNED, not live: baseline={baseline} perturbed={perturbed} \
         relative change {rel_change} < required ratio {ratio}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measured_passes() {
        let m = Measured::new(1.85, "cargo bench -p bench --bench seal", "2026-11-05T00:00:00Z")
            .expect("non-empty provenance");
        let v = Verdict::pass(&[m]).expect("measured values pass");
        assert_eq!(v, Verdict::Pass);
        assert_eq!(v.source_tag(), "pass(measured)");
    }

    #[test]
    fn pinned_in_runtime_list_is_refused() {
        // Falsifier: construct a Pinned 12.4 and attempt the pass over it.
        let pinned = Pinned::new(12.4, "crates/bench/src/bin/ctq_report.rs", 224);
        let v = Verdict::pass_sources(&[Source::Pinned(pinned)]);
        assert_eq!(
            v,
            Verdict::Refused(RefusalReason::PinnedConstantCannotPass)
        );
        assert_eq!(v.source_tag(), "refused(pinned_constant_cannot_pass)");
    }

    #[test]
    fn pinned_row_renders_refused_not_pass() {
        let pinned = Pinned::new(1.30, "ctq_report.rs", 224);
        let tag = pinned.source_tag();
        assert_eq!(tag, "const(ctq_report.rs:224)");
        let v = Verdict::pass_sources(&[Source::Pinned(pinned)]);
        let rendered = match &v {
            Verdict::Refused(_) => "REFUSED",
            Verdict::Pass => "PASS",
            Verdict::Unmeasured => "UNMEASURED",
        };
        assert_eq!(rendered, "REFUSED");
    }

    #[test]
    fn mixed_measured_and_pinned_is_refused() {
        let m = Measured::new(9.1, "ctq06", "2026-11-05T00:00:00Z").expect("ok");
        let p = Pinned::new(8.5, "prd.md", 10);
        let v = Verdict::pass_sources(&[Source::Measured(m), Source::Pinned(p)]);
        assert_eq!(
            v,
            Verdict::Refused(RefusalReason::PinnedConstantCannotPass)
        );
    }

    #[test]
    fn unmeasured_source_refuses() {
        let v = Verdict::pass_sources(&[Source::Unmeasured]);
        assert_eq!(
            v,
            Verdict::Refused(RefusalReason::PinnedConstantCannotPass)
        );
    }

    #[test]
    fn empty_measured_command_refuses_at_construction() {
        // Tamper test: a "measured" row with an empty timestamp.
        let r = Measured::new(12.4, "cargo bench", "");
        assert_eq!(r.unwrap_err(), RefusalReason::EmptyProvenance);
        let r2 = Measured::new(12.4, "", "2026-11-05T00:00:00Z");
        assert_eq!(r2.unwrap_err(), RefusalReason::EmptyProvenance);
    }

    #[test]
    fn empty_list_cannot_pass() {
        let v = Verdict::pass(&[]);
        assert_eq!(
            v.unwrap_err(),
            RefusalReason::PinnedConstantCannotPass
        );
        assert_eq!(
            Verdict::pass_sources(&[]),
            Verdict::Unmeasured
        );
    }

    #[test]
    fn assert_live_passes_on_real_perturbation() {
        assert_live(1.85, 2.10, 0.05);
    }

    #[test]
    #[should_panic(expected = "looks PINNED")]
    fn assert_live_flags_pinned_constant() {
        assert_live(12.4, 12.4, 0.01);
    }

    #[test]
    fn type_system_rejects_pinned_in_pass() {
        // The compile-time non-starter is the SIGNATURE itself: `pass` takes
        // `&[Measured]`, so a Pinned cannot be passed. Witness it via a
        // helper generic over nothing — this test only compiles because the
        // pinned value stays out of the measured list.
        let m = Measured::new(2.5, "cmd", "ts").expect("ok");
        let _pinned = Pinned::new(12.4, "x.rs", 1);
        // Uncommenting the next line must fail to compile:
        // let _ = Verdict::pass(&[_pinned]);
        assert!(Verdict::pass(&[m]).is_ok());
    }
}
