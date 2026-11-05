//! aq_slab — zero-copy egress NIF over an mmap'd wire-shm-layout slab (L19).
//!
//! Layout law (wire-shm/src/ring.rs:3-5 + wire-shm lib.rs): segment is
//! `[bookkeeping 4096 B][slot 0] ... [slot capacity-1]`; slot = 16-byte BE
//! header `[magic][crc32][payload_len][ordinal]` + up to MAX_PAYLOAD = 4080
//! payload bytes.
//!
//! The slab resource owns the mapping; `enif_make_resource_binary` wraps the
//! mmap'd frame bytes directly — the binary term's bytes ARE the slab bytes,
//! no copy (resbin.rs:40-52 precedent, declared erl_nif_api_funcs.h:134).
//!
//! Frame validation is `read_frame_into`'s law set IN MEMORY (ring.rs:158-189):
//! magic, stored-ordinal match, checked_add budget, crc32. Failure is the
//! atom `error`, never a projected binary over bad bytes.
//!
//! Lease recycling (spec AqLease free-list bitmask): slot i leased iff bit i
//! of an AtomicU64 mask is set; capacity > 64 is a typed open refusal; lease
//! dtor clears the bit (fetch_and AcqRel, the AND/OR serialization law);
//! stamp refuses a leased slot (SlabSlotLeased).
//!
//! Immutability law (resbin.rs:28-38): a projected frame binary freezes its
//! slot for the binary's life — the plain ring's silent overwrite (ring.rs:110)
//! is SUSPENDED under the NIF boundary; stamp refuses instead.
//!
//! Alloc law (S12): the per-frame hot path allocates nothing; counted allocs
//! are only the one-shot resource allocs at open/lease (crate::alloc_spy;
//! BEAM court reads cgap_alloc_stats/0 deltas).
//!
//! Dirty classification (spec): open = DirtyIo; frame/window/stamp/lease =
//! CPU (crc32 over 4080 B ~ 1.3 us). Durability (msync) is a separate NIF
//! (aq_slab_sync), landed here as the sixth AQ NIF (DirtyIo, direct msync(2) —
//!
//! Deviations from spec (each disclosed in the owning lane's receipt):
//! ingress uses enif_inspect_binary (stamp arg is a binary term; same law,
//! smaller ABI surface than enif_inspect_iovec); windows (lane L23) follow
//! the spec exactly — enif_make_sub_binary over the frame resource binary
//! (erl_nif_api_funcs.h:99), no payload copy, parent pins the slab;
//! open validation applies
//! WireRing::open's size-mismatch law (ring.rs:57-71) on the mapped bytes
//! instead of a second WireRing handle (Darwin second-handle law,
//! shm.rs:31-51).
//!
//! SHIM-COURT anti-vacuity mutation: AQ_MUTATE_CRC_SKIP=1 (read once into a
//! OnceLock) makes validate_frame skip the crc32 comparison. The BEAM-side
//! court MUST flip: corrupt-slot projection succeeds under the mutation and
//! fails without it — a court that cannot flip is FAIL.

use std::ffi::CString;
use std::fs::File;
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::raw::{c_char, c_int, c_void};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicPtr, AtomicU64, Ordering};

use crate::alloc_spy::record_alloc;
use crate::resources::{ErlNifResourceType, ErlNifResourceTypeInit};
use crate::{ErlNifBinary, ErlNifEnv, ErlNifTerm, enif_get_long, make_atom};
use wire_shm::{BOOK_LEN, HEADER_LEN, MAX_PAYLOAD, MAGIC, SLOT_LEN, crc32_ieee};

// mmap externs — bare extern "C" (wire-shm zero-dep law: zero crates, not
// zero syscalls; same declaration set as shm.rs's private block).

const O_RDWR: c_int = 0x0002;
const PROT_READ_WRITE: c_int = 0x1 | 0x2;
const MAP_SHARED: c_int = 0x0001;
/// ERL_NIF_DIRTY_JOB_IO_BOUND (erl_drv_nif.h:51: `= 2`).
pub(crate) const ERL_NIF_DIRTY_JOB_IO_BOUND: u32 = 0x02;

type OffT = i64;
type SizeT = usize;

/// `#define MS_SYNC 0x0010` (macOS SDK <sys/mman.h>:188 — NOT Linux's 4;
/// 0x0004 here would be MS_KILLPAGES, silently non-durable).
const MS_SYNC: c_int = 0x0010;

extern "C" {
    #[link_name = "open"]
    fn c_open(path: *const c_char, oflag: c_int, ...) -> c_int;
    fn msync(addr: *mut c_void, len: SizeT, flags: c_int) -> c_int;
    fn mmap(
        addr: *mut c_void,
        len: SizeT,
        prot: c_int,
        flags: c_int,
        fd: c_int,
        offset: OffT,
    ) -> *mut c_void;
    fn munmap(addr: *mut c_void, len: SizeT) -> c_int;
}

