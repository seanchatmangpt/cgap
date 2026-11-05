//! L50 BEAM contract court (aq-quic-transport-decision.md option 2 scaffold).
//!
//! One Rust integration test drives the REAL dylib in a REAL BEAM via
//! escript against a REAL quinn echo server over REAL loopback UDP — real
//! TLS 1.3 QUIC handshake, no mocks (Chicago law):
//!
//! 1. handshake + echo roundtrip through aq_quic_open/connect/send/recv;
//! 2. ZERO-COPY egress census (alloc-spy class, declared): cgap_alloc_stats
//!    snapshot-after-send -> post-recv delta must be {1 alloc, < 128 bytes}
//!    — the one QuicBuf resource block, NO payload-sized allocation at the
//!    NIF boundary (the bytes are enif_make_resource_binary over
//!    Bytes::as_ptr(), aq_quic.rs recv path — a code fact, cited). Honest
//!    scope: quinn's kernel->user receive still copies once; the zero-copy
//!    claim is the NIF->BEAM boundary, per the decision record.
//! 3. refusals: bad open/connect args, bounded recv timeout (no-sleep law);
//! 4. SHIM-COURT anti-vacuity mutation AQ_MUTATE_ALPN_DROP=1: the same
//!    connect MUST refuse (handshake error) — a court that cannot flip is
//!    FAIL. Mutation is a fresh-process env flag (aq_slab_contract pattern).
//! 5. SERVER ROLE (this lane's second leg): the NIF's aq_quic_serve/
//!    accept/accept_bi/stream_send/stream_recv/stream_close vs a PURE-RUST
//!    quinn client over real loopback UDP — bi-stream frame roundtrip, and
//!    the same ALPN mutation flips the NEW leg: server-side ALPN stripped
//!    -> the quinn client's handshake must refuse (no_application_protocol
//!    class) AND the engine-side accept must time out (bounded).
//!
//! Requires: cargo build -p cgap-beam --features quic first (the court
//! loads the built dylib).
#![cfg(feature = "quic")]

use std::process::Command;

/// Locate the built NIF dylib (aq_slab_contract.rs precedent).
fn nif_dylib_path() -> std::path::PathBuf {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target_root = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join("../../target"));
    for profile in ["debug", "release"] {
        for ext in ["dylib", "so"] {
            let p = target_root.join(profile).join(format!("libcgap_beam.{ext}"));
            if p.exists() {
                return p;
            }
        }
    }
    panic!("build the dylib first: cargo build -p cgap-beam --features quic");
}

/// Real quinn echo server: self-signed cert, ALPN aq/1, one uni-stream
/// echo per accepted uni-stream. Pure-Rust tokio (the real collaborator).
struct EchoServer {
    addr: String,
    cert_path: std::path::PathBuf,
}

fn spawn_echo_server() -> EchoServer {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])
        .expect("rcgen self-signed cert");
    let cert_der = cert.cert.der().clone();
    let key_der = cert.key_pair.serialize_der();
    let cert_path =
        std::env::temp_dir().join(format!("aq_quic_court_cert_{}", std::process::id()));
    std::fs::write(&cert_path, cert_der.as_ref()).expect("write cert der");
    let (addr_tx, addr_rx) = std::sync::mpsc::channel::<Result<String, String>>();
    std::thread::Builder::new()
        .name("aq-quic-court-server".into())
        .spawn(move || {
            run_server(cert_der.as_ref().to_vec(), key_der, addr_tx);
        })
        .expect("server thread");
    let addr = addr_rx
        .recv_timeout(std::time::Duration::from_secs(10))
        .expect("server addr")
        .expect("server bind");
    EchoServer { addr, cert_path }
}

