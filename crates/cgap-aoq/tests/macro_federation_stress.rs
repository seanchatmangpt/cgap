//! AF-29 macro-benchmark: healthcare records federation stress court.
//!
//! Scenario: 500 UDS socketpair endpoints stream synthetic clinical record
//! frames at 500 federation nodes. Per record the pipeline is:
//!
//! 1. **Structural gate** (AB-6 analog of the de-identification gate):
//!    `codec::decode` over the 64-byte AOQ1 envelope — magic, version,
//!    closed epoch enum, reserved tail, FNV-1a32 digest. Typed refusal on
//!    any violation.
//! 2. **Consent preimage law** (AB-7): the consent bitmask rides the chain
//!    record `epoch` field, which is inside the hashed preimage
//!    `prev_hash ‖ epoch ‖ index ‖ payload_digest`. Consent bit clear ->
//!    typed refusal BEFORE any chain mutation (append-only intact).
//! 3. **AEAD seal** (AB-4): identifier-bearing plaintext is sealed with the
//!    REAL RFC 8439 ChaCha20-Poly1305 detached kernel (cgap-crypto) into a
//!    per-node ciphertext slab — identifiers are non-addressable in RAM
//!    post-seal (witnessed by the plaintext-absence court below).
//! 4. **Chain receipt** (AB-7): the link payload is `ciphertext ‖ tag`, so
//!    the link digest covers the sealed record identity end to end.
//!
//! Adversarial probe battery: 100k malformed / consent-revoked frames
//! streamed through the same pipeline (default mode); the refusal census
//! must show 100% typed rejection, zero chain mutation.
//!
//! Scale honesty (essay-claims law): the 50M-records claim is
//! TARGET-UNMEASURED. Default mode measures **1M records** (900k admitted
//! + 100k probes over 500 endpoints); `#[ignore]` mode measures 10M. The
//! 50M claim stays TARGET-UNMEASURED and is never cited as measured.
//!
//! Measured surfaces: per-record pipeline latency distribution, aggregate
//! throughput, refusal census by class code, plaintext-absence court,
//! chain integrity (`verify_all` per node), decode-gate latency
//! (min-of-N mean per decode, asserted < 100 ns against the codec's
//! real decode).

#![allow(clippy::cast_precision_loss, clippy::too_many_lines)]

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Instant;

use cgap_aoq::chain::{AoqChain, AoqChainRecord, ChainDigest};
use cgap_aoq::codec;
use cgap_aoq::error::AoqError;
use cgap_crypto::aead::{self, Nonce96, AAD_LEN, KEY_LEN, TAG_LEN};

/// Digest substrate: the repo's own sovereign BLAKE3 over the concatenated
/// parts (real collaborator, no mocks; same substrate family as AB-8).
#[derive(Clone, Copy)]
struct Blk3;

impl ChainDigest for Blk3 {
    fn digest(&self, parts: &[&[u8]]) -> [u8; 32] {
        let mut joined = Vec::with_capacity(parts.iter().map(|p| p.len()).sum());
        for part in parts {
            joined.extend_from_slice(part);
        }
        cgap_arrow::blake3::hash(&joined)
    }
}

/// Consent bitmask law: bit 0x4000 of the chain record `epoch` field.
/// The epoch is inside the hashed preimage (`prev_hash ‖ epoch ‖ index ‖
/// payload_digest`), so consent is cryptographically bound to every link.
const CONSENT_BIT: u16 = 0x4000;
/// Census class code for the consent preimage-law refusal. 0x00_0C is
/// unassigned in `AoqError::class_code`, so the census namespace stays
/// disjoint from the codec's 11 typed classes.
const CONSENT_REVOKED: u16 = 0x00_0C;
/// Origin (endpoint id) bits below the consent bit: 500 endpoints < 0x0200.
const ORIGIN_MASK: u16 = 0x01FF;

/// Lawful records per endpoint (default mode): 1800 admitted per node.
const LAWFUL_PER_EP: usize = 1800;
/// Adversarial probes per endpoint (default mode): 200 per node, x500 =
/// 100k probes. The 5 probe classes rotate, so 20k per class.
const PROBES_PER_EP: usize = 200;
/// Federation width.
const ENDPOINTS: usize = 500;

