//! Authority tier: the 80-byte `PhiLease` and its monotonic `Tick` (lane A16).
//!
//! Spec: `docs/jira/v26.11.1/lanes/A16-chatman-pqc.md` §3.2 (byte layout) and
//! §5 (lease mechanics).
//!
//! Wave 1 froze the ABI (structs, consts, compile-time layout pins). Wave 2
//! (this file) lands the §5 core engine: the monotonic `tick()` advance, the
//! exhaustion TRAP (§5, lane-A17 trap discipline), and `Phi_lease` snapshot
//! production for seal time. The PQC seal itself stays Wave 3 (§4); the
//! unsealed engine path is the §4 UNSUPPORTED clause.
//!
//! Layout convention matches lane A04 `cgap-core/src/frame.rs` (Wave 1 ABI
//! freeze): `#[repr(C)]`, per-field offset docs, core-only `const _` asserts,
//! no `static_assertions` dependency.

#![allow(dead_code)] // Wave-1 ABI consts not yet read by the Wave-3 PQC path.

/// `lease_id`: 32 bytes, random at lease creation (§3.2).
pub const LEASE_ID_LEN: usize = 32;
/// `tick`: 64-bit unsigned little-endian, monotonic, starts at 0 (§3.2).
pub const TICK_LEN: usize = 8;
/// `tick_ceiling`: 64-bit unsigned little-endian, fixed at creation (§3.2).
pub const TICK_CEILING_LEN: usize = 8;
/// `issued_root`: 32-byte BLAKE3 digest binding the lease to the key epoch
/// (§3.2; key-epoch rotation is the §3.4 UNKNOWN — not decided here).
pub const ISSUED_ROOT_LEN: usize = 32;
/// Total `Phi_lease` wire size: `32 + 8 + 8 + 32` bytes (§3.2).
pub const PHI_LEASE_LEN: usize = 80;

/// Monotonic tick counter newtype (§5).
///
/// The field is private and the ONLY constructor from a raw value is
/// crate-internal (`Tick(0)` inside `Lease::new`). There is no `set_tick`,
/// no `Sub` impl, and no public or crate API that accepts a caller-supplied
/// tick: rewind is unrepresentable in the type, not merely unchecked (§5).
pub struct Tick(u64);

/// A live lease epoch: the authority term `Phi_lease` (§3.2) plus the §5
/// mechanics that make its tick counter monotonic and its exhaustion a trap.
///
/// Fields are private; the 80-byte wire form exists only as the read-only
/// [`PhiLease`] snapshot produced at seal time (`Lease::phi`, §5 "produced
/// only at seal time ... no public path exposes mutable lease bytes").
pub struct Lease {
    /// 32 B, supplied at construction; randomness is the constructor
    /// caller's policy (this zero-dependency crate owns no RNG — recorded
    /// in the lane receipt as divergence D1).
    lease_id: [u8; LEASE_ID_LEN],
    /// Monotonic counter, starts at 0 (§3.2, §5).
    tick: Tick,
    /// Fixed at construction; must be > 0 (§5).
    tick_ceiling: u64,
    /// 32 B BLAKE3 digest binding the lease to its key epoch (§3.2).
    issued_root: [u8; ISSUED_ROOT_LEN],
}

impl Lease {
    /// Creates a lease with a zero tick and a fixed ceiling (§3.2, §5).
    ///
    /// TRAPS (panic) when `tick_ceiling == 0`: §5 fixes the only ceiling
    /// policy as "ceiling > 0", and a zero-ceiling lease is exhausted at
    /// birth — an invariant failure of the abstraction, not a recoverable
    /// condition (same trap discipline as §5 exhaustion).
    pub fn new(
        lease_id: [u8; LEASE_ID_LEN],
        tick_ceiling: u64,
        issued_root: [u8; ISSUED_ROOT_LEN],
    ) -> Self {
        if tick_ceiling == 0 {
            panic!(
                "cgap-crypto: Phi_lease tick_ceiling must be > 0 (§5); got 0 \
                 — refusing a lease that is exhausted at birth"
            );
        }
        Lease {
            lease_id,
            tick: Tick(0),
            tick_ceiling,
            issued_root,
        }
    }