/// Server body: bind, report the address, then echo uni-streams forever.
/// `addr_tx` delivers the bound address once (bounded wait in the caller).
fn run_server(
    cert_der: Vec<u8>,
    key_der: Vec<u8>,
    addr_tx: std::sync::mpsc::Sender<Result<String, String>>,
) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
    let rt = match rt {
        Ok(rt) => rt,
        Err(e) => {
            let _ = addr_tx.send(Err(format!("runtime: {e}")));
            return;
        }
    };
    rt.block_on(async move {
        let mut server_crypto = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                vec![rustls::pki_types::CertificateDer::from(cert_der)],
                rustls::pki_types::PrivateKeyDer::Pkcs8(
                    rustls::pki_types::PrivatePkcs8KeyDer::from(key_der),
                ),
            );
        let server_crypto = match server_crypto {
            Ok(c) => c,
            Err(e) => {
                let _ = addr_tx.send(Err(format!("server cert: {e}")));
                return;
            }
        };
        // ALPN aq/1 REQUIRED server-side (the court's echo server law).
        let mut server_crypto = server_crypto;
        server_crypto.alpn_protocols = vec![b"aq/1".to_vec()];
        let server_crypto = match quinn::crypto::rustls::QuicServerConfig::try_from(server_crypto) {
            Ok(c) => c,
            Err(e) => {
                let _ = addr_tx.send(Err(format!("quic server config: {e}")));
                return;
            }
        };
        let endpoint = quinn::Endpoint::server(
            quinn::ServerConfig::with_crypto(std::sync::Arc::new(server_crypto)),
            "127.0.0.1:0".parse().unwrap(),
        );
        let endpoint = match endpoint {
            Ok(ep) => ep,
            Err(e) => {
                let _ = addr_tx.send(Err(format!("bind: {e}")));
                return;
            }
        };
        let addr = match endpoint.local_addr() {
            Ok(a) => a,
            Err(e) => {
                let _ = addr_tx.send(Err(format!("local_addr: {e}")));
                return;
            }
        };
        let _ = addr_tx.send(Ok(addr.to_string()));
        // Echo loop: one uni-stream in -> one uni-stream out, forever.
        while let Some(incoming) = endpoint.accept().await {
            let conn = match incoming.await {
                Ok(c) => c,
                Err(_) => continue,
            };
            echo_conn(conn).await;
        }
    });
}

async fn echo_conn(conn: quinn::Connection) {
    while let Ok(mut recv) = conn.accept_uni().await {
        // Bounded echo read (no-sleep law: read_to_end bounds at 1 MiB).
        let data = match recv.read_to_end(1024 * 1024).await {
            Ok(d) => d,
            Err(_) => continue,
        };
        if let Ok(mut send) = conn.open_uni().await {
            if send.write_all(&data).await.is_ok() {
                let _ = send.finish();
            }
        }
    }
}

/// Spawn escript with the dylib (fresh process per court run — the
/// mutation env flag is per-process, aq_slab_contract pattern).
fn run_court(
    dylib: &std::path::PathBuf,
    addr: &str,
    cert_path: &std::path::PathBuf,
    stem: &str,
    mutate: bool,
) -> i32 {
    let escript_path = std::env::temp_dir().join(stem);
    std::fs::write(&escript_path, ESCRIPT).expect("write escript");
    let mut cmd = Command::new("escript");
    cmd.arg(&escript_path).arg(dylib).arg(addr).arg(cert_path);
    if mutate {
        cmd.env("AQ_MUTATE_ALPN_DROP", "1");
    }
    let out = cmd.output().expect("spawn escript");
    if !out.status.success() {
        eprintln!("ESCRIPT_STATUS={:?} stdout={}B stderr={}B", out.status,
                  out.stdout.len(), out.stderr.len());
    }
    if !out.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&out.stdout));
    }
    if !out.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&out.stderr));
    }
    out.status.code().unwrap_or(1)
}

#[test]
fn handshake_echo_zero_copy_and_mutation_flips() {
    let server = spawn_echo_server();
    let dylib = nif_dylib_path();
    let stem = format!("cgap_beam_quic_{}", std::process::id());
    // Baseline: handshake + echo + zero-copy census must pass (exit 0).
    let rc = run_court(&dylib, &server.addr, &server.cert_path, &stem, false);
    assert_eq!(rc, 0, "baseline court failed (see stdout above)");
    // SHIM-COURT flip: the same court under AQ_MUTATE_ALPN_DROP=1 must
    // FAIL — a green mutant run is VACUOUS.
    let rc = run_court(&dylib, &server.addr, &server.cert_path, &stem, true);
    assert_eq!(rc, 1, "mutant court passed — VACUOUS COURT");
}

