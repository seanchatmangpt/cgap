//! aq_quic — phase-2 QUIC scaffold: quinn behind the LANE-02 NIF boundary
//! (lane L50; decision record aq-quic-transport-decision.md, option 2).
//!
//! ARCHITECTURE
//!
//! One dedicated non-BEAM OS thread ("aq-quic-engine") owns a tokio
//! **current-thread** runtime and every quinn object. NIF calls hand jobs to
//! the engine over a `std::sync::mpsc` channel and block the DirtyIo
//! scheduler on the reply. This is the decision record's scheduler-pinning
//! composition:
//!
//! - **Pinning mechanism (disclosed)**: on Linux the engine thread pins
//!   itself with `sched_setaffinity` to core 0 (best-effort, failure
//!   tolerated — pinning is a performance law, not a correctness law); on
//!   macOS the OS offers no per-thread CPU affinity, so the pinning
//!   guarantee there is structural — a single dedicated named thread is the
//!   only thread quinn/tokio work ever runs on (no foreign executor pool,
//!   no BEAM scheduler ever touches the QUIC state machine), and the
//!   current-thread runtime is what enforces single-thread execution.
//! - **Zero-copy egress (disclosed scope)**: `aq_quic_recv` hands back a
//!   resource-binary view whose bytes ARE quinn's received buffer
//!   (`enif_make_resource_binary` over `Bytes::as_ptr()` — the
//!   `aq_slab_frame`/`resbin` precedent). The honest scope: zero copies at
//!   the NIF→BEAM boundary; quinn's own UDP receive still copies once from
//!   the kernel into its receive buffer — the decision record's claim is
//!   exactly this boundary, not the kernel.
//!
//! Bounded waits only (no-sleep law): handshake, send, and recv are wrapped
//! in `tokio::time::timeout`; the NIF-side reply wait is bounded by the
//! engine-side timeout, never a sleep.
//!
//! Deviations from a full transport (scaffolding scale, per dispatch):
//! client role only in the engine (the court's echo server is pure-Rust
//! tokio in `tests/aq_quic_contract.rs`); uni-stream echo; ALPN fixed
//! `aq/1`.
//!
//! SHIM-COURT anti-vacuity mutation: `AQ_MUTATE_ALPN_DROP=1` makes
//! `aq_quic_connect` offer NO ALPN. The court's server requires `aq/1`, so
//! the handshake must refuse (rustls `no_application_protocol` alert). The
//! BEAM court MUST flip: connect fails under the mutation and succeeds
//! without it — a court that cannot flip is FAIL. The flag is read per
//! connect call (fresh escript process per court run, the aq_slab_contract
//! pattern — no cross-court shared state).

#![allow(non_camel_case_types)]

use crate::{ErlNifEnv, ErlNifTerm, make_atom};
use std::os::raw::{c_int, c_void};

