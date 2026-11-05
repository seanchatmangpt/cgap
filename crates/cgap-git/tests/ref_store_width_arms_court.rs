//! RC-02 width-arm witness court (red-11 findings 1 and 5).
//!
//! Chicago discipline: real collaborators (real cargo builds against the
//! real crate source), assert final state (exit codes + stderr text), no
//! mocks. Anti-vacuity: the court mutates /tmp sandbox copies of the
//! crate and proves each witness actually flips the build; a court that
//! cannot flip is FAIL. The shared tree is never mutated.
//!
//! Witness under court:
//!   - W1: `std`-feature-on-OS-less-target = loud compile_error naming
//!     `--no-default-features` (replaces raw E0463, red-11 finding 5).
//!   - W2: width contracts of the never-compiled cfg arms (impl16,
//!     implcell) are compiled on every target via mirror modules
//!     (red-11 finding 1); breaking one breaks the build everywhere.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Recursively copy `src_dir` into `dst_dir/src`.
fn copy_src(src_dir: &PathBuf, dst_root: &Path) -> std::io::Result<()> {
    let dst_src = dst_root.join("src");
    fs::create_dir_all(&dst_src)?;
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            fs::copy(entry.path(), dst_src.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn cargo_bin() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string())
}

/// Standalone sandbox crate: the workspace Cargo.toml cannot stand alone
/// (workspace inheritance), so the court mints its own minimal manifest
/// with the same feature set (default = ["std"]).
fn make_sandbox(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rc02-court-{}-{}", tag, std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("sandbox root");
    let manifest = format!(
        "[package]\n\
         name = \"cgap-git-rc02-{}\"\n\
         version = \"0.0.0\"\n\
         edition = \"2021\"\n\
         [lib]\n\
         path = \"src/lib.rs\"\n\
         [features]\n\
         default = [\"std\"]\n\
         std = []\n",
        tag
    );
    fs::write(root.join("Cargo.toml"), manifest).expect("sandbox manifest");
    copy_src(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"),
        &root,
    )
    .expect("sandbox src copy");
    root
}