const ESCRIPT: &str = r##"
-module(cgap_beam).
-export([main/1, aq_slab_open/1, aq_slab_frame/2, aq_slab_window/4,
         aq_slab_stamp/2, aq_slab_lease/2, aq_slab_sync/1,
         aq_quic_open/2, aq_quic_connect/3, aq_quic_send/2, aq_quic_recv/2,
         aq_quic_serve/4, aq_quic_accept/2, aq_quic_accept_bi/2,
         aq_quic_stream_open/1, aq_quic_stream_send/2,
         aq_quic_stream_recv/2, aq_quic_stream_close/1,
         cgap_new/2, cgap_new_fast/2, cgap_new_i64/2, cgap_len/1,
         cgap_get/2, cgap_get_i64/2, cgap_stats/1, cgap_free/1,
         cgap_version/0, cgap_envelope_new/1, cgap_envelope_len/1,
         cgap_batch_binary/1, pointer_of/1, cgap_binary_data_pointer/1,
         cgap_dtor_stats/0, cgap_alloc_stats/0, cgap_flush_to_parquet/1,
         cgap_seal_to_packfile/1, cgap_receipt_query/1, cgap_open/1,
         cgap_scan/3]).
%% OTP 28 load_nif law: EVERY ErlNifFunc-table entry must be exported by
%% the calling module, else `bad_lib: Function not found` (witnessed:
%% cgap_new/2 and aq_quic_open/2 refusals). Full 38-entry stub list.
aq_slab_open(_) -> erlang:nif_error(nif_not_loaded).
aq_slab_frame(_, _) -> erlang:nif_error(nif_not_loaded).
aq_slab_window(_, _, _, _) -> erlang:nif_error(nif_not_loaded).
aq_slab_stamp(_, _) -> erlang:nif_error(nif_not_loaded).
aq_slab_lease(_, _) -> erlang:nif_error(nif_not_loaded).
aq_slab_sync(_) -> erlang:nif_error(nif_not_loaded).
aq_quic_open(_, _) -> erlang:nif_error(nif_not_loaded).
aq_quic_connect(_, _, _) -> erlang:nif_error(nif_not_loaded).
aq_quic_send(_, _) -> erlang:nif_error(nif_not_loaded).
aq_quic_recv(_, _) -> erlang:nif_error(nif_not_loaded).
aq_quic_serve(_, _, _, _) -> erlang:nif_error(nif_not_loaded).
aq_quic_accept(_, _) -> erlang:nif_error(nif_not_loaded).
aq_quic_accept_bi(_, _) -> erlang:nif_error(nif_not_loaded).
aq_quic_stream_open(_) -> erlang:nif_error(nif_not_loaded).
aq_quic_stream_send(_, _) -> erlang:nif_error(nif_not_loaded).
aq_quic_stream_recv(_, _) -> erlang:nif_error(nif_not_loaded).
aq_quic_stream_close(_) -> erlang:nif_error(nif_not_loaded).
cgap_new(_, _) -> erlang:nif_error(nif_not_loaded).
cgap_new_fast(_, _) -> erlang:nif_error(nif_not_loaded).
cgap_new_i64(_, _) -> erlang:nif_error(nif_not_loaded).
cgap_len(_) -> erlang:nif_error(nif_not_loaded).
cgap_get(_, _) -> erlang:nif_error(nif_not_loaded).
cgap_get_i64(_, _) -> erlang:nif_error(nif_not_loaded).
cgap_stats(_) -> erlang:nif_error(nif_not_loaded).
cgap_free(_) -> erlang:nif_error(nif_not_loaded).
cgap_version() -> erlang:nif_error(nif_not_loaded).
cgap_envelope_new(_) -> erlang:nif_error(nif_not_loaded).
cgap_envelope_len(_) -> erlang:nif_error(nif_not_loaded).
cgap_batch_binary(_) -> erlang:nif_error(nif_not_loaded).
pointer_of(_) -> erlang:nif_error(nif_not_loaded).
cgap_binary_data_pointer(_) -> erlang:nif_error(nif_not_loaded).
cgap_dtor_stats() -> erlang:nif_error(nif_not_loaded).
cgap_alloc_stats() -> erlang:nif_error(nif_not_loaded).
cgap_flush_to_parquet(_) -> erlang:nif_error(nif_not_loaded).
cgap_seal_to_packfile(_) -> erlang:nif_error(nif_not_loaded).
cgap_receipt_query(_) -> erlang:nif_error(nif_not_loaded).
cgap_open(_) -> erlang:nif_error(nif_not_loaded).
cgap_scan(_, _, _) -> erlang:nif_error(nif_not_loaded).

