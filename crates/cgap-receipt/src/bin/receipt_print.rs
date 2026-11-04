//! receipt_print — CLI probe for the cgap-receipt engine.
//!
//! Env-driven so any shell gate can mint a header block:
//!   CGAP_PLANE (required), CGAP_TARGET (required),
//!   CGAP_ALLOC_SIG (optional).
//!
//! Exit ABI: 0 header printed; 1 DISQUALIFIED_PROFILE (non-release build)
//! with the ledger line on stderr; 2 typed capture failure (identity,
//! cores, clock).

use cgap_receipt::{ReceiptHeader, ReceiptError, DISQUALIFIED_PROFILE};

fn main() {
    // Leak to 'static: the plane/target consts live for the process life.
    let plane: &'static str =
        Box::leak(std::env::var("CGAP_PLANE").unwrap_or_default().into_boxed_str());
    let target: &'static str =
        Box::leak(std::env::var("CGAP_TARGET").unwrap_or_default().into_boxed_str());
    if plane.is_empty() || target.is_empty() {
        eprintln!("usage: CGAP_PLANE=... CGAP_TARGET=... receipt_print");
        std::process::exit(64);
    }
    let alloc = std::env::var("CGAP_ALLOC_SIG").ok().filter(|s| !s.is_empty());
    let alloc_ref = alloc.as_deref();
    let repo_dir = std::env::var("CGAP_REPO_DIR").ok().filter(|s| !s.is_empty());
    let repo_ref = repo_dir.as_deref();
    match ReceiptHeader::emit(plane, target, alloc_ref, repo_ref) {
        Ok(text) => {
            print!("{text}");
            std::process::exit(0);
        }
        Err(e @ ReceiptError::DisqualifiedProfile { .. }) => {
            eprintln!("{}", ReceiptHeader::ledger_line(&e));
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("RECEIPT-CAPTURE-FAIL: {e}");
            std::process::exit(2);
        }
    }
}

// silence unused-import lint if DISQUALIFIED_PROFILE only used in messages
const _: &str = DISQUALIFIED_PROFILE;
