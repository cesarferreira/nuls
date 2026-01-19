{
  description = "NuShell-inspired ls with colorful table output, human-readable sizes, and recency-aware timestamps.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        # Build dependencies for rust
        packages = rec {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "nuls";
            version = "0.2.0";
            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            nativeBuildInputs = with pkgs; [
              pkg-config
            ];
            # Build inputs for nix-darwin
            buildInputs =
              with pkgs;
              [ ]
              ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                pkgs.darwin.apple_sdk.frameworks.Security
                pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
              ];
            meta = with pkgs.lib; {
              description = "NuShell-inspired ls with colorful table output, human-readable sizes, and recency-aware timestamps.";
              homepage = "https://github.com/cesarferreira/nuls";
              license = licenses.mit;
              maintainers = [ ];
              mainProgram = "nuls";
            };
          };
          # Alias to reference it with nuls instead of default
          nuls = self.packages.${system}.default;

          # Execute with `nix run github:cesarferreira/nuls`
          apps = {
            default = {
              type = "app";
              program = "${self.packages.${system}.default}/bin/nuls";
            };
          };

          # Devtools for nix develop
          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              rustc
              cargo
              rust-analyzer
              rustfmt
              clippy
            ];
          };
        };
      }
    );
}