/// Process-shared slab state, shared by the AqSlab resource, every AqLease
/// (Arc — the spec's "Rust-side Arc inside the resource, never a raw borrow"
/// law), and every projected binary (which pins the AqSlab resource).
struct SlabState {
    map: *mut u8,
    map_len: usize,
    capacity: u32,
    /// Bit i set = slot i leased for write. fetch_or/fetch_and serialize
    /// stamp-vs-dtor from any scheduler thread.
    free_mask: AtomicU64,
    /// Backing file; kept open for the slab's life (no name reopen).
    file: File,
}
// SAFETY: process-shared memory through `map` (ShmRing precedent);
// slot mutation goes through atomics or NIF-call-confined slices.
unsafe impl Send for SlabState {}
unsafe impl Sync for SlabState {}

/// Typed refusal set. BEAM surface maps every variant to the `error` atom
/// (cgap_len convention); the types exist for tests, the receipt, fail-loud
/// debugging.
#[derive(Debug)]
enum SlabErr {
    Open(String),
    CapacityTooWide(u32),
    BadMagic(u32),
    StaleOrdinal(u32, u32),
    FrameTooLarge(usize),
    CrcMismatch(u32, u32),
    BudgetOverflow(usize, usize),
    SlotLeased(u32),
    StaleWrite(u64),
    Sync(String),
    PathTooLong(usize),
    EmptyPath,
}

/// SHIM-COURT mutation flag: AQ_MUTATE_CRC_SKIP=1 skips crc32 in
/// validate_frame. Read once; default OFF.
fn mutate_crc_skip() -> bool {
    static M: OnceLock<bool> = OnceLock::new();
    *M.get_or_init(|| std::env::var("AQ_MUTATE_CRC_SKIP").as_deref() == Ok("1"))
}

// ---------------------------------------------------------------------------
// In-memory frame law (read_frame_into's law set over mmap'd bytes)
// ---------------------------------------------------------------------------

/// BE u32 read of header field at `at`.
fn be32(h: &[u8; HEADER_LEN], at: usize) -> u32 {
    u32::from_be_bytes([h[at], h[at + 1], h[at + 2], h[at + 3]])
}

/// Read the 16-byte header at slot `idx` off the mapping. Stack-only.
unsafe fn read_header(map: *mut u8, idx: u32) -> [u8; HEADER_LEN] {
    let off = BOOK_LEN + idx as usize * SLOT_LEN;
    let mut h = [0u8; HEADER_LEN];
    std::ptr::copy_nonoverlapping(map.add(off) as *const u8, h.as_mut_ptr(), HEADER_LEN);
    h
}

/// Validate the frame at `ordinal % capacity` in memory over the mapping and
/// return the payload length. Zero alloc; law set of read_frame_into
/// (ring.rs:158-189).
unsafe fn validate_frame(map: *mut u8, capacity: u32, ordinal: u32) -> Result<usize, SlabErr> {
    let idx = ordinal % capacity;
    let h = read_header(map, idx);
    let magic = be32(&h, 0);
    if magic != MAGIC {
        return Err(SlabErr::BadMagic(magic));
    }
    let stored = be32(&h, 12);
    if stored != ordinal {
        return Err(SlabErr::StaleOrdinal(ordinal, stored));
    }
    let plen = be32(&h, 8) as usize;
    if plen > MAX_PAYLOAD {
        return Err(SlabErr::FrameTooLarge(plen));
    }
    let base = BOOK_LEN + idx as usize * SLOT_LEN;
    let payload = std::slice::from_raw_parts(map.add(base + HEADER_LEN), plen);
    let expect = be32(&h, 4);
    let found = crc32_ieee(payload);
    // SHIM-COURT mutation point: AQ_MUTATE_CRC_SKIP=1 skips this comparison.
    // The court (tests/aq_slab_contract.rs) must flip on this line.
    if mutate_crc_skip() {
        return Ok(plen);
    }
    if found != expect {
        return Err(SlabErr::CrcMismatch(expect, found));
    }
    Ok(plen)
}

/// Stamp header + payload into slot `idx` in place, then advance the
/// bookkeeping next_ordinal (frame BEFORE ordinal, ring.rs:119-121 order law).
unsafe fn stamp_into(
    map: *mut u8,
    map_len: usize,
    capacity: u32,
    free_mask: &AtomicU64,
    payload: &[u8],
    ordinal: u32,
) -> Result<(), SlabErr> {
    if payload.len() > MAX_PAYLOAD {
        return Err(SlabErr::FrameTooLarge(payload.len()));
    }
    let idx = ordinal % capacity;
    // Lease law: a leased slot cannot be re-stamped (SlabSlotLeased).
    if free_mask.load(Ordering::Acquire) & (1u64 << idx) != 0 {
        return Err(SlabErr::SlotLeased(idx));
    }
    let base = BOOK_LEN + idx as usize * SLOT_LEN;
    if base + HEADER_LEN + payload.len() > map_len {
        return Err(SlabErr::BudgetOverflow(base + HEADER_LEN + payload.len(), map_len));
    }
    let crc = crc32_ieee(payload);
    let mut h = [0u8; HEADER_LEN];
    h[0..4].copy_from_slice(&MAGIC.to_be_bytes());
    h[4..8].copy_from_slice(&crc.to_be_bytes());
    h[8..12].copy_from_slice(&(payload.len() as u32).to_be_bytes());
    h[12..16].copy_from_slice(&ordinal.to_be_bytes());
    let slot = map.add(base);
    std::ptr::copy_nonoverlapping(h.as_ptr(), slot as *mut u8, HEADER_LEN);
    std::ptr::copy_nonoverlapping(payload.as_ptr(), slot.add(HEADER_LEN) as *mut u8, payload.len());
    Ok(())
}

