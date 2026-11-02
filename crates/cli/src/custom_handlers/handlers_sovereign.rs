//! CLI-2 handlers: sovereign surfaces (gate, p28, crypto, gpu, beam, court).

use clap_noun_verb::{NounVerbError, Result};
use serde_json::{json, Map, Value};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn fail<T>(what: &str, detail: String) -> Result<T> {
    Err(NounVerbError::execution_error(format!("{what}: {detail}")))
}

/// Shared: build a valid 128B-header CGAP frame (A04 layout via cgap-gate's
/// public constants) carrying a 32-byte payload, then run it through the
/// real penstock ingress twice — admit + one corruption refusal.
fn ingest_valid_frame() -> Result<(usize, u32, u32)> {
    use cgap_gate::penstock::{
        ingress, MAGIC, OFF_ATTESTATION, OFF_FLAGS, OFF_MAGIC, OFF_OABI_DIGEST,
        OFF_PAYLOAD_BYTE_LEN, OFF_VERSION, P28_FLAG_ROOT, P28_LICENSE_MAGIC,
        P28_SAFE_HARBOR_CEILING_CENTS, P28_VERSION, CGAP_OK,
    };
    let payload_len = 32usize;
    let mut frame = vec![0u8; 128 + payload_len];
    frame[OFF_MAGIC..OFF_MAGIC + 4].copy_from_slice(&MAGIC);
    frame[OFF_VERSION] = 1;
    frame[OFF_FLAGS] = 0;
    frame[OFF_PAYLOAD_BYTE_LEN..OFF_PAYLOAD_BYTE_LEN + 8]
        .copy_from_slice(&(payload_len as u64).to_le_bytes());
    let digest = cgap_blake3::hash(&frame[128..]);
    frame[OFF_OABI_DIGEST..OFF_OABI_DIGEST + 32].copy_from_slice(&digest);
    let a = OFF_ATTESTATION;
    frame[a..a + 4].copy_from_slice(&P28_LICENSE_MAGIC);
    frame[a + 4..a + 6].copy_from_slice(&P28_VERSION.to_le_bytes());
    frame[a + 6..a + 8].copy_from_slice(&P28_FLAG_ROOT.to_le_bytes());
    frame[a + 8..a + 16].copy_from_slice(&P28_SAFE_HARBOR_CEILING_CENTS.to_le_bytes());
    frame[a + 16..a + 48].copy_from_slice(&digest); // nonzero origin authority
    let status = ingress(&frame, frame.len(), None);
    if status != CGAP_OK {
        return fail("gate dam-ingest", format!("valid frame refused: {status:#x}"));
    }
    // Anti-vacuity: break the magic; ingress must refuse typed.
    let mut bad = frame.clone();
    bad[0] = b'X';
    let bad_status = ingress(&bad, bad.len(), None);
    if bad_status == CGAP_OK {
        return fail("gate dam-ingest", "corrupt frame admitted — vacuous".to_owned());
    }
    Ok((frame.len(), status, bad_status))
}

/// `gate dam-ingest` — real penstock ingress of a valid DAM evidence frame
/// plus a corruption-refusal witness (fail closed).
pub fn gate_dam_ingest(_inputs: Map<String, Value>) -> Result<Value> {
    let (frame_len, status, bad_status) = ingest_valid_frame()?;
    Ok(json!({
        "frame_len": frame_len,
        "status": format!("{status:#010x}"),
        "corrupt_status": format!("{bad_status:#010x}"),
        "corrupt_refused": bad_status != status,
    }))
}

