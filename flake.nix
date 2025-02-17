{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane = {
      url = "github:ipetkov/crane";
    };
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    st = {
      url = "github:lcolonq/st";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, ... }@inputs:
    inputs.flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import inputs.rust-overlay) ];
        };
        inherit (pkgs) lib;

        native = rec {
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            targets = [ "x86_64-unknown-linux-gnu" ];
          };
          craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;
          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              (lib.hasSuffix "\.html" path) ||
              (lib.hasSuffix "\.js" path) ||
              (lib.hasSuffix "\.css" path) ||
              (lib.hasInfix "/assets/" path) ||
              (craneLib.filterCargoSources path type)
            ;
          };
          nativeBuildInputs = [
            pkgs.pkg-config
          ];
          buildInputs = [
            pkgs.openssl.dev
            pkgs.glfw
            pkgs.xorg.libX11 
            pkgs.xorg.libXcursor 
            pkgs.xorg.libXi 
            pkgs.xorg.libXrandr
            pkgs.xorg.libXinerama
            pkgs.libxkbcommon 
            pkgs.xorg.libxcb  
            pkgs.libglvnd
            pkgs.alsa-lib
          ];
          commonArgs = {
            inherit src nativeBuildInputs buildInputs;
            strictDeps = true;
            CARGO_BUILD_TARGET = "x86_64-unknown-linux-gnu";
            inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
          };
          cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
            doCheck = false;
          });
          renderer = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
            cargoExtraArgs = "-p renderer";
          });
        };

        wasm = rec {
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            targets = [ "wasm32-unknown-unknown" ];
          };
          craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;
          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              (lib.hasSuffix "\.html" path) ||
              (lib.hasSuffix "\.js" path) ||
              (lib.hasSuffix "\.css" path) ||
              (lib.hasInfix "/assets/" path) ||
              (craneLib.filterCargoSources path type)
            ;
          };
          commonArgs = {
            inherit src;
            strictDeps = true;
            CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
            buildInputs = [];
            inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
            wasm-bindgen-cli = pkgs.buildWasmBindgenCli rec {
              src = pkgs.fetchCrate {
                pname = "wasm-bindgen-cli";
                version = "0.2.100";
                hash = "sha256-3RJzK7mkYFrs7C/WkhW9Rr4LdP5ofb2FdYGz1P7Uxog=";
              };
              cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
                inherit src;
                inherit (src) pname version;
                hash = "sha256-qsO12332HSjWCVKtf1cUePWWb9IdYUmT+8OPj/XP2WE=";
              };
            };
          };
          throwshade = craneLib.buildTrunkPackage (commonArgs // rec {
            pname = "throwshade";
            cargoExtraArgs = "-p throwshade";
            cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
              inherit cargoExtraArgs;
              doCheck = false;
            });
            preBuild = ''
              cd ./crates/throwshade
            '';
            postBuild = ''
              mv ./dist ../..
              cd ../..
            '';
          });
        };
      in
        {
          packages = {
            inherit native wasm;
            st = inputs.st.packages.x86_64-linux.st;
          };

          devShells.default = native.craneLib.devShell {
            packages = native.nativeBuildInputs ++ native.buildInputs ++ [
              pkgs.trunk
              pkgs.rust-analyzer
              pkgs.glxinfo
              pkgs.cmake
            ];
            LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${
              pkgs.lib.makeLibraryPath [
                pkgs.xorg.libX11 
                pkgs.xorg.libXcursor 
                pkgs.xorg.libXi 
                pkgs.libxkbcommon 
                pkgs.xorg.libxcb  
                pkgs.libglvnd
              ]
            }";
          };
        });
}