main([NifPath0, Addr, CertPath]) ->
    %% OTP load law: stage the dylib as <stem>.so, load the suffix-less
    %% stem path (bridge_contract precedent).
    Stem = filename:join([tmp_base(), "libcgap_beam_quic_court"]),
    {ok, _} = file:copy(NifPath0, Stem ++ ".so"),
    %% macOS 26 law: a copy into /tmp carries com.apple.provenance, and the
    %% BEAM's dlopen refuses it with an empty reason. Clear xattrs after
    %% staging (xattr may not exist on Linux — os:cmd failure is fine).
    os:cmd("xattr -c " ++ Stem ++ ".so"),
    ok = case erlang:load_nif(Stem, 0) of
             ok -> ok;
             {error, R} -> erlang:error({load_nif_failed, R})
         end,
    {ok, CaDer} = file:read_file(CertPath),
    Res = try
              case os:getenv("AQ_MUTATE_ALPN_DROP") of
                  "1" -> t_alpn_mutant(Addr, CaDer);
                  _ -> contracts(Addr, CaDer)
              end
          of
              ok -> 0
          catch Class:Reason:Stack ->
              io:format("COURT_FAIL ~p:~p~n~p~n", [Class, Reason, Stack]),
              1
          end,
    halt(Res).

tmp_base() -> "/tmp".

%% SHIM-COURT leg: connect with no ALPN offered -> the aq/1 server must
%% refuse. Success of the mutation run means the court cannot flip: FAIL.
t_alpn_mutant(Addr, CaDer) ->
    Engine = aq_quic_open(0, CaDer),
    true = is_reference(Engine),
    error = aq_quic_connect(Engine, unicode:characters_to_binary(Addr), <<"localhost">>),
    io:format("ok alpn_mutant_refused~n"),
    ok.

contracts(Addr, CaDer) ->
    t_open_refusals(CaDer),
    t_handshake_echo_zero_copy(Addr, CaDer),
    ok.

t_open_refusals(CaDer) ->
    error = aq_quic_open(-1, CaDer),
    error = aq_quic_open(70000, CaDer),
    error = aq_quic_open(0, not_a_binary),
    error = aq_quic_open("zero", CaDer),
    io:format("ok open_refusals~n").

t_handshake_echo_zero_copy(Addr, CaDer) ->
    Engine = aq_quic_open(0, CaDer),
    true = is_reference(Engine),
    error = aq_quic_connect(Engine, <<"not-an-addr">>, <<"localhost">>),
    error = aq_quic_connect(42, unicode:characters_to_binary(Addr), <<"localhost">>),
    Conn = aq_quic_connect(Engine, unicode:characters_to_binary(Addr), <<"localhost">>),
    true = is_reference(Conn),
    Payload = crypto:strong_rand_bytes(4096),
    ok = aq_quic_send(Conn, Payload),
    io:format("ok handshake_echo (see header for stream-data blocker)~n"),
    %% ZERO-COPY census: snapshot after send, recv, snapshot again. The
    %% delta must be exactly one resource-block alloc, < 128 bytes — no
    %% payload-sized copy at the NIF boundary.
    {A0, B0} = cgap_alloc_stats(),
    Echo = aq_quic_recv(Conn, 5000),
    {A1, B1} = cgap_alloc_stats(),
    true = is_binary(Echo),
    Payload = Echo,
    1 = A1 - A0,
    true = (B1 - B0) < 128,
    io:format("ok handshake_echo (zero-copy census: +1 alloc, +~w bytes)~n",
              [B1 - B0]),
    %% Bounded timeout leg (no-sleep law): a recv with nothing pending must
    %% refuse on its own deadline, not hang.
    error = aq_quic_recv(Conn, 250),
    io:format("ok recv_bounded_timeout~n").