// ---------------------------------------------------------------------------
// Bookkeeping (segment header: u64 BE next_ordinal at 0..8, u32 BE capacity
// at 8..12 — wire-shm ring.rs NEXT_ORD_OFF/CAP_OFF)
// ---------------------------------------------------------------------------

unsafe fn read_next_ordinal(map: *mut u8) -> u64 {
    let mut b = [0u8; 8];
    std::ptr::copy_nonoverlapping(map as *const u8, b.as_mut_ptr(), 8);
    u64::from_be_bytes(b)
}

unsafe fn write_next_ordinal(map: *mut u8, next: u64) {
    let b = next.to_be_bytes();
    std::ptr::copy_nonoverlapping(b.as_ptr(), map as *mut u8, 8);
}

unsafe fn read_capacity(map: *mut u8) -> u32 {
    let mut b = [0u8; 4];
    std::ptr::copy_nonoverlapping(map.add(8) as *const u8, b.as_mut_ptr(), 4);
    u32::from_be_bytes(b)
}

// ---------------------------------------------------------------------------
// Resources: AqSlabRes + AqLeaseRes
// ---------------------------------------------------------------------------

/// The slab resource: Arc-held shared state. Dtor drops the Arc; when the
/// LAST Arc drops (resource dtor or lease dtor, whichever is last), the
/// mapping is torn down (munmap before fd close).
struct AqSlabRes {
    state: std::sync::Arc<SlabState>,
}

/// The lease resource. Dtor clears its slot bit via the same Arc — safe even
/// if the slab resource term was GC'd first, because the lease's Arc clone
/// keeps the SlabState (and mapping) alive past the slab resource itself.
struct AqLeaseRes {
    state: std::sync::Arc<SlabState>,
    slot: u32,
}

// SAFETY: SlabState is Send+Sync; the Arc clones move it across scheduler
// threads (BEAM GC fires dtors on arbitrary schedulers).
unsafe impl Send for AqSlabRes {}
unsafe impl Send for AqLeaseRes {}

static AQ_SLAB_TYPE: AtomicPtr<ErlNifResourceType> = AtomicPtr::new(std::ptr::null_mut());
static AQ_LEASE_TYPE: AtomicPtr<ErlNifResourceType> = AtomicPtr::new(std::ptr::null_mut());

/// Slab resource dtor: drop the Arc. Mapping teardown happens when the last
/// owner drops (see SlabDrop).
extern "C" fn aq_slab_dtor(_env: *mut ErlNifEnv, obj: *mut c_void) {
    unsafe { std::ptr::drop_in_place(obj.cast::<AqSlabRes>()) };
}

/// Lease resource dtor: clear the slot's bit (at-most-once dtor, VM-guaranteed
/// resources.rs:167-179 ledger), AcqRel so the fetch_and serializes against
/// stamp's Acquire load + fetch_or.
extern "C" fn aq_lease_dtor(_env: *mut ErlNifEnv, obj: *mut c_void) {
    unsafe {
        let l = &mut *(obj.cast::<AqLeaseRes>());
        l.state.free_mask.fetch_and(!(1u64 << l.slot), Ordering::AcqRel);
        std::ptr::drop_in_place(l as *mut AqLeaseRes);
    }
}

/// Mapping teardown: munmap BEFORE the File field's close (our Drop runs
/// first, then fields drop in declaration order; `file` is the last field).
impl Drop for SlabState {
    fn drop(&mut self) {
        unsafe { munmap(self.map as *mut c_void, self.map_len) };
    }
}

/// Open both AQ resource types at load (fail-closed boot: returning false
/// fails the whole load_nif — the resources.rs:217-225 law; this OTP refuses
/// resource-type opens outside load/upgrade).
pub(crate) fn open_aq_types(env: *mut ErlNifEnv) -> bool {
    static SLAB_INIT: std::sync::OnceLock<ErlNifResourceTypeInit> = std::sync::OnceLock::new();
    static LEASE_INIT: std::sync::OnceLock<ErlNifResourceTypeInit> = std::sync::OnceLock::new();
    let slab_init = SLAB_INIT.get_or_init(|| {
        crate::resources::resource_type_init_dtor(aq_slab_dtor)
    });
    let lease_init = LEASE_INIT.get_or_init(|| {
        crate::resources::resource_type_init_dtor(aq_lease_dtor)
    });
    unsafe fn open_ty(
        env: *mut ErlNifEnv,
        name: &[u8],
        init: &ErlNifResourceTypeInit,
    ) -> *mut ErlNifResourceType {
        let mut tried: c_int = 0;
        resources_extern::enif_open_resource_type_x(
            env,
            name.as_ptr() as *const c_char,
            init as *const _,
            1 | 2, // ERL_NIF_RT_CREATE | ERL_NIF_RT_TAKEOVER
            &mut tried,
        )
    }
    let slab_ty = unsafe { open_ty(env, b"AqSlab\0", slab_init) };
    if slab_ty.is_null() {
        return false;
    }
    let lease_ty = unsafe { open_ty(env, b"AqLease\0", lease_init) };
    if lease_ty.is_null() {
        return false;
    }
    AQ_SLAB_TYPE.store(slab_ty, Ordering::Relaxed);
    AQ_LEASE_TYPE.store(lease_ty, Ordering::Relaxed);
    true
}