    /// 32 B lease identity (§3.2).
    pub fn lease_id(&self) -> &[u8; LEASE_ID_LEN] {
        &self.lease_id
    }

    /// Fixed tick ceiling (§3.2).
    pub fn tick_ceiling(&self) -> u64 {
        self.tick_ceiling
    }

    /// 32 B key-epoch binding digest (§3.2; rotation story is the §3.4
    /// UNKNOWN — this crate never decides it in prose).
    pub fn issued_root(&self) -> &[u8; ISSUED_ROOT_LEN] {
        &self.issued_root
    }

    /// Current tick value, pure read (observability/courts). Reading is not
    /// rewinding: no API mutates the counter except [`Lease::tick`].
    pub fn current_tick(&self) -> u64 {
        self.tick.0
    }

    /// §5: "returns the current value and advances it." One tick per sealed
    /// receipt; `chatman::seal_unsealed` is the only intended caller.
    ///
    /// TRAPS (panic in hosted builds; `panic = "abort"` makes this a real
    /// trap on wasm32) when the counter is exhausted: `tick ==
    /// tick_ceiling` means authority is spent. §5: an error return "would
    /// invite callers to handle authority exhaustion by minting a new lease
    /// mid-delta, silently splitting one epoch into two" — the trap makes
    /// the quiet continuation impossible; a lease that must continue mints
    /// a NEW lease through [`Lease::new`].
    pub fn tick(&mut self) -> u64 {
        let t = self.tick.0;
        if t >= self.tick_ceiling {
            panic!(
                "cgap-crypto: Phi_lease authority EXHAUSTED: tick {} == \
                 tick_ceiling {} (lease_id {:?}) — §5 exhaustion trap; mint a \
                 new lease instead of continuing this epoch",
                t, self.tick_ceiling, self.lease_id
            );
        }
        self.tick.0 = t + 1;
        t
    }

    /// Produces the 80-byte [`PhiLease`] snapshot for seal time (§5: bytes
    /// come from internal state; nothing mutable is exposed).
    ///
    /// `pub(crate)`: `chatman::seal_unsealed` is the only caller — phi bytes
    /// without a consumed tick must not leak past the crate boundary.
    /// Defense-in-depth: also TRAPS when already exhausted, so no phi with
    /// `tick >= tick_ceiling` can ever be emitted by this crate.
    pub(crate) fn phi(&self) -> PhiLease {
        let t = self.tick.0;
        if t >= self.tick_ceiling {
            panic!(
                "cgap-crypto: refusing to emit Phi_lease with tick {} >= \
                 tick_ceiling {} (lease_id {:?})",
                t, self.tick_ceiling, self.lease_id
            );
        }
        PhiLease {
            lease_id: self.lease_id,
            tick: t,
            tick_ceiling: self.tick_ceiling,
            issued_root: self.issued_root,
        }
    }
}

/// 80-byte lease epoch: `lease_id || tick || tick_ceiling || issued_root`
/// (§3.2).
///
/// `#[repr(C)]`, gap-free by construction: offsets 0, 32, 40, 48; total
/// exactly 80 bytes; multi-byte integers little-endian (§3.2). The struct
/// mirrors the canonical byte string one-to-one, so the §3.2 encoding has no
/// serialization choices to make.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhiLease {
    /// offset 0, 32 B — random at lease creation (§3.2).
    pub lease_id: [u8; LEASE_ID_LEN],
    /// offset 32, 8 B — u64 LE, monotonic, starts at 0 (§3.2, §5).
    pub tick: u64,
    /// offset 40, 8 B — u64 LE, fixed at creation; ceiling > 0 (§5).
    pub tick_ceiling: u64,
    /// offset 48, 32 B — BLAKE3 digest binding the lease to the key epoch
    /// (§3.2).
    pub issued_root: [u8; ISSUED_ROOT_LEN],
}

