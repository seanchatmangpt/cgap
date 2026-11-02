//! SP1: bare-metal enclave self-play entry (FA10 maximal-consumer pattern,
//! `docs/jira/v26.11.2/adversarial/forensic-lane-10.md:81` - every
//! subsystem named by ONE representative real call in ONE link, so the
//! size census sees the whole engine, not a fixture).
//!
//! Built ONLY for `target_os = "none"` (riscv32imc XIP class). On host
//! builds this file compiles to an empty stub so `cargo test -p cgap`
//! stays green; the real body is cfg-gated below.
#![cfg_attr(target_os = "none", no_main)]
#![cfg_attr(target_os = "none", no_std)]

/// Host stub: host `cargo test` builds this bin target for the host triple,
/// so it must carry a `main`.
#[cfg(not(target_os = "none"))]
#[allow(dead_code)]
fn main() {}

/// The none-target body.
#[cfg(target_os = "none")]
mod imp {
    use core::arch::asm;
    use core::hint::black_box;
    use core::sync::atomic::{AtomicU32, Ordering};

    /// Volatile egress: results of every representative subsystem call are
    /// XOR-accumulated here; read back volatilely in the halt loop so the
    /// calls cannot be dead-code-eliminated (forensic-lane-10 method).
    static EGRESS: AtomicU32 = AtomicU32::new(0);

    /// Bare-metal bump allocator: the pipeline seams carry `alloc`-typed
    /// signatures (git pack writer), so the link needs a global allocator.
    /// Bump-only (no free); the entry runs one pass and halts.
    struct Bump;

    static mut HEAP: [u8; 8192] = [0u8; 8192];
    static HEAP_TOP: AtomicU32 = AtomicU32::new(0);

    unsafe impl Sync for Bump {}