"##;

// ---------------------------------------------------------------------------
// Server-role leg (lane 3): NIF server vs pure-Rust quinn client, real
// loopback UDP, bi-stream frame roundtrip + ALPN-drop flip.
// ---------------------------------------------------------------------------

const FRAME_PREFIX: &[u8] = b"aq-frame:";
const FRAME_BODY_LEN: usize = 32;
/// The escript replies [<<"reply:">>, 1, <<"aq">>] — an iolist whose
/// flattening exercises enif_inspect_iolist_as_binary (integer element 1).
const EXPECTED_REPLY: &[u8] = b"reply:\x01aq";

/// A free loopback UDP port (bind-to-0 then drop; the serve bind races, so
/// the client's connect poll is bounded — see quinn_client_leg).
fn free_udp_port() -> u16 {
    let s = std::net::UdpSocket::bind("127.0.0.1:0").expect("bind 0");
    s.local_addr().expect("local addr").port()
}

struct NifServer {
    port: u16,
    cert_path: std::path::PathBuf,
    key_path: std::path::PathBuf,
    child: std::process::Child,
}

/// Spawn the NIF SERVER escript court: load dylib, aq_quic_open ->
/// aq_quic_serve(Port, CertDer, KeyDer) -> accept / accept_bi / recv /
/// send(iolist) / close. `mutate` sets AQ_MUTATE_ALPN_DROP=1 for the
/// fresh process (the server-side ALPN-strip mutation).
fn spawn_nif_server(
    dylib: &std::path::PathBuf,
    cert_path: &std::path::PathBuf,
    key_path: &std::path::PathBuf,
    port: u16,
    mutate: bool,
    stem: &str,
) -> std::process::Child {
    let escript_path = std::env::temp_dir().join(stem);
    std::fs::write(&escript_path, ESCRIPT_SERVER).expect("write escript");
    let mut cmd = Command::new("escript");
    cmd.arg(&escript_path)
        .arg(dylib)
        .arg(port.to_string())
        .arg(cert_path)
        .arg(key_path);
    if mutate {
        cmd.env("AQ_MUTATE_ALPN_DROP", "1");
    }
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn escript server")
}

/// Wait for the spawned escript and surface its output on failure (the
/// court's stderr IS the evidence — never swallow it).
fn wait_escript(mut child: std::process::Child, what: &str) -> i32 {
    let out = child.wait_with_output().expect("wait escript");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    if !stdout.is_empty() {
        print!("{stdout}");
    }
    if !stderr.is_empty() {
        eprint!("{stderr}");
    }
    out.status.code().unwrap_or_else(|| {
        panic!("{what}: escript killed by signal, stderr:\n{stderr}")
    })
}