// §3.2 layout pins — compile-time; a field reorder that preserves total size
// still trips the offset pins.
const _: () = assert!(core::mem::size_of::<PhiLease>() == PHI_LEASE_LEN);
const _: () = assert!(core::mem::align_of::<PhiLease>() == 8);
const _: () = assert!(core::mem::offset_of!(PhiLease, lease_id) == 0);
const _: () = assert!(core::mem::offset_of!(PhiLease, tick) == 32);
const _: () = assert!(core::mem::offset_of!(PhiLease, tick_ceiling) == 40);
const _: () = assert!(core::mem::offset_of!(PhiLease, issued_root) == 48);
const _: () =
    assert!(PHI_LEASE_LEN == LEASE_ID_LEN + TICK_LEN + TICK_CEILING_LEN + ISSUED_ROOT_LEN);

impl PhiLease {
    /// The canonical 80-byte encoding: `lease_id || tick(LE) ||
    /// tick_ceiling(LE) || issued_root` (§3.2 — fixed order, little-endian,
    /// no serialization choices).
    pub fn to_bytes(&self) -> [u8; PHI_LEASE_LEN] {
        let mut out = [0u8; PHI_LEASE_LEN];
        out[0..32].copy_from_slice(&self.lease_id);
        out[32..40].copy_from_slice(&self.tick.to_le_bytes());
        out[40..48].copy_from_slice(&self.tick_ceiling.to_le_bytes());
        out[48..80].copy_from_slice(&self.issued_root);
        out
    }

    /// Decodes the canonical 80-byte encoding (courts / wire parsing; the
    /// crate never rebuilds a live `Lease` from bytes — a decoded `PhiLease`
    /// is an inert snapshot, not authority).
    pub fn from_bytes(bytes: &[u8; PHI_LEASE_LEN]) -> Self {
        let mut lease_id = [0u8; LEASE_ID_LEN];
        lease_id.copy_from_slice(&bytes[0..32]);
        let mut tick_bytes = [0u8; TICK_LEN];
        tick_bytes.copy_from_slice(&bytes[32..40]);
        let mut ceiling_bytes = [0u8; TICK_CEILING_LEN];
        ceiling_bytes.copy_from_slice(&bytes[40..48]);
        let mut issued_root = [0u8; ISSUED_ROOT_LEN];
        issued_root.copy_from_slice(&bytes[48..80]);
        PhiLease {
            lease_id,
            tick: u64::from_le_bytes(tick_bytes),
            tick_ceiling: u64::from_le_bytes(ceiling_bytes),
            issued_root,
        }
    }
}

/// Lane AE8 additive type-state guard: an affine `Fresh → Spent` state
/// machine wrapping the frozen §5 [`Lease`] WITHOUT touching its internals.
///
/// The frozen pin stands unchanged: `Tick` stays a private monotonic
/// newtype, exhaustion still TRAPS via `Lease::tick` (delegated), and there
/// is still no API that accepts a caller-supplied tick. What the guard adds
/// is a TYPE-level law on top: a spent guard is `LeaseGuard<Spent>` and no
/// method converts it back to `LeaseGuard<Fresh>` — rewind-at-the-type-level
/// is unrepresentable, witnessable as E0599 (see `guard_compile_rejection`
/// court below and its CONTROL mutation).
pub mod guard {
    use super::{Lease, ISSUED_ROOT_LEN, LEASE_ID_LEN};
    use core::marker::PhantomData;

    /// Fresh state: the guard may still spend ticks.
    pub struct Fresh;
    /// Terminal state: no tick path exists on this type at all.
    pub struct Spent;

    /// Affine guard over a frozen [`Lease`]. `S` is `Fresh` or `Spent`.
    pub struct LeaseGuard<S> {
        lease: Lease,
        _state: PhantomData<S>,
    }

    impl<S> LeaseGuard<S> {
        /// Pure read-through of the frozen monotonic counter (§5).
        pub fn current_tick(&self) -> u64 {
            self.lease.current_tick()
        }