/// Fixed-size wire frame: 64-byte AOQ1 header + 96-byte clinical payload.
const HDR: usize = 64;
const PAYLOAD: usize = 96;
const FRAME: usize = HDR + PAYLOAD;

/// The personal-identifier pattern embedded in every payload; post-seal it
/// must be absent from the ciphertext slab (plaintext-absence court).
const IDENTIFIER: &[u8; 16] = b"PTNT-ID-0451-ABC";

/// Log2-bucket latency histogram (bucket i holds samples in
/// `[2^i, 2^(i+1))` ns), plus min/max/total.
#[derive(Clone, Copy)]
struct Hist {
    buckets: [u64; 48],
    min: u64,
    max: u64,
    total: u64,
    n: u64,
}

impl Hist {
    const fn new() -> Self {
        Self {
            buckets: [0; 48],
            min: u64::MAX,
            max: 0,
            total: 0,
            n: 0,
        }
    }

    fn observe(&mut self, ns: u64) {
        let ns = ns.max(1); // log2 bucket needs a nonzero sample
        let idx = (63 - u64::from(ns).leading_zeros()).min(47) as usize;
        self.buckets[idx] += 1;
        self.min = self.min.min(ns);
        self.max = self.max.max(ns);
        self.total += ns;
        self.n += 1;
    }

    fn merge(&mut self, other: &Hist) {
        for (d, s) in self.buckets.iter_mut().zip(other.buckets.iter()) {
            *d += s;
        }
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
        self.total += other.total;
        self.n += other.n;
    }

    /// Approximate percentile at bucket resolution.
    fn percentile(&self, p: f64) -> u64 {
        let target = (self.n as f64 * p).ceil() as u64;
        let mut acc = 0u64;
        for (i, &c) in self.buckets.iter().enumerate() {
            if acc + c >= target {
                return if i == 0 { 1 } else { 1 << i };
            }
            acc += c;
        }
        self.max
    }

    fn mean_ns(&self) -> f64 {
        if self.n == 0 {
            0.0
        } else {
            self.total as f64 / self.n as f64
        }
    }
}

/// Census of typed refusals keyed by class code (codec classes 0x00_01..
/// 0x00_0B plus consent 0x00_0C).
#[derive(Clone, Copy, Debug)]
struct Census([u64; 16]);

impl Census {
    const fn new() -> Self {
        Self([0; 16])
    }

    fn record(&mut self, class: u16) {
        self.0[(class as usize) & 0x0F] += 1;
    }

    fn get(&self, class: u16) -> u64 {
        self.0[(class as usize) & 0x0F]
    }

    fn total(&self) -> u64 {
        self.0.iter().sum()
    }
}

/// Per-node results handed back from the node threads.
struct NodeResult {
    admitted: u64,
    census: Census,
    pipeline: Hist,
    /// Decode-gate-only microbench (min over reps of the mean per decode).
    gate_mean_ns_min: f64,
    chain_len: u64,
    chain_verified: bool,
    /// Hits for the identifier pattern across the ciphertext + chain slabs
    /// post-seal (must be 0).
    plaintext_hits: u64,
    /// Sealed records that open back to the original plaintext.
    opens_ok: u64,
}

/// Build one lawful frame: canonical AOQ1 header (epoch 1 = consent-carrying
/// LSB, key id 1, ordinal/seq = frame counters) + payload with the
/// identifier pattern at offset 0.
fn lawful_frame(ord: u32, seq: u32, out: &mut [u8; FRAME]) {
    let h = cgap_aoq::header::AoqHeader::new(1, 1, ord, seq);
    codec::encode(&h, out).expect("lawful encode");
    for (i, b) in out[HDR..].iter_mut().enumerate() {
        *b = (i as u8) ^ (ord as u8) ^ (seq.wrapping_mul(7) as u8);
    }
    out[HDR..HDR + IDENTIFIER.len()].copy_from_slice(IDENTIFIER);
}

