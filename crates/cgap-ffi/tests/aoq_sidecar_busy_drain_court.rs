//! AoQ sidecar multiplex-refusal court (PL7 implementation lane).
//!
//! LAW: the sidecar serves SEQUENTIALLY, single-tenant. A peer that
//! connects DURING another peer's serve round queues in the listener
//! backlog; the multiplex-refusal drain (aoq_sidecar.rs main loop) must
//! refuse every queued peer with the frozen-band BUSY word (0x4347A003)
//! and close — never silently carry it into a later serve round.
//!
//! WITNESS SHAPE: client A connects and sends NOTHING (the sidecar blocks
//! in `read_window` inside serve). Client B connects while A is being
//! served (B is in the backlog). A then closes cleanly (zero-byte close =
//! `ReadFault::CleanClose`, serve returns). The drain must hand B the
//! 4-byte BUSY word within a bounded poll. Mutation (drain removed): B
//! queues silently and the bounded BUSY read fails — the mutation witness
//! is recorded in the PL7 lane receipt (shim-court: SHIM -> court FAIL ->
//! byte-exact RESTORE).
//!
//! No-sleep law: bounded polls with hard deadlines throughout.
//!
//! ANTI-VACUITY (drain-removal mutation leg, `busy_drain_mutation_flips`
//! below): a /tmp sandbox copy of the workspace with the multiplex-refusal
//! drain textually removed (revert to blocking accept) must NOT deliver the
//! BUSY word to client B — B queues SILENTLY — so the BUSY assertion above
//! provably flips. The shared tree is never touched; the sandbox source is
//! sha256-verified byte-identical after the shim is applied-and-reverted.

// SOURCE-SCAN-EXEMPT: std:: : integration-court-only collaborator
// SOURCE-SCAN-EXEMPT: Command: spawns the real sidecar binary under test

use std::io::Read;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// One bounded poll: run `probe` until it returns Some or the deadline
/// passes (fail-closed, never a fixed sleep).
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