        /// 32 B lease identity read-through (§3.2).
        pub fn lease_id(&self) -> &[u8; LEASE_ID_LEN] {
            self.lease.lease_id()
        }

        /// 32 B key-epoch binding read-through (§3.2).
        pub fn issued_root(&self) -> &[u8; ISSUED_ROOT_LEN] {
            self.lease.issued_root()
        }

        /// Fixed ceiling read-through (§3.2).
        pub fn tick_ceiling(&self) -> u64 {
            self.lease.tick_ceiling()
        }
    }

    impl LeaseGuard<Fresh> {
        /// Mints a guard around a fresh lease (zero tick, ceiling > 0;
        /// §5 — delegates the zero-ceiling trap to [`Lease::new`]).
        pub fn mint(
            lease_id: [u8; LEASE_ID_LEN],
            tick_ceiling: u64,
            issued_root: [u8; ISSUED_ROOT_LEN],
        ) -> Self {
            LeaseGuard {
                lease: Lease::new(lease_id, tick_ceiling, issued_root),
                _state: PhantomData,
            }
        }

        /// §5 advance, delegated verbatim: pre-advance value returned,
        /// counter incremented, exhaustion still TRAPS (frozen pin — the
        /// guard does NOT soften the panic into a typed refusal; that
        /// decision belongs to the §5 owner, recorded as DESIGN-TARGET in
        /// the lane-08 receipt).
        pub fn spend_tick(&mut self) -> u64 {
            self.lease.tick()
        }

        /// Affine transition `Fresh → Spent`. Consumes the guard by value;
        /// there is no inverse.
        pub fn retire(self) -> LeaseGuard<Spent> {
            LeaseGuard {
                lease: self.lease,
                _state: PhantomData,
            }
        }
    }

    impl LeaseGuard<Spent> {
        /// Terminal state: unwraps the frozen lease (its tick is wherever
        /// spending left it — monotonic, never reset).
        pub fn release(self) -> Lease {
            self.lease
        }
        // DELIBERATE ABSENCE (the AE8 law): no `rewind`, no `to_fresh`, no
        // `Into<LeaseGuard<Fresh>>`, no `Default`. A spent guard cannot go
        // back; the compiler, not a runtime check, enforces it.
    }
}

#[cfg(test)]
mod guard_courts {
    use super::guard::{Fresh, LeaseGuard, Spent};
    use super::Lease;

    /// AE8 runtime court: spending through the guard advances the frozen
    /// counter exactly as `Lease::tick` does (strictly increasing,
    /// read-through agrees). Anti-vacuity mutation: a guard whose
    /// `spend_tick` returned without advancing would flip this (same M1
    /// class as the wave-7 `monotonicity_court`).
    #[test]
    fn guard_spend_sequence_is_monotonic_delegated() {
        let mut g = LeaseGuard::mint([7u8; 32], 16, [9u8; 32]);
        for expected in 0..4u64 {
            assert_eq!(g.spend_tick(), expected, "pre-advance value");
            assert_eq!(g.current_tick(), expected + 1, "read-through");
        }
        let spent = g.retire();
        assert_eq!(spent.current_tick(), 4);
        let lease: Lease = spent.release();
        assert_eq!(lease.current_tick(), 4, "release preserves the tick");
    }

    /// AE8 runtime court: `retire` is affine-by-value — after moving the
    /// guard into `retire`, the `Fresh` binding is gone; `Spent` exposes no
    /// `spend_tick`. The runtime part pins observable state; the compile
    /// part is pinned by `guard_compile_rejection`.
    #[test]
    fn retire_consumes_and_terminal_state_has_no_tick_path() {
        let g: LeaseGuard<Fresh> = LeaseGuard::mint([1u8; 32], 8, [3u8; 32]);
        assert_eq!(g.current_tick(), 0);
        let spent: LeaseGuard<Spent> = g.retire();
        assert_eq!(spent.current_tick(), 0, "retire does not spend a tick");
        let _lease = spent.release();
    }

