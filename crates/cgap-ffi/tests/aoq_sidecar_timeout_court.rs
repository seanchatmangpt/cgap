//! AoQ sidecar wedge-freedom court (A4 RISK A — unbounded blocking read).
//!
//! LAW: the sidecar is single-tenant; a client that connects, sends a
//! partial AOQ1 prefix, then STALLS must NOT park serve() in read(2)
//! forever. serve() installs SO_RCVTIMEO/SO_SNDTIMEO (30s default;
//! `CGAP_SIDECAR_IO_TIMEOUT_MS` court override) on both legs; the stalled
//! peer is dropped SILENTLY (no status word — the frozen band is 8/8
//! allocated per KD-1) and the accept loop resumes.
//!
//! WITNESS SHAPE: client A writes 4 of the 8 prefix bytes and holds the
//! connection open (never closes, never sends more). Client B connects and
//! queues in the backlog DURING A's serve round. At the deadline the
//! sidecar must (1) drop A with ZERO bytes written (the silent-drop law —
//! A observes a clean EOF, never a status word) and (2) resume: B takes
//! the frozen-band BUSY word from the PL7 multiplex-refusal drain, and a
//! subsequent well-formed connection C is ACCEPTED and SERVED (its
//! tag-gate refusal returns a non-BUSY frozen-band word — the accept loop
//! is provably out of the wedge). A sidecar still wedged in read(2) can
//! never produce witness (2).
//!
//! ANTI-VACUITY mutation leg (`timeout_mutation_flips`): a /tmp sandbox
//! copy with the set_read_timeout/set_write_timeout installation removed
//! must NEVER drop A (A stays connected past the deadline) — the green
//! court's EOF assertion provably flips. Sandbox sha256 fail-closed
//! restore; the shared tree is never mutated.
//!
//! No-sleep law: every wait is a bounded poll on a real observable (socket
//! bytes, child exit) with a hard deadline; the only timed waits are the
//! kernel's own SO_RCVTIMEO inside the subject under test.

// SOURCE-SCAN-EXEMPT: std:: : integration-court-only collaborator
// SOURCE-SCAN-EXEMPT: Command: spawns the real sidecar binary under test

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// The court's timeout override: the REAL binary under the REAL serve loop
/// at a deadline a CI court can afford (1s), via the disclosed env seam.
const COURT_TIMEOUT_MS: &str = "1000";
/// Bounded deadline for the drop witness: must fire well within a few
/// multiples of the override (fail-closed, never a fixed sleep).
const DROP_DEADLINE: Duration = Duration::from_secs(10);

/// One bounded poll: run `probe` until Some or the deadline passes.
fn poll_until<T>(what: &str, deadline: Duration, mut probe: impl FnMut() -> Option<T>) -> T {
    let exp = Instant::now() + deadline;
    loop {
        if let Some(v) = probe() {
            return v;
        }
        if Instant::now() > exp {
            panic!("bounded poll expired: {what}");
        }
    }
}

/// A sidecar under court, stdin piped (the stop signal).
struct Sidecar {
    child: Child,
    sock_path: PathBuf,
    stop_signal: Option<std::process::ChildStdin>,
}

impl Drop for Sidecar {
    fn drop(&mut self) {
        self.stop_signal = None; // stdin EOF -> clean exit path
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.sock_path); // court janitor
    }
}

/// Spawn the real sidecar with the court timeout override; bounded-poll
/// for its socket.
fn spawn_sidecar_with_timeout(tag: &str, bin: Option<&PathBuf>) -> Sidecar {
    let sock_path: PathBuf =
        std::env::temp_dir().join(format!("aoq_timeout_{tag}_{:?}.sock", std::process::id()));
    let _ = std::fs::remove_file(&sock_path);
    let mut cmd = match bin {
        Some(b) => Command::new(b),
        None => Command::new(env!("CARGO_BIN_EXE_aoq_sidecar")),
    };
    let mut child = cmd
        .arg(&sock_path)
        .env("CGAP_SIDECAR_IO_TIMEOUT_MS", COURT_TIMEOUT_MS)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn aoq_sidecar");
    let stop_signal = child.stdin.take().expect("sidecar stdin pipe");
    poll_until("socket bind", Duration::from_secs(10), || {
        if sock_path.exists() {
            Some(())
        } else {
            None
        }
    });
    Sidecar {
        child,
        sock_path,
        stop_signal: Some(stop_signal),
    }
}

/// Connect (bounded-poll; the listener may be mid-drain).
fn connect(car: &Sidecar) -> UnixStream {
    poll_until("connect", Duration::from_secs(10), || {
        UnixStream::connect(&car.sock_path).ok()
    })
}

