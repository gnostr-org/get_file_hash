{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          packages = [
            cargo-msrv
            git-cliff
            just
            nushell
            protobuf
            taplo
          ];

          nativeBuildInputs = [
            (lib.hiPrio rust-bin.nightly."2026-02-21".rustfmt)
            (rust-bin.stable.latest.default.override {
              extensions = [ "llvm-tools-preview" ];
            })
            rust-analyzer
          ];

          shellHook = ''
            export N34_RELAY_BASE_DIR=relay_base
            export RUST_LOG=debug
          '';
        };

        packages.default =
          let
            manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
          in
          with pkgs;
          rustPlatform.buildRustPackage {
            pname = manifest.name;
            version = manifest.version;
            cargoLock.lockFile = ./Cargo.lock;
            src = lib.cleanSource ./.;

            nativeBuildInputs = [ protobuf ];

            meta = {
              inherit (manifest) description homepage;
              license = lib.licenses.agpl3Plus;
            };
          };
      }
    );
}
