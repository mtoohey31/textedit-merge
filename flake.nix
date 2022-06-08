{
  description = "textedit-merge";

  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    nixpkgs-mozilla.url = "github:mozilla/nixpkgs-mozilla";
  };

  outputs = { self, nixpkgs, utils, nixpkgs-mozilla }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          overlays = [ nixpkgs-mozilla.overlays.rust ];
          inherit system;
        };
        rustChannel = pkgs.rustChannelOf {
          date = "2022-06-07";
          channel = "nightly";
          sha256 = "AtkXIw+pi4J37uyzD08YmsydJj9v8tsYirVZzaEvUnY=";
        };
      in
      rec {
        devShells.default = pkgs.mkShell {
          packages = [
            rustChannel.rust
            pkgs.rust-analyzer
          ];
          shellHook = ''
            export RUST_SRC_PATH="${rustChannel.rust-src}/lib/rustlib/src/rust/library"
          '';
        };
      });
}