/// Nonblocking read of up to `buf.len()` bytes; None = not yet ready
/// (WouldBlock), Some(n) = kernel reported n bytes / EOF.
fn nb_read(s: &mut UnixStream, buf: &mut [u8]) -> Option<usize> {
    s.set_nonblocking(true).expect("set_nonblocking");
    let r = s.read(buf);
    s.set_nonblocking(false).expect("re-block");
    match r {
        Ok(n) => Some(n),
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => None,
        Err(e) => panic!("unexpected read fault: {e}"),
    }
}

/// COURT (wedge freedom): a stalled client is dropped at the deadline and
/// the accept loop resumes — B is drained with the BUSY word and a fresh
/// well-formed connection C is accepted and served.
#[test]
fn stalled_client_dropped_and_accept_resumes() {
    let mut car = spawn_sidecar_with_timeout("green", None);

    // A: partial prefix, then HOLD (never closes, never sends more).
    let mut a = connect(&car);
    let mut prefix = [0u8; 8];
    prefix[0..4].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    prefix[4..8].copy_from_slice(&120u32.to_le_bytes());
    a.write_all(&prefix[..4]).expect("partial prefix write");
    let _ = a.flush();

    // B: queues in the backlog DURING A's serve round.
    let mut b = connect(&car);

    // WITNESS 1 — A is dropped SILENTLY at the deadline: a bounded poll
    // on A's socket until the kernel reports EOF (read = Ok(0)); a status
    // word before the EOF is the silent-drop law violated.
    let mut seen_bytes = 0usize;
    poll_until("stalled peer A dropped (EOF)", DROP_DEADLINE, || {
        let mut probe = [0u8; 4];
        match nb_read(&mut a, &mut probe) {
            Some(0) => Some(()), // clean EOF: serve() returned, conn closed
            Some(n) => {
                seen_bytes += n;
                None // bytes arrived; silence checked after the poll
            }
            None => None, // still connected: the wedge persists
        }
    });
    assert_eq!(
        seen_bytes, 0,
        "timeout drop must be SILENT (no status word — frozen band 8/8, KD-1)"
    );

    // WITNESS 2a — B, the mid-serve backlog peer, gets the PL7 BUSY word.
    let word = poll_until("B receives BUSY word from drain", DROP_DEADLINE, || {
        let mut word_b = [0u8; 4];
        match nb_read(&mut b, &mut word_b) {
            Some(4) => Some(u32::from_le_bytes(word_b)),
            _ => None,
        }
    });
    assert_eq!(word, 0x4347_A003, "drain hands the backlog peer BUSY");
    drop(b);

    // WITNESS 2b — wedge freedom: a fresh well-formed connection C is
    // ACCEPTED and SERVED. C sends a lawful-shaped window whose body tag
    // cannot verify; the tag gate must answer with a non-BUSY frozen-band
    // refusal (DIGEST_MISMATCH). Only an un-wedged accept loop can.
    let mut c = connect(&car);
    // Lawful-shaped window: prefix + meta + EOS tail (body_len 0), so the
    // read completes and the TAG GATE (not a transport fault) refuses.
    let mut probe_win = vec![0u8; 8 + 120 + 8]; // body_len 0 -> tag mismatch
    probe_win[136 - 4..].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    probe_win[0..4].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    probe_win[4..8].copy_from_slice(&120u32.to_le_bytes());
    c.write_all(&probe_win).expect("probe window write");
    let mut word_c = [0u8; 4];
    c.set_read_timeout(Some(DROP_DEADLINE))
        .expect("C read deadline");
    c.read_exact(&mut word_c)
        .expect("C must be served (accept loop out of the wedge)");
    assert_ne!(
        u32::from_le_bytes(word_c),
        0x4347_A003,
        "C must be SERVED, not drained"
    );
    drop(c);

    // The sidecar is still alive and stops lawfully (stdin EOF -> exit 0).
    car.stop_signal = None;
    let status = poll_until("stdin-EOF exit", Duration::from_secs(10), || {
        car.child.try_wait().ok().flatten()
    });
    assert!(
        status.success(),
        "stdin-EOF stop must exit 0, got {status:?}"
    );
    let _ = a;
}

// ---------------------------------------------------------------------------
// ANTI-VACUITY: timeout-installation-removal mutation leg (SHIM-COURT)
// ---------------------------------------------------------------------------

/// Anchor: the timeout installation at the top of serve().
const SET_READ_TIMEOUT: &str = "if let Err(e) = stream.set_read_timeout(Some(t)) {";