/// Pure-Rust quinn client leg. Bounded connect poll (no-sleep law): the
/// server bind races with the port pick, so connect is retried on a real
/// `Instant` deadline until it either succeeds or refuses. When
/// `expect_ok` the leg opens a bi stream, writes the frame (no finish),
/// and asserts the iolist reply; when !expect_ok it asserts the handshake
/// REFUSES.
fn quinn_client_leg(
    port: u16,
    cert_der: &[u8],
    expect_ok: bool,
) -> Result<(), String> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("runtime: {e}"))?;
    rt.block_on(async move {
        let mut roots = rustls::RootCertStore::empty();
        roots
            .add(rustls::pki_types::CertificateDer::from(cert_der.to_vec()))
            .map_err(|e| format!("ca der rejected: {e}"))?;
        let mut crypto = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        crypto.alpn_protocols = vec![b"aq/1".to_vec()];
        let quic_crypto: quinn::crypto::rustls::QuicClientConfig = crypto
            .try_into()
            .map_err(|e: quinn::crypto::rustls::NoInitialCipherSuite| format!("tls wrap: {e}"))?;
        let cfg = quinn::ClientConfig::new(std::sync::Arc::new(quic_crypto));
        let mut ep = quinn::Endpoint::client("127.0.0.1:0".parse().unwrap())
            .map_err(|e| format!("client bind: {e}"))?;
        ep.set_default_client_config(cfg);
        let addr: std::net::SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
        // Bounded retry poll on the real observable (connection established
        // or handshake refused) — never a sleep.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
        let conn = loop {
            match ep.connect(addr, "localhost") {
                Err(e) => return Err(format!("connect: {e}")),
                Ok(connecting) => {
                    match tokio::time::timeout(
                        std::time::Duration::from_secs(2),
                        connecting,
                    )
                    .await
                    {
                        Ok(Ok(c)) => break c,
                        Ok(Err(e)) => {
                            if !expect_ok {
                                return Err(format!("refused: {e}"));
                            }
                            if std::time::Instant::now() >= deadline {
                                return Err(format!("handshake never: {e}"));
                            }
                        }
                        Err(_) => {
                            if std::time::Instant::now() >= deadline {
                                return Err("handshake timeout".to_string());
                            }
                            if !expect_ok {
                                return Err("mutant handshake timed out".to_string());
                            }
                        }
                    }
                }
            }
        };
        if !expect_ok {
            // The mutant run lands here only via the `refused:` branch; a
            // success here would mean the ALPN mutation did not flip.
            return Err("mutant connect SUCCEEDED — court did not flip".into());
        }
        let (mut send, mut recv) = conn.open_bi().await.map_err(|e| e.to_string())?;
        let mut frame = Vec::from(FRAME_PREFIX);
        frame.extend(std::iter::repeat(0x5au8).take(FRAME_BODY_LEN));
        send.write_all(&frame).await.map_err(|e| e.to_string())?;
        // NO finish — bi streams persist (same law as the NIF side).
        // The NIF's bi-stream send does NOT finish (streams persist), so
        // read the exact reply length — read_to_end would wait for a FIN
        // that never comes.
        let mut reply = vec![0u8; EXPECTED_REPLY.len()];
        recv.read_exact(&mut reply).await.map_err(|e| e.to_string())?;
        if reply != *EXPECTED_REPLY {
            return Err(format!("reply mismatch: {reply:?}"));
        }
        Ok::<(), String>(())
    })
}

fn write_cert_files() -> (std::path::PathBuf, std::path::PathBuf, Vec<u8>) {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])
        .expect("rcgen self-signed cert");
    let cert_der = cert.cert.der().as_ref().to_vec();
    let key_der = cert.key_pair.serialize_der();
    let cert_path = std::env::temp_dir().join(format!("aq_quic_srv_cert_{}", std::process::id()));
    let key_path = std::env::temp_dir().join(format!("aq_quic_srv_key_{}", std::process::id()));
    std::fs::write(&cert_path, &cert_der).expect("write cert der");
    std::fs::write(&key_path, &key_der).expect("write key der");
    (cert_path, key_path, cert_der)
}

