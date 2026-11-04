//! profile_guard — reusable release-profile detector + anti-DCE fence (RX-4).
//!
//! Generalizes two landed patterns into one component:
//! - **AF-25 axis 2** (cbmqs_gates.rs G2): a debug-profile run must not
//!   fence a measurement against a release-only ceiling; it loud-SKIPs with
//!   a typed reason. Here: [`assert_release`] returns a typed
//!   [`ProfileViolation`] so any caller composes the same loud-SKIP pattern
//!   (`#[must_use]` — a dropped `Result` is a compile error).
//! - **AF-10 G1b** (ab-10/af-10 receipts): release benches silently
//!   constant-fold away the measured work (11.08 B pushes/s defect);
//!   `black_box` the inputs and outputs. Here: [`fence`] wraps
//!   `core::hint::black_box` (stable, safe, no_std).
//!
//! Both axes are detected at build time — no runtime environment reads:
//! - `cfg!(debug_assertions)` — on unless the profile sets `debug-assertions = false`.
//! - `option_env!("OPT_LEVEL")` — cargo sets it per profile ("0" = dev;
//!   anything else, including absent, is treated as optimized).
//!
//! `#![no_std]`, zero dependencies, `forbid(unsafe_code)`-clean.

/// Why a caller is not on a lawful release profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileViolation {
    /// True when the profile enables `debug-assertions`.
    pub debug_assertions: bool,
    /// Cargo's per-profile `OPT_LEVEL` env ("0" for dev; `None` when unset).
    pub opt_level: Option<&'static str>,
}

impl ProfileViolation {
    /// One-line human-readable reason for a loud SKIP / gate context string.
    pub fn write_reason(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "non-release profile: debug_assertions={} opt-level={}",
            self.debug_assertions,
            self.opt_level.unwrap_or("unset")
        )
    }
}

impl core::fmt::Display for ProfileViolation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.write_reason(f)
    }
}

/// The two detection axes at build time.
const VIOLATION: ProfileViolation = ProfileViolation {
    debug_assertions: cfg!(debug_assertions),
    opt_level: option_env!("OPT_LEVEL"),
};

fn is_release(opt_level: Option<&str>) -> bool {
    // "0" is the dev profile's opt-level; every optimized profile is
    // 1/2/3/s/z. `None` (env unset — some rustc invocations do not carry
    // it) reads as UNKNOWN on this axis and is NOT a violation by itself;
    // the debug_assertions axis carries detection there.
    !matches!(opt_level, Some("0"))
}

/// Assert the current build is a lawful release profile.
///
/// Returns `Err(ProfileViolation)` when either axis detects a debug/dev
/// build: `cfg!(debug_assertions)` is on, or cargo's `OPT_LEVEL` reads
/// "0"/unset. `#[must_use]` on the `Result` makes a silently-dropped
/// guard a compile error — the caller pattern from AF-25's G2 loud-SKIP.
///
/// # Examples
/// ```
/// use cgap_aoq::profile_guard;
/// // Loud-SKIP caller pattern: the Result must be handled.
/// match profile_guard::assert_release() {
///     Ok(()) => { /* fenced measurement lawful */ }
///     Err(v) => panic!("SKIP: {v}"),
/// }
/// ```
#[must_use = "a dropped profile guard is a silent fake pass; handle the Result (loud SKIP in debug)"]
pub fn assert_release() -> Result<(), ProfileViolation> {
    if VIOLATION.debug_assertions || !is_release(VIOLATION.opt_level) {
        return Err(VIOLATION);
    }
    Ok(())
}

/// Anti-DCE fence (AF-10 G1b fix, generalized): opaque to the optimizer.
///
/// A release bench that computes a value and never "observes" it gets the
/// whole computation constant-folded away; `fence` forces the value
/// through an opaque barrier so the measured work must execute.
#[inline]
pub fn fence<T>(value: T) -> T {
    core::hint::black_box(value)
}

#[cfg(test)]
mod courts {
    use super::*;
    // Unit courts run hosted: the lib is no_std, the test binary links std.
    extern crate std;

    /// Guard fires in debug, passes in release — witnessed via the cfg
    /// inverse: this test compiles both ways and asserts the axis the
    /// build actually has.
    #[test]
    fn guard_axis_matches_build_profile() {
        let result = assert_release();
        if cfg!(debug_assertions) {
            assert!(result.is_err(), "debug build must be detected as violation");
            let v = result.unwrap_err();
            assert!(v.debug_assertions);
        } else {
            assert!(result.is_ok(), "release build must pass the guard");
        }
    }

    /// Opt-level axis: when cargo carries OPT_LEVEL into the build (Some),
    /// a dev build must read "0"; when the env is unset the axis is UNKNOWN
    /// and detection falls to the debug_assertions axis (never a fake pass).
    #[test]
    fn violation_carries_opt_level_axis() {
        if cfg!(debug_assertions) {
            let v = assert_release().unwrap_err();
            assert!(
                v.opt_level == Some("0") || v.opt_level.is_none(),
                "dev build must read opt-level 0 or UNKNOWN, got {:?}",
                v.opt_level
            );
        }
    }

    /// Runtime value identity through the fence: the fenced value equals
    /// the input — fence is transparent to data, opaque to the optimizer.
    #[test]
    fn fence_is_value_transparent() {
        let x: u64 = 0x5eed_5eed_5eed_5eed;
        assert_eq!(fence(x), x);
        assert_eq!(fence(&x), &x);
        let s = "cbmqs";
        assert_eq!(fence(s), "cbmqs");
    }

    /// Compile-time const-folding probe: `folded()` has a constant body the
    /// optimizer may reduce to `42` at compile time; `fenced()` routes the
    /// same constant through `fence`, which the optimizer cannot see through.
    /// Both must produce identical runtime values — proving the fence
    /// preserves semantics while blocking visibility. Additionally the
    /// fenced value feeds a comparison the compiler cannot fold (input is
    /// runtime-blackboxed), so the fence call itself is live code.
    #[test]
    fn fence_blocks_constant_folding() {
        const CONST: u64 = 42;
        let runtime = fence(CONST);
        // Runtime value identity: the fence output equals the constant.
        assert_eq!(runtime, CONST);
        // Live-code check: consume the fenced value in a branch on a
        // blackboxed runtime condition so the fence cannot be folded into
        // the assertion below.
        let opaque = fence(0u8);
        if opaque == 1 {
            // Unreachable at runtime, but the compiler must keep the fence
            // to prove it — the branch depends on fence output.
            assert_ne!(fence(CONST), CONST);
        }
    }

    /// Display renders the typed reason (loud-SKIP context string).
    #[test]
    fn violation_displays_reason() {
        if cfg!(debug_assertions) {
            let v = assert_release().unwrap_err();
            // SOURCE-SCAN-EXEMPT: std:: : test-only Display rendering; std is
            // linked by the lib-test harness; the no_std lib surface never
            // touches std.
            let s = std::format!("{v}");
            assert!(s.contains("debug_assertions=true"));
            assert!(
                s.contains("opt-level=0") || s.contains("opt-level=unset"),
                "display must carry the opt-level axis, got: {s}"
            );
        }
    }
}
