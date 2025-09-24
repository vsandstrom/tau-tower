{
  description = "Tau webradio server - Nix flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        craneLib = crane.mkLib pkgs;
      in
    {
    packages.default = craneLib.buildPackage {
      src = craneLib.cleanCargoSource ./.;

      nativeBuildInputs = with pkgs; [
        pkg-config
        perl
        rustPlatform.bindgenHook
      ];

      doCheck = false;
    };
  });

}