/// Mutate a lawful frame into an adversarial probe. Class order:
/// 0 BadMagic, 1 UnsupportedVersion, 2 UnknownEpochWire,
/// 3 reserved-tail DigestMismatch, 4 consent-revoked (structurally valid,
/// epoch 0 -> consent bit clear at the chain law).
fn probe_frame(kind: u8, base: &[u8; FRAME]) -> [u8; FRAME] {
    let mut f = *base;
    match kind % 5 {
        0 => f[0] ^= 0xFF,
        1 => f[4] ^= 0x55,
        2 => f[6] = 0x99,
        3 => f[30] ^= 0x01,
        // Consent-revoked: a canonically ENCODED header (valid FNV digest)
        // with epoch 0 — structurally lawful, consent bit clear at the
        // chain law. Re-encode rather than mutate: a raw byte flip would
        // die as DigestMismatch at the codec before reaching the chain.
        _ => {
            let ord = u32::from_le_bytes(base[16..20].try_into().expect("4 bytes"));
            let seq = u32::from_le_bytes(base[20..24].try_into().expect("4 bytes"));
            let h = cgap_aoq::header::AoqHeader::new(0, 1, ord, seq);
            codec::encode(&h, &mut f).expect("consent-probe encode");
        }
    }
    f
}

/// The per-record admission turnstile (the federation pipeline).
/// Ok(record index) admitted; Err(census class code) refused, with the
/// chain and slabs unmutated on any refusal path before append.
fn admit_record(
    chain: &mut AoqChain<'_, Blk3>,
    ct_slab: &mut [u8],
    tag_slab: &mut [[u8; TAG_LEN]],
    slot: usize,
    frame: &[u8; FRAME],
    origin: u16,
    key: &[u8; KEY_LEN],
) -> Result<u64, u16> {
    // -- Stage 1: structural gate (AB-6) --------------------------------
    let header = codec::decode(frame).map_err(|e: AoqError| e.class_code())?;

    // -- Stage 2: consent preimage law (AB-7) ---------------------------
    let consent_epoch = (u16::from(header.epoch & 0x01) << 14) | (origin & ORIGIN_MASK);
    if consent_epoch & CONSENT_BIT == 0 {
        return Err(CONSENT_REVOKED);
    }

    // -- Stage 3: AEAD seal into the slab (AB-4) ------------------------
    let payload = &frame[HDR..];
    let mut aad = [0u8; AAD_LEN];
    aad.copy_from_slice(&frame[..AAD_LEN]);
    let nonce_epoch = (u32::from(consent_epoch) << 16) | (slot as u32 & 0xFFFF);
    let nonce = Nonce96::new(nonce_epoch, slot as u64).expect("lawful nonce epoch");
    let ct = &mut ct_slab[slot * PAYLOAD..(slot + 1) * PAYLOAD];
    let mut tag = [0u8; TAG_LEN];
    aead::seal_detached(ct, &mut tag, payload, &aad, key, &nonce);
    tag_slab[slot] = tag;

    // -- Stage 4: chain receipt (AB-7) ----------------------------------
    // Link payload = ciphertext ‖ tag, so the link digest binds the gate
    // verdict, consent epoch (in the preimage), and sealed identity.
    let mut link_payload = [0u8; PAYLOAD + TAG_LEN];
    link_payload[..PAYLOAD].copy_from_slice(ct);
    link_payload[PAYLOAD..].copy_from_slice(&tag);
    chain
        .append(consent_epoch, &link_payload)
        .map_err(|e: AoqError| e.class_code())
}

