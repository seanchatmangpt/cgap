//! ML-DSA-65 phase-5 alloc census (A3 census pattern): counting global
//! allocator proves 0 allocations across keygen / sign / verify.
//! Separate test binary so the process-wide counter is not raced by
//! sibling tests. Non-vacuity: a probe Vec must move the counter.
use cgap_crypto::mldsa as m;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOCS: AtomicUsize = AtomicUsize::new(0);

struct Counting;
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::SeqCst);
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        System.dealloc(p, l)
    }
}

#[global_allocator]
static A: Counting = Counting;

fn splitmix_bytes(state: &mut u64, out: &mut [u8]) {
    for b in out.iter_mut() {
        *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_4A0D_A71B_D881);
        *b = (z ^ (z >> 31)) as u8;
    }
}

#[test]
fn alloc_census_keygen_sign_verify() {
    let probe = Vec::<u8>::with_capacity(64);
    let before_probe = ALLOCS.load(Ordering::SeqCst);
    assert!(before_probe > 0, "counter never fired (vacuous census)");
    drop(probe);

    let mut sm = 0x5EED_0065u64;
    let mut xi = [0u8; 32];
    splitmix_bytes(&mut sm, &mut xi);
    let mut ek = [0u8; m::PK_BYTES];
    let mut dk = [0u8; m::SK_BYTES];
    ALLOCS.store(0, Ordering::SeqCst);
    m::keygen(&xi, &mut ek, &mut dk);
    assert_eq!(ALLOCS.load(Ordering::SeqCst), 0, "keygen allocated");
    let tr: [u8; 64] = dk[64..128].try_into().unwrap();
    let mu = m::compute_mu(&tr, b"census");
    let rnd = [0x5Au8; 32];
    m::sign(&dk, &mu, &rnd).expect("sign");
    assert_eq!(ALLOCS.load(Ordering::SeqCst), 0, "sign allocated");
    assert!(m::verify(&ek, &mu, &m::sign(&dk, &mu, &rnd).unwrap()));
    assert_eq!(ALLOCS.load(Ordering::SeqCst), 0, "verify allocated");
}