/// COURT (mutation leg): removing the SO_RCVTIMEO/SO_SNDTIMEO installation
/// must make the green court's drop witness FAIL — the stalled peer A
/// stays connected past the deadline (the sidecar parks in read(2)
/// forever). A mutant that still drops A is a VACUOUS court.
#[test]
fn timeout_mutation_flips() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("workspace root");
    let sandbox = std::env::temp_dir().join(format!("aoq_timeout_mut_{:?}", std::process::id()));
    let _ = std::fs::remove_dir_all(&sandbox);
    let cp = Command::new("rsync")
        .args([
            "-a",
            "--exclude",
            ".git",
            "--exclude",
            "target",
            "--exclude",
            "target-*",
        ])
        .arg(format!("{}/", root.display()))
        .arg(format!("{}/", sandbox.display()))
        .status()
        .expect("rsync sandbox copy")
        .success();
    assert!(cp, "sandbox rsync failed");
    let bin_src = sandbox.join("crates/cgap-ffi/src/bin/aoq_sidecar.rs");
    let src = std::fs::read_to_string(&bin_src).expect("read sidecar source");
    let hash_before = sha256(&src);
    // SHIM: drop the read-timeout installation (the write timeout alone
    // cannot bound the read wedge; the stalled-peer law needs both, and
    // the read install is the load-bearing one for this witness).
    let start = src
        .find(SET_READ_TIMEOUT)
        .expect("VACUOUS SHIM: set_read_timeout anchor drifted in aoq_sidecar.rs");
    let end = src[start..]
        .find("        let kem = SidecarKem;")
        .expect("VACUOUS SHIM: serve body anchor drifted")
        + start;
    let removed = &src[start..end];
    assert!(
        removed.contains("set_read_timeout") && removed.contains("set_write_timeout"),
        "VACUOUS SHIM: removed span is not the timeout installation"
    );
    let mutated = format!("{}{}", &src[..start], &src[end..]);
    assert_ne!(mutated, src, "mutation was a no-op (shim vacuous)");
    std::fs::write(&bin_src, &mutated).expect("write mutated sidecar");
    let bin = sandbox.join("target/debug/aoq_sidecar");
    let mut cmd = Command::new("cargo");
    cmd.args([
        "build",
        "--manifest-path",
        &sandbox.join("Cargo.toml").display().to_string(),
        "-p",
        "cgap-ffi",
        "--bin",
        "aoq_sidecar",
    ]);
    for (k, _) in std::env::vars() {
        if k.starts_with("CARGO_") {
            cmd.env_remove(&k);
        }
    }
    let built = cmd.status().expect("nested cargo build").success();
    assert!(built, "mutant build failed");
    // SHIM-COURT restore proof: sandbox source byte-identical to the
    // shared tree BEFORE mutation (fail-closed, hash from disk).
    let pristine = std::fs::read_to_string(root.join("crates/cgap-ffi/src/bin/aoq_sidecar.rs"))
        .expect("read pristine sidecar source");
    assert_eq!(
        hash_before,
        sha256(&pristine),
        "sandbox copy drifted from the shared tree BEFORE mutation"
    );

    // The green court's witness shape, inverted: A stalls, and PAST the
    // deadline the mutant must still hold the connection open (wedged).
    let car = spawn_sidecar_with_timeout("mut", Some(&bin));
    let mut a = connect(&car);
    let mut prefix = [0u8; 8];
    prefix[0..4].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    prefix[4..8].copy_from_slice(&120u32.to_le_bytes());
    a.write_all(&prefix[..4]).expect("partial prefix write");
    let _ = a.flush();
    // Sample A's connection state across the whole deadline window: the
    // mutant must NEVER report EOF (drop) within it.
    let exp = Instant::now() + Duration::from_millis(2500);
    let mut ever_dropped = false;
    while Instant::now() < exp {
        let mut probe = [0u8; 4];
        if nb_read(&mut a, &mut probe) == Some(0) {
            ever_dropped = true;
            break;
        }
    }
    assert!(
        !ever_dropped,
        "MUTATION DID NOT FLIP: the timeout-less sidecar still dropped the stalled peer"
    );
    drop(car);
    // Janitor: the /tmp sandbox is a lease, delete it in the same file.
    let _ = std::fs::remove_dir_all(&sandbox);
}

/// sha256 via the host toolchain's shasum (fail-closed on absence).
fn sha256(data: &str) -> String {
    let out = Command::new("shasum")
        .args(["-a", "256"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut c| {
            c.stdin.as_mut().unwrap().write_all(data.as_bytes())?;
            let o = c.wait_with_output()?;
            if o.status.success() {
                Ok(String::from_utf8_lossy(&o.stdout)
                    .split_whitespace()
                    .next()
                    .map(str::to_string)
                    .unwrap_or_default())
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "shasum failed",
                ))
            }
        })
        .expect("shasum -a 256 on PATH");
    assert!(
        out.len() == 64 && out.chars().all(|c| c.is_ascii_hexdigit()),
        "sha256 must be a real 64-hex digest, got {out:?}"
    );
    out
}