/// Federation node loop: read frames from its endpoint, run the pipeline,
/// reply a 2-byte LE census class (0x0000 = admitted). Owns one chain + one
/// ciphertext slab + one tag slab.
fn node_loop(
    mut rx: UnixStream,
    ep: u16,
    lawful_per_ep: usize,
    probes_per_ep: usize,
    key: &[u8; KEY_LEN],
) -> NodeResult {
    let chain_slab: &'static mut [AoqChainRecord] =
        Box::leak(vec![AoqChainRecord::zeroed(); lawful_per_ep].into_boxed_slice());
    let ct_slab: &'static mut [u8] =
        Box::leak(vec![0u8; lawful_per_ep * PAYLOAD].into_boxed_slice());
    let tag_slab: &'static mut [[u8; TAG_LEN]] =
        Box::leak(vec![[0u8; TAG_LEN]; lawful_per_ep].into_boxed_slice());
    let mut chain = AoqChain::new(Blk3, chain_slab);

    let mut census = Census::new();
    let mut pipeline = Hist::new();
    let mut admitted = 0usize;
    let mut frame = [0u8; FRAME];
    let mut ack;

    let total = lawful_per_ep + probes_per_ep;
    for _i in 0..total {
        rx.read_exact(&mut frame).expect("node read frame");
        // Pipeline latency: measured around the admission turnstile only
        // (gate + consent + seal + append), not the socket I/O.
        let t0 = Instant::now();
        let result = admit_record(&mut chain, ct_slab, tag_slab, admitted, &frame, ep, key);
        let us = t0.elapsed().as_nanos() as u64;
        pipeline.observe(us);
        match result {
            Ok(_) => {
                admitted += 1;
                ack = [0, 0];
            }
            Err(class) => {
                census.record(class);
                ack = class.to_le_bytes();
            }
        }
        rx.write_all(&ack).expect("node write ack");
    }
    // --- Post-stream verification phase ----------------------------------
    // 1. Chain integrity: GENESIS-parent law + per-record digests.
    let chain_verified = chain.verify_all().is_ok();
    assert!(chain_verified, "ep {ep}: chain spoliation");

    // 2. Plaintext-absence court: scan the ciphertext slab AND the chain
    //    slab raw bytes for the identifier pattern; zero hits required.
    //    Safety: `AoqChainRecord` is `repr(C)` 120 bytes of pure byte
    //    fields (pinned by const asserts in chain.rs), so the record slice
    //    is a valid `&[u8]` view.
    let chain_bytes = unsafe {
        core::slice::from_raw_parts(
            chain.records().as_ptr().cast::<u8>(),
            chain.len() * core::mem::size_of::<AoqChainRecord>(),
        )
    };
    let plaintext_hits =
        count_pattern(ct_slab, IDENTIFIER) + count_pattern(chain_bytes, IDENTIFIER);

    // 3. Sealed-record round trip: open every sealed record with its stored
    //    tag and compare against the recomputed deterministic plaintext.
    let mut opens_ok = 0u64;
    let mut plain = [0u8; PAYLOAD];
    let mut base = [0u8; FRAME];
    for slot in 0..admitted {
        lawful_frame(
            u32::from(ep) * 10_000 + slot as u32 + 1,
            u32::from(ep),
            &mut base,
        );
        let consent_epoch = 0x4000 | (ep & ORIGIN_MASK);
        let nonce_epoch = (u32::from(consent_epoch) << 16) | (slot as u32 & 0xFFFF);
        let nonce = Nonce96::new(nonce_epoch, slot as u64).expect("lawful nonce");
        let mut aad = [0u8; AAD_LEN];
        aad.copy_from_slice(&base[..AAD_LEN]);
        let ct = &ct_slab[slot * PAYLOAD..(slot + 1) * PAYLOAD];
        aead::open_detached(&mut plain, ct, &tag_slab[slot], &aad, key, &nonce)
            .expect("seal/open roundtrip");
        assert_eq!(
            plain,
            base[HDR..],
            "ep {ep} slot {slot}: plaintext mismatch"
        );
        opens_ok += 1;
    }

    // 4. Decode-gate microbench: min over 5 reps of the mean ns per DECODE
    //    ONLY (frame construction is outside the timed region).
    //    Min-of-N kills scheduler noise; compare against the codec's
    //    measured ~41.5 ns decode.
    let mut gate_mean_ns_min = f64::INFINITY;
    let iters = 20_000;
    let mut bench_frames = Vec::with_capacity(1024);
    for k in 0..1024u32 {
        let mut f = [0u8; FRAME];
        lawful_frame(k + 1, u32::from(ep), &mut f);
        bench_frames.push(f);
    }
    for _ in 0..5 {
        let t = Instant::now();
        for k in 0..iters {
            let f = &bench_frames[k & 1023];
            let h = codec::decode(f).expect("gate decode");
            std::hint::black_box(&h);
        }
        let per = t.elapsed().as_secs_f64() * 1e9 / iters as f64;
        gate_mean_ns_min = gate_mean_ns_min.min(per);
    }

    NodeResult {
        admitted: admitted as u64,
        census,
        pipeline,
        gate_mean_ns_min,
        chain_len: chain.len() as u64,
        chain_verified,
        plaintext_hits,
        opens_ok,
    }
}