/// Minimal local re-declaration of the resource-type open call (the
/// resources.rs externs are private to that module; same C ABI, same VM
/// symbols — the resbin.rs:32-45 re-declaration precedent).
mod resources_extern {
    use super::*;
    use crate::resources::{ErlNifResourceType, ErlNifResourceTypeInit};
    extern "C" {
        pub(crate) fn enif_open_resource_type_x(
            env: *mut ErlNifEnv,
            name_str: *const c_char,
            init: *const ErlNifResourceTypeInit,
            flags: c_int,
            tried: *mut c_int,
        ) -> *mut ErlNifResourceType;
    }
}

/// Alloc + fully initialize an AqSlabRes over `state` (take-before-allocate /
/// restore-on-fail discipline — no dtor-on-garbage window, resbin.rs:148-156
/// ledger class). Returns the obj pointer; caller makes the term and releases
/// the NIF-local reference.
unsafe fn alloc_slab(state: SlabState) -> Option<*mut c_void> {
    let ty = AQ_SLAB_TYPE.load(Ordering::Relaxed);
    if ty.is_null() {
        return None;
    }
    let obj = bin_extern::enif_alloc_resource(ty, std::mem::size_of::<AqSlabRes>());
    if obj.is_null() {
        return None;
    }
    record_alloc(std::mem::size_of::<AqSlabRes>());
    obj.cast::<AqSlabRes>().write(AqSlabRes {
        state: std::sync::Arc::new(state),
    });
    Some(obj)
}

/// Typed borrow of the slab behind a resource term (envelope_of precedent).
unsafe fn slab_of(env: *mut ErlNifEnv, term: ErlNifTerm) -> Option<*mut AqSlabRes> {
    let ty = AQ_SLAB_TYPE.load(Ordering::Relaxed);
    if ty.is_null() {
        return None;
    }
    let mut obj: *mut c_void = std::ptr::null_mut();
    if bin_extern::enif_get_resource(env, term, ty, &mut obj) == 0 || obj.is_null() {
        return None;
    }
    Some(obj.cast::<AqSlabRes>())
}

/// Typed borrow of the lease behind a resource term.
unsafe fn lease_of(env: *mut ErlNifEnv, term: ErlNifTerm) -> Option<*mut AqLeaseRes> {
    let ty = AQ_LEASE_TYPE.load(Ordering::Relaxed);
    if ty.is_null() {
        return None;
    }
    let mut obj: *mut c_void = std::ptr::null_mut();
    if bin_extern::enif_get_resource(env, term, ty, &mut obj) == 0 || obj.is_null() {
        return None;
    }
    Some(obj.cast::<AqLeaseRes>())
}

mod bin_extern {
    use super::*;
    use crate::ErlNifBinary;
    extern "C" {
        /// void* enif_alloc_resource(...) (erl_nif_api_funcs.h:113)
        pub(crate) fn enif_alloc_resource(
            ty: *mut ErlNifResourceType,
            size: usize,
        ) -> *mut c_void;
        /// void enif_release_resource(void*) (:114)
        pub(crate) fn enif_release_resource(obj: *mut c_void);
        /// ERL_NIF_TERM enif_make_resource(ErlNifEnv*, void*) (:115)
        pub(crate) fn enif_make_resource(env: *mut ErlNifEnv, obj: *mut c_void) -> ErlNifTerm;
        /// int enif_get_resource(ErlNifEnv*, ERL_NIF_TERM, ErlNifResourceType*, void**) (:116)
        pub(crate) fn enif_get_resource(
            env: *mut ErlNifEnv,
            term: ErlNifTerm,
            ty: *mut ErlNifResourceType,
            objp: *mut *mut c_void,
        ) -> c_int;
        /// ERL_NIF_TERM enif_make_resource_binary(ErlNifEnv*, void*, const void*, size_t)
        /// (erl_nif_api_funcs.h:134 — the resbin.rs extern precedent)
        pub(crate) fn enif_make_resource_binary(
            env: *mut ErlNifEnv,
            obj: *mut c_void,
            data: *const c_void,
            size: usize,
        ) -> ErlNifTerm;
        /// int enif_inspect_binary(ErlNifEnv*, ERL_NIF_TERM, ErlNifBinary*)
        /// (erl_nif_api_funcs.h:41)
        pub(crate) fn enif_inspect_binary(
            env: *mut ErlNifEnv,
            term: ErlNifTerm,
            bin: *mut ErlNifBinary,
        ) -> c_int;
        /// ERL_NIF_TERM enif_make_sub_binary(ErlNifEnv*, ERL_NIF_TERM,
        /// size_t pos, size_t size) (erl_nif_api_funcs.h:99)
        pub(crate) fn enif_make_sub_binary(
            env: *mut ErlNifEnv,
            src: ErlNifTerm,
            pos: usize,
            size: usize,
        ) -> ErlNifTerm;
    }
}

// ---------------------------------------------------------------------------
// Open (shared logic; called from the DirtyIo NIF)
// ---------------------------------------------------------------------------