/// `p28 attest-verify` — build a real 80-byte P28LicenseAttestation (Root
/// lineage), validate, round-trip the wire bytes, and refuse a
/// ceiling-overflow mutation (fail-closed witness).
pub fn p_28_attest_verify(_inputs: Map<String, Value>) -> Result<Value> {
    use cgap_p28::attestation::{
        P28LicenseAttestation, ATTESTATION_LEN, FLAG_LINEAGE_MASK, FLAG_RESERVED_MASK,
        FLAG_ROOT, SAFE_HARBOR_CEILING_MAX_CENTS,
    };
    let att = P28LicenseAttestation {
        license_magic: *b"P28S",
        version: 1,
        attestation_flags: FLAG_ROOT,
        safe_harbor_ceiling_cents: SAFE_HARBOR_CEILING_MAX_CENTS,
        origin_authority_digest: [7u8; 32],
        upstream_receipt: [0u8; 32],
    };
    att.validate_intrinsic()
        .map_err(|e| NounVerbError::execution_error(format!("valid attestation refused: {e:?}")))?;
    let bytes = att.to_bytes();
    if bytes.len() != ATTESTATION_LEN {
        return fail("p28 attest-verify", format!("wire len {}", bytes.len()));
    }
    let back = cgap_p28::attestation::P28LicenseAttestation::from_bytes(&bytes)
        .map_err(|e| NounVerbError::execution_error(format!("wire decode refused: {e:?}")))?;
    back.validate_intrinsic()
        .map_err(|e| NounVerbError::execution_error(format!("roundtrip refused: {e:?}")))?;
    if back != att {
        return fail("p28 attest-verify", "roundtrip field drift".to_owned());
    }
    if back.attestation_flags & FLAG_RESERVED_MASK != 0
        || back.attestation_flags & FLAG_LINEAGE_MASK != FLAG_ROOT
    {
        return fail("p28 attest-verify", "lineage law broken".to_owned());
    }
    // Anti-vacuity: ceiling above the $50M safe-harbor MUST refuse.
    let mut corrupt = att;
    corrupt.safe_harbor_ceiling_cents = SAFE_HARBOR_CEILING_MAX_CENTS + 1;
    if corrupt.validate_intrinsic().is_ok() {
        return fail("p28 attest-verify", "over-ceiling admitted — vacuous".to_owned());
    }
    Ok(json!({
        "wire_len": bytes.len(),
        "license_urn": cgap_gate::penstock::P28_LICENSE_URN,
        "origin_authority_digest": hex(&att.origin_authority_digest),
        "roundtrip_ok": true,
        "over_ceiling_refused": true,
    }))
}

/// Shared: mint a real Lease, seal one DeltaS under it, verify the sealed
/// receipt, and return (receipt bytes, tick after seal).
fn seal_one() -> Result<(Vec<u8>, u64)> {
    use cgap_crypto::chatman::{admissions_root, seal_unsealed, verify_unsealed, DeltaS};
    use cgap_crypto::lease::Lease;
    let mut lease = Lease::new([1u8; 32], 8, [2u8; 32]);
    let from = cgap_blake3::hash(b"cgap-cli:from");
    let to = cgap_blake3::hash(b"cgap-cli:to");
    let admissions = admissions_root(&[&from, &to]);
    let delta = DeltaS::new(from, to, 3, admissions);
    let receipt = seal_unsealed(delta, &mut lease);
    verify_unsealed(&receipt)
        .map_err(|e| NounVerbError::execution_error(format!("sealed receipt refused: {e:?}")))?;
    let tick = lease.current_tick();
    if tick != 1 {
        return fail("crypto seal", format!("tick {tick} != 1 after one seal"));
    }
    Ok((receipt.to_bytes().to_vec(), tick))
}

/// `crypto seal` — real keyed seal under a live Lease; round-trip the
/// receipt wire bytes and assert the tick advanced exactly once.
pub fn crypto_seal(_inputs: Map<String, Value>) -> Result<Value> {
    use cgap_crypto::chatman::UNSEALED_RECEIPT_LEN;
    let (bytes, tick) = seal_one()?;
    if bytes.len() != UNSEALED_RECEIPT_LEN {
        return fail("crypto seal", format!("wire len {} != {}", bytes.len(), UNSEALED_RECEIPT_LEN));
    }
    Ok(json!({
        "receipt_len": bytes.len(),
        "tick_after_seal": tick,
        "receipt_digest": hex(&cgap_blake3::hash(&bytes)),
    }))
}

/// `crypto receipt-verify` — verify the sealed receipt, then refuse a
/// single-bit tamper of the wire copy (fail-closed witness).
pub fn crypto_receipt_verify(_inputs: Map<String, Value>) -> Result<Value> {
    use cgap_crypto::chatman::{verify_unsealed, UnsealedReceipt};
    let (bytes, _tick) = seal_one()?;
    let arr: [u8; cgap_crypto::chatman::UNSEALED_RECEIPT_LEN] =
        bytes.clone().try_into().map_err(|v: Vec<u8>| {
            NounVerbError::execution_error(format!("len {}", v.len()))
        })?;
    let receipt = UnsealedReceipt::from_bytes(&arr)
        .map_err(|e| NounVerbError::execution_error(format!("decode refused: {e:?}")))?;
    verify_unsealed(&receipt)
        .map_err(|e| NounVerbError::execution_error(format!("verify refused: {e:?}")))?;
    // Anti-vacuity: flip one bit in the wire copy; decode MUST refuse.
    let mut tampered = arr;
    tampered[10] ^= 0x01;
    if UnsealedReceipt::from_bytes(&tampered).is_ok()
        && verify_unsealed(&UnsealedReceipt::from_bytes(&tampered).unwrap()).is_ok()
    {
        return fail("crypto receipt-verify", "tampered receipt verified — vacuous".to_owned());
    }
    Ok(json!({
        "receipt_len": arr.len(),
        "verified": true,
        "tamper_refused": true,
    }))
}

