# Receipt: R-03 native quinn ALPN `aoq/1`

Standing: **PARTIAL**

| Field | Value |
|---|---|
| Receipt ID | r03-aoq-alpn-native |
| Date | 2026-11-23 |
| Subject | ECOSYSTEM-SPEC-27.1.1 R-03 (native Quinn path) |
| Risk-body ALPN | `aoq/1` (pinned; historical `aq/1` retired, not dual-admitted) |
| WebTransport / h3 / edge-proxy | ABSENT (deferred E9) — not invented CLOSED |
| UDP-443 | typed **BLOCKED** (`PermissionDenied`, uid 1000, unprivileged) |
| Foundry | never claimed ALIVE |
| Machine | Linux x86_64 (cursor cloud agent) |
| Base SHA | `3d22363ba670e4d986b9fbaebde4b668de9a27f9` |

## Landed

1. `crates/cgap-beam/src/aq_quic.rs` — `AQ_ALPN = b"aq/1"` → `AOQ_ALPN = b"aoq/1"`;
   client + server TLS configs negotiate that string only. Unit pin
   `r03_aoq_alpn_pin::aoq_alpn_is_risk_body_string`.
2. `crates/cgap-beam/tests/aq_quic_contract.rs` — BEAM echo/server ALPN
   literals → `aoq/1` (escript not present on this host; not re-run).
3. `crates/cgap-beam/tests/aoq_alpn_native_court.rs` — pure-Rust Chicago court:
   baseline `aoq/1` handshake; ALPN drop refuse; legacy `aq/1` refuse;
   UDP-443 typed LOCKED|BLOCKED|UNSUPPORTED probe.

## Commands + EXIT (this session)

```text
cargo test -p cgap-beam --features quic --test aoq_alpn_native_court -- --nocapture
EXIT 0
  aoq_alpn_byte_pin ... ok
  baseline_aoq_alpn_handshake_exit_0 ... ok
  alpn_drop_refuses ... ok
  legacy_aq1_client_refuses_against_aoq1_server ... ok
  udp_443_listen_locked_or_typed_blocked ... ok
  (stderr: UDP-443 BLOCKED: PermissionDenied (unprivileged uid))

cargo test -p cgap-beam --features quic r03_aoq_alpn_pin -- --nocapture
EXIT 0
  aq_quic::r03_aoq_alpn_pin::aoq_alpn_is_risk_body_string ... ok

cargo build -p cgap-beam --features quic
EXIT 0

cargo build -p cgap-beam
EXIT 0
```

## Standing honesty

**PARTIAL** — ALPN pinned + courted on the native quinn path; UDP-443 not
LOCKED; WebTransport/h3 edge still ABSENT. CLOSED only if the full R-03
body is met.

## Exact next

1. Privileged / `CAP_NET_BIND_SERVICE` (or equivalent) UDP-443 listen+connect
   court that reaches LOCKED with `aoq/1`, **or** a documented production
   bind path that does not require root on the court host.
2. WebTransport/h3 edge remains deferred E9 — do not invent CLOSED.
3. When escript/OTP is available: re-run `aq_quic_contract` BEAM legs under
   the new ALPN pin (flip still required).

## Anti-vacuity

- Drop ALPN → refuse (ALPN-shaped).
- Offer retired `aq/1` → refuse (no silent dual-admit).
- Court that cannot flip is FAIL.