    unsafe impl core::alloc::GlobalAlloc for Bump {
        unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
            // Single-core enclave: one hart runs the entry; the atomic
            // top-pointer is the only shared state.
            let base = core::ptr::addr_of_mut!(HEAP) as usize;
            let top = HEAP_TOP.load(Ordering::SeqCst) as usize;
            let aligned = (base + top + layout.align() - 1) & !(layout.align() - 1);
            if aligned + layout.size() > base + 8192 {
                return core::ptr::null_mut();
            }
            HEAP_TOP.store((aligned - base + layout.size()) as u32, Ordering::SeqCst);
            aligned as *mut u8
        }
        unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
    }

    #[global_allocator]
    static ALLOCATOR: Bump = Bump;

    #[panic_handler]
    fn ph(_: &core::panic::PanicInfo) -> ! {
        loop {
            unsafe { asm!("wfi") }
        }
    }

    /// SOVEREIGN ENTRY. One representative REAL call per subsystem.
    #[no_mangle]
    pub extern "C" fn cgap_self_play_start() -> ! {
        let mut egress: u32 = 0;

        // --- P28 (A17, 80 B): root license, validated intrinsically ----
        let att = cgap_p28_root();
        egress ^= att.validate_intrinsic().is_ok() as u32;

        // --- pipeline wiring against the real license -----------------
        // (EnclavePipeline::new re-validates; a refused construction is a
        // typed PipelineError, never a panic.)
        let pip = match cgap::pipeline::EnclavePipeline::new(&att) {
            Ok(p) => p,
            Err(_) => halt_with(0xDEAD_0001),
        };

        // --- gate (A04/A19): penstock ingress probe, 128 B header -----
        // Byte pins mirror cgap_core::frame (A04): FRAME_MAGIC b"CGAP",
        // FRAME_VERSION 1, FRAME_HEADER_LEN 128. The core crate is
        // host-only (not no_std-clean), so the none-target image carries
        // the pinned literals; a core-side pin change trips the p28/gate
        // intrinsic laws here and at cgap_core::frame independently.
        #[cfg(not(feature = "host"))]
        let mut frame = {
            let mut f = [0u8; 128];
            f[0..4].copy_from_slice(b"CGAP");
            f[4] = 1u8;
            f
        };
        #[cfg(feature = "host")]
        let mut frame = {
            let mut f = [0u8; cgap_core::frame::FRAME_HEADER_LEN];
            f[0..4].copy_from_slice(&cgap_core::frame::FRAME_MAGIC);
            f[4] = cgap_core::frame::FRAME_VERSION;
            f
        };
        let status = pip
            .stage(cgap::pipeline::PipelineStage::Ingress)
            .ingress_status(black_box(&frame), frame.len());
        egress ^= status;

        // --- blake3: real digest over the frame -----------------------
        let digest = (cgap_blake3_hash())(&frame);
        egress ^= digest[0] as u32;

        // --- arrow: presence mask const surface -----------------------
        let m = cgap_arrow::bitmask::PresenceMask::<256, 32>::ALL_VALID;
        egress ^= cgap_arrow::bitmask::PresenceMask::<256, 32>::MASK_BYTES as u32;

        // --- bn3: rule compiler handle (unit struct, Copy) ------------
        let rc = pip.rule_compiler();
        egress ^= core::ptr::addr_of!(*rc) as u32;

        // --- storage: DRed cascade over caller-leased scratch (REP4) --
        // HOST-ONLY stage (SP1 feature decision): cgap-storage is not
        // no_std-clean; on the none target this stage compiles OUT and the
        // egress carries the typed host-only marker bit instead.
        #[cfg(feature = "host")]
        {
            let derived_deps = [0u64; 4];
            let del_mask = [0b0001u64];
            let alt_support = [0u32; 32];
            let counts = [0u32; 1];
            let adj = [0u32; 1];
            let mut purged = [0u64; 4];
            let mut keep = [0u64; 4];
            let mut worklist = [0u32; 32];
            let mut out = [0u32; 0];
            let cascaded = cgap_storage::dred::cascade_retract(
                &derived_deps,
                &del_mask,
                &alt_support,
                32,
                &counts,
                &adj,
                &mut purged,
                &mut keep,
                &mut worklist,
                0,
                &mut out,
            );
            egress ^= match cascaded {
                Ok(n) => n,
                Err(_) => 0x8000_0000,
            };
        }
        #[cfg(not(feature = "host"))]
        {
            egress ^= 0x8000_0000; // HOST-ONLY cascade stage absent (typed marker)
        }

        // --- parquet: fnv1a64 block digest over the frame (no_std) ----
        egress ^= cgap_parquet::footer::fnv1a64(&frame) as u32;

        // --- git: crc32 + sha1 over the frame (no_std surface) --------
        egress ^= cgap_git::pack_write::crc32(&frame) as u32;
        egress ^= u32::from_le_bytes([
            cgap_git::pack_write::sha1(&frame)[0],
            cgap_git::pack_write::sha1(&frame)[1],
            cgap_git::pack_write::sha1(&frame)[2],
            cgap_git::pack_write::sha1(&frame)[3],
        ]);

        EGRESS.store(egress, Ordering::SeqCst);
        halt_with(EGRESS.load(Ordering::SeqCst));
    }

    fn halt_with(_code: u32) -> ! {
        // Egress word stays visible in EGRESS for a debugger / GDB stub.
        loop {
            unsafe {
                read_egress();
                asm!("wfi")
            }
        }
    }

    fn read_egress() -> u32 {
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!(EGRESS).cast::<u32>()) }
    }

    fn cgap_p28_root() -> cgap_p28::attestation::P28LicenseAttestation {
        cgap_p28::attestation::P28LicenseAttestation {
            license_magic: cgap_p28::attestation::LICENSE_MAGIC,
            version: cgap_p28::attestation::ATTESTATION_VERSION,
            attestation_flags: cgap_p28::attestation::FLAG_ROOT,
            safe_harbor_ceiling_cents: cgap_p28::attestation::SAFE_HARBOR_CEILING_MAX_CENTS,
            origin_authority_digest: [0u8; 32],
            upstream_receipt: [0u8; 32],
        }
    }

    #[inline(never)]
    fn cgap_blake3_hash() -> fn(&[u8]) -> [u8; 32] {
        cgap_blake3::hash
    }
}