/// `gpu info` — real Metal-shared-buffer probe over a live cgap-arrow Buf:
/// availability is observed (typed Some/None), never assumed.
pub fn gpu_info(_inputs: Map<String, Value>) -> Result<Value> {
    let buf = cgap_arrow::buffer::Buf::new(4096);
    std::hint::black_box(buf.as_ptr());
    let handle = cgap_gpu::MetalSharedBuffer::from_buf(&buf);
    match handle {
        Some(h) => {
            if h.len() != 4096 {
                return fail("gpu info", format!("handle len {} != 4096", h.len()));
            }
            Ok(json!({ "available": true, "wrapped_bytes": h.len() }))
        }
        None => Ok(json!({
            "available": false,
            "reason": "Metal device unavailable in this context (typed None from from_buf)",
        })),
    }
}

/// `beam info` — cgap-beam is cdylib-only (no rlib) and the CLI crate
/// forbids unsafe, so the ring cannot be linked in-process. Real
/// collaborator via subprocess instead: run the beam crate's own SPSC ring
/// tests and report the harness's actual counts (bounded 600 s deadline).
pub fn beam_info(_inputs: Map<String, Value>) -> Result<Value> {
    run_crate_tests(
        "beam info",
        &["test", "-p", "cgap-beam", "spsc"],
        "cargo test -p cgap-beam spsc",
    )
}

/// Shared: run one cargo test command under a bounded deadline poll, parse
/// the harness result line, refuse on failure/timeout (POLL-STEP pattern).
fn run_crate_tests(what: &str, args: &[&str], label: &str) -> Result<Value> {
    use std::io::Read;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};
    const DEADLINE: Duration = Duration::from_secs(600);
    const STEP: Duration = Duration::from_millis(250);
    let started = Instant::now();
    let mut child = Command::new("cargo")
        .args(args)
        .current_dir("/Users/sac/cgap")
        .env("CARGO_TARGET_DIR", "/tmp/cgap-court-cli-target")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| NounVerbError::execution_error(format!("{what}: spawn refused: {e:?}")))?;
    // SLEEP-CLASS: POLL-STEP — child exit is the observable; deadline
    // bounds the wait, expiry kills and refuses typed.
    let status = loop {
        if let Some(st) = child
            .try_wait()
            .map_err(|e| NounVerbError::execution_error(format!("{what}: wait failed: {e:?}")))?
        {
            break st;
        }
        if started.elapsed() > DEADLINE {
            let _ = child.kill();
            return fail(what, "harness exceeded 600s deadline".to_owned());
        }
        std::thread::sleep(STEP);
    };
    let mut out = String::new();
    if child
        .stdout
        .take()
        .expect("piped stdout present")
        .read_to_string(&mut out)
        .is_err()
    {
        return fail(what, "stdout read failed".to_owned());
    }
    let result_line = out
        .lines()
        .find(|l| l.contains("test result:"))
        .ok_or_else(|| {
            NounVerbError::execution_error(format!(
                "{what}: no result line; tail: {}",
                out.lines().last().unwrap_or("")
            ))
        })?;
    let passed: usize = result_line
        .split("passed")
        .next()
        .and_then(|p| p.rsplit('.').next())
        .and_then(|p| p.trim().parse().ok())
        .ok_or_else(|| {
            NounVerbError::execution_error(format!("{what}: unparseable result: {result_line}"))
        })?;
    let failed: usize = result_line
        .split("failed")
        .next()
        .and_then(|p| p.rsplit(&['.', ';'][..]).next())
        .and_then(|p| p.trim().parse().ok())
        .unwrap_or(0);
    if status.success() && failed == 0 && passed > 0 {
        Ok(json!({
            "harness": label,
            "exit": status.code().unwrap_or(0),
            "passed": passed,
            "failed": failed,
            "elapsed_s": started.elapsed().as_secs(),
        }))
    } else {
        fail(what, format!("harness failing: exit {:?}, line: {result_line}", status.code()))
    }
}

/// `court verdict-run` — run the real cgap-court Chicago harness
/// (`cargo test -p cgap-court --test court_verdict`) as a subprocess and
/// report its actual pass/fail counts (bounded 600 s deadline poll).
pub fn court_verdict_run(_inputs: Map<String, Value>) -> Result<Value> {
    run_crate_tests(
        "court verdict-run",
        &["test", "-p", "cgap-court", "--test", "court_verdict"],
        "cargo test -p cgap-court --test court_verdict",
    )
}
