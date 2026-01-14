{
  description = "vibes - Vibe coding swiss army knife of enhancements";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "llvm-tools" ];
        };
        isLinux = pkgs.stdenv.isLinux;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rust
            pkgs.just
            pkgs.cargo-nextest
            pkgs.cargo-mutants
            pkgs.cargo-watch
            pkgs.sccache
            # CLI recording
            pkgs.vhs
            # Cloudflare tunnel client
            pkgs.cloudflared
            # Native build deps for CozoDB/RocksDB
            pkgs.clang
            pkgs.llvmPackages.libclang
            pkgs.zstd
            pkgs.pkg-config
          ] ++ pkgs.lib.optionals isLinux [
            # mold is a faster linker (Linux only)
            pkgs.mold
          ];

          # Required for bindgen to find libclang
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

          # Use sccache to cache Rust compilation artifacts
          RUSTC_WRAPPER = "sccache";
          SCCACHE_CACHE_SIZE = "24G";

          shellHook = ''
            # Auto-install cargo-llvm-cov if missing (consistent across all platforms)
            if ! command -v cargo-llvm-cov &> /dev/null; then
              echo "Installing cargo-llvm-cov..."
              cargo install cargo-llvm-cov --quiet
            fi

            echo "vibes dev shell loaded (sccache enabled)"
            echo "  just              - list commands"
            echo "  just test         - run tests"
            echo "  just dev          - watch mode"
            echo "  just coverage     - test coverage report"
            echo "  sccache --show-stats - show cache statistics"
          '' + pkgs.lib.optionalString isLinux ''
            # Use mold linker on Linux for faster linking
            export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang
            export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=clang
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
          '';
        };
      });
}
