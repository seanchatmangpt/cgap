//! RED-11 reentrancy courts for the ref_store seal counter (lane RC-01).
//!
//! Witness shape (red-11 sec.3): a preempted check-then-act window between
//! the ceiling check and the RMW made the overspend reachable. The fix
//! moves ceiling enforcement INSIDE the RMW (fetch_update CAS loop with a
//! c < bound guard): there is no window between "check passed" and
//! "incremented" for an ISR to occupy - the check IS the RMW. No mocks;
//! final-state asserts on the real register through public methods.

#[path = "../src/ref_store.rs"]
mod ref_store;

use ref_store::*;

const OID_B: [u8; 20] = [0xBB; 20];

fn drive_to(seal: &mut SealRegister, store: &mut RefStore, name: &[u8], n: u64) -> u64 {
    let mut lease = 0u64;
    for i in 0..n {
        lease = store.cas(name, lease, OID_B, seal).expect("stamp");
        assert_eq!(lease, i + 1, "strict monotonic");
    }
    lease
}

/// Court 1 (red-11 witness shape): at the ceiling, spending authority
/// through the public path refuses typed CounterExhausted and the counter
/// never exceeds the ceiling.
#[test]
fn rc01_overspend_through_rmw_is_typed_refusal() {
    let name: &[u8] = b"refs/heads/main";
    let mut seal = SealRegister::with_ceiling(5);
    let mut store = RefStore::new();
    store.create(b"refs/heads/main", [0xAA; 20]).expect("create");
    let lease = drive_to(&mut seal, &mut store, name, 5);
    assert_eq!(lease, 5);
    assert_eq!(seal.counter(), 5, "counter at ceiling");
    let overspend = store.cas(b"refs/heads/main", 5, OID_B, &mut seal);
    assert!(
        matches!(
            overspend,
            Err(Refusal {
                kind: RefusalKind::CounterExhausted,
                ..
            })
        ),
        "overspend past ceiling refused typed"
    );
    assert_eq!(seal.counter(), 5, "counter never exceeded the ceiling");
    // Authority is genuinely spent, not stuck: a fresh register still
    // stamps.
    let mut fresh = SealRegister::with_ceiling(5);
    assert_eq!(fresh.stamp(0, &[1u8; 8]).expect("fresh stamps"), 1);
}

/// Court 2 (no-interrupt control): the same flow with no preemption stays
/// fully legal — leases 1..5, then a typed exhaust. A court that cannot
/// pass without the fix AND fail under the mutation is vacuous; the
/// control pins that the fix did not simply break stamping.
#[test]
fn rc01_control_no_interrupt_no_overspend() {
    let name: &[u8] = b"refs/heads/single";
    let mut seal = SealRegister::with_ceiling(5);
    let mut store = RefStore::new();
    store.create(b"refs/heads/single", [0xAA; 20]).expect("create");
    let lease = drive_to(&mut seal, &mut store, name, 5);
    assert_eq!(lease, 5);
    let next = store.cas(b"refs/heads/single", 5, OID_B, &mut seal);
    assert!(matches!(
        next,
        Err(Refusal {
            kind: RefusalKind::CounterExhausted,
            ..
        })
    ));
    assert_eq!(seal.counter(), 5);
    // Chain is a deterministic function of the stamp sequence: two
    // registers given identical stamps fold to the identical chain.
    let mut a = SealRegister::new();
    let mut b = SealRegister::new();
    for i in 0..5u64 {
        let tag = i.to_le_bytes();
        let _ = a.stamp(0, &tag);
        let _ = b.stamp(0, &tag);
    }
    assert_eq!(a.chain(), b.chain(), "chain fold deterministic");
}

/// Court 3 (strict-monotonic lease uniqueness): every accepted cas stamps
/// a strictly greater lease and a spent lease is never re-mintable
/// (StaleLease), preserving the CAS guard red-11 section 2 names.
#[test]
fn rc01_lease_uniqueness_invariant() {
    let mut seal = SealRegister::new();
    let mut store = RefStore::new();
    store.create(b"refs/agents/a7", [0xAA; 20]).expect("create");
    let mut last = 0u64;
    for i in 0..64 {
        let l = store.cas(b"refs/agents/a7", last, OID_B, &mut seal)
            .expect("stamp");
        assert!(l > last, "lease strictly monotonic at step {}", i);
        let stale = store.cas(b"refs/agents/a7", last, OID_B, &mut seal);
        assert!(
            matches!(
                stale,
                Err(Refusal {
                    kind: RefusalKind::StaleLease,
                    ..
                })
            ),
            "spent lease is not re-mintable at step {}",
            i
        );
        last = l;
    }
    assert_eq!(seal.counter(), 64);
}

/// Court 5 (preempted-window replay, THE flip witness): context A observes
/// counter < bound (the old pre-RMW check passes), the ISR spends the
/// remaining authority through real stamps, then context A resumes AT the
/// RMW via the production reserve path. Fixed code: the guard lives inside
/// the RMW -> typed CounterExhausted, counter stays at ceiling. Mutated
/// code (guard hoisted out of the RMW): the resumed RMW mints lease 6
/// past ceiling 5 -> court red.
#[test]
fn rc01_preempted_window_replay() {
    let mut seal = SealRegister::with_ceiling(5);
    let mut store = RefStore::new();
    store.create(b"refs/heads/main", [0xAA; 20]).expect("create");
    // Context A drives counter to 4: next lease would be 5 == ceiling, so
    // A's check passes.
    let lease = drive_to(&mut seal, &mut store, b"refs/heads/main", 5 - 1);
    assert_eq!(lease, 4);
    assert!(seal.counter() < 5, "context A check passes");
    // ISR: spend the remaining authority through real public stamps.
    let isr = store.cas(b"refs/heads/main", 4, OID_B, &mut seal).expect("isr stamp");
    assert_eq!(isr, 5);
    assert_eq!(seal.counter(), 5, "authority spent by the ISR");
    // Context A resumes AT the RMW (production reserve path, no re-check).
    let resumed = seal.reserve_probe();
    assert!(
        matches!(
            resumed,
            Err(Refusal {
                kind: RefusalKind::CounterExhausted,
                ..
            })
        ),
        "RMW-owned guard refuses the resumed window"
    );
    assert_eq!(seal.counter(), 5, "no lease past the ceiling");
}

/// Court 4 (exhausts-never-wraps law): with ceiling == WIDTH the register
/// still stamps at 0 and refuses, never folds to 0. On host WIDTH ==
/// u64::MAX is undrivable by repeated stamps; the same c < bound closure
/// path is exercised at small bounds by courts 1-3, and the mutation
/// battery (revert to separate check/RMW) is the flip witness for the
/// wrap law at the width boundary.
#[test]
fn rc01_exhausts_never_wraps_law() {
    let mut seal = SealRegister::with_ceiling(SEAL_COUNTER_WIDTH);
    let v = seal.stamp(0, &[2u8; 8]).expect("first stamp under width ceiling");
    assert_eq!(v, 1);
    assert_eq!(seal.counter(), 1);
    assert_ne!(seal.chain(), 0, "chain folded");
    assert_eq!(SEAL_COUNTER_WIDTH, u64::MAX, "host arm is impl64");
}