/// mmap the segment file at `path` and validate it against the wire-shm law:
/// file size == BOOK_LEN + capacity*SLOT_LEN (ring.rs:57-71 law), bookkeeping
/// capacity matches, capacity <= 64 (u64 mask law). Returns the SlabState.
fn open_slab(path: &[u8]) -> Result<SlabState, SlabErr> {
    if path.is_empty() {
        return Err(SlabErr::EmptyPath);
    }
    let cpath = CString::new(path).map_err(|_| SlabErr::Open("nul in path".into()))?;
    let fd = unsafe { c_open(cpath.as_ptr(), O_RDWR) };
    if fd < 0 {
        return Err(SlabErr::Open(format!("open failed: {}", std::io::Error::last_os_error())));
    }
    // SAFETY: fd is a fresh owning descriptor from open(2).
    let file = unsafe { File::from_raw_fd(fd as RawFd) };
    let map_len = match file.metadata() {
        Ok(m) => m.len() as usize,
        Err(e) => return Err(SlabErr::Open(e.to_string())),
    };
    let capacity_hint = map_len.saturating_sub(BOOK_LEN) / SLOT_LEN;
    if capacity_hint == 0 || capacity_hint > 64 {
        return Err(SlabErr::CapacityTooWide(capacity_hint as u32));
    }
    let map = unsafe {
        mmap(
            std::ptr::null_mut(),
            map_len,
            PROT_READ_WRITE,
            MAP_SHARED,
            file.as_raw_fd(),
            0,
        )
    };
    if map == std::ptr::null_mut() && map_len > 0 {
        return Err(SlabErr::Open(format!("mmap failed: {}", std::io::Error::last_os_error())));
    }
    let map = map as *mut u8;
    // Wire-shm open law on the mapped bytes: bookkeeping capacity must match
    // the file-size-derived capacity (ring.rs:57-71 size-mismatch law).
    let cap_field = unsafe { read_capacity(map) };
    if cap_field as usize != capacity_hint {
        return Err(SlabErr::Open(format!(
            "segment size mismatch: capacity field {} vs size-implied {}",
            cap_field, capacity_hint
        )));
    }
    // First-page touch (validation) happens implicitly below via read_capacity;
    // also sanity-check the bookkeeping ordinal is a plausible u64.
    let _ = unsafe { read_next_ordinal(map) };
    Ok(SlabState {
        map,
        map_len,
        capacity: cap_field,
        free_mask: AtomicU64::new(0),
        file,
    })
}

// ---------------------------------------------------------------------------
// NIF surface (all errors = the atom `error`, cgap_len convention)
// ---------------------------------------------------------------------------

/// aq_slab_open(PathBinary) -> Slab :: reference() | error. DirtyIo.
pub(crate) extern "C" fn aq_slab_open(
    env: *mut ErlNifEnv,
    argc: c_int,
    argv: *const ErlNifTerm,
) -> ErlNifTerm {
    let args = unsafe { std::slice::from_raw_parts(argv, argc.max(0) as usize) };
    let mut bin = ErlNifBinary {
        size: 0,
        data: std::ptr::null_mut(),
        ref_bin: std::ptr::null_mut(),
        __spare__: [std::ptr::null_mut(); 2],
    };
    if argc != 1
        || unsafe { bin_extern::enif_inspect_binary(env, args[0], &mut bin) } == 0
    {
        return make_atom(env, "error");
    }
    let path = unsafe { std::slice::from_raw_parts(bin.data, bin.size) };
    match open_slab(path) {
        Ok(state) => unsafe {
            match alloc_slab(state) {
                Some(obj) => {
                    let term = bin_extern::enif_make_resource(env, obj);
                    bin_extern::enif_release_resource(obj);
                    term
                }
                None => make_atom(env, "error"),
            }
        },
        Err(_) => make_atom(env, "error"),
    }
}

/// aq_slab_frame(Slab, Ordinal) -> Binary :: binary() | error. CPU.
/// Zero-copy egress: enif_make_resource_binary over the mmap'd frame bytes.
pub(crate) extern "C" fn aq_slab_frame(
    env: *mut ErlNifEnv,
    argc: c_int,
    argv: *const ErlNifTerm,
) -> ErlNifTerm {
    let args = unsafe { std::slice::from_raw_parts(argv, argc.max(0) as usize) };
    let mut ord = 0i64;
    if argc != 2 || unsafe { enif_get_long(env, args[1], &mut ord) } == 0 || ord < 0 {
        return make_atom(env, "error");
    }
    let Some(slab) = (unsafe { slab_of(env, args[0]) }) else {
        return make_atom(env, "error");
    };
    unsafe {
        let st = &(*slab).state;
        let plen = match validate_frame(st.map, st.capacity, ord as u32) {
            Ok(n) => n,
            Err(_) => return make_atom(env, "error"),
        };
        let idx = (ord as u32) % st.capacity;
        let base = BOOK_LEN + idx as usize * SLOT_LEN + HEADER_LEN;
        // SAFETY: obj is the live resource the binary term pins (the term pins
        // it for this call and the returned binary adds its own keep-alive);
        // base..base+plen are validated in-bounds mapped bytes.
        bin_extern::enif_make_resource_binary(
            env,
            slab as *mut c_void,
            st.map.add(base) as *const c_void,
            plen,
        )
    }
}