/// Run cargo against a sandbox manifest with its own isolated target dir.
fn sandbox_build(root: &Path, args: &[&str]) -> (bool, String) {
    let target_dir = root.join("target");
    let out = Command::new(cargo_bin())
        .args(args)
        .arg("--manifest-path")
        .arg(root.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("spawn cargo");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

/// Bounded, observable cleanup (not a sleep): remove the sandbox at test
/// end; failure to remove is asserted, never silent.
fn rm_sandbox(root: &PathBuf) {
    fs::remove_dir_all(root).expect("sandbox cleanup");
}

#[test]
fn w1_riscv_default_features_fails_loud_not_e0463() {
    let sb = make_sandbox("w1");
    let (ok, out) = sandbox_build(&sb, &["build", "--target", "riscv32imc-unknown-none-elf"]);
    assert!(
        !ok,
        "default-feature riscv32imc build must NOT pass; got:\n{}",
        out
    );
    assert!(
        out.contains("--no-default-features"),
        "failure must carry our loud witness message; got:\n{}",
        out
    );
    assert!(
        out.contains("feature is enabled on an OS-less target"),
        "witness message must name the OS-less-target cause; got:\n{}",
        out
    );
    rm_sandbox(&sb);
}

#[test]
fn w2_riscv_no_cas_target_refuses_loud_without_optin() {
    // ADV-11 fix: rv32imc has NO legacy CAS key on the current nightly,
    // so the no-default-features build must now fail LOUD with the typed
    // implcell gate (never a silent implcell arm).
    let sb = make_sandbox("w2");
    let (ok, out) = sandbox_build(
        &sb,
        &[
            "build",
            "--target",
            "riscv32imc-unknown-none-elf",
            "--no-default-features",
        ],
    );
    assert!(
        !ok,
        "no-CAS rv32imc build without the critical opt-in must refuse; got:\n{}",
        out
    );
    assert!(
        out.contains("implcell_critical_provided"),
        "typed refusal must name the opt-in cfg; got:\n{}",
        out
    );
    assert!(
        out.contains("no CAS-capable atomic"),
        "typed refusal must name the missing CAS capability; got:\n{}",
        out
    );
    rm_sandbox(&sb);
}

#[test]
fn w2b_riscv_optin_builds_green() {
    // With the firmware opt-in the rv32imc kernel build compiles: the
    // acceptance is visible in the build command (RUSTFLAGS --cfg).
    let sb = make_sandbox("w2b");
    let target_dir = sb.join("target");
    let out = Command::new(cargo_bin())
        .args([
            "build",
            "--target",
            "riscv32imc-unknown-none-elf",
            "--no-default-features",
        ])
        .arg("--manifest-path")
        .arg(sb.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RUSTFLAGS", "--cfg implcell_critical_provided")
        .output()
        .expect("spawn cargo");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success(),
        "opt-in rv32imc build must compile; got:\n{}",
        text
    );
    rm_sandbox(&sb);
}

#[test]
fn w6_discriminator_table_matches_selected_arm() {
    // The independent (pointer-width-family) EXPECTED_ARM must agree with
    // the ladder's SELECTED_ARM on the host: asserted inside the crate
    // at compile time; this leg additionally pins the host arm identity
    // so a silent family slide cannot pass without this court flipping.
    assert_eq!(
        cgap_git::ref_store::SEAL_COUNTER_WIDTH,
        u64::MAX,
        "host must select impl64; if this flips the cfg families slid"
    );
}

#[test]
fn w7_anti_vacuity_discriminator_mutation_flips() {
    // Mutation: in a /tmp sandbox, force the discriminator's EXPECTED_ARM
    // leg to the impl32 value on the host (a lie about the cfg family).
    // The compile-time cross-check must fail the build with the
    // discriminator message — proving the table is a real check, not a
    // self-agreeing decoration.
    let sb = make_sandbox("w7");
    let path = sb.join("src/ref_store.rs");
    let src = fs::read_to_string(&path).expect("read sandbox ref_store.rs");
    let mutated = src.replacen(
        "    #[cfg(target_pointer_width = \"64\")]\n    const EXPECTED_ARM: u8 = ARM_IMPL64;",
        "    #[cfg(target_pointer_width = \"64\")]\n    const EXPECTED_ARM: u8 = ARM_IMPL32;",
        1,
    );
    assert_ne!(
        src, mutated,
        "mutation must apply (EXPECTED_ARM leg not found)"
    );
    fs::write(&path, mutated).expect("write mutated ref_store.rs");
    let (ok, out) = sandbox_build(&sb, &["build"]);
    assert!(
        !ok,
        "discriminator must fail the build when its table is wrong; got:\n{}",
        out
    );
    assert!(
        out.contains("cfg-arm discriminator"),
        "failure must be the discriminator cross-check; got:\n{}",
        out
    );
    rm_sandbox(&sb);
}

#[test]
fn w3_host_width_contract_impl64() {
    // Host compiles the impl64 arm: its width contract must hold.
    assert_eq!(cgap_git::ref_store::SEAL_COUNTER_WIDTH, u64::MAX);
}

#[test]
fn w4_anti_vacuity_break_implcell_width_constant() {
    // Mutation: break the implcell mirror width constant in a /tmp
    // sandbox copy; the witness must fail the build. Flip law: if this
    // leg cannot flip, W2's pass is vacuous.
    let sb = make_sandbox("w4");
    let path = sb.join("src/ref_store.rs");
    let src = fs::read_to_string(&path).expect("read sandbox ref_store.rs");
    let mutated = src.replacen(
        "pub const WIDTH: u64 = usize::MAX as u64;\n    }\n    const _: () = assert!(\n        mirror_implcell::WIDTH",
        "pub const WIDTH: u64 = u32::MAX as u64;\n    }\n    const _: () = assert!(\n        mirror_implcell::WIDTH",
        1,
    );
    assert_ne!(src, mutated, "mutation must apply (width const not found)");
    fs::write(&path, mutated).expect("write mutated ref_store.rs");
    let (ok, out) = sandbox_build(&sb, &["build"]);
    assert!(
        !ok,
        "witness must fail the build on a broken width constant; got:\n{}",
        out
    );
    assert!(
        out.contains("implcell arm width contract violated"),
        "failure must be the width-contract assertion; got:\n{}",
        out
    );
    rm_sandbox(&sb);
}

#[test]
fn w5_anti_vacuity_remove_std_gate() {
    // Mutation: the W1 gate's compile_error cfg is flipped to fire on the
    // HOST default build (target_pointer_width = "64" replaces
    // target_os = "none"). The host default-feature build must now fail
    // with the witness message — proving the compile_error fires when its
    // cfg holds (non-vacuous gate, not cfg that can never be true).
    let sb = make_sandbox("w5");
    let path = sb.join("src/ref_store.rs");
    let src = fs::read_to_string(&path).expect("read sandbox ref_store.rs");
    let mutated = src.replacen(
        "target_os = \"none\"))]",
        "target_pointer_width = \"64\"))]",
        1,
    );
    assert_ne!(src, mutated, "mutation must apply (gate cfg not found)");
    fs::write(&path, mutated).expect("write mutated ref_store.rs");
    let (ok, out) = sandbox_build(&sb, &["build"]);
    assert!(
        !ok,
        "witness gate must flip the host default build red; got:\n{}",
        out
    );
    assert!(
        out.contains("--no-default-features"),
        "host flip must produce the witness message; got:\n{}",
        out
    );
    rm_sandbox(&sb);
}
