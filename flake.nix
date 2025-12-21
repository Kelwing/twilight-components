{
  description = "Build twilight-components";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # Use the toolchain specified in rust-toolchain.toml
        rustToolchainFor = p: p.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        rustFilter = path: type: (craneLib.filterCargoSources path type);

        # Override the crane toolchain with the one in rust-toolchain.toml
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchainFor;

        # Common arguments can be set here to avoid repeating them later
        # Note: changes here will rebuild all dependency crates
        commonArgs = {
          src = pkgs.lib.cleanSourceWith {
            src = ./.; # The original, unfiltered source
            filter = rustFilter;
          };
          strictDeps = true;

          buildInputs = [
            pkgs.libgit2
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
        };

        twilight-components = craneLib.buildPackage (
          commonArgs
          // {
            cargoArtifacts = craneLib.buildDepsOnly commonArgs;

            # Additional environment variables or build phases/hooks can be set
            # here *without* rebuilding all dependency crates
            # MY_CUSTOM_VAR = "some value";
          }
        );
      in
      {
        checks = {
          inherit twilight-components;
        };

        packages.default = twilight-components;

        apps.default = flake-utils.lib.mkApp {
          drv = twilight-components;
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

          # Extra inputs can be added here; cargo and rustc are provided by default.
          packages = [
            pkgs.sqlx-cli
            pkgs.bacon
            pkgs.nixd
            pkgs.nil
            pkgs.cocogitto
            pkgs.cargo-edit
          ];
        };
      }
    );
}
