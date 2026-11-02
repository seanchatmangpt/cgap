//! AE7 — enclave tracking allocator interceptor (v26.11.2 countermeasure
//! lane 07).
//!
//! A reusable counting/budgeting global-allocator TYPE for hot-path
//! zero-alloc courts. REGISTRATION DIVERGENCE (recorded, lane receipt):
//! the `#[global_allocator]` REGISTRATION is deliberately left to each
//! court binary (rustc E0152: a lib-level registration collides with the
//! per-binary counting allocators already landed across cgap test
//! targets — cgap-gate/penstock.rs, cgap-parquet/rle.rs, cgap-bn3/
//! unify.rs and six `crates/cgap/tests/*` files each install their own).
//! A court binary declares the type + installs it; the landed form is
//! `crates/cgap/tests/alloc_boundary.rs`.
//!
//! The type is core-only types + a std eprintln diagnostic (cgap-core is
//! a host-only crate per the SP1 gate; `#![no_std]` is declared nowhere
//! in this crate).
//!
//! Budget semantics: [`start_hot_path_budget`] arms a process-global
//! budget (max allocation COUNT, max heap BYTES). Any alloc/realloc/
//! alloc_zeroed while armed that pushes either counter over budget is a
//! BREACH: the offending site label and observed counts are recorded in
//! survives-disarm statics ([`last_breach`]), the interceptor fully
//! DISARMS, prints a best-effort stderr diagnostic naming the site, and
//! then serves the allocation from the INNER allocator so the process
//! stays alive; the court reads [`last_breach`] / [`window_counts`]
//! after the window and FAILS ITSELF with a panic naming the site.
//!
//! BREACH PATH (measured this session, receipted in the lane report):
//! panicking DIRECTLY from inside a `GlobalAlloc` method ABORTS the
//! process ("failed to initiate panic, error 3" -> SIGABRT), and the
//! null-return route also aborts — `handle_alloc_error` panics are NOT
//! catchable on this toolchain, so the court harness dies with exit
//! 134/SIGABRT before it can name the site. Serving real memory is the
//! measured-working design: post-breach execution continues with real
//! (counted-then-untracked) allocations until the window's guard drops
//! and the court asserts on the sticky breach record.
//!
//! SEQUENCING LAW: budget state is process-global atomics, so courts in
//! one binary MUST run sequentially (single `#[test]` driver; harness
//! sibling threads pollute a global window — measured: a parallel
//! positive-control test leaked allocs into the ingress window). See
//! `crates/cgap/tests/alloc_boundary.rs`.
//!
//! DEPTH LAW (measured defect, fixed): a breach zeroes DEPTH while the
//! window's guards are still alive; a plain `fetch_sub` disarm wraps
//! DEPTH to usize::MAX and leaks ARMED into every later window. Disarm
//! is saturating (compare_exchange loop).

// SOURCE-SCAN-EXEMPT: alloc:: : path-included test-harness allocator module (#[path]-included by crates/cgap/tests/alloc_boundary.rs); tests-only collaborator, never part of the shipped kernel surface; the no_std invariant table itself is unchanged.
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

// OWNER-THREAD LAW (measured, this session): budget state is
// process-global, but the budget LAW is about the hot-path thread.
// Witnessed: under the libtest harness the HARNESS thread allocates
// concurrently (a stray 4 B alloc) and breaches the window on a foreign
// thread — flake only under the full suite, never standalone (timing).
// Law: only allocations issued from the owner thread's STACK REGION are
// counted/enforced; foreign-thread allocations pass through untracked.
//
// Discriminator: the stack ADDRESS of an anchor local captured at arm
// time. Same-thread calls during the window run at comparable frame
// depth (the window brackets a shallow loop), so their stack pointers
// stay within a 1 MiB band of the anchor. Foreign threads' stacks are
// separate mmap mappings (macOS places them TBs apart), so their stack
// pointers fall outside the band. Allocation-free: two stack reads and
// atomics only — thread::current() was measured to allocate its fallback
// Thread inside the allocator (infinite recursion -> stack overflow).
static OWNER_SP: AtomicUsize = AtomicUsize::new(0);
const SP_BAND: usize = 1 << 20; // 1 MiB

#[inline]
fn is_owner_thread() -> bool {
    let probe = 0u8;
    let sp = &probe as *const u8 as usize;
    let anchor = OWNER_SP.load(Ordering::Acquire);
    anchor != 0 && sp.abs_diff(anchor) < SP_BAND
}

