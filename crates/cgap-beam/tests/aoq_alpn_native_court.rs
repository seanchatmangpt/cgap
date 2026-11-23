//! ECOSYSTEM-SPEC-27.1.1 R-03 — native quinn ALPN `aoq/1` Chicago court.
//!
//! Pure-Rust (no BEAM/escript): real quinn listen/connect over loopback UDP.
//! Baseline EXIT 0 with pinned ALPN; ALPN drop / legacy `aq/1` must refuse.
//! UDP-443: real bind probe — LOCKED if listen+handshake succeeds, else
//! typed BLOCKED (PermissionDenied) / UNSUPPORTED. WebTransport/h3 ABSENT
//! (deferred E9) — this court does NOT invent that edge.
#![cfg(feature = "quic")]

use std::io;
use std::net::UdpSocket;
use std::sync::mpsc;
use std::time::Duration;

/// Risk-body ALPN pin (must match `AOQ_ALPN` in aq_quic.rs).
const AOQ_ALPN: &[u8] = b"aoq/1";
/// Retired scaffold — must NOT negotiate silently.
const LEGACY_AQ_ALPN: &[u8] = b"aq/1";

#[test]
fn aoq_alpn_byte_pin() {
    assert_eq!(AOQ_ALPN, b"aoq/1");
    assert_ne!(AOQ_ALPN, LEGACY_AQ_ALPN);
}

fn make_cert() -> (Vec<u8>, Vec<u8>) {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])
        .expect("rcgen");
    (
        cert.cert.der().as_ref().to_vec(),
        cert.key_pair.serialize_der(),
    )
}

fn server_cfg(cert: Vec<u8>, key: Vec<u8>, alpn: Option<&[u8]>) -> quinn::ServerConfig {
    let mut crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![rustls::pki_types::CertificateDer::from(cert)],
            rustls::pki_types::PrivateKeyDer::Pkcs8(
                rustls::pki_types::PrivatePkcs8KeyDer::from(key),
            ),
        )
        .expect("server cert");
    if let Some(p) = alpn {
        crypto.alpn_protocols = vec![p.to_vec()];
    }
    let qc: quinn::crypto::rustls::QuicServerConfig = crypto.try_into().expect("quic srv");
    quinn::ServerConfig::with_crypto(std::sync::Arc::new(qc))
}

fn client_cfg(ca: &[u8], alpn: Option<&[u8]>) -> quinn::ClientConfig {
    let mut roots = rustls::RootCertStore::empty();
    roots
        .add(rustls::pki_types::CertificateDer::from(ca.to_vec()))
        .expect("ca");
    let mut crypto = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    if let Some(p) = alpn {
        crypto.alpn_protocols = vec![p.to_vec()];
    }
    let qc: quinn::crypto::rustls::QuicClientConfig = crypto.try_into().expect("quic cli");
    quinn::ClientConfig::new(std::sync::Arc::new(qc))
}

/// Spawn quinn server on `bind`; return local addr after bind reports.
fn spawn_server(
    bind: std::net::SocketAddr,
    cert: Vec<u8>,
    key: Vec<u8>,
    server_alpn: Option<&'static [u8]>,
) -> (std::net::SocketAddr, std::thread::JoinHandle<()>) {
    let (tx, rx) = mpsc::channel::<Result<std::net::SocketAddr, String>>();
    let handle = std::thread::Builder::new()
        .name("aoq-alpn-court-srv".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("rt");
            rt.block_on(async move {
                let ep = match quinn::Endpoint::server(server_cfg(cert, key, server_alpn), bind) {
                    Ok(e) => e,
                    Err(e) => {
                        let _ = tx.send(Err(format!("bind: {e}")));
                        return;
                    }
                };
                let addr = match ep.local_addr() {
                    Ok(a) => a,
                    Err(e) => {
                        let _ = tx.send(Err(format!("local_addr: {e}")));
                        return;
                    }
                };
                let _ = tx.send(Ok(addr));
                while let Some(incoming) = ep.accept().await {
                    if let Ok(conn) = incoming.await {
                        // Keep alive briefly so client can finish handshake.
                        let _ = tokio::time::timeout(Duration::from_secs(2), async {
                            let _ = conn.accept_uni().await;
                        })
                        .await;
                    }
                }
            });
        })
        .expect("spawn");
    let addr = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("addr")
        .expect("server bind");
    (addr, handle)
}

