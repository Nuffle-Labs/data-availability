{ inputs, pkgs, ... }:

{
  devcontainer.enable = true;
  difftastic.enable = true;
  dotenv.enable = true;

  languages = {
    python = {
      enable = true;
    };
    go.enable = true;
    nix.enable = true;
    c.enable = true;
    cplusplus.enable = true;
    rust = {
      enable = true;
      # https://devenv.sh/reference/options/#languagesrustchannel
      channel = "stable";
      # version = "1.77";
      components = [
        "rustc"
        "cargo"
        "clippy"
        "rustfmt"
        "rust-src"
      ];
    };
  };

  # https://devenv.sh/basics/
  env.GREET = "devenv";
  env.LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];


  # https://devenv.sh/packages/
  packages = with pkgs; [
    git
    openssl
    rust-analyzer
    ninja
    protobuf
    # bun
    (inputs.nixpkgs-working-bun.legacyPackages.${system}.bun)
  ];

  enterShell = ''
    Hello world!
  '';

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"
    git --version | grep "2.42.0"
    cargo test --workspace --all-features
  '';
  # https://devenv.sh/pre-commit-hooks/
  pre-commit.hooks = {
    # execute example shell from Markdown files
    mdsh.enable = true;
    # format Python code
    black.enable = true;

    # shellcheck.enable = true;
    check-json.enable = true;
    check-toml.enable = true;
    check-yaml.enable = true;
    clippy.enable = true;
    detect-private-keys.enable = true;
    flake-checker.enable = true;
    gofmt.enable = true;
    gotest.enable = true;
    rustfmt.enable = true;
    cargo-check.enable = true;
  };
  # https://devenv.sh/services/
  # services.postgres.enable = true;

  # https://devenv.sh/processes/

  scripts = {
    # The sidecar used to interact with a live network
    sidecar.exec = "RUST_LOG=debug cargo run --bin near-da-http-api -- -c http-config.json";
  };
  processes = {
    set-key.exec = "docker compose up near-localnet-set-key";
    localnet.exec = "docker compose up --build near-localnet";
    sidecar.exec = "RUST_LOG=debug cargo run --bin near-da-http-api -- -c test/http-sidecar.json";
  };
  # See full reference at https://devenv.sh/reference/options/
}