/// Pass-through tracking allocator: forwards every call to the inner
/// allocator unchanged; while armed, counts allocs/bytes and enforces
/// the armed budget (breach -> record + disarm + serve).
pub struct EnclaveTrackingAlloc<A> {
    /// The real allocator (e.g. `std::alloc::System` in host courts).
    pub inner: A,
}

// Process-global interceptor state (see module doc re: single-driver
// sequencing).
static ARMED: AtomicBool = AtomicBool::new(false);
static DEPTH: AtomicUsize = AtomicUsize::new(0);
static MAX_ALLOCS: AtomicUsize = AtomicUsize::new(0);
static MAX_BYTES: AtomicUsize = AtomicUsize::new(0);
static COUNT_ALLOCS: AtomicUsize = AtomicUsize::new(0);
static COUNT_BYTES: AtomicUsize = AtomicUsize::new(0);
static SITE_PTR: AtomicUsize = AtomicUsize::new(0);
static SITE_LEN: AtomicUsize = AtomicUsize::new(0);
static BREACH: AtomicBool = AtomicBool::new(false);
static BREACH_ALLOCS: AtomicUsize = AtomicUsize::new(0);
static BREACH_BYTES: AtomicUsize = AtomicUsize::new(0);
static BREACH_OFFENDING: AtomicUsize = AtomicUsize::new(0);

/// One budget-breach record (survives guard drop / disarm; cleared only
/// by [`reset_breach`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Breach {
    /// The `site` label passed to the armed [`start_hot_path_budget`].
    pub site: &'static str,
    /// Allocation count observed at breach time.
    pub allocs: usize,
    /// Heap bytes observed at breach time.
    pub bytes: usize,
    /// Size in bytes of the allocation that crossed the budget.
    pub offending_bytes: usize,
}

/// RAII guard for a hot-path budget window; drop = disarm (saturating,
/// depth-counted — see the module doc DEPTH LAW).
#[derive(Debug)]
pub struct AllocGuard {
    _priv: (),
}

impl Drop for AllocGuard {
    fn drop(&mut self) {
        disarm_hot_path_budget();
    }
}

/// Arm a hot-path allocation budget. Nested calls increment depth; the
/// budget applies while ANY guard is alive (innermost values win).
pub fn start_hot_path_budget(
    site: &'static str,
    max_allocs: usize,
    max_bytes: usize,
) -> AllocGuard {
    DEPTH.fetch_add(1, Ordering::AcqRel);
    SITE_PTR.store(site.as_ptr() as usize, Ordering::Release);
    SITE_LEN.store(site.len(), Ordering::Release);
    MAX_ALLOCS.store(max_allocs, Ordering::Release);
    MAX_BYTES.store(max_bytes, Ordering::Release);
    COUNT_ALLOCS.store(0, Ordering::Release);
    COUNT_BYTES.store(0, Ordering::Release);
    ARMED.store(true, Ordering::Release);
    {
        let anchor = 0u8;
        OWNER_SP.store((&anchor as *const u8).addr(), Ordering::Release);
    }
    AllocGuard { _priv: () }
}

/// Disarm (guard drop). Idempotent, depth-counted, SATURATING at zero
/// (module doc DEPTH LAW).
pub fn disarm_hot_path_budget() {
    let mut d = DEPTH.load(Ordering::Acquire);
    loop {
        if d == 0 {
            return;
        }
        match DEPTH.compare_exchange_weak(d, d - 1, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => {
                if d == 1 {
                    ARMED.store(false, Ordering::Release);
                    MAX_ALLOCS.store(0, Ordering::Release);
                    MAX_BYTES.store(0, Ordering::Release);
                    OWNER_SP.store(0, Ordering::Release);
                }
                return;
            }
            Err(cur) => d = cur,
        }
    }
}

/// Last breach record, if the interceptor breached since [`reset_breach`].
pub fn last_breach() -> Option<Breach> {
    if !BREACH.load(Ordering::Acquire) {
        return None;
    }
    let ptr = SITE_PTR.load(Ordering::Acquire) as *const u8;
    let len = SITE_LEN.load(Ordering::Acquire);
    // SAFETY: SITE_PTR/SITE_LEN are only ever stored from a &'static str
    // passed to `start_hot_path_budget`; static-lifetime bytes.
    let site = unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr, len)) };
    Some(Breach {
        site,
        allocs: BREACH_ALLOCS.load(Ordering::Acquire),
        bytes: BREACH_BYTES.load(Ordering::Acquire),
        offending_bytes: BREACH_OFFENDING.load(Ordering::Acquire),
    })
}