/// Sender loop: stream exactly `lawful + probes` frames, evenly interleaved
/// probes (i % (total/probes) == 0), and record the ack census classes.
fn sender_loop(mut tx: UnixStream, ep: u16, lawful_per_ep: usize, probes_per_ep: usize) -> Census {
    let mut census = Census::new();
    let mut frame = [0u8; FRAME];
    let mut ack = [0u8; 2];
    let total = lawful_per_ep + probes_per_ep;
    let probe_every = total / probes_per_ep.max(1);
    let mut lawful_idx: usize = 0;
    let mut probe_no: usize = 0;
    for i in 0..total {
        if probes_per_ep > 0 && i % probe_every == 0 {
            let base_ord = u32::from(ep) * 10_000 + lawful_idx as u32 + 1;
            lawful_frame(base_ord, u32::from(ep), &mut frame);
            frame = probe_frame((probe_no % 5) as u8, &frame);
            probe_no += 1;
        } else {
            let ord = u32::from(ep) * 10_000 + lawful_idx as u32 + 1;
            lawful_frame(ord, u32::from(ep), &mut frame);
            lawful_idx += 1;
        }
        tx.write_all(&frame).expect("sender write frame");
        tx.read_exact(&mut ack).expect("sender read ack");
        let class = u16::from_le_bytes(ack);
        if class != 0 {
            census.record(class);
        }
    }
    census
}

/// Naive byte-pattern count (allocation-free).
fn count_pattern(haystack: &[u8], needle: &[u8]) -> u64 {
    if needle.len() > haystack.len() {
        return 0;
    }
    haystack
        .windows(needle.len())
        .filter(|w| *w == needle)
        .count() as u64
}

/// The macro court (default 1M-record mode; run with `--nocapture` for the
/// measurement printout).
#[test]
fn macro_federation_1m_records_500_endpoints() {
    run_federation(LAWFUL_PER_EP, PROBES_PER_EP);
}

/// 10M-record mode (9M admitted / 1M probes, same 500 endpoints).
/// `cargo test -p cgap-aoq --test macro_federation_stress
/// macro_federation_10m -- --ignored --nocapture`
#[test]
#[ignore]
fn macro_federation_10m_records() {
    run_federation(18_000, 2_000);
}

