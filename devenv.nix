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
    go.package = pkgs.go_1_21;
    nix.enable = true;
    c.enable = true;
    cplusplus.enable = true;
    rust = {
      enable = true;
      targets = [
        "wasm32-unknown-unknown"
      ];
      # https://devenv.sh/reference/options/#languagesrustchannel
      channel = "stable";
      components = [
        "rustc"
        "cargo"
        "clippy"
        "rustfmt"
        "rust-src"
      ];
    };
  };

  env.LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];


  # https://devenv.sh/packages/
  packages = with pkgs; [
    git
    openssl
    rust-analyzer
    ninja
    protobuf
    just

    # bun without bugs
    (inputs.nixpkgs-working-bun.legacyPackages.${system}.bun)
  ];

  enterShell = ''
    echo "Welcome to devshell! Printing info.."
    devenv info

    echo "Printing legacy just commands.."
    just
  '';

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"

    # Near localnet
    wait_for_port 5888

    # Sidecar
    wait_for_port 3030


    test-rust
    test-eth
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
    # FIXME: Doesnt work because we setup sidecar etc gotest.enable = true;
    rustfmt.enable = true;
    cargo-check.enable = true;
  };
  # https://devenv.sh/services/
  # services.postgres.enable = true;

  # https://devenv.sh/processes/

  scripts = {
    # The sidecar used to interact with a live network
    sidecar.exec = "RUST_LOG=debug cargo run --bin near-da-sidecar -- -c http-config.json";

    # Test rust workspace
    test-rust.exec = "cargo test --workspace --all-features";

    # Test near da contract on eth
    test-eth.exec = ''
      cd eth
      bun install
      bun run lint
      forge build --sizes
      forge config
      forge test --gas-report
    '';

    # Generate a changelog 
    changelog.exec = "git-cliff > CHANGELOG.md";

    # Enrich JSON file with environment variable values
    enrich.exec = ''scripts/enrich.sh http-config.template.json http-config.json'';
  };
  processes = {
    set-key.exec = "docker compose up near-localnet-set-key";
    localnet.exec = "docker compose up --build near-localnet";
    sidecar.exec = "RUST_LOG=debug cargo run --bin near-da-sidecar -- -c test/http-sidecar.json";
  };
  # See full reference at https://devenv.sh/reference/options/
}