/// Window [offset, len) bounds law vs the frame's payload length plen:
/// off + len <= plen (overflow checked). Returns Some((off, len)) or None
/// (typed refusal `error` at the NIF boundary). Testable seam so the
/// over-read mutation court can flip against the exact law the NIF applies.
fn window_bounds(plen: usize, off: usize, len: usize) -> Option<(usize, usize)> {
    // SHIM-COURT mutation point: AQ_MUTATE_WINDOW_OOB_SKIP=1 skips the
    // bounds comparison — an over-reading window must be caught by this
    // check; a court that cannot flip on it is FAIL.
    if mutate_window_oob_skip() {
        return Some((off, len));
    }
    off.checked_add(len).map(|end| end <= plen).unwrap_or(false)
        .then_some((off, len))
}

/// SHIM-COURT mutation flag: AQ_MUTATE_WINDOW_OOB_SKIP=1 skips the window
/// bounds law. Read once; default OFF.
fn mutate_window_oob_skip() -> bool {
    static M: OnceLock<bool> = OnceLock::new();
    *M.get_or_init(|| std::env::var("AQ_MUTATE_WINDOW_OOB_SKIP").as_deref() == Ok("1"))
}

/// aq_slab_window(Slab, Ordinal, Offset, Len) -> Binary :: binary() | error.
/// CPU. Zero-copy window: enif_make_sub_binary over the frame resource
/// binary (erl_nif_api_funcs.h:99) — the sub-binary references the parent
/// refc binary, 0 payload bytes copied; the parent pins the slab resource.
pub(crate) extern "C" fn aq_slab_window(
    env: *mut ErlNifEnv,
    argc: c_int,
    argv: *const ErlNifTerm,
) -> ErlNifTerm {
    let args = unsafe { std::slice::from_raw_parts(argv, argc.max(0) as usize) };
    let mut ord = 0i64;
    let mut off = 0i64;
    let mut len = 0i64;
    if argc != 4
        || unsafe { enif_get_long(env, args[1], &mut ord) } == 0
        || unsafe { enif_get_long(env, args[2], &mut off) } == 0
        || unsafe { enif_get_long(env, args[3], &mut len) } == 0
        || ord < 0
        || off < 0
        || len < 0
    {
        return make_atom(env, "error");
    }
    let Some(slab) = (unsafe { slab_of(env, args[0]) }) else {
        return make_atom(env, "error");
    };
    unsafe {
        let st = &(*slab).state;
        let plen = match validate_frame(st.map, st.capacity, ord as u32) {
            Ok(n) => n,
            Err(_) => return make_atom(env, "error"),
        };
        let (off, len) = match window_bounds(plen, off as usize, len as usize) {
            Some(pair) => pair,
            None => return make_atom(env, "error"),
        };
        // Frame resource binary over the mapped payload (the parent refc
        // binary pins the slab resource), then the spec'd enif_make_sub_binary
        // window slice — references the parent, no payload copy.
        let frame = bin_extern::enif_make_resource_binary(
            env,
            slab as *mut c_void,
            st.map.add(BOOK_LEN + (ord as u32) % st.capacity as usize * SLOT_LEN + HEADER_LEN)
                as *const c_void,
            plen,
        );
        // SAFETY: off+len <= plen (window_bounds law), so pos..pos+size are
        // within the parent binary's extent; the sub-binary keeps the parent
        // (and transitively the resource + mapping) alive — refcount law.
        bin_extern::enif_make_sub_binary(env, frame, off, len)
    }
}

/// aq_slab_stamp(Slab, PayloadBinary) -> ok | error. CPU.
/// Writes the frame at the bookkeeping next_ordinal's slot (no-lease,
/// overwrite-legal mode: lawful only when no frame binary was projected from
/// the slot — spec §4), refusing a leased slot (SlabSlotLeased) and a stale
/// next_ordinal wrap; frame BEFORE ordinal store (ring.rs:119-121).
pub(crate) extern "C" fn aq_slab_stamp(
    env: *mut ErlNifEnv,
    argc: c_int,
    argv: *const ErlNifTerm,
) -> ErlNifTerm {
    let args = unsafe { std::slice::from_raw_parts(argv, argc.max(0) as usize) };
    let mut bin = ErlNifBinary {
        size: 0,
        data: std::ptr::null_mut(),
        ref_bin: std::ptr::null_mut(),
        __spare__: [std::ptr::null_mut(); 2],
    };
    if argc != 2
        || unsafe { bin_extern::enif_inspect_binary(env, args[1], &mut bin) } == 0
    {
        return make_atom(env, "error");
    }
    let Some(slab) = (unsafe { slab_of(env, args[0]) }) else {
        return make_atom(env, "error");
    };
    unsafe {
        let st = &(*slab).state;
        let next = read_next_ordinal(st.map);
        if next > u32::MAX as u64 {
            return make_atom(env, "error"); // StaleWrite
        }
        let payload = std::slice::from_raw_parts(bin.data, bin.size);
        if let Err(_) = stamp_into(st.map, st.map_len, st.capacity, &st.free_mask, payload, next as u32)
        {
            return make_atom(env, "error");
        }
        // Frame BEFORE ordinal (ring.rs:119-121 order law).
        write_next_ordinal(st.map, next + 1);
        make_atom(env, "ok")
    }
}

