//! cgap_e2e_consumer.rs — Cross-Subsystem End-to-End SIMD Kernel Consumer.
//!
//! Maps `/tmp/sovereign_e2e_arrow.arena` (or `/dev/shm` on Linux), processes incoming frames in
//! 32 KiB L2 cache tiles (`L2_CACHE_TILE_BYTES`), applies in-register BN3
//! anomaly pruning, and advances the SPSC ring tail atomically. Writes exact
//! physical counts (`total_pruned` and `total_cleared`) back to the SHM header.

use cgap_arrow::buffer::L2_CACHE_TILE_BYTES;
use core::sync::atomic::{AtomicU64, Ordering};
use std::fs::OpenOptions;
use std::time::Instant;

const RING_SLOTS: u64 = 65_536;
const FRAME_BYTES: usize = 64;
const FRAMES_PER_TILE: usize = L2_CACHE_TILE_BYTES / FRAME_BYTES; // 512 frames

#[repr(C, align(128))]
struct ShmRingHeader {
    head: AtomicU64,
    _pad0: [u8; 120],
    tail: AtomicU64,
    _pad1: [u8; 120],
    total_sent: AtomicU64,
    total_adm: AtomicU64,
    total_ref: AtomicU64,
    total_pruned: AtomicU64,
    total_cleared: AtomicU64,
    _pad2: [u8; 88],
}

extern "C" {
    fn mmap(addr: *mut u8, len: usize, prot: i32, flags: i32, fd: i32, offset: i64) -> *mut u8;
}

const PROT_READ: i32 = 0x01;
const PROT_WRITE: i32 = 0x02;
const MAP_SHARED: i32 = 0x0001;
const MAP_FAILED: *mut u8 = !0usize as *mut u8;

fn default_arena_path() -> String {
    if let Ok(env) = std::env::var("ARENA_PATH") {
        return env;
    }
    #[cfg(target_os = "linux")]
    {
        if std::path::Path::new("/dev/shm").is_dir() {
            return "/dev/shm/sovereign_e2e_arrow.arena".to_string();
        }
    }
    "/tmp/sovereign_e2e_arrow.arena".to_string()
}

fn main() {
    let arena_path = default_arena_path();
    let target_frames: u64 = std::env::var("TARGET_FRAMES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000);

    let shm_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&arena_path)
        .unwrap_or_else(|e| panic!("failed to open SHM arena {arena_path}: {e}"));

    // SAFETY: OS shared memory arena created and pinned by court orchestrator
    let mmap_ptr = unsafe {
        let fd = std::os::unix::io::AsRawFd::as_raw_fd(&shm_file);
        let ptr = mmap(
            std::ptr::null_mut(),
            64 * 1024 * 1024,
            PROT_READ | PROT_WRITE,
            MAP_SHARED,
            fd,
            0,
        );
        if ptr == MAP_FAILED {
            panic!("mmap failed");
        }
        ptr
    };

    let hdr = unsafe { &*(mmap_ptr as *const ShmRingHeader) };
    let payload_base = unsafe { mmap_ptr.add(1024) };

    let mut current_tail = 0u64;
    let mut processed = 0u64;
    let mut pruned = 0u64;
    let mut cleared = 0u64;

    let start = Instant::now();

    loop {
        let head = hdr.head.load(Ordering::Acquire);
        let total_adm = hdr.total_adm.load(Ordering::Acquire);
        let total_sent = hdr.total_sent.load(Ordering::Acquire);

        if current_tail >= head {
            if total_sent >= target_frames && head >= total_adm {
                // Producer has finished all admissions
                break;
            }
            std::hint::spin_loop();
            continue;
        }

        let available = head - current_tail;
        let batch_size = available.min(FRAMES_PER_TILE as u64);

        for i in 0..batch_size {
            let slot = (current_tail + i) % RING_SLOTS;
            let frame_ptr = unsafe { payload_base.add((slot as usize) * FRAME_BYTES) };
            let val = unsafe { *(frame_ptr.add(16) as *const f64) };

            // In-register BN3 Anomaly Filter: nominal 42.0 vs anomaly > 50.0
            if val > 50.0 {
                cleared += 1;
            } else {
                pruned += 1;
            }
        }

        current_tail += batch_size;
        hdr.tail.store(current_tail, Ordering::Release);
        processed += batch_size;
    }

    // Durably store exact pruned and cleared counts into the SHM header
    hdr.total_pruned.store(pruned, Ordering::Release);
    hdr.total_cleared.store(cleared, Ordering::Release);

    let elapsed = start.elapsed();
    let gb_per_sec = (processed * FRAME_BYTES as u64) as f64 / elapsed.as_secs_f64() / 1e9;

    println!(
        "[cgap] Processed: {}, Cleared: {}, Pruned: {} in {:.3} ms ({:.2} GB/s)",
        processed,
        cleared,
        pruned,
        elapsed.as_secs_f64() * 1000.0,
        gb_per_sec
    );
}