#[cfg(feature = "quic")]
mod imp {
    use crate::alloc_spy::record_alloc;
    use crate::resources::{ErlNifResourceType, ErlNifResourceTypeInit};
    use crate::{ErlNifBinary, ErlNifEnv, ErlNifTerm, enif_get_long, make_atom};
    use bytes::Bytes;
    use std::collections::HashMap;
    use std::net::SocketAddr;
    use std::os::raw::{c_char, c_int, c_void};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicPtr, Ordering};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    /// The ALPN law: one protocol, `aq/1`.
    pub(crate) const AQ_ALPN: &[u8] = b"aq/1";

    /// Bounded handshake/send/open budget (tokio timeout; never a sleep).
    const OP_TIMEOUT: Duration = Duration::from_secs(5);
    /// Open is NOT a normal op: it includes engine-thread spawn, the tokio
    /// runtime build, TWO rustls config builds and the endpoint bind. Under
    /// fleet load (parallel court runs on a shared box) 5 s was witnessed
    /// too tight — the reply itself is bounded, never a sleep (no-sleep
    /// law: bounded poll on the real observable `ready_rx`).
    const OPEN_TIMEOUT: Duration = Duration::from_secs(30);

    // -----------------------------------------------------------------------
    // Resource types (opened at load; fail-closed boot, resources.rs law)
    // -----------------------------------------------------------------------

    static QUIC_ENGINE_TYPE: AtomicPtr<ErlNifResourceType> = AtomicPtr::new(std::ptr::null_mut());
    static QUIC_CONN_TYPE: AtomicPtr<ErlNifResourceType> = AtomicPtr::new(std::ptr::null_mut());
    static QUIC_BUF_TYPE: AtomicPtr<ErlNifResourceType> = AtomicPtr::new(std::ptr::null_mut());
    static QUIC_LISTENER_TYPE: AtomicPtr<ErlNifResourceType> =
        AtomicPtr::new(std::ptr::null_mut());
    static QUIC_STREAM_TYPE: AtomicPtr<ErlNifResourceType> =
        AtomicPtr::new(std::ptr::null_mut());

    /// The engine resource: a cloneable sender into the engine thread.
    struct QuicEngineRes {
        inner: Arc<EngineInner>,
    }
    // SAFETY: EngineInner is Send (mpsc Sender is Send; only the engine
    // thread touches quinn objects); resource dtors fire on arbitrary
    // scheduler threads.
    unsafe impl Send for QuicEngineRes {}

    /// The connection resource: engine handle + connection id.
    struct QuicConnRes {
        inner: Arc<EngineInner>,
        id: u64,
    }
    unsafe impl Send for QuicConnRes {}

    /// The received-buffer resource: OWNS quinn's received `Bytes`. The
    /// projected resource binary pins this resource; its dtor drops the
    /// Bytes (terminal reclamation, the resbin.rs ledger law).
    struct QuicBufRes {
        bytes: Bytes,
    }
    unsafe impl Send for QuicBufRes {}

    /// The QUIC listener resource: server-role handle to the engine's bound
    /// endpoint. The endpoint itself lives ONLY in `EngineState.listener` —
    /// the resource is an engine handle, never an owned socket (one-engine
    /// law).
    struct QuicListenerRes {
        inner: Arc<EngineInner>,
    }
    unsafe impl Send for QuicListenerRes {}

    /// The bi-directional stream resource: engine handle + (conn_id,
    /// stream_id) key into `EngineState.streams`.
    struct QuicStreamRes {
        inner: Arc<EngineInner>,
        conn_id: u64,
        stream_id: u64,
    }
    unsafe impl Send for QuicStreamRes {}

    extern "C" fn quic_engine_dtor(_env: *mut ErlNifEnv, obj: *mut c_void) {
        unsafe { std::ptr::drop_in_place(obj.cast::<QuicEngineRes>()) };
    }

    /// Conn dtor: best-effort graceful close (the engine drops the
    /// `quinn::Connection`; its Drop initiates the close). Send failure
    /// (dead engine) tolerated — a GC thread must never panic.
    extern "C" fn quic_conn_dtor(_env: *mut ErlNifEnv, obj: *mut c_void) {
        unsafe {
            let c = &mut *(obj.cast::<QuicConnRes>());
            let _ = c.inner.tx.send(Job::Close { id: c.id });
            std::ptr::drop_in_place(c as *mut QuicConnRes);
        }
    }

    extern "C" fn quic_buf_dtor(_env: *mut ErlNifEnv, obj: *mut c_void) {
        unsafe { std::ptr::drop_in_place(obj.cast::<QuicBufRes>()) };
    }

    /// Listener dtor: clears the engine's server endpoint (GC thread safety:
    /// send failure tolerated).
    extern "C" fn quic_listener_dtor(_env: *mut ErlNifEnv, obj: *mut c_void) {
        unsafe {
            let l = &mut *(obj.cast::<QuicListenerRes>());
            let _ = l.inner.tx.send(Job::CloseListener);
            std::ptr::drop_in_place(l as *mut QuicListenerRes);
        }
    }

    /// Stream dtor: removes the (conn_id, stream_id) entry; dropping the
    /// quinn RecvStream implicitly STOPs it and dropping the SendStream
    /// RESETs it (quinn 0.11 drop semantics — the explicit-close law for a
    /// GC thread that must never panic).
    extern "C" fn quic_stream_dtor(_env: *mut ErlNifEnv, obj: *mut c_void) {
        unsafe {
            let s = &mut *(obj.cast::<QuicStreamRes>());
            let _ = s.inner.tx.send(Job::CloseStream {
                conn_id: s.conn_id,
                stream_id: s.stream_id,
            });
            std::ptr::drop_in_place(s as *mut QuicStreamRes);
        }
    }

    /// Open all five QUIC resource types at load (fail-closed boot:
    /// returning false fails the whole load_nif — resources.rs:217-225 law;
    /// this OTP refuses resource-type opens outside load/upgrade).
    pub(crate) fn open_quic_types(env: *mut ErlNifEnv) -> bool {
        static ENGINE_INIT: std::sync::OnceLock<ErlNifResourceTypeInit> = std::sync::OnceLock::new();
        static CONN_INIT: std::sync::OnceLock<ErlNifResourceTypeInit> = std::sync::OnceLock::new();
        static BUF_INIT: std::sync::OnceLock<ErlNifResourceTypeInit> = std::sync::OnceLock::new();
        static LISTENER_INIT: std::sync::OnceLock<ErlNifResourceTypeInit> =
            std::sync::OnceLock::new();
        static STREAM_INIT: std::sync::OnceLock<ErlNifResourceTypeInit> =
            std::sync::OnceLock::new();
        let engine_init = ENGINE_INIT
            .get_or_init(|| crate::resources::resource_type_init_dtor(quic_engine_dtor));
        let conn_init =
            CONN_INIT.get_or_init(|| crate::resources::resource_type_init_dtor(quic_conn_dtor));
        let buf_init = BUF_INIT.get_or_init(|| crate::resources::resource_type_init_dtor(quic_buf_dtor));
        let listener_init = LISTENER_INIT
            .get_or_init(|| crate::resources::resource_type_init_dtor(quic_listener_dtor));
        let stream_init =
            STREAM_INIT.get_or_init(|| crate::resources::resource_type_init_dtor(quic_stream_dtor));
        unsafe fn open_ty(
            env: *mut ErlNifEnv,
            name: &[u8],
            init: &ErlNifResourceTypeInit,
        ) -> *mut ErlNifResourceType {
            let mut tried: c_int = 0;
            quic_extern::enif_open_resource_type_x(
                env,
                name.as_ptr() as *const c_char,
                init as *const _,
                1 | 2, // ERL_NIF_RT_CREATE | ERL_NIF_RT_TAKEOVER
                &mut tried,
            )
        }
        let engine_ty = unsafe { open_ty(env, b"AqQuicEngine\0", engine_init) };
        if engine_ty.is_null() {
            return false;
        }
        let conn_ty = unsafe { open_ty(env, b"AqQuicConn\0", conn_init) };
        if conn_ty.is_null() {
            return false;
        }
        let buf_ty = unsafe { open_ty(env, b"AqQuicBuf\0", buf_init) };
        if buf_ty.is_null() {
            return false;
        }
        let listener_ty = unsafe { open_ty(env, b"AqQuicListener\0", listener_init) };
        if listener_ty.is_null() {
            return false;
        }
        let stream_ty = unsafe { open_ty(env, b"AqQuicStream\0", stream_init) };
        if stream_ty.is_null() {
            return false;
        }
        QUIC_ENGINE_TYPE.store(engine_ty, Ordering::Relaxed);
        QUIC_CONN_TYPE.store(conn_ty, Ordering::Relaxed);
        QUIC_BUF_TYPE.store(buf_ty, Ordering::Relaxed);
        QUIC_LISTENER_TYPE.store(listener_ty, Ordering::Relaxed);
        QUIC_STREAM_TYPE.store(stream_ty, Ordering::Relaxed);
        true
    }

    /// Local re-declaration of the enif_* calls this module needs (the
    /// aq_slab.rs:339-351 re-declaration precedent; same C ABI, same VM
    /// symbols the BEAM resolves at dlopen).
    mod quic_extern {
        use super::*;
        extern "C" {
            pub(crate) fn enif_open_resource_type_x(
                env: *mut ErlNifEnv,
                name_str: *const c_char,
                init: *const crate::resources::ErlNifResourceTypeInit,
                flags: c_int,
                tried: *mut c_int,
            ) -> *mut ErlNifResourceType;
            pub(crate) fn enif_alloc_resource(
                ty: *mut ErlNifResourceType,
                size: usize,
            ) -> *mut c_void;
            pub(crate) fn enif_release_resource(obj: *mut c_void);
            pub(crate) fn enif_make_resource(env: *mut ErlNifEnv, obj: *mut c_void) -> ErlNifTerm;
            pub(crate) fn enif_get_resource(
                env: *mut ErlNifEnv,
                term: ErlNifTerm,
                ty: *mut ErlNifResourceType,
                objp: *mut *mut c_void,
            ) -> c_int;
            /// enif_make_resource_binary (erl_nif_api_funcs.h:134) — the
            /// zero-copy egress call: `data` is `Bytes::as_ptr()` of the
            /// received buffer (the code-level zero-copy fact, cited in the
            /// receipt).
            pub(crate) fn enif_make_resource_binary(
                env: *mut ErlNifEnv,
                obj: *mut c_void,
                data: *const c_void,
                size: usize,
            ) -> ErlNifTerm;
            pub(crate) fn enif_release_binary(bin: *mut ErlNifBinary);
            pub(crate) fn enif_inspect_binary(
                env: *mut ErlNifEnv,
                term: ErlNifTerm,
                bin: *mut ErlNifBinary,
            ) -> c_int;
            /// `int enif_inspect_iolist_as_binary(ErlNifEnv*,
            ///   ERL_NIF_TERM, ErlNifBinary*);` (erl_nif_api_funcs.h:100)
            /// — send-path iolist support: flattens a (possibly improper)
            /// iolist of binaries and 0..255 integers into one contiguous
            /// view. Export witness: `nm -gU beam.smp` shows
            /// `_enif_inspect_iolist_as_binary`.
            pub(crate) fn enif_inspect_iolist_as_binary(
                env: *mut ErlNifEnv,
                term: ErlNifTerm,
                bin: *mut ErlNifBinary,
            ) -> c_int;
        }
    }

    // -----------------------------------------------------------------------
    // Engine thread: dedicated, pinned (Linux), current-thread tokio runtime
    // -----------------------------------------------------------------------

    enum Job {
        /// Bind the client endpoint (engine-thread startup job: quinn
        /// Endpoints are runtime-bound, so the bind must run inside the
        /// engine's runtime).
        Open {
            port: u16,
            ca_der: Vec<u8>,
            reply: std::sync::mpsc::Sender<Result<(), String>>,
        },
        Connect {
            addr: SocketAddr,
            server_name: String,
            reply: std::sync::mpsc::Sender<Result<u64, String>>,
        },
        Send {
            id: u64,
            data: Vec<u8>,
            reply: std::sync::mpsc::Sender<Result<(), String>>,
        },
        /// Receive one uni-stream echo and hand back quinn's OWN buffer.
        Recv {
            id: u64,
            timeout_ms: u64,
            reply: std::sync::mpsc::Sender<Result<Bytes, String>>,
        },
        Close {
            id: u64,
        },
        // --- server role (same engine thread, same Job mpsc) ----------------
        /// Bind the SERVER endpoint (rustls ServerConfig, ALPN aq/1
        /// REQUIRED server-side — the SHIM-COURT mutation strips it).
        Serve {
            port: u16,
            cert_der: Vec<u8>,
            key_der: Vec<u8>,
            reply: std::sync::mpsc::Sender<Result<(), String>>,
        },
        /// Accept one inbound connection off the listener (bounded).
        Accept {
            timeout_ms: u64,
            reply: std::sync::mpsc::Sender<Result<u64, String>>,
        },
        /// Open one client-initiated bi-directional stream.
        OpenBi {
            conn_id: u64,
            reply: std::sync::mpsc::Sender<Result<u64, String>>,
        },
        /// Write on an open bi stream: `write_all`, NO `finish` — bi
        /// streams persist for further frames.
        SendOn {
            conn_id: u64,
            stream_id: u64,
            data: Vec<u8>,
            reply: std::sync::mpsc::Sender<Result<(), String>>,
        },
        /// Read one chunk off an open bi stream and hand back quinn's OWN
        /// buffer (the zero-copy egress law, same as client Recv).
        RecvOn {
            conn_id: u64,
            stream_id: u64,
            timeout_ms: u64,
            reply: std::sync::mpsc::Sender<Result<Bytes, String>>,
        },
        /// Accept a peer-initiated bi stream (server side of OpenBi).
        AcceptBi {
            conn_id: u64,
            timeout_ms: u64,
            reply: std::sync::mpsc::Sender<Result<u64, String>>,
        },
        /// Drop the stream (recv STOP + send RESET via quinn drop
        /// semantics).
        CloseStream {
            conn_id: u64,
            stream_id: u64,
        },
        /// Listener dtor job: clear the server endpoint.
        CloseListener,
    }

    struct EngineInner {
        tx: std::sync::mpsc::Sender<Job>,
    }

    struct EngineState {
        ep: Option<quinn::Endpoint>,
        /// Server endpoint bound by Job::Serve (one listener per engine —
        /// the one-engine law; the listener resource is only a handle).
        listener: Option<quinn::Endpoint>,
        /// Trust anchor captured at Open, applied per connection (the ALPN
        /// mutation is a per-connect decision, so the client config is
        /// built per connect).
        ca_der: Vec<u8>,
        conns: HashMap<u64, quinn::Connection>,
        /// Bi-directional streams keyed (conn_id, stream_id); both ids are
        /// engine-assigned counters (NOT quinn's wire stream ids — the
        /// engine's own address space, same class as conn ids).
        streams: HashMap<(u64, u64), (quinn::RecvStream, quinn::SendStream)>,
        next_id: u64,
        next_stream_id: u64,
    }

    /// SHIM-COURT mutation flag (read PER CALL — each court run is a fresh
    /// escript process; no OnceLock, no cross-court shared state).
    /// AQ_MUTATE_ALPN_DROP=1: offer no ALPN -> the aq/1-requiring peer must
    /// refuse the handshake.
    fn mutate_alpn_drop() -> bool {
        std::env::var("AQ_MUTATE_ALPN_DROP").as_deref() == Ok("1")
    }

    fn pin_current_thread_best_effort() {
        // Linux: sched_setaffinity to core 0. macOS: no per-thread affinity
        // exists in the OS — the structural pin (dedicated thread +
        // current-thread runtime) is the whole mechanism there.
        #[cfg(target_os = "linux")]
        {
            // sched_setaffinity (x86_64). Best-effort by design.
            const SYS_SETAFFINITY: std::os::raw::c_long = 203;
            unsafe extern "C" {
                fn syscall(num: std::os::raw::c_long, ...) -> std::os::raw::c_long;
            }
            let mut set = [0u64; 16]; // 1024 CPUs
            set[0] = 1; // core 0
            let rc = unsafe {
                syscall(
                    SYS_SETAFFINITY,
                    0usize,
                    std::mem::size_of::<[u64; 16]>(),
                    set.as_ptr(),
                )
            };
            let _ = rc; // tolerated failure (performance law, not correctness)
        }
    }

    fn client_config(ca_der: &[u8], offer_alpn: bool) -> Result<quinn::ClientConfig, String> {
        let mut roots = rustls::RootCertStore::empty();
        roots
            .add(rustls::pki_types::CertificateDer::from(ca_der.to_vec()))
            .map_err(|e| format!("ca der rejected: {e}"))?;
        let mut crypto = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        if offer_alpn {
            crypto.alpn_protocols = vec![AQ_ALPN.to_vec()];
        }
        // quinn 0.11.12 wraps a rustls ClientConfig via TryFrom (the
        // `QuicClientConfig::new` ctor is private in this version; TryFrom
        // derives the initial cipher suite from the provider).
        let quic_crypto: quinn::crypto::rustls::QuicClientConfig =
            crypto.try_into().map_err(|e: quinn::crypto::rustls::NoInitialCipherSuite| {
                format!("quic tls wrap: {e}")
            })?;
        Ok(quinn::ClientConfig::new(Arc::new(quic_crypto)))
    }

    /// Server TLS config: self-PEM-free DER cert+key, `alpn_protocols`
    /// REQUIRED to contain `aq/1` server-side (the QUIC-TLS law: a server
    /// with no ALPN refuses every client offering one; the SHIM-COURT
    /// mutation strips the list and the handshake must answer
    /// `no_application_protocol`).
    fn server_config(
        cert_der: Vec<u8>,
        key_der: Vec<u8>,
        offer_alpn: bool,
    ) -> Result<quinn::ServerConfig, String> {
        let cert = rustls::pki_types::CertificateDer::from(cert_der);
        let key = rustls::pki_types::PrivatePkcs8KeyDer::from(key_der);
        let mut crypto = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert], rustls::pki_types::PrivateKeyDer::Pkcs8(key))
            .map_err(|e| format!("server cert: {e}"))?;
        if offer_alpn {
            crypto.alpn_protocols = vec![AQ_ALPN.to_vec()];
        }
        let quic_crypto: quinn::crypto::rustls::QuicServerConfig =
            crypto
                .try_into()
                .map_err(|e: quinn::crypto::rustls::NoInitialCipherSuite| {
                    format!("quic tls server wrap: {e}")
                })?;
        // `with_crypto` supplies the default handshake-token key
        // (`ServerConfig::new` takes an explicit token key in 0.11.19 —
        // the quinn 0.11.12 example-server precedent).
        Ok(quinn::ServerConfig::with_crypto(Arc::new(quic_crypto)))
    }

    /// One job, executed on the engine thread under the runtime.
    async fn run_job(st: &mut EngineState, job: Job) {
        match job {
            Job::Open {
                port,
                ca_der,
                reply,
            } => {
                // Validate the CA EARLY (a bad DER must fail open, not the
                // first connect).
                if let Err(e) = client_config(&ca_der, true) {
                    let _ = reply.send(Err(e));
                    return;
                }
                match quinn::Endpoint::client(SocketAddr::from(([127, 0, 0, 1], port))) {
                    Ok(mut ep) => {
                        match client_config(&ca_der, true) {
                            Ok(cfg) => ep.set_default_client_config(cfg),
                            Err(e) => {
                                let _ = reply.send(Err(e));
                                return;
                            }
                        }
                        st.ca_der = ca_der;
                        st.ep = Some(ep);
                        let _ = reply.send(Ok(()));
                    }
                    Err(e) => {
                        let _ = reply.send(Err(format!("bind: {e}")));
                    }
                }
            }
            Job::Connect {
                addr,
                server_name,
                reply,
            } => {
                let Some(ep) = st.ep.as_ref() else {
                    let _ = reply.send(Err("engine not opened".into()));
                    return;
                };
                // Per-connect client config: the SHIM-COURT mutation
                // (AQ_MUTATE_ALPN_DROP=1) lands HERE — no ALPN offered.
                let offer_alpn = !mutate_alpn_drop();
                let cfg = match client_config(&st.ca_der, offer_alpn) {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = reply.send(Err(e));
                        return;
                    }
                };
                match ep.connect_with(cfg, addr, &server_name) {
                    Ok(connecting) => {
                        // Bounded handshake (tokio timeout — never a sleep).
                        match tokio::time::timeout(OP_TIMEOUT, connecting).await {
                            Ok(Ok(conn)) => {
                                let id = st.next_id;
                                st.next_id += 1;
                                st.conns.insert(id, conn);
                                let _ = reply.send(Ok(id));
                            }
                            Ok(Err(e)) => {
                                eprintln!("AQ-DIAG connect handshake error: {e}");
                                let _ = reply.send(Err(format!("handshake: {e}")));
                            }
                            Err(_) => {
                                eprintln!("AQ-DIAG connect handshake timeout");
                                let _ = reply.send(Err("handshake timeout".into()));
                            }
                        }
                    }
                    Err(e) => {
                        let _ = reply.send(Err(format!("connect: {e}")));
                    }
                }
            }
            Job::Send { id, data, reply } => {
                let Some(conn) = st.conns.get(&id) else {
                    let _ = reply.send(Err(format!("no conn {id}")));
                    return;
                };
                let fut = async {
                    let mut s = conn.open_uni().await.map_err(|e| e.to_string())?;
                    s.write_all(&data).await.map_err(|e| e.to_string())?;
                    s.finish().map_err(|e| e.to_string())?;
                    Ok::<(), String>(())
                };
                match tokio::time::timeout(OP_TIMEOUT, fut).await {
                    Ok(r) => {
                        let _ = reply.send(r);
                    }
                    Err(_) => {
                        let _ = reply.send(Err("send timeout".into()));
                    }
                }
            }
            Job::Recv {
                id,
                timeout_ms,
                reply,
            } => {
                let Some(conn) = st.conns.get(&id) else {
                    let _ = reply.send(Err(format!("no conn {id}")));
                    return;
                };
                let budget = Duration::from_millis(timeout_ms);
                let fut = async {
                    // Accept the server's echo uni-stream, then hand back
                    // quinn's buffer: a SINGLE-chunk stream's Bytes move out
                    // untouched (zero-copy fast path at the boundary). A
                    // multi-chunk stream (payloads spanning QUIC packets,
                    // e.g. the court's 4096B echo at ~1424B/packet) drains
                    // to FIN and concatenates — one copy, disclosed.
                    let mut s = conn.accept_uni().await.map_err(|e| e.to_string())?;
                    let mut first: Option<Bytes> = None;
                    let mut acc: Vec<u8> = Vec::new();
                    loop {
                        match s.read_chunk(usize::MAX, true).await {
                            Ok(Some(chunk)) => {
                                if first.is_none() && acc.is_empty() {
                                    first = Some(chunk.bytes);
                                } else {
                                    if let Some(b) = first.take() {
                                        acc.extend_from_slice(&b);
                                    }
                                    acc.extend_from_slice(&chunk.bytes);
                                }
                            }
                            Ok(None) => break,
                            Err(e) => return Err(e.to_string()),
                        }
                    }
                    match (first, acc.is_empty()) {
                        (Some(b), _) => Ok(b),
                        (None, true) => Err("eof before data".to_string()),
                        (None, false) => Ok(Bytes::from(acc)),
                    }
                };
                match tokio::time::timeout(budget, fut).await {
                    Ok(r) => {
                        let _ = reply.send(r);
                    }
                    Err(_) => {
                        let _ = reply.send(Err("recv timeout".into()));
                    }
                }
            }
            Job::Close { id } => {
                st.conns.remove(&id);
            }
            Job::Serve {
                port,
                cert_der,
                key_der,
                reply,
            } => {
                // The mutation is a per-Serve decision (read PER CALL — the
                // fresh-process court law).
                let offer_alpn = !mutate_alpn_drop();
                let cfg = match server_config(cert_der, key_der, offer_alpn) {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = reply.send(Err(e));
                        return;
                    }
                };
                match quinn::Endpoint::server(cfg, SocketAddr::from(([127, 0, 0, 1], port))) {
                    Ok(ep) => {
                        st.listener = Some(ep);
                        let _ = reply.send(Ok(()));
                    }
                    Err(e) => {
                        let _ = reply.send(Err(format!("serve bind: {e}")));
                    }
                }
            }
            Job::Accept { timeout_ms, reply } => {
                let Some(ep) = st.listener.as_ref() else {
                    let _ = reply.send(Err("no listener".into()));
                    return;
                };
                let fut = async {
                    eprintln!("AQ-DIAG server accept waiting for incoming...");
                    let incoming = ep.accept().await.ok_or_else(|| "listener closed".to_string())?;
                    eprintln!("AQ-DIAG server incoming connection received, awaiting handshake...");
                    let conn = incoming.await.map_err(|e| e.to_string())?;
                    eprintln!("AQ-DIAG server connection established!");
                    Ok::<quinn::Connection, String>(conn)
                };
                match tokio::time::timeout(Duration::from_millis(timeout_ms), fut).await {
                    Ok(Ok(conn)) => {
                        let id = st.next_id;
                        st.next_id += 1;
                        st.conns.insert(id, conn);
                        let _ = reply.send(Ok(id));
                    }
                    Ok(Err(e)) => {
                        eprintln!("AQ-DIAG accept handshake error: {e}");
                        let _ = reply.send(Err(format!("accept: {e}")));
                    }
                    Err(_) => {
                        // Bounded timeout (no-sleep law): the mutant court
                        // RELIES on this deadline firing.
                        eprintln!("AQ-DIAG accept timeout firing");
                        let _ = reply.send(Err("accept timeout".into()));
                    }
                }
            }
            Job::OpenBi { conn_id, reply } => {
                let Some(conn) = st.conns.get(&conn_id) else {
                    let _ = reply.send(Err(format!("no conn {conn_id}")));
                    return;
                };
                let fut = async {
                    let (send, recv) = conn.open_bi().await.map_err(|e| e.to_string())?;
                    Ok::<_, String>((send, recv))
                };
                match tokio::time::timeout(OP_TIMEOUT, fut).await {
                    Ok(Ok((send, recv))) => {
                        let sid = st.next_stream_id;
                        st.next_stream_id += 1;
                        st.streams.insert((conn_id, sid), (recv, send));
                        let _ = reply.send(Ok(sid));
                    }
                    Ok(Err(e)) => {
                        let _ = reply.send(Err(e));
                    }
                    Err(_) => {
                        let _ = reply.send(Err("open_bi timeout".into()));
                    }
                }
            }
            Job::SendOn {
                conn_id,
                stream_id,
                data,
                reply,
            } => {
                let Some((_, send)) = st.streams.get_mut(&(conn_id, stream_id)) else {
                    let _ = reply.send(Err(format!("no stream ({conn_id},{stream_id})")));
                    return;
                };
                // write_all WITHOUT finish: bi streams persist (a finished
                // send side would signal end-of-stream to the peer).
                let fut = async {
                    send.write_all(&data).await.map_err(|e| e.to_string())
                };
                match tokio::time::timeout(OP_TIMEOUT, fut).await {
                    Ok(r) => {
                        let _ = reply.send(r);
                    }
                    Err(_) => {
                        let _ = reply.send(Err("send_on timeout".into()));
                    }
                }
            }
            Job::RecvOn {
                conn_id,
                stream_id,
                timeout_ms,
                reply,
            } => {
                let Some((recv, _)) = st.streams.get_mut(&(conn_id, stream_id)) else {
                    let _ = reply.send(Err(format!("no stream ({conn_id},{stream_id})")));
                    return;
                };
                let budget = Duration::from_millis(timeout_ms);
                let fut = async {
                    let chunk = recv
                        .read_chunk(usize::MAX, true)
                        .await
                        .map_err(|e| e.to_string())?
                        .ok_or_else(|| "eof before data".to_string())?;
                    Ok::<Bytes, String>(chunk.bytes)
                };
                match tokio::time::timeout(budget, fut).await {
                    Ok(r) => {
                        let _ = reply.send(r);
                    }
                    Err(_) => {
                        let _ = reply.send(Err("recv_on timeout".into()));
                    }
                }
            }
            Job::AcceptBi {
                conn_id,
                timeout_ms,
                reply,
            } => {
                let Some(conn) = st.conns.get(&conn_id) else {
                    let _ = reply.send(Err(format!("no conn {conn_id}")));
                    return;
                };
                let fut = async {
                    // quinn 0.11's accept_bi yields (SendStream, RecvStream)
                    // — the map value type is (RecvStream, SendStream).
                    conn.accept_bi().await.map(|(send, recv)| (send, recv)).map_err(|e| e.to_string())
                };
                match tokio::time::timeout(Duration::from_millis(timeout_ms), fut).await {
                    Ok(Ok((send, recv))) => {
                        let sid = st.next_stream_id;
                        st.next_stream_id += 1;
                        st.streams.insert((conn_id, sid), (recv, send));
                        let _ = reply.send(Ok(sid));
                    }
                    Ok(Err(e)) => {
                        let _ = reply.send(Err(e));
                    }
                    Err(_) => {
                        let _ = reply.send(Err("accept_bi timeout".into()));
                    }
                }
            }
            Job::CloseStream {
                conn_id,
                stream_id,
            } => {
                st.streams.remove(&(conn_id, stream_id));
            }
            Job::CloseListener => {
                st.listener = None;
            }
        }
    }

    /// Spawn the pinned engine thread and run the Open job to completion:
    /// a published engine resource is always an OPENED (bound) engine.
    fn spawn_engine_with_open(port: u16, ca_der: Vec<u8>) -> Result<Arc<EngineInner>, String> {
        let (tx, rx) = channel::<Job>();
        let (ready_tx, ready_rx) = channel::<Result<(), String>>();
        let ready_tx_thread = ready_tx.clone();
        std::thread::Builder::new()
            .name("aq-quic-engine".into())
            .spawn(move || {
                pin_current_thread_best_effort();
                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = ready_tx_thread.send(Err(format!("runtime: {e}")));
                        return;
                    }
                };
                // The runtime must be driven CONTINUOUSLY (the aq-quic
                // engine is the only thread allowed to touch quinn): a std
                // blocking rx.recv() between jobs would starve the runtime —
                // quinn's endpoint driver task would not be polled between
                // jobs, so inbound datagrams (handshake retries, echo
                // traffic) would sit unread while idle. A forwarder thread
                // feeds the std mpsc into an unbounded tokio channel, and
                // block_on runs the job loop forever on the engine thread.
                let (job_tx, mut job_rx) = tokio::sync::mpsc::unbounded_channel::<Job>();
                let fwd = std::thread::Builder::new()
                    .name("aq-quic-jobs".into())
                    .spawn(move || {
                        while let Ok(job) = rx.recv() {
                            if job_tx.send(job).is_err() {
                                break;
                            }
                        }
                    });
                if fwd.is_err() {
                    let _ = ready_tx_thread.send(Err("engine jobs thread".into()));
                    return;
                }
                rt.block_on(async {
                    let mut st = EngineState {
                        ep: None,
                        listener: None,
                        ca_der: Vec::new(),
                        conns: HashMap::new(),
                        streams: HashMap::new(),
                        next_id: 0,
                        next_stream_id: 0,
                    };
                    while let Some(job) = job_rx.recv().await {
                        run_job(&mut st, job).await;
                    }
                });
            })
            .map_err(|e| format!("engine thread: {e}"))?;
        // The Open job is the first in the queue; bounded wait on the reply
        // (no sleep).
        tx.send(Job::Open {
            port,
            ca_der,
            reply: ready_tx,
        })
        .map_err(|_| "engine died before open".to_string())?;
        match ready_rx.recv_timeout(OPEN_TIMEOUT) {
            Ok(r) => {
                if let Err(ref e) = r {
                    eprintln!("AQ-DIAG open failed: {e}");
                }
                r?;
                Ok(Arc::new(EngineInner { tx }))
            }
            Err(e) => {
                eprintln!("AQ-DIAG open wait: {e}");
                Err(format!("engine open wait: {e}"))
            }
        }
    }

    // -----------------------------------------------------------------------
    // NIF surface (all errors = the atom `error`, cgap_len convention)
    // -----------------------------------------------------------------------

    unsafe fn engine_of(env: *mut ErlNifEnv, term: ErlNifTerm) -> Option<*mut QuicEngineRes> {
        let ty = QUIC_ENGINE_TYPE.load(Ordering::Relaxed);
        if ty.is_null() {
            return None;
        }
        let mut obj: *mut c_void = std::ptr::null_mut();
        if quic_extern::enif_get_resource(env, term, ty, &mut obj) == 0 || obj.is_null() {
            return None;
        }
        Some(obj.cast::<QuicEngineRes>())
    }

    unsafe fn conn_of(env: *mut ErlNifEnv, term: ErlNifTerm) -> Option<*mut QuicConnRes> {
        let ty = QUIC_CONN_TYPE.load(Ordering::Relaxed);
        if ty.is_null() {
            return None;
        }
        let mut obj: *mut c_void = std::ptr::null_mut();
        if quic_extern::enif_get_resource(env, term, ty, &mut obj) == 0 || obj.is_null() {
            return None;
        }
        Some(obj.cast::<QuicConnRes>())
    }

    fn inspect_arg<'a>(env: *mut ErlNifEnv, term: ErlNifTerm) -> Option<&'a [u8]> {
        let mut bin = ErlNifBinary {
            size: 0,
            data: std::ptr::null_mut(),
            ref_bin: std::ptr::null_mut(),
            __spare__: [std::ptr::null_mut(); 2],
        };
        if unsafe { quic_extern::enif_inspect_binary(env, term, &mut bin) } == 0 {
            return None;
        }
        // SAFETY: enif_inspect_binary handed us bin.size readable bytes,
        // valid for the NIF call (the caller's term pins the binary).
        Some(unsafe { std::slice::from_raw_parts(bin.data, bin.size) })
    }

    /// Allocate + fully initialize + term a QUIC resource (the aq_slab
    /// alloc_slab discipline: init writes the complete value before the
    /// term exists; the NIF-local reference is released immediately after
    /// enif_make_resource — the term is the owner from then on).
    unsafe fn alloc_quic_res<T>(
        env: *mut ErlNifEnv,
        ty: *mut ErlNifResourceType,
        init: impl FnOnce(*mut c_void),
    ) -> Option<ErlNifTerm> {
        if ty.is_null() {
            return None;
        }
        let obj = quic_extern::enif_alloc_resource(ty, std::mem::size_of::<T>());
        if obj.is_null() {
            return None;
        }
        record_alloc(std::mem::size_of::<T>());
        init(obj);
        let term = quic_extern::enif_make_resource(env, obj);
        quic_extern::enif_release_resource(obj);
        Some(term)
    }

    /// `aq_quic_open(Port, CaCertDer) -> Engine :: reference() | error`.
    /// DirtyIo: spawns the pinned engine thread and binds the client
    /// endpoint (a real socket bind). `CaCertDer` is the trust anchor for
    /// server-cert verification (the court's self-signed test cert).
    #[no_mangle]
    pub(crate) extern "C" fn aq_quic_open(
        env: *mut ErlNifEnv,
        argc: c_int,
        argv: *const ErlNifTerm,
    ) -> ErlNifTerm {
        let args = unsafe { std::slice::from_raw_parts(argv, argc.max(0) as usize) };
        let mut port = 0i64;
        if argc != 2
            || unsafe { enif_get_long(env, args[0], &mut port) } == 0
            || !(0..=u16::MAX as i64).contains(&port)
        {
            return make_atom(env, "error");
        }
        let Some(ca_der) = inspect_arg(env, args[1]) else {
            return make_atom(env, "error");
        };
        let ca_der = ca_der.to_vec();
        let inner = match spawn_engine_with_open(port as u16, ca_der) {
            Ok(inner) => inner,
            Err(_) => return make_atom(env, "error"),
        };
        unsafe {
            alloc_quic_res::<QuicEngineRes>(env, QUIC_ENGINE_TYPE.load(Ordering::Relaxed), |obj| {
                obj.cast::<QuicEngineRes>().write(QuicEngineRes { inner });
            })
            .unwrap_or_else(|| make_atom(env, "error"))
        }
    }

    /// `aq_quic_connect(Engine, Addr, ServerName) -> Conn :: reference() |
    /// error`. DirtyIo: real TLS 1.3 QUIC handshake on the engine thread
    /// (bounded 5 s tokio timeout). Addr is "ip:port" text; ServerName is
    /// the SNI. `AQ_MUTATE_ALPN_DROP=1` is the SHIM-COURT mutation: no ALPN
    /// offered, an aq/1-requiring peer must refuse.
    #[no_mangle]
    pub(crate) extern "C" fn aq_quic_connect(
        env: *mut ErlNifEnv,
        argc: c_int,
        argv: *const ErlNifTerm,
    ) -> ErlNifTerm {
        let args = unsafe { std::slice::from_raw_parts(argv, argc.max(0) as usize) };
        if argc != 3 {
            return make_atom(env, "error");
        }
        let Some(engine) = (unsafe { engine_of(env, args[0]) }) else {
            return make_atom(env, "error");
        };
        let Some(addr) = inspect_arg(env, args[1]) else {
            return make_atom(env, "error");
        };
        let Some(name) = inspect_arg(env, args[2]) else {
            return make_atom(env, "error");
        };
        let Ok(addr) = std::str::from_utf8(addr)
            .map_err(|_| ())
            .and_then(|s| s.parse::<SocketAddr>().map_err(|_| ()))
        else {
            return make_atom(env, "error");
        };
        let Ok(name) = std::str::from_utf8(name).map(|s| s.to_string()) else {
            return make_atom(env, "error");
        };
        // SAFETY: engine_of validated the resource type; the borrow is
        // confined to this NIF call (the argument term pins the resource).
        let inner = unsafe { Arc::clone(&*std::ptr::addr_of!((*engine).inner)) };
        let (reply_tx, reply_rx) = channel();
        if inner
            .tx
            .send(Job::Connect {
                addr,
                server_name: name,
                reply: reply_tx,
            })
            .is_err()
        {
            return make_atom(env, "error");
        }
        match reply_rx.recv() {
            Ok(Ok(id)) => unsafe {
                alloc_quic_res::<QuicConnRes>(env, QUIC_CONN_TYPE.load(Ordering::Relaxed), |obj| {
                    obj.cast::<QuicConnRes>()
                        .write(QuicConnRes { inner, id });
                })
                .unwrap_or_else(|| make_atom(env, "error"))
            },
            _ => make_atom(env, "error"),
        }
    }

    /// `aq_quic_send(Conn, Data) -> ok | error`. DirtyIo: one uni-stream
    /// write on the engine thread (bounded 5 s tokio timeout).
    #[no_mangle]
    pub(crate) extern "C" fn aq_quic_send(
        env: *mut ErlNifEnv,
        argc: c_int,
        argv: *const ErlNifTerm,
    ) -> ErlNifTerm {
        let args = unsafe { std::slice::from_raw_parts(argv, argc.max(0) as usize) };
        if argc != 2 {
            return make_atom(env, "error");
        }
        let Some(conn) = (unsafe { conn_of(env, args[0]) }) else {
            return make_atom(env, "error");
        };
        let Some(data) = inspect_arg(env, args[1]) else {
            return make_atom(env, "error");
        };
        // SAFETY: conn_of validated the resource type (see aq_quic_connect).
        let inner = unsafe { Arc::clone(&*std::ptr::addr_of!((*conn).inner)) };
        let id = unsafe { std::ptr::addr_of!((*conn).id).read() };
        let (reply_tx, reply_rx) = channel();
        let data = data.to_vec();
        if inner
            .tx
            .send(Job::Send {
                id,
                data,
                reply: reply_tx,
            })
            .is_err()
        {
            return make_atom(env, "error");
        }
        match reply_rx.recv() {
            Ok(Ok(())) => make_atom(env, "ok"),
            _ => make_atom(env, "error"),
        }
    }

    /// `aq_quic_recv(Conn, TimeoutMs) -> Binary :: binary() | error`.
    /// DirtyIo. ZERO-COPY EGRESS: the returned binary is a resource binary
    /// whose bytes ARE quinn's received buffer — `enif_make_resource_binary`
    /// over `Bytes::as_ptr()` — no payload bytes are copied at the NIF
    /// boundary (the QuicBuf resource owns the Bytes; its dtor reclaims).
    #[no_mangle]
    pub(crate) extern "C" fn aq_quic_recv(
        env: *mut ErlNifEnv,
        argc: c_int,
        argv: *const ErlNifTerm,
    ) -> ErlNifTerm {
        let args = unsafe { std::slice::from_raw_parts(argv, argc.max(0) as usize) };
        let mut timeout_ms = 0i64;
        if argc != 2
            || unsafe { enif_get_long(env, args[1], &mut timeout_ms) } == 0
            || !(0..=60_000).contains(&timeout_ms)
        {
            return make_atom(env, "error");
        }
        let Some(conn) = (unsafe { conn_of(env, args[0]) }) else {
            return make_atom(env, "error");
        };
        // SAFETY: conn_of validated the resource type (see aq_quic_connect).
        let inner = unsafe { Arc::clone(&*std::ptr::addr_of!((*conn).inner)) };
        let id = unsafe { std::ptr::addr_of!((*conn).id).read() };
        let (reply_tx, reply_rx) = channel();
        if inner
            .tx
            .send(Job::Recv {
                id,
                timeout_ms: timeout_ms as u64,
                reply: reply_tx,
            })
            .is_err()
        {
            return make_atom(env, "error");
        }
        let Ok(Ok(bytes)) = reply_rx.recv() else {
            return make_atom(env, "error");
        };
        // Project quinn's buffer: the binary term pins the QuicBuf resource,
        // the resource owns the Bytes — zero bytes copied here (resbin.rs
        // ledger law: a projection must not show up as an allocation). The
        // ONLY counted alloc on this path is the one QuicBuf resource block
        // (record_alloc above) — the BEAM court asserts exactly that census.
        unsafe { project_bytes(env, bytes) }
    }

    /// Zero-copy egress, shared by the client and stream recv paths: the
    /// binary term pins a QuicBuf resource whose Bytes ARE quinn's received
    /// buffer (no payload bytes copied at the NIF boundary).
    unsafe fn project_bytes(env: *mut ErlNifEnv, bytes: Bytes) -> ErlNifTerm {
        let ptr = bytes.as_ptr();
        let len = bytes.len();
        unsafe {
            let obj = quic_extern::enif_alloc_resource(
                QUIC_BUF_TYPE.load(Ordering::Relaxed),
                std::mem::size_of::<QuicBufRes>(),
            );
            if obj.is_null() {
                return make_atom(env, "error");
            }
            record_alloc(std::mem::size_of::<QuicBufRes>());
            obj.cast::<QuicBufRes>().write(QuicBufRes { bytes });
            let term =
                quic_extern::enif_make_resource_binary(env, obj, ptr as *const c_void, len);
            quic_extern::enif_release_resource(obj);
            term
        }
    }

    /// Listener-handle borrow (the engine_of/conn_of discipline).
    unsafe fn listener_of(env: *mut ErlNifEnv, term: ErlNifTerm) -> Option<*mut QuicListenerRes> {
        let ty = QUIC_LISTENER_TYPE.load(Ordering::Relaxed);
        if ty.is_null() {
            return None;
        }
        let mut obj: *mut c_void = std::ptr::null_mut();
        if quic_extern::enif_get_resource(env, term, ty, &mut obj) == 0 || obj.is_null() {
            return None;
        }
        Some(obj.cast::<QuicListenerRes>())
    }

    /// Stream-handle borrow.
    unsafe fn stream_of(env: *mut ErlNifEnv, term: ErlNifTerm) -> Option<*mut QuicStreamRes> {
        let ty = QUIC_STREAM_TYPE.load(Ordering::Relaxed);
        if ty.is_null() {
            return None;
        }
        let mut obj: *mut c_void = std::ptr::null_mut();
        if quic_extern::enif_get_resource(env, term, ty, &mut obj) == 0 || obj.is_null() {
            return None;
        }
        Some(obj.cast::<QuicStreamRes>())
    }

    /// Send-path argument inspect: iolist first (binaries + 0..255
    /// integers, improper tails included), plain binary fallback. Returns
    /// an owned copy for the Job (the engine thread outlives the NIF call).
    fn inspect_iolist(env: *mut ErlNifEnv, term: ErlNifTerm) -> Option<Vec<u8>> {
        let mut bin = ErlNifBinary {
            size: 0,
            data: std::ptr::null_mut(),
            ref_bin: std::ptr::null_mut(),
            __spare__: [std::ptr::null_mut(); 2],
        };
        unsafe {
            if quic_extern::enif_inspect_iolist_as_binary(env, term, &mut bin) != 0 {
                let bytes =
                    std::slice::from_raw_parts(bin.data, bin.size).to_vec();
                quic_extern::enif_release_binary(&mut bin);
                return Some(bytes);
            }
            if quic_extern::enif_inspect_binary(env, term, &mut bin) != 0 {
                return Some(std::slice::from_raw_parts(bin.data, bin.size).to_vec());
            }
        }
        None
    }

    /// `aq_quic_serve(Engine, Port, CertDer, KeyDer) -> Listener ::
    /// reference() | error`. DirtyIo: rustls ServerConfig build + real
    /// UDP bind on the engine thread. ALPN `aq/1` is REQUIRED server-side
    /// (SHIM-COURT mutation `AQ_MUTATE_ALPN_DROP=1` strips it — the
    /// handshake must then refuse).
    #[no_mangle]
    pub(crate) extern "C" fn aq_quic_serve(
        env: *mut ErlNifEnv,
        argc: c_int,
        argv: *const ErlNifTerm,
    ) -> ErlNifTerm {
        let args = unsafe { std::slice::from_raw_parts(argv, argc.max(0) as usize) };
        let mut port = 0i64;
        if argc != 4
            || unsafe { enif_get_long(env, args[1], &mut port) } == 0
            || !(0..=u16::MAX as i64).contains(&port)
        {
            return make_atom(env, "error");
        }
        let Some(engine) = (unsafe { engine_of(env, args[0]) }) else {
            return make_atom(env, "error");
        };
        let Some(cert_der) = inspect_arg(env, args[2]) else {
            return make_atom(env, "error");
        };
        let Some(key_der) = inspect_arg(env, args[3]) else {
            return make_atom(env, "error");
        };
        // SAFETY: engine_of validated the resource type (aq_quic_connect).
        let inner = unsafe { Arc::clone(&*std::ptr::addr_of!((*engine).inner)) };
        let (reply_tx, reply_rx) = channel();
        if inner
            .tx
            .send(Job::Serve {
                port: port as u16,
                cert_der: cert_der.to_vec(),
                key_der: key_der.to_vec(),
                reply: reply_tx,
            })
            .is_err()
        {
            return make_atom(env, "error");
        }
        match reply_rx.recv() {
            Ok(Ok(())) => unsafe {
                alloc_quic_res::<QuicListenerRes>(
                    env,
                    QUIC_LISTENER_TYPE.load(Ordering::Relaxed),
                    |obj| {
                        obj.cast::<QuicListenerRes>()
                            .write(QuicListenerRes { inner: inner.clone() })
                    },
                )
                .unwrap_or_else(|| make_atom(env, "error"))
            },
            _ => make_atom(env, "error"),
        }
    }

    /// `aq_quic_accept(Listener, TimeoutMs) -> Conn :: reference() |
    /// error`. DirtyIo: bounded server-side accept of one inbound QUIC
    /// connection (the mutant court relies on the deadline firing).
    #[no_mangle]
    pub(crate) extern "C" fn aq_quic_accept(
        env: *mut ErlNifEnv,
        argc: c_int,
        argv: *const ErlNifTerm,
    ) -> ErlNifTerm {
        let args = unsafe { std::slice::from_raw_parts(argv, argc.max(0) as usize) };
        let mut timeout_ms = 0i64;
        if argc != 2
            || unsafe { enif_get_long(env, args[1], &mut timeout_ms) } == 0
            || !(0..=60_000).contains(&timeout_ms)
        {
            return make_atom(env, "error");
        }
        let Some(listener) = (unsafe { listener_of(env, args[0]) }) else {
            return make_atom(env, "error");
        };
        // SAFETY: listener_of validated the resource type.
        let inner = unsafe { Arc::clone(&*std::ptr::addr_of!((*listener).inner)) };
        let (reply_tx, reply_rx) = channel();
        if inner
            .tx
            .send(Job::Accept {
                timeout_ms: timeout_ms as u64,
                reply: reply_tx,
            })
            .is_err()
        {
            return make_atom(env, "error");
        }
        match reply_rx.recv() {
            Ok(Ok(id)) => unsafe {
                alloc_quic_res::<QuicConnRes>(env, QUIC_CONN_TYPE.load(Ordering::Relaxed), |obj| {
                    obj.cast::<QuicConnRes>().write(QuicConnRes { inner, id });
                })
                .unwrap_or_else(|| make_atom(env, "error"))
            },
            _ => make_atom(env, "error"),
        }
    }

    /// `aq_quic_accept_bi(Conn, TimeoutMs) -> Stream :: reference() |
    /// error`. DirtyIo: accept a peer-initiated bi-directional stream.
    #[no_mangle]
    pub(crate) extern "C" fn aq_quic_accept_bi(
        env: *mut ErlNifEnv,
        argc: c_int,
        argv: *const ErlNifTerm,
    ) -> ErlNifTerm {
        let args = unsafe { std::slice::from_raw_parts(argv, argc.max(0) as usize) };
        let mut timeout_ms = 0i64;
        if argc != 2
            || unsafe { enif_get_long(env, args[1], &mut timeout_ms) } == 0
            || !(0..=60_000).contains(&timeout_ms)
        {
            return make_atom(env, "error");
        }
        let Some(conn) = (unsafe { conn_of(env, args[0]) }) else {
            return make_atom(env, "error");
        };
        let inner = unsafe { Arc::clone(&*std::ptr::addr_of!((*conn).inner)) };
        let id = unsafe { std::ptr::addr_of!((*conn).id).read() };
        let (reply_tx, reply_rx) = channel();
        if inner
            .tx
            .send(Job::AcceptBi {
                conn_id: id,
                timeout_ms: timeout_ms as u64,
                reply: reply_tx,
            })
            .is_err()
        {
            return make_atom(env, "error");
        }
        stream_reply(env, inner, id, reply_rx)
    }

    /// `aq_quic_stream_open(Conn) -> Stream :: reference() | error`.
    /// DirtyIo: open one client-initiated bi-directional stream.
    #[no_mangle]
    pub(crate) extern "C" fn aq_quic_stream_open(
        env: *mut ErlNifEnv,
        argc: c_int,
        argv: *const ErlNifTerm,
    ) -> ErlNifTerm {
        let args = unsafe { std::slice::from_raw_parts(argv, argc.max(0) as usize) };
        if argc != 1 {
            return make_atom(env, "error");
        }
        let Some(conn) = (unsafe { conn_of(env, args[0]) }) else {
            return make_atom(env, "error");
        };
        let inner = unsafe { Arc::clone(&*std::ptr::addr_of!((*conn).inner)) };
        let id = unsafe { std::ptr::addr_of!((*conn).id).read() };
        let (reply_tx, reply_rx) = channel();
        if inner.tx.send(Job::OpenBi { conn_id: id, reply: reply_tx }).is_err() {
            return make_atom(env, "error");
        }
        stream_reply(env, inner, id, reply_rx)
    }

    /// `aq_quic_stream_send(Stream, IoData) -> ok | error`. DirtyIo.
    /// IoData may be a binary OR an iolist (enif_inspect_iolist_as_binary,
    /// send-path iolist support). write_all, NO finish — bi streams
    /// persist for further frames.
    #[no_mangle]
    pub(crate) extern "C" fn aq_quic_stream_send(
        env: *mut ErlNifEnv,
        argc: c_int,
        argv: *const ErlNifTerm,
    ) -> ErlNifTerm {
        let args = unsafe { std::slice::from_raw_parts(argv, argc.max(0) as usize) };
        if argc != 2 {
            return make_atom(env, "error");
        }
        let Some(stream) = (unsafe { stream_of(env, args[0]) }) else {
            return make_atom(env, "error");
        };
        let Some(data) = inspect_iolist(env, args[1]) else {
            return make_atom(env, "error");
        };
        let inner = unsafe { Arc::clone(&*std::ptr::addr_of!((*stream).inner)) };
        let conn_id = unsafe { std::ptr::addr_of!((*stream).conn_id).read() };
        let stream_id = unsafe { std::ptr::addr_of!((*stream).stream_id).read() };
        let (reply_tx, reply_rx) = channel();
        if inner
            .tx
            .send(Job::SendOn {
                conn_id,
                stream_id,
                data,
                reply: reply_tx,
            })
            .is_err()
        {
            return make_atom(env, "error");
        }
        match reply_rx.recv() {
            Ok(Ok(())) => make_atom(env, "ok"),
            _ => make_atom(env, "error"),
        }
    }

    /// `aq_quic_stream_recv(Stream, TimeoutMs) -> Binary :: binary() |
    /// error`. DirtyIo. ZERO-COPY EGRESS (the project_bytes law — same
    /// QuicBufRes census as aq_quic_recv).
    #[no_mangle]
    pub(crate) extern "C" fn aq_quic_stream_recv(
        env: *mut ErlNifEnv,
        argc: c_int,
        argv: *const ErlNifTerm,
    ) -> ErlNifTerm {
        let args = unsafe { std::slice::from_raw_parts(argv, argc.max(0) as usize) };
        let mut timeout_ms = 0i64;
        if argc != 2
            || unsafe { enif_get_long(env, args[1], &mut timeout_ms) } == 0
            || !(0..=60_000).contains(&timeout_ms)
        {
            return make_atom(env, "error");
        }
        let Some(stream) = (unsafe { stream_of(env, args[0]) }) else {
            return make_atom(env, "error");
        };
        let inner = unsafe { Arc::clone(&*std::ptr::addr_of!((*stream).inner)) };
        let conn_id = unsafe { std::ptr::addr_of!((*stream).conn_id).read() };
        let stream_id = unsafe { std::ptr::addr_of!((*stream).stream_id).read() };
        let (reply_tx, reply_rx) = channel();
        if inner
            .tx
            .send(Job::RecvOn {
                conn_id,
                stream_id,
                timeout_ms: timeout_ms as u64,
                reply: reply_tx,
            })
            .is_err()
        {
            return make_atom(env, "error");
        }
        let Ok(Ok(bytes)) = reply_rx.recv() else {
            return make_atom(env, "error");
        };
        unsafe { project_bytes(env, bytes) }
    }

    /// `aq_quic_stream_close(Stream) -> ok | error`. DirtyIo (engine
    /// round-trip; the drops do the wire work — recv STOP + send RESET).
    #[no_mangle]
    pub(crate) extern "C" fn aq_quic_stream_close(
        env: *mut ErlNifEnv,
        argc: c_int,
        argv: *const ErlNifTerm,
    ) -> ErlNifTerm {
        let args = unsafe { std::slice::from_raw_parts(argv, argc.max(0) as usize) };
        if argc != 1 {
            return make_atom(env, "error");
        }
        let Some(stream) = (unsafe { stream_of(env, args[0]) }) else {
            return make_atom(env, "error");
        };
        let inner = unsafe { Arc::clone(&*std::ptr::addr_of!((*stream).inner)) };
        let conn_id = unsafe { std::ptr::addr_of!((*stream).conn_id).read() };
        let stream_id = unsafe { std::ptr::addr_of!((*stream).stream_id).read() };
        // Fire-and-forget: the Job carries no reply (the engine's map
        // removal is the entire effect; the drop semantics do the wire
        // work). Send failure (dead engine) is the atom `error`.
        if inner
            .tx
            .send(Job::CloseStream {
                conn_id,
                stream_id,
            })
            .is_err()
        {
            return make_atom(env, "error");
        }
        make_atom(env, "ok")
    }

    /// Shared stream-resource reply path (alloc + term or error).
    fn stream_reply(
        env: *mut ErlNifEnv,
        inner: Arc<EngineInner>,
        conn_id: u64,
        reply_rx: std::sync::mpsc::Receiver<Result<u64, String>>,
    ) -> ErlNifTerm {
        match reply_rx.recv() {
            Ok(Ok(sid)) => unsafe {
                alloc_quic_res::<QuicStreamRes>(
                    env,
                    QUIC_STREAM_TYPE.load(Ordering::Relaxed),
                    |obj| {
                        obj.cast::<QuicStreamRes>().write(QuicStreamRes {
                            inner,
                            conn_id,
                            stream_id: sid,
                        })
                    },
                )
                .unwrap_or_else(|| make_atom(env, "error"))
            },
            _ => make_atom(env, "error"),
        }
    }
}

