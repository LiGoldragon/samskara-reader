{
  description = "Samskara Reader — read-only MCP server for samskara world state";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    criome-cozo = { url = "github:LiGoldragon/criome-cozo"; flake = false; };
    samskara-core = { url = "github:LiGoldragon/samskara-core"; flake = false; };
  };

  outputs = { self, nixpkgs, flake-utils, crane, fenix, criome-cozo, samskara-core, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rustToolchain = fenix.packages.${system}.latest.toolchain;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (craneLib.filterCargoSources path type);
        };

        commonArgs = {
          inherit src;
          pname = "samskara-reader";
          postUnpack = ''
            depDir=$(dirname $sourceRoot)
            cp -rL ${criome-cozo} $depDir/criome-cozo
            cp -rL ${samskara-core} $depDir/samskara-core
          '';
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      in
      {
        packages.default = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });

        checks = {
          build = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
          });
        };

        devShells.default = craneLib.devShell {
          packages = with pkgs; [ rust-analyzer sqlite jujutsu ];
        };
      }
    );
}