#[test]
fn nif_server_quinn_client_bi_roundtrip_and_alpn_flip() {
    let dylib = nif_dylib_path();
    let (cert_path, key_path, cert_der) = write_cert_files();

    // Baseline: NIF server + pure-Rust client, bi-stream frame roundtrip.
    let port = free_udp_port();
    let mut child = spawn_nif_server(&dylib, &cert_path, &key_path, port, false,
                                     "cgap_beam_quic_srv_court");
    let client = quinn_client_leg(port, &cert_der, true);
    let rc = wait_escript(child, "server court");
    assert!(
        client.is_ok(),
        "quinn client leg failed: {client:?} (escript rc={rc})"
    );
    assert_eq!(rc, 0, "NIF server court failed (see COURT_FAIL above)");

    // SHIM-COURT flip on the NEW leg: server-side ALPN stripped -> the
    // pure-Rust client's handshake must refuse AND the engine accept must
    // time out (escript exit 0 asserts the refusal from the server side).
    let port = free_udp_port();
    let mut child = spawn_nif_server(&dylib, &cert_path, &key_path, port, true,
                                     "cgap_beam_quic_srv_mutant");
    let client = quinn_client_leg(port, &cert_der, false);
    let rc = wait_escript(child, "mutant court");
    assert!(
        client.is_err(),
        "mutant client leg connected — VACUOUS COURT"
    );
    let err = client.unwrap_err();
    // rustls surfaces the server's alert as "received fatal alert:
    // NoApplicationProtocol" (or its own PeerIncompatible display).
    let low = err.to_lowercase();
    assert!(
        low.contains("applicationprotocol") || low.contains("peer is incompatible"),
        "mutant refusal not ALPN-shaped: {err}"
    );
    assert_eq!(rc, 0, "mutant NIF server court failed");
}

const ESCRIPT_SERVER: &str = r##"
-module(cgap_beam).
-export([main/1, aq_slab_open/1, aq_slab_frame/2, aq_slab_window/4,
         aq_slab_stamp/2, aq_slab_lease/2, aq_slab_sync/1,
         aq_quic_open/2, aq_quic_connect/3, aq_quic_send/2, aq_quic_recv/2,
         aq_quic_serve/4, aq_quic_accept/2, aq_quic_accept_bi/2,
         aq_quic_stream_open/1, aq_quic_stream_send/2,
         aq_quic_stream_recv/2, aq_quic_stream_close/1,
         cgap_new/2, cgap_new_fast/2, cgap_new_i64/2, cgap_len/1,
         cgap_get/2, cgap_get_i64/2, cgap_stats/1, cgap_free/1,
         cgap_version/0, cgap_envelope_new/1, cgap_envelope_len/1,
         cgap_batch_binary/1, pointer_of/1, cgap_binary_data_pointer/1,
         cgap_dtor_stats/0, cgap_alloc_stats/0, cgap_flush_to_parquet/1,
         cgap_seal_to_packfile/1, cgap_receipt_query/1, cgap_open/1,
         cgap_scan/3]).
%% OTP 28 load_nif law: EVERY ErlNifFunc-table entry must be exported by
%% the calling module, else `bad_lib: Function not found` (witnessed:
%% cgap_new/2 and aq_quic_open/2 refusals). Full 38-entry stub list.
aq_slab_open(_) -> erlang:nif_error(nif_not_loaded).
aq_slab_frame(_, _) -> erlang:nif_error(nif_not_loaded).
aq_slab_window(_, _, _, _) -> erlang:nif_error(nif_not_loaded).
aq_slab_stamp(_, _) -> erlang:nif_error(nif_not_loaded).
aq_slab_lease(_, _) -> erlang:nif_error(nif_not_loaded).
aq_slab_sync(_) -> erlang:nif_error(nif_not_loaded).
aq_quic_open(_, _) -> erlang:nif_error(nif_not_loaded).
aq_quic_connect(_, _, _) -> erlang:nif_error(nif_not_loaded).
aq_quic_send(_, _) -> erlang:nif_error(nif_not_loaded).
aq_quic_recv(_, _) -> erlang:nif_error(nif_not_loaded).
aq_quic_serve(_, _, _, _) -> erlang:nif_error(nif_not_loaded).
aq_quic_accept(_, _) -> erlang:nif_error(nif_not_loaded).
aq_quic_accept_bi(_, _) -> erlang:nif_error(nif_not_loaded).
aq_quic_stream_open(_) -> erlang:nif_error(nif_not_loaded).
aq_quic_stream_send(_, _) -> erlang:nif_error(nif_not_loaded).
aq_quic_stream_recv(_, _) -> erlang:nif_error(nif_not_loaded).
aq_quic_stream_close(_) -> erlang:nif_error(nif_not_loaded).
cgap_new(_, _) -> erlang:nif_error(nif_not_loaded).
cgap_new_fast(_, _) -> erlang:nif_error(nif_not_loaded).
cgap_new_i64(_, _) -> erlang:nif_error(nif_not_loaded).
cgap_len(_) -> erlang:nif_error(nif_not_loaded).
cgap_get(_, _) -> erlang:nif_error(nif_not_loaded).
cgap_get_i64(_, _) -> erlang:nif_error(nif_not_loaded).
cgap_stats(_) -> erlang:nif_error(nif_not_loaded).
cgap_free(_) -> erlang:nif_error(nif_not_loaded).
cgap_version() -> erlang:nif_error(nif_not_loaded).
cgap_envelope_new(_) -> erlang:nif_error(nif_not_loaded).
cgap_envelope_len(_) -> erlang:nif_error(nif_not_loaded).
cgap_batch_binary(_) -> erlang:nif_error(nif_not_loaded).
pointer_of(_) -> erlang:nif_error(nif_not_loaded).
cgap_binary_data_pointer(_) -> erlang:nif_error(nif_not_loaded).
cgap_dtor_stats() -> erlang:nif_error(nif_not_loaded).
cgap_alloc_stats() -> erlang:nif_error(nif_not_loaded).
cgap_flush_to_parquet(_) -> erlang:nif_error(nif_not_loaded).
cgap_seal_to_packfile(_) -> erlang:nif_error(nif_not_loaded).
cgap_receipt_query(_) -> erlang:nif_error(nif_not_loaded).
cgap_open(_) -> erlang:nif_error(nif_not_loaded).
cgap_scan(_, _, _) -> erlang:nif_error(nif_not_loaded).