/// aq_slab_lease(Slab, Ordinal) -> Lease :: reference() | error. CPU.
/// Sets the slot's free-list bit (fetch_or AcqRel); already-set bit is the
/// `error` atom (already leased). The lease resource's dtor clears the bit.
pub(crate) extern "C" fn aq_slab_lease(
    env: *mut ErlNifEnv,
    argc: c_int,
    argv: *const ErlNifTerm,
) -> ErlNifTerm {
    let args = unsafe { std::slice::from_raw_parts(argv, argc.max(0) as usize) };
    let mut ord = 0i64;
    if argc != 2 || unsafe { enif_get_long(env, args[1], &mut ord) } == 0 || ord < 0 {
        return make_atom(env, "error");
    }
    let Some(slab) = (unsafe { slab_of(env, args[0]) }) else {
        return make_atom(env, "error");
    };
    unsafe {
        let st = &(*slab).state;
        let slot = (ord as u32) % st.capacity;
        // fetch_or returns the previous mask: a set bit means already leased.
        let prev = st.free_mask.fetch_or(1u64 << slot, Ordering::AcqRel);
        if prev & (1u64 << slot) != 0 {
            return make_atom(env, "error");
        }
        let ty = AQ_LEASE_TYPE.load(Ordering::Relaxed);
        if ty.is_null() {
            // Backout: clear the bit we set (failure stays recoverable).
            st.free_mask.fetch_and(!(1u64 << slot), Ordering::AcqRel);
            return make_atom(env, "error");
        }
        let obj = bin_extern::enif_alloc_resource(ty, std::mem::size_of::<AqLeaseRes>());
        if obj.is_null() {
            st.free_mask.fetch_and(!(1u64 << slot), Ordering::AcqRel);
            return make_atom(env, "error");
        }
        record_alloc(std::mem::size_of::<AqLeaseRes>());
        obj.cast::<AqLeaseRes>().write(AqLeaseRes {
            state: std::sync::Arc::clone(&(*slab).state),
            slot,
        });
        let term = bin_extern::enif_make_resource(env, obj);
        bin_extern::enif_release_resource(obj);
        term
    }
}

// ---------------------------------------------------------------------------
// Sync (durability)
// ---------------------------------------------------------------------------

/// msync(MS_SYNC) the full mapping to its backing file. `Ok(())` = flushed;
/// `Err` maps any msync failure (the lease Arc keeps the mapping alive, so
/// failure here is an OS-level refusal, never a use-after-free).
fn sync_slab(st: &SlabState) -> Result<(), SlabErr> {
    let rc = unsafe { msync(st.map as *mut c_void, st.map_len, MS_SYNC) };
    if rc != 0 {
        return Err(SlabErr::Sync(std::io::Error::last_os_error().to_string()));
    }
    Ok(())
}

