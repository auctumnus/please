{
  description = "`please` cli";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs =
    inputs@{
      flake-parts,
      nixpkgs,
      rust-overlay,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      perSystem =
        {
          config,
          system,
          ...
        }:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };

          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
              "llvm-tools-preview"
            ];
          };

          nativeBuildInputs =
            with pkgs;
            [
              rustToolchain
              pkg-config
            ]
            ++ lib.optionals stdenv.isLinux [
              cargo-llvm-cov
            ];

          buildInputs =
            with pkgs;
            [
              openssl
              postgresql
            ]
            ++ lib.optionals stdenv.isDarwin [
              apple-sdk_11
            ];

        in
        {
          devShells.default = pkgs.mkShell {
            inherit nativeBuildInputs buildInputs;
          };

          packages.default = pkgs.rustPlatform.buildRustPackage {
            pname = "please-cli";
            version = "0.1.0";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            inherit nativeBuildInputs buildInputs;

            meta = with pkgs.lib; {
              description = "`please` cli";
              license = licenses.mit;
              maintainers = [ ];
            };
          };

          packages.please = config.packages.default;
        };
    };
}
