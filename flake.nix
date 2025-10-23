{
  inputs = {
    teleia.url = "github:lcolonq/teleia";
    nixpkgs.follows = "teleia/nixpkgs";
    st = {
      url = "github:lcolonq/st";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ self, nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      native = {
        renderer = inputs.teleia.native.build ./. "newton_renderer";
        renderer-nonnix = pkgs.stdenv.mkDerivation {
          name = "newton-renderer-nonnix";
          phases = [ "installPhase" ];
          installPhase = ''
            mkdir $out
            cp -rv ${native.renderer}/bin/newton_renderer $out
            chmod +w $out/newton_renderer
            patchelf --remove-rpath $out/newton_renderer
            patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 $out/newton_renderer
            strip $out/newton_renderer
            chmod -w $out/newton_renderer
          '';
        };
      };
      wasm = {
        shader = (inputs.teleia.wasm.build ./. "newton_shader").overrideAttrs (cur: prev: {
          preBuild = ''
            cd ./crates/shader
          '';
          postBuild = ''
            mv ./dist ../..
            cd ../..
          '';
        });
      };
    in {
      packages.${system} = {
        inherit native wasm;
        st = inputs.st.packages.x86_64-linux.st;
      };
      devShells.${system}.default = inputs.teleia.shell.overrideAttrs (final: prev: {
        buildInputs = prev.buildInputs ++ [
        ];
      });
    };
}