/// Clear the sticky breach record (between courts).
pub fn reset_breach() {
    BREACH.store(false, Ordering::Release);
}

/// Observed counts of the current window (diagnostics).
pub fn window_counts() -> (usize, usize) {
    (
        COUNT_ALLOCS.load(Ordering::Acquire),
        COUNT_BYTES.load(Ordering::Acquire),
    )
}

// One breach path, shared by all entry points. DISARM FIRST so the
// diagnostic/continuation allocations cannot recursively re-breach.
// Never panics from inside the allocator (module doc: measured abort).
#[cold]
#[inline(never)]
fn breach(offending_bytes: usize) {
    let allocs = COUNT_ALLOCS.load(Ordering::Acquire);
    let bytes = COUNT_BYTES.load(Ordering::Acquire);
    BREACH_ALLOCS.store(allocs, Ordering::Release);
    BREACH_BYTES.store(bytes, Ordering::Release);
    BREACH_OFFENDING.store(offending_bytes, Ordering::Release);
    BREACH.store(true, Ordering::Release);
    ARMED.store(false, Ordering::Release);
    DEPTH.store(0, Ordering::Release);
    MAX_ALLOCS.store(0, Ordering::Release);
    MAX_BYTES.store(0, Ordering::Release);
    // Best-effort diagnostic naming the site (mid-run; std is live).
    eprintln!(
        "ALLOC BUDGET BREACH: site={:?} allocs={} bytes={} offending={}",
        site_label(),
        allocs,
        bytes,
        offending_bytes
    );
}

/// Decode the armed site label from the statics (SAFETY: stored from a
/// &'static str by start_hot_path_budget; static-lifetime bytes).
fn site_label() -> &'static str {
    let ptr = SITE_PTR.load(Ordering::Acquire) as *const u8;
    let len = SITE_LEN.load(Ordering::Acquire);
    unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr, len)) }
}

#[inline]
fn over_budget(size: usize) -> bool {
    let allocs = COUNT_ALLOCS.fetch_add(1, Ordering::AcqRel) + 1;
    let bytes = COUNT_BYTES.fetch_add(size, Ordering::AcqRel) + size;
    let max_a = MAX_ALLOCS.load(Ordering::Acquire);
    let max_b = MAX_BYTES.load(Ordering::Acquire);
    allocs > max_a || bytes > max_b
}

// SAFETY (all impls): full forward to `inner` with identical semantics;
// the tracking path touches only lock-free atomics; the breach path
// disarms BEFORE serving, so continuation allocations are untracked.
unsafe impl<A: GlobalAlloc> GlobalAlloc for EnclaveTrackingAlloc<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // ARMED first: is_owner_thread reads the current-thread TLS
        // handle, which is only valid once std runtime init has run; a
        // pre-init startup allocation must pass straight through.
        if ARMED.load(Ordering::Acquire) && is_owner_thread() && over_budget(layout.size()) {
            breach(layout.size());
        }
        unsafe { self.inner.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if ARMED.load(Ordering::Acquire) && is_owner_thread() && over_budget(layout.size()) {
            breach(layout.size());
        }
        unsafe { self.inner.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if ARMED.load(Ordering::Acquire) && is_owner_thread() && over_budget(new_size) {
            breach(new_size);
            // Contents must survive a realloc; the disarm above already
            // happened, so this copy is untracked (System.realloc would
            // preserve contents; the serve-fresh form must copy).
            let nl =
                Layout::from_size_align(new_size, layout.align()).expect("realloc layout valid");
            let fresh = unsafe { self.inner.alloc(nl) };
            if !fresh.is_null() {
                unsafe {
                    core::ptr::copy_nonoverlapping(ptr, fresh, layout.size().min(new_size));
                }
            }
            return fresh;
        }
        unsafe { self.realloc_passthrough(ptr, layout, new_size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.inner.dealloc(ptr, layout) }
    }
}

impl<A: GlobalAlloc> EnclaveTrackingAlloc<A> {
    // Realloc passthrough split out so the breach branch above stays
    // #[cold]-separable from the hot forward.
    unsafe fn realloc_passthrough(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { self.inner.realloc(ptr, layout, new_size) }
    }
}