    /// AE8 compile-rejection court: a program that attempts state inversion
    /// (`Spent -> Fresh` via a `rewind()` call) MUST fail to compile against
    /// the real crate. Observed as a REAL rustc subprocess; the court
    /// demands an E0599-class rejection.
    ///
    /// CONTROL mutation (anti-vacuity, run in the same court): the identical
    /// call shape against an in-file stand-in that HAS `rewind` must
    /// COMPILE — proving the baseline rejection is attributable to the
    /// guard's missing rewind (the AE8 law), not to rustc noise or a path
    /// typo. If the guard ever grew a rewind, the BASELINE would compile,
    /// `find_rlib` would return None or the baseline rustc would exit 0 —
    /// either way this court flips.
    #[test]
    fn guard_compile_rejection_state_inversion_is_unrepresentable() {
        // CONTROL first: same call shape, stand-in WITH rewind -> compiles.
        let control = std::env::temp_dir().join("ae8_guard_control.rs");
        std::fs::write(
            &control,
            "pub struct G; pub struct F;\n\
             impl G { pub fn rewind(self) -> F { F } }\n\
             fn main() {\n\
             let g = G;\n\
             let _f: F = g.rewind();\n\
             }\n",
        )
        .expect("write control mutant");
        let out = std::process::Command::new("rustc")
            .arg("--edition")
            .arg("2021")
            .arg("--crate-type")
            .arg("bin")
            .arg(&control)
            .arg("-o")
            .arg(control.with_extension("bin"))
            .output()
            .expect("spawn rustc for control mutant");
        assert!(
            out.status.success(),
            "AE8 CONTROL DEATH: the rewind-shaped call does not compile \
             even against a stand-in that HAS rewind — the baseline \
             rejection below is not attributable to the guard API; stderr: {}",
            std::string::String::from_utf8_lossy(&out.stderr)
        );

        // BASELINE: the same call against the REAL crate MUST be rejected.
        // The workspace target layout is nonstandard, so the extern paths
        // come from a shallow artifact scan near the test binary.
        // BK-3 takeover: the single-artifact scan paired a stale blake3 rlib
        // with the fresh crypto rmeta and died with E0460 (wrong-reason).
        // Enumerate ALL candidate (rmeta, rlib) pairs from the build dirs:
        //   - any pair where the mutant COMPILES -> VACUOUS GATE (hard fail)
        //   - any pair rejecting with E0599/E0433 -> law witnessed, take it
        //   - pairs dying with stale-metadata E0460 -> unusable, skipped
        // No usable pair -> wrong-reason death (fail closed).
        let build_root =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/build");
        let mut cryptos = std::vec::Vec::new();
        let mut blakes = std::vec::Vec::new();
        for (crate_dir, name, ext, out) in [
            (
                "cgap-crypto",
                "libcgap_crypto",
                ".rmeta",
                &mut cryptos,
            ),
            ("cgap-blake3", "libcgap_blake3", ".rlib", &mut blakes),
        ] {
            let _ = crate_dir;
            for h in std::fs::read_dir(build_root.join(crate_dir))
                .into_iter()
                .flatten()
                .flatten()
            {
                let out_dir = h.path().join("out");
                for e in std::fs::read_dir(&out_dir).into_iter().flatten().flatten() {
                    let p = e.path();
                    let n = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if n.starts_with(name) && n.ends_with(ext) {
                        out.push(p);
                    }
                }
                let _ = name;
                let _ = ext;
            }
            out.sort();
        }
        assert!(
            !cryptos.is_empty() && !blakes.is_empty(),
            "AE8 SETUP DEATH: no cgap_crypto rmeta / cgap_blake3 rlib under \
             {} — run cargo build first",
            build_root.display()
        );
        let mutant = std::env::temp_dir().join("ae8_guard_mutant.rs");
        std::fs::write(
            &mutant,
            "fn main() {\n\
             let g = cgap_crypto::lease::guard::LeaseGuard::<\n\
             cgap_crypto::lease::guard::Fresh>::mint([0u8; 32], 8, [0u8; 32]);\n\
             let spent = g.retire();\n\
             let _back: cgap_crypto::lease::guard::LeaseGuard<\n\
             cgap_crypto::lease::guard::Fresh> = spent.rewind();\n\
             }\n",
        )
        .expect("write mutant");
        let mut saw_vacuous: Option<std::string::String> = None;
        let mut saw_reject = false;
        for crypto in &cryptos {
            for blake3 in &blakes {
                let out = std::process::Command::new("rustc")
                    .arg("--edition=2021")
                    .arg("--emit=metadata")
                    .arg("--crate-type=lib")
                    .arg(format!("--extern=cgap_crypto={}", crypto.display()))
                    .arg(format!("--extern=cgap_blake3={}", blake3.display()))
                    .arg("-L")
                    .arg(format!("dependency={}", crypto.parent().unwrap().display()))
                    .arg("-L")
                    .arg(format!("dependency={}", blake3.parent().unwrap().display()))
                    .arg(&mutant)
                    .output()
                    .expect("spawn rustc for mutant");
                let stderr = std::string::String::from_utf8_lossy(&out.stderr);
                if out.status.success() {
                    saw_vacuous = Some(stderr.into_owned());
                } else if stderr.contains("E0599") || stderr.contains("E0433") {
                    saw_reject = true;
                }
            }
        }
        assert!(
            saw_vacuous.is_none(),
            "AE8 VACUOUS GATE: `Spent::rewind` COMPILED against the real \
             crate — the type-level no-rewind law is absent; stderr: {}",
            saw_vacuous.unwrap_or_default()
        );
        assert!(
            saw_reject,
            "AE8 WRONG-REASON DEATH: no usable (rmeta, rlib) pair rejected \
             the mutant with an E0599/E0433-style method-resolution signature"
        );
        let _ = std::fs::remove_file(&control);
        let _ = std::fs::remove_file(&mutant);
    }

