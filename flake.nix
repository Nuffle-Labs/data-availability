{
  description = "Development nix flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ 
          (import rust-overlay) 
          (self: prevPkgs: {
              nodejs = prevPkgs.nodejs-16_x;
          })
        ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustVersion = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml);
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };
      in
      {
        stdenv = pkgs.fastStdenv;
        devShell = pkgs.mkShell {
          LIBCLANG_PATH = pkgs.libclang.lib + "/lib/";
          LD_LIBRARY_PATH = "${pkgs.stdenv.cc.cc.lib}/lib/:/usr/local/lib";
          PROTOC = pkgs.protobuf + "/bin/protoc";

          NIXPKGS_ALLOW_INSECURE=1;

          nativeBuildInputs = with pkgs; [
            bashInteractive
            taplo
            clang
            just
            cmake
            openssl
            protobuf
            pkg-config
            # clang
            llvmPackages.bintools
            llvmPackages.libclang
            protobuf
            rust-cbindgen
            
            # Should be go 1.19
            go
            gopls
            python3Full

            # Note: needs impure flake to build contracts, ignore for now
            nodejs_20
            # nodejs_16
            # yarn

          ];
          buildInputs = with pkgs; [
              (rustVersion.override { extensions = [ "rust-src" ]; })
          ];
          permittedInsecurePackages = [
                "nodejs-16.20.1"
          ];

        };
  });
}