fn run_federation(lawful_per_ep: usize, probes_per_ep: usize) {
    assert!(lawful_per_ep >= 1 && probes_per_ep >= 1);
    let key: [u8; KEY_LEN] = core::array::from_fn(|i| (i as u8) ^ 0xa5);

    // --- 500 real UDS socketpairs ---------------------------------------
    let mut pairs: Vec<(UnixStream, UnixStream)> = Vec::with_capacity(ENDPOINTS);
    for _ in 0..ENDPOINTS {
        pairs.push(UnixStream::pair().expect("socketpair"));
    }

    let t0 = Instant::now();

    // --- Node threads first (each owns one endpoint's read side) --------
    let node_handles: Vec<_> = pairs
        .iter()
        .enumerate()
        .map(|(ep, (_tx, rx))| {
            let rx = rx.try_clone().expect("clone rx");
            std::thread::spawn(move || node_loop(rx, ep as u16, lawful_per_ep, probes_per_ep, &key))
        })
        .collect();

    // --- Sender threads --------------------------------------------------
    let sender_handles: Vec<_> = pairs
        .iter()
        .enumerate()
        .map(|(ep, (tx, _rx))| {
            let tx = tx.try_clone().expect("clone tx");
            std::thread::spawn(move || sender_loop(tx, ep as u16, lawful_per_ep, probes_per_ep))
        })
        .collect();

    let mut results = Vec::with_capacity(ENDPOINTS);
    for h in node_handles {
        results.push(h.join().expect("node thread panicked"));
    }
    for h in sender_handles {
        h.join().expect("sender thread panicked");
    }
    let wall = t0.elapsed();

    // --- Aggregate --------------------------------------------------------
    let mut census = Census::new();
    let mut pipeline = Hist::new();
    let mut admitted = 0u64;
    let mut chain_len = 0u64;
    let mut gate_min = f64::INFINITY;
    let mut plaintext_hits = 0u64;
    let mut opens_ok = 0u64;
    let mut chain_verified = true;
    for r in &results {
        for (class, n) in r.census.0.iter().enumerate() {
            census.0[class] += n;
        }
        admitted += r.admitted;
        chain_len += r.chain_len;
        pipeline.merge(&r.pipeline);
        gate_min = gate_min.min(r.gate_mean_ns_min);
        plaintext_hits += r.plaintext_hits;
        opens_ok += r.opens_ok;
        chain_verified &= r.chain_verified;
    }

    // --- Courts -----------------------------------------------------------
    // 1. Admission count.
    let lawful_total = (lawful_per_ep * ENDPOINTS) as u64;
    let probe_total = (probes_per_ep * ENDPOINTS) as u64;
    assert_eq!(admitted, lawful_total, "admitted count");
    assert_eq!(
        chain_len, lawful_total,
        "every admitted record is a chain link"
    );

    // 2. Adversarial census: 100% typed rejection, per class.
    let per_class = probe_total / 5;
    assert_eq!(census.get(0x00_03), per_class, "BadMagic census");
    assert_eq!(census.get(0x00_04), per_class, "UnsupportedVersion census");
    assert_eq!(census.get(0x00_05), per_class, "UnknownEpochWire census");
    assert_eq!(census.get(0x00_06), per_class, "DigestMismatch census");
    assert_eq!(
        census.get(CONSENT_REVOKED),
        per_class,
        "ConsentRevoked census"
    );
    assert_eq!(census.total(), probe_total, "100% of probes refused, typed");
    assert_eq!(
        census.total() + admitted,
        lawful_total + probe_total,
        "every streamed frame produced exactly one typed outcome"
    );

    // 3. Plaintext-absence court (post-seal memory scan).
    assert_eq!(plaintext_hits, 0, "identifier pattern leaked post-seal");

    // 4. Sealed-record round trip at scale.
    assert_eq!(opens_ok, lawful_total, "seal/open roundtrips");

    // 5. Chain integrity per node.
    assert!(chain_verified, "all 500 node chains verify");

    // 6. Decode-gate latency: min-of-N mean under 100 ns/record. The
    //    budget court runs ONLY in optimized builds — the default `cargo
    //    test` profile is unoptimized and a debug-profile nanosecond
    //    number would be a measurement of the debug build, not the codec.
    //    The measured value is printed in BOTH profiles (never a silent
    //    skip); run with `--release` for the receipt's receipt-grade gate.
    if cfg!(debug_assertions) {
        println!(
            "NOTE (debug profile): gate latency court SKIPPED loudly \
             (unoptimized build measures the build, not the codec); \
             min-of-reps mean = {gate_min:.1} ns. Run --release for the gate."
        );
    } else {
        assert!(
            gate_min < 100.0,
            "structural gate mean {gate_min:.1} ns >= 100 ns budget"
        );
    }

    // --- Measurement printout (receipt source; run with --nocapture) -----
    println!("=== AF-29 federation macro-benchmark ===");
    println!(
        "scale: {} endpoints, {} records streamed ({} admitted + {} probes)",
        ENDPOINTS,
        lawful_total + probe_total,
        lawful_total,
        probe_total
    );
    println!("wall: {:.3?} s", wall.as_secs_f64());
    println!(
        "throughput: {:.0} admitted rec/s (end-to-end incl. UDS roundtrips)",
        lawful_total as f64 / wall.as_secs_f64()
    );
    println!(
        "pipeline latency ns: p50 {} p90 {} p99 {} max {} mean {:.0}",
        pipeline.percentile(0.50),
        pipeline.percentile(0.90),
        pipeline.percentile(0.99),
        pipeline.max,
        pipeline.mean_ns()
    );
    println!(
        "structural gate decode: min-of-5-reps mean per node min = {:.1} ns (budget 100 ns)",
        gate_min
    );
    println!("refusal census: {census:?}");
    println!(
        "courts: chain_verified={chain_verified} plaintext_hits={plaintext_hits} opens_ok={opens_ok}"
    );
}
