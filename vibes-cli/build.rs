//! Build script to embed workspace root for binary lookup.
//!
//! This enables connectivity tests to find locally-copied binaries even when
//! CARGO_TARGET_DIR points to a shared location across worktrees.

use std::path::PathBuf;

fn main() {
    // CARGO_MANIFEST_DIR points to vibes-cli/, workspace root is one level up
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = PathBuf::from(&manifest_dir)
        .parent()
        .expect("vibes-cli should be in workspace root")
        .to_string_lossy()
        .to_string();

    // Emit as compile-time environment variable
    println!("cargo::rustc-env=VIBES_WORKSPACE_ROOT={workspace_root}");

    // Re-run if Cargo.toml changes (standard practice)
    println!("cargo::rerun-if-changed=Cargo.toml");
}