    /// Locates a built crate artifact (`name` + `ext`, e.g.
    /// `libcgap_crypto` + `.rmeta`) near this test binary. The workspace
    /// target layout here is nonstandard (artifacts live under
    /// `build/<crate>/<hash>/out/`), so the search walks up from the test
    /// executable scanning shallowly.
    fn find_artifact(name: &str, ext: &str) -> Option<std::path::PathBuf> {
        fn name_matches(n: &str, name: &str, ext: &str) -> bool {
            n.starts_with(name) && n.ends_with(ext)
        }
        fn shallow(dir: &std::path::Path, name: &str, ext: &str) -> Option<std::path::PathBuf> {
            // BK-3 takeover: pick the NEWEST matching artifact by mtime, not
            // the first in read_dir order — build/<crate>/<hash>/out holds
            // stale hash dirs and an old blake3 rlib made the AE8 mutant
            // compile die with E0460 (wrong-reason) instead of E0599.
            let mut best: Option<(std::time::SystemTime, std::path::PathBuf)> = None;
            for e in std::fs::read_dir(dir).ok()?.flatten() {
                let p = e.path();
                let matches = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| name_matches(n, name, ext))
                    .unwrap_or(false);
                if !matches {
                    continue;
                }
                let mtime = e.metadata().and_then(|m| m.modified()).unwrap_or(
                    std::time::SystemTime::UNIX_EPOCH,
                );
                if best.as_ref().map_or(true, |(t, _)| mtime > *t) {
                    best = Some((mtime, p));
                }
            }
            best.map(|(_, p)| p)
        }
        let mut dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
        loop {
            if let Some(p) = shallow(&dir, name, ext) {
                return Some(p);
            }
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        let hit =
                            shallow(&p, name, ext).or_else(|| shallow(&p.join("out"), name, ext));
                        if hit.is_some() {
                            return hit;
                        }
                        // third level: <dir>/<hash>/out
                        if let Ok(subs) = std::fs::read_dir(&p) {
                            for q in subs.flatten().map(|e| e.path()) {
                                if q.is_dir() {
                                    if let Some(hit) = shallow(&q.join("out"), name, ext) {
                                        return Some(hit);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !dir.pop() {
                return None;
            }
        }
    }
}
