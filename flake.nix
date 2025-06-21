{
  description = "A responsive, color-enabled ASCII video player for the terminal";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        # Development shell
        devShells.default = pkgs.mkShell {
          name = "ascii-player-dev";
          buildInputs = with pkgs; [
            # Rust toolchain
            rustc
            cargo
            rust-analyzer
            rustfmt
            clippy

            # Core dependency for video processing
            ffmpeg
            ffmpeg-full

            # Build dependencies
            pkg-config
            openssl
            
            # Development tools
            git
            gh
            just
            
            # Testing and debugging
            gdb
          ];
          
          # Environment variables for Rust development
          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.ffmpeg.dev}/lib/pkgconfig";
          
          shellHook = ''
            echo "🎬 ASCII Player Development Environment"
            echo "======================================="
            echo "Rust toolchain: $(rustc --version 2>/dev/null || echo 'not found')"
            echo "FFmpeg: $(ffmpeg -version 2>/dev/null | head -1 || echo 'not found')"
            echo ""
            echo "Available commands:"
            echo "  cargo build     - Build the project"
            echo "  cargo run       - Run the project"
            echo "  cargo test      - Run tests"
            echo "  just --list     - Show all available tasks"
            echo ""
          '';
        };

        # Build package
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "ascii-player";
          version = "0.1.0";
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          
          buildInputs = with pkgs; [
            ffmpeg
            openssl
          ];
          
          # Skip tests during build (will be run separately)
          doCheck = false;
          
          meta = with pkgs.lib; {
            description = "A responsive, color-enabled ASCII video player for the terminal";
            homepage = "https://github.com/gapul/ascii-player";
            license = licenses.mit;
            maintainers = [ "yuki" ];
            platforms = platforms.unix;
          };
        };

        # Apps
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/ascii-player";
        };
      }
    );
}