#[cfg(not(feature = "quic"))]
mod imp {
    use crate::{ErlNifEnv, ErlNifTerm, make_atom};
    use std::os::raw::c_int;

    /// Loud skip (gate exit-code law): without the `quic` feature the NIF
    /// surface is present in the func table but every call answers the
    /// `error` atom — never a silent fake pass, never a load failure.
    pub(crate) fn open_quic_types(_env: *mut ErlNifEnv) -> bool {
        true
    }

    macro_rules! quic_stub {
        ($name:ident) => {
            #[no_mangle]
            pub(crate) extern "C" fn $name(
                env: *mut ErlNifEnv,
                _argc: c_int,
                _argv: *const ErlNifTerm,
            ) -> ErlNifTerm {
                make_atom(env, "error")
            }
        };
    }

    quic_stub!(aq_quic_open);
    quic_stub!(aq_quic_connect);
    quic_stub!(aq_quic_send);
    quic_stub!(aq_quic_recv);
    quic_stub!(aq_quic_serve);
    quic_stub!(aq_quic_accept);
    quic_stub!(aq_quic_accept_bi);
    quic_stub!(aq_quic_stream_open);
    quic_stub!(aq_quic_stream_send);
    quic_stub!(aq_quic_stream_recv);
    quic_stub!(aq_quic_stream_close);
}

pub(crate) use imp::*;