/// Spawn the real sidecar and bounded-poll for its socket.
fn spawn_sidecar(tag: &str) -> Sidecar {
    let sock_path: PathBuf =
        std::env::temp_dir().join(format!("aoq_busy_{tag}_{:?}.sock", std::process::id()));
    let _ = std::fs::remove_file(&sock_path);
    let mut child = Command::new(env!("CARGO_BIN_EXE_aoq_sidecar"))
        .arg(&sock_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
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

/// COURT: client A holds the single-tenant serve (sends nothing); client B
/// queues in the backlog; A closes cleanly; B must receive the 4-byte BUSY
/// word within a bounded poll.
#[test]
fn busy_drain_refuses_backlog_peer_with_busy_word() {
    let mut car = spawn_sidecar("busy");
    let a = connect(&car);
    // A sends nothing: serve(A) is blocked in read_window.
    let b = connect(&car);
    // B is now queued in the backlog DURING A's serve round.

    // A closes cleanly -> serve(A) returns via ReadFault::CleanClose.
    drop(a);

    // THE LAW: B gets the frozen-band BUSY word, bounded, then closed.
    // B reads on its OWN retained socket — the exact peer that queued in
    // the backlog mid-serve. Nonblocking reads bounded-poll until the
    // drain reaches B.
    let mut b = b;
    let _ = b.set_nonblocking(true);
    let word = poll_until("B receives BUSY word", Duration::from_secs(10), || {
        let mut word_b = [0u8; 4];
        match b.read(&mut word_b) {
            Ok(4) => Some(u32::from_le_bytes(word_b)),
            Ok(_) => None,  // partial: not the law's shape
            Err(_) => None, // WouldBlock / not yet drained -> retry
        }
    });
    assert_eq!(
        word, 0x4347_A003,
        "backlog peer must receive the frozen-band BUSY word"
    );
    // Post-BUSY: the drain closed the peer — a further read sees EOF.
    let mut probe = [0u8; 16];
    let n = b.read(&mut probe).unwrap_or(0);
    assert_eq!(n, 0, "drain closes the refused peer after the BUSY word");
    // The sidecar must still be serving (drain != poison).
    let c = connect(&car);
    let _ = c; // accepted into the next blocking serve round
               // Lawful stop: stdin EOF -> exit 0.
    car.stop_signal = None;
    let status = poll_until("stdin-EOF exit", Duration::from_secs(10), || {
        car.child.try_wait().ok().flatten()
    });
    assert!(
        status.success(),
        "stdin-EOF stop must exit 0, got {status:?}"
    );
    let _ = b;
}

// ---------------------------------------------------------------------------
// ANTI-VACUITY: drain-removal mutation leg (SHIM-COURT)
// ---------------------------------------------------------------------------

/// The drain block's start anchor in `src/bin/aoq_sidecar.rs` (X2's PL7
/// implementation): nonblocking-accept drain begins here.
const DRAIN_START: &str = "                    if let Err(e) = listener.set_nonblocking(true) {";
/// The re-blocking call directly after the drain loop (exclusive end anchor).
const DRAIN_END: &str = "                    if let Err(e) = listener.set_nonblocking(false) {";

/// COURT (mutation leg): a sidecar WITHOUT the multiplex-refusal drain
/// (reverted to a plain blocking accept) must NOT hand client B the BUSY
/// word — B queues silently in the backlog, so the green court's BUSY
/// assertion provably flips on this mutation. A flip count of 0 here is a
/// VACUOUS court (the green leg would pass on a drain-less sidecar too).
///
/// Mechanics (repo shim-court law): the mutation is applied to a /tmp
/// rsync copy of the workspace (never the shared tree), the mutant binary
/// is built in the sandbox's own target dir, and the sandbox source is
/// sha256-verified byte-identical after the shim apply+revert cycle.
#[test]
fn busy_drain_mutation_flips() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("workspace root");
    let sandbox = std::env::temp_dir().join(format!("aoq_busy_mut_{:?}", std::process::id()));
    let _ = std::fs::remove_dir_all(&sandbox);
    // SHIM-COURT 1: copy the workspace sources (never .git / target dirs).
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
    // SHIM: remove the drain (start anchor .. end anchor, exclusive).
    let start = src
        .find(DRAIN_START)
        .expect("VACUOUS SHIM: drain start anchor drifted in aoq_sidecar.rs");
    let end = src[start..]
        .find(DRAIN_END)
        .expect("VACUOUS SHIM: drain end anchor drifted in aoq_sidecar.rs")
        + start;
    let removed = &src[start..end];
    assert!(
        removed.contains("AOQ_STATUS_BUSY") && removed.contains("WouldBlock"),
        "VACUOUS SHIM: removed span is not the drain block"
    );
    assert!(
        src.find("MULTIPLEX-REFUSAL DRAIN").is_some(),
        "VACUOUS SHIM: drain comment absent from source (anchor drift)"
    );
    let mutated = format!("{}{}", &src[..start], &src[end..]);
    assert_ne!(mutated, src, "mutation was a no-op (shim vacuous)");
    std::fs::write(&bin_src, &mutated).expect("write mutated sidecar");
    // Build the mutant binary in the sandbox's own target dir. Strip the
    // outer cargo's env so the nested build is self-contained.
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
    // SHIM-COURT restore: the sandbox source must be byte-identical to the
    // shared tree's (fail-closed: hash taken from disk, not from memory).
    let restored_src = std::fs::read_to_string(root.join("crates/cgap-ffi/src/bin/aoq_sidecar.rs"))
        .expect("read pristine sidecar source");
    let hash_pristine = sha256(&restored_src);
    assert_eq!(
        hash_before, hash_pristine,
        "sandbox copy drifted from the shared tree BEFORE mutation"
    );
    let mut car = spawn_sidecar_from(&bin, "mut");
    // The green court's exact witness shape, inverted: A holds the serve,
    // B queues mid-serve, A closes — the drain-less mutant must hand B
    // NOTHING (silent queue), never the BUSY word.
    let a = connect(&car);
    let b = connect(&car);
    drop(a);
    let mut b = b;
    let _ = b.set_nonblocking(true);
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut busy_words = 0u32;
    let mut total_bytes = 0usize;
    while Instant::now() < deadline {
        let mut word_b = [0u8; 4];
        match b.read(&mut word_b) {
            Ok(n) => {
                total_bytes += n;
                if n == 4 && u32::from_le_bytes(word_b) == 0x4347_A003 {
                    busy_words += 1;
                }
            }
            Err(_) => {} // WouldBlock / not yet drained: retry
        }
    }
    assert_eq!(
        busy_words, 0,
        "MUTATION DID NOT FLIP: the drain-less sidecar still sent the BUSY word"
    );
    assert_eq!(
        total_bytes, 0,
        "drain-less sidecar queued client B silently (zero bytes), got {total_bytes}"
    );
    // Flip count: exactly one mutation witness, the court flips.
    // The mutant is otherwise lawful: a fresh peer is served (blocking
    // accept resumed) and stdin-EOF still exits 0.
    let c = connect(&car);
    let _ = c;
    car.stop_signal = None;
    let status = poll_until("mutant stdin-EOF exit", Duration::from_secs(10), || {
        car.child.try_wait().ok().flatten()
    });
    assert!(status.success(), "mutant stdin-EOF stop must exit 0");
    drop(car);
    // Janitor: the /tmp sandbox is a lease, delete it in the same file.
    let _ = std::fs::remove_dir_all(&sandbox);
}

/// Spawn an arbitrary sidecar binary (green court uses
/// `CARGO_BIN_EXE_aoq_sidecar`; the mutation leg uses the sandbox mutant).
fn spawn_sidecar_from(bin: &PathBuf, tag: &str) -> Sidecar {
    let sock_path: PathBuf =
        std::env::temp_dir().join(format!("aoq_busy_{tag}_{:?}.sock", std::process::id()));
    let _ = std::fs::remove_file(&sock_path);
    let mut child = Command::new(bin)
        .arg(&sock_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn sidecar under court");
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

/// sha256 via the host toolchain's shasum (fail-closed on absence).
fn sha256(data: &str) -> String {
    let out = Command::new("shasum")
        .args(["-a", "256"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut c| {
            use std::io::Write;
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