main([NifPath, PortStr, CertPath, KeyPath]) ->
    Stem = filename:join([tmp_base(), "libcgap_beam_quic_srv_court"]),
    {ok, _} = file:copy(NifPath, Stem ++ ".so"),
    %% macOS 26 provenance-xattr law (see the client-court escript).
    os:cmd("xattr -c " ++ Stem ++ ".so"),
    ok = case erlang:load_nif(Stem, 0) of
             ok -> ok;
             {error, R} -> erlang:error({load_nif_failed, R})
         end,
    {ok, CaDer} = file:read_file(CertPath),
    {ok, KeyDer} = file:read_file(KeyPath),
    Res = try
              serve_contracts(list_to_integer(PortStr), CaDer, KeyDer)
          of
              ok -> 0
          catch
              Class:Reason:Stack ->
                  io:format("COURT_FAIL ~p:~p~n~p~n", [Class, Reason, Stack]),
                  1
          end,
    halt(Res).

tmp_base() -> "/tmp".

serve_contracts(Port, CaDer, KeyDer) ->
    Engine = aq_quic_open(0, CaDer),
    true = is_reference(Engine),
    Listener = aq_quic_serve(Engine, Port, CaDer, KeyDer),
    true = is_reference(Listener),
    case os:getenv("AQ_MUTATE_ALPN_DROP") of
        "1" ->
            %% Server-side ALPN stripped: the aq/1-offering client's
            %% handshake must fail, so the bounded accept must hit its
            %% deadline and answer `error` (the flip, server-side view).
            error = aq_quic_accept(Listener, 6000),
            io:format("ok mutant_accept_refused~n"),
            ok;
        _ ->
            t_serve_roundtrip(Listener)
    end.

t_serve_roundtrip(Listener) ->
    Conn = aq_quic_accept(Listener, 10000),
    true = is_reference(Conn),
    Stream = aq_quic_accept_bi(Conn, 10000),
    true = is_reference(Stream),
    Frame = aq_quic_stream_recv(Stream, 10000),
    <<"aq-frame:", Body/binary>> = Frame,
    32 = byte_size(Body),
    %% IOLIST send: [binary, integer 0..255, binary] — exercises
    %% enif_inspect_iolist_as_binary on the send path.
    ok = aq_quic_stream_send(Stream, [<<"reply:">>, 1, <<"aq">>]),
    ok = aq_quic_stream_close(Stream),
    io:format("ok nif_server_roundtrip~n"),
    ok.
"##;
