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
      native = {
        renderer = inputs.teleia.native.build ./. "renderer";
      };
      wasm = {
        throwshade = inputs.teleia.wasm.build ./. "throwshade";
      };
    in {
      packages.${system} = {
        inherit native wasm;
        st = inputs.st.packages.x86_64-linux.st;
      };

      devShells.${system}.default = inputs.teleia.shell;
    };
}