/// `aq_slab_sync(Slab) -> ok | error`. DirtyIo: msync is blocking disk I/O
/// (the spec's durability NIF; never on a normal scheduler).
pub(crate) extern "C" fn aq_slab_sync(
    env: *mut ErlNifEnv,
    argc: c_int,
    argv: *const ErlNifTerm,
) -> ErlNifTerm {
    let args = unsafe { std::slice::from_raw_parts(argv, argc.max(0) as usize) };
    if argc != 1 {
        return make_atom(env, "error");
    }
    let Some(slab) = (unsafe { slab_of(env, args[0]) }) else {
        return make_atom(env, "error");
    };
    match sync_slab(&unsafe { (*slab).state.as_ref() }) {
        Ok(()) => make_atom(env, "ok"),
        Err(_) => make_atom(env, "error"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wire_shm::{BOOK_LEN, HEADER_LEN, MAGIC, MAX_PAYLOAD, SLOT_LEN};

    /// Build a real wire-shm-layout segment file in a fresh tempdir.
    fn make_segment(capacity: u32) -> (std::path::PathBuf, ()) {
        static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "aq_slab_test_{}_{}_{}",
            std::process::id(),
            capacity,
            N.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("seg.wire");
        let total = BOOK_LEN + capacity as usize * SLOT_LEN;
        let mut seg = vec![0u8; total];
        seg[0..8].copy_from_slice(&0u64.to_be_bytes());
        seg[8..12].copy_from_slice(&capacity.to_be_bytes());
        std::fs::write(&path, &seg).unwrap();
        (path, ())
    }

    #[test]
    fn open_and_stamp_frame_roundtrip() {
        let (path, _) = make_segment(4);
        let st = open_slab(path.as_os_str().to_str().unwrap().as_bytes()).ok().unwrap();
        let map = st.map;
        unsafe {
            stamp_into(map, st.map_len, st.capacity, &st.free_mask, b"hello", 0).unwrap();
            write_next_ordinal(map, 1);
            stamp_into(map, st.map_len, st.capacity, &st.free_mask, b"world!", 1).unwrap();
            let plen = validate_frame(map, st.capacity, 0).unwrap();
            assert_eq!(plen, 5);
            let plen1 = validate_frame(map, st.capacity, 1).unwrap();
            assert_eq!(plen1, 6);
            // Unstamped slot: magic 0 -> BadMagic refusal.
            assert!(matches!(
                validate_frame(map, st.capacity, 2),
                Err(SlabErr::BadMagic(_))
            ));
            // Overwrite-legal stamp to slot 0 (ordinal 4, no lease, no live
            // projection in this test) — wrap addressing law.
            stamp_into(map, st.map_len, st.capacity, &st.free_mask, b"zz", 4).unwrap();
            // Stored-ordinal law: ordinal 0 no longer matches slot 0.
            assert!(matches!(
                validate_frame(map, st.capacity, 0),
                Err(SlabErr::StaleOrdinal(_, _))
            ));
            assert_eq!(validate_frame(map, st.capacity, 4).unwrap(), 2);
        }
        drop(st);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn crc_mismatch_and_budget_laws() {
        let (path, _) = make_segment(2);
        let st = open_slab(path.as_os_str().to_str().unwrap().as_bytes()).ok().unwrap();
        let map = st.map;
        unsafe {
            stamp_into(map, st.map_len, st.capacity, &st.free_mask, b"payload", 0).unwrap();
            // Corrupt one payload byte in place.
            let base = BOOK_LEN + HEADER_LEN;
            // Skip the crc-skip mutation for the negative leg.
            std::ptr::copy_nonoverlapping(b"X".as_ptr(), map.add(base), 1);
            match (validate_frame(map, st.capacity, 0), mutate_crc_skip()) {
                (Err(SlabErr::CrcMismatch(exp, _found)), false) => {
                    let _ = exp;
                }
                (Ok(_), true) => { /* mutation active: projection would succeed */ }
                other => panic!("unexpected: {other:?}"),
            }
            // FrameTooLarge: plen over budget is refused (wire input).
            let big = [0u8; MAX_PAYLOAD + 1];
            assert!(
                stamp_into(map, st.map_len, st.capacity, &st.free_mask, &big[..], 1).is_err()
            );
            assert!(
                stamp_into(map, st.map_len, st.capacity, &st.free_mask, &[0u8; MAX_PAYLOAD], 1)
                    .is_ok()
            );
        }
        drop(st);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn lease_mask_and_dtor_law() {
        let (path, _) = make_segment(4);
        let st = std::sync::Arc::new(
            open_slab(path.as_os_str().to_str().unwrap().as_bytes()).ok().unwrap(),
        );
        // Lease slot 2 (fetch_or) — stamp to it must refuse (SlabSlotLeased).
        let prev = st.free_mask.fetch_or(1u64 << 2, Ordering::AcqRel);
        assert_eq!(prev & (1u64 << 2), 0, "first lease must win");
        unsafe {
            match stamp_into(st.map, st.map_len, st.capacity, &st.free_mask, b"no", 2) {
                Err(SlabErr::SlotLeased(2)) => {}
                other => panic!("expected SlotLeased, got {other:?}"),
            }
        }
        // Dtor law: clear the bit, stamp succeeds.
        st.free_mask.fetch_and(!(1u64 << 2), Ordering::AcqRel);
        unsafe {
            stamp_into(st.map, st.map_len, st.capacity, &st.free_mask, b"yes", 2).unwrap();
        }
        let _ = std::fs::remove_dir_all(
            std::path::Path::new(path.as_os_str().to_str().unwrap()).parent().unwrap(),
        );
    }

    #[test]
    fn sync_flushes_mapped_segment() {
        let (path, _) = make_segment(2);
        let st = open_slab(path.as_os_str().to_str().unwrap().as_bytes()).ok().unwrap();
        unsafe {
            stamp_into(map_ptr(&st), st.map_len, st.capacity, &st.free_mask, b"durable", 0).unwrap();
        }
        sync_slab(&st).unwrap();
        // Re-open the same file fresh: the flushed frame must validate.
        let st2 = open_slab(path.as_os_str().to_str().unwrap().as_bytes()).ok().unwrap();
        unsafe {
            assert_eq!(validate_frame(map_ptr(&st2), st2.capacity, 0).unwrap(), 7);
        }
        drop(st);
        drop(st2);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    /// Helper so tests can share the raw map pointer without an extra unsafe
    /// block at each call site.
    fn map_ptr(st: &SlabState) -> *mut u8 {
        st.map
    }

    #[test]
    fn open_refusals() {
        // Capacity > 64 refused (u64 mask law).
        let dir = std::env::temp_dir().join(format!("aq_slab_wide_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("wide.wire");
        let total = BOOK_LEN + 65 * SLOT_LEN;
        let mut seg = vec![0u8; total];
        seg[8..12].copy_from_slice(&65u32.to_be_bytes());
        std::fs::write(&path, &seg).unwrap();
        let mut st = open_slab(path.as_os_str().to_str().unwrap().as_bytes());
        assert!(matches!(st, Err(SlabErr::CapacityTooWide(65))));
        // Capacity-field vs size mismatch (ring.rs:57-71 law).
        let path2 = dir.join("mismatch.wire");
        let seg2 = vec![0u8; BOOK_LEN + 2 * SLOT_LEN];
        std::fs::write(&path2, &seg2).unwrap();
        st = open_slab(path2.as_os_str().to_str().unwrap().as_bytes());
        assert!(matches!(st, Err(SlabErr::Open(_))));
        // Missing file.
        st = open_slab(b"/nonexistent/aq/seg.wire");
        assert!(matches!(st, Err(SlabErr::Open(_))));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