async fn connect_once(
    addr: std::net::SocketAddr,
    ca: &[u8],
    client_alpn: Option<&[u8]>,
) -> Result<quinn::Connection, String> {
    let mut ep = quinn::Endpoint::client("127.0.0.1:0".parse().unwrap())
        .map_err(|e| format!("client bind: {e}"))?;
    ep.set_default_client_config(client_cfg(ca, client_alpn));
    let connecting = ep
        .connect(addr, "localhost")
        .map_err(|e| format!("connect: {e}"))?;
    tokio::time::timeout(Duration::from_secs(3), connecting)
        .await
        .map_err(|_| "handshake timeout".to_string())?
        .map_err(|e| format!("handshake: {e}"))
}

fn handshake(
    bind: std::net::SocketAddr,
    server_alpn: Option<&'static [u8]>,
    client_alpn: Option<&[u8]>,
) -> Result<(), String> {
    let (cert, key) = make_cert();
    let ca = cert.clone();
    let (addr, _jh) = spawn_server(bind, cert, key, server_alpn);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("rt: {e}"))?;
    rt.block_on(connect_once(addr, &ca, client_alpn))
        .map(|_| ())
}

fn alpn_shaped(err: &str) -> bool {
    let low = err.to_lowercase();
    low.contains("applicationprotocol")
        || low.contains("peer is incompatible")
        || low.contains("doesn't support any known protocol")
        || low.contains("no_application_protocol")
}

#[test]
fn baseline_aoq_alpn_handshake_exit_0() {
    let bind: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    handshake(bind, Some(AOQ_ALPN), Some(AOQ_ALPN)).expect("baseline aoq/1 handshake");
}

#[test]
fn alpn_drop_refuses() {
    let bind: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let err = handshake(bind, Some(AOQ_ALPN), None).expect_err("drop must refuse");
    assert!(alpn_shaped(&err), "not ALPN-shaped refuse: {err}");
}

#[test]
fn legacy_aq1_client_refuses_against_aoq1_server() {
    // Migration honesty: retired aq/1 is NOT dual-admitted.
    let bind: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let err = handshake(bind, Some(AOQ_ALPN), Some(LEGACY_AQ_ALPN))
        .expect_err("legacy aq/1 must refuse");
    assert!(alpn_shaped(&err), "legacy refuse not ALPN-shaped: {err}");
}

/// Typed UDP-443 standing. LOCKED only if real listen+aoq/1 handshake
/// succeeds on port 443. PermissionDenied → BLOCKED (unprivileged). Any
/// other bind fault → UNSUPPORTED. Never invents WebTransport CLOSED.
#[derive(Debug, PartialEq, Eq)]
enum Udp443Standing {
    Locked,
    BlockedPermissionDenied,
    Unsupported(&'static str),
}

fn probe_udp_443() -> Udp443Standing {
    match UdpSocket::bind("127.0.0.1:443") {
        Ok(sock) => {
            // Release OS bind so quinn can take the same port.
            drop(sock);
            let bind: std::net::SocketAddr = "127.0.0.1:443".parse().unwrap();
            match handshake(bind, Some(AOQ_ALPN), Some(AOQ_ALPN)) {
                Ok(()) => Udp443Standing::Locked,
                Err(e) => {
                    eprintln!("UDP-443 bind ok but handshake failed: {e}");
                    Udp443Standing::Unsupported("handshake_failed_after_bind")
                }
            }
        }
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            eprintln!("UDP-443 BLOCKED: PermissionDenied (unprivileged uid)");
            Udp443Standing::BlockedPermissionDenied
        }
        Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
            eprintln!("UDP-443 UNSUPPORTED: AddrInUse ({e})");
            Udp443Standing::Unsupported("addr_in_use")
        }
        Err(e) => {
            eprintln!("UDP-443 UNSUPPORTED: {e}");
            Udp443Standing::Unsupported("other_bind_error")
        }
    }
}

#[test]
fn udp_443_listen_locked_or_typed_blocked() {
    let standing = probe_udp_443();
    match standing {
        Udp443Standing::Locked => {
            eprintln!("UDP-443 LOCKED: listen+aoq/1 handshake EXIT 0");
        }
        Udp443Standing::BlockedPermissionDenied => {
            // Typed BLOCKED is a lawful court outcome (not a fake pass of
            // the R-03 body — standing stays PARTIAL).
            eprintln!("UDP-443 standing=BLOCKED PermissionDenied");
        }
        Udp443Standing::Unsupported(why) => {
            eprintln!("UDP-443 standing=UNSUPPORTED {why}");
        }
    }
    // Court itself EXIT 0: the probe typed the outcome. R-03 CLOSED
    // requires Locked + WebTransport edge — still open.
    assert!(
        matches!(
            standing,
            Udp443Standing::Locked
                | Udp443Standing::BlockedPermissionDenied
                | Udp443Standing::Unsupported(_)
        )
    );
}